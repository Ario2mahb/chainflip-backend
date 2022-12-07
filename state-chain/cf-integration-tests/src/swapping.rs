//! Contains tests related to liquidity, pools and swapping
use crate::{ALICE, CHARLIE};
use frame_support::{
	assert_ok,
	traits::{Hooks, OnNewAccount},
};
use state_chain_runtime::{
	chainflip::address_derivation::AddressDerivation, AccountRoles, Call, EpochInfo,
	EthereumIngressEgress, EthereumInstance, Event, LiquidityPools, LiquidityProvider, Origin,
	Runtime, Swapping, System, Validator, Witnesser,
};

use cf_primitives::{
	chains::{assets::eth, Ethereum},
	AccountId, AccountRole, Asset, AssetAmount, ExchangeRate, ForeignChainAddress, TradingPosition,
};
use cf_traits::{AddressDerivationApi, LiquidityPoolApi, LpProvisioningApi};
use pallet_cf_ingress_egress::IngressWitness;

#[test]
fn can_swap_assets() {
	super::genesis::default().build().execute_with(|| {
		// Register the liquidity provider account.
		let liquidity_provider: AccountId = AccountId::from([0xFF; 32]);
		AccountRoles::on_new_account(&liquidity_provider);
		assert_ok!(LiquidityProvider::register_lp_account(Origin::signed(
			liquidity_provider.clone()
		)));

		// Register the relayer account.
		let relayer: AccountId = AccountId::from([0xFE; 32]);
		AccountRoles::on_new_account(&relayer);
		assert_ok!(AccountRoles::register_account_role(
			Origin::signed(relayer.clone()),
			AccountRole::Relayer
		));

		let egress_address = [1u8; 20];

		// Provide liquidity to the exchange pool.
		assert_ok!(LiquidityProvider::provision_account(
			&liquidity_provider,
			Asset::Flip,
			1_000_000
		));
		assert_ok!(LiquidityProvider::provision_account(
			&liquidity_provider,
			Asset::Usdc,
			10_000_000
		));
		assert_ok!(LiquidityProvider::provision_account(
			&liquidity_provider,
			Asset::Eth,
			1_000_000
		));

		// Gives Flip : USDC a 1:10 ratio.
		assert_ok!(LiquidityProvider::open_position(
			Origin::signed(liquidity_provider.clone()),
			Asset::Flip,
			TradingPosition::ClassicV3 {
				range: Default::default(),
				volume_0: 100_000,
				volume_1: 1_000_000
			}
		));
		assert_eq!(
			LiquidityPools::swap_rate(&Asset::Flip, 0u128),
			ExchangeRate::from_rational(10, 1)
		);

		// Gives Eth : USDC a 1 : 5 ratio.
		assert_ok!(LiquidityProvider::open_position(
			Origin::signed(liquidity_provider),
			Asset::Eth,
			TradingPosition::ClassicV3 {
				range: Default::default(),
				volume_0: 200_000,
				volume_1: 1_000_000
			}
		));
		assert_eq!(
			LiquidityPools::swap_rate(&Asset::Eth, 0u128),
			ExchangeRate::from_rational(5, 1)
		);

		System::reset_events();
		// Test swap
		assert_ok!(Swapping::register_swap_intent(
			Origin::signed(relayer),
			Asset::Eth,
			Asset::Flip,
			ForeignChainAddress::Eth(egress_address),
			0u16,
		));

		// Note the ingress address here
		let ingress_address =
			<AddressDerivation as AddressDerivationApi<Ethereum>>::generate_address(
				eth::Asset::Eth,
				pallet_cf_ingress_egress::IntentIdCounter::<Runtime, EthereumInstance>::get(),
			)
			.expect("Should be able to generate a valid eth address.");

		System::assert_has_event(Event::EthereumIngressEgress(
			pallet_cf_ingress_egress::Event::StartWitnessing {
				ingress_address,
				ingress_asset: eth::Asset::Eth,
			},
		));

		const SWAP_AMOUNT: AssetAmount = 10_000;
		// Define the ingress call
		let ingress_call =
			Box::new(Call::EthereumIngressEgress(pallet_cf_ingress_egress::Call::do_ingress {
				ingress_witnesses: vec![IngressWitness {
					ingress_address,
					asset: eth::Asset::Eth,
					amount: SWAP_AMOUNT,
					tx_hash: Default::default(),
				}],
			}));

		// Get the current authorities to witness the ingress.
		let nodes = Validator::current_authorities();
		let current_epoch = Validator::current_epoch();
		for node in &nodes {
			assert_ok!(Witnesser::witness_at_epoch(
				Origin::signed(node.clone()),
				ingress_call.clone(),
				current_epoch
			));
		}

		System::assert_has_event(Event::EthereumIngressEgress(
			pallet_cf_ingress_egress::Event::IngressCompleted {
				ingress_address,
				asset: eth::Asset::Eth,
				amount: SWAP_AMOUNT,
				tx_hash: Default::default(),
			},
		));

		System::assert_has_event(Event::Swapping(pallet_cf_swapping::Event::SwapIngressReceived {
			ingress_address: ForeignChainAddress::Eth(ingress_address.to_fixed_bytes()),
			swap_id: pallet_cf_swapping::SwapIdCounter::<Runtime>::get(),
			ingress_amount: SWAP_AMOUNT,
		}));

		// Performs the actual swap during on_idle hooks.
		let _ = Swapping::on_idle(1, 1_000_000_000_000);

		// Flip: $10, Eth: $5
		// 10_000 Eth = about 5_000 Flips - slippage
		// TODO: Calculate this using the exchange rate.
		const EXPECTED_OUTPUT: AssetAmount = 4545;
		System::assert_has_event(Event::EthereumIngressEgress(
			pallet_cf_ingress_egress::Event::EgressScheduled {
				asset: eth::Asset::Flip,
				amount: EXPECTED_OUTPUT,
				egress_address: egress_address.into(),
			},
		));
		// Flip: 100_000 -> 95_455: -4545, USDC: 1_000_000 -> 1_047_619: +47_619
		// TODO: Use exchange rates instead of magic numbers.
		assert_eq!(
			LiquidityPools::get_liquidity(&Asset::Flip),
			(100_000 - EXPECTED_OUTPUT, 1_047_619)
		);

		// Eth: 200_000 -> 210_000: +10_000, USDC: 1_000_000 -> 952_381: -47_619
		assert_eq!(LiquidityPools::get_liquidity(&Asset::Eth), (200_000 + SWAP_AMOUNT, 952_381));

		// Egress the asset out during on_idle.
		let _ = EthereumIngressEgress::on_idle(1, 1_000_000_000_000);

		// Swapped asset is broadcasted out into the Ethereum chain. This completes the Swap action.
		System::assert_has_event(Event::EthereumThresholdSigner(
			pallet_cf_threshold_signature::Event::ThresholdSignatureRequest {
				request_id: 1,
				ceremony_id: 1,
				key_id: vec![
					3, 106, 138, 135, 164, 185, 128, 208, 210, 182, 238, 29, 65, 19, 108, 86, 107,
					153, 17, 26, 90, 110, 67, 218, 145, 182, 247, 80, 16, 106, 240, 177, 79,
				],
				signatories: [ALICE.into(), CHARLIE.into()].into(),
				payload: hex_literal::hex!(
					"2d7163ee98544e0484c111577e5da357edcb29cea63a227de2a1b8dc4f4e0783"
				)
				.into(),
			},
		));

		System::assert_has_event(Event::EthereumIngressEgress(
			pallet_cf_ingress_egress::Event::BatchBroadcastRequested {
				fetch_batch_size: 1,
				egress_batch_size: 1,
			},
		));
	});
}