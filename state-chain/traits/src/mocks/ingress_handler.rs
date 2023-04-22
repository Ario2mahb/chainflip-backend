use crate::{Chainflip, IngressApi};
use cf_chains::{
	address::ForeignChainAddress, eth::assets::any, CcmIngressMetadata, Chain, ForeignChain,
};
use cf_primitives::{BasisPoints, IntentId};
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::marker::PhantomData;

use super::{MockPallet, MockPalletStorage};

pub struct MockIngressHandler<C, T>(PhantomData<(C, T)>);

impl<C, T> MockPallet for MockIngressHandler<C, T> {
	const PREFIX: &'static [u8] = b"MockIngressHandler";
}

enum SwapOrLp {
	Swap,
	Lp,
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct SwapIntent<C: Chain, T: Chainflip> {
	pub ingress_address: ForeignChainAddress,
	pub ingress_asset: <C as Chain>::ChainAsset,
	pub egress_asset: any::Asset,
	pub egress_address: ForeignChainAddress,
	pub relayer_commission_bps: BasisPoints,
	pub relayer_id: <T as frame_system::Config>::AccountId,
	pub message_metadata: Option<CcmIngressMetadata>,
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct LpIntent<C: Chain, T: Chainflip> {
	pub ingress_address: ForeignChainAddress,
	pub ingress_asset: <C as Chain>::ChainAsset,
	pub lp_account: <T as frame_system::Config>::AccountId,
}

impl<C: Chain, T: Chainflip> MockIngressHandler<C, T> {
	fn get_new_intent(
		swap_or_lp: SwapOrLp,
		asset: <C as Chain>::ChainAsset,
	) -> (IntentId, ForeignChainAddress) {
		let intent_id = <Self as MockPalletStorage>::mutate_value(
			match swap_or_lp {
				SwapOrLp::Swap => b"SWAP_INTENT_ID",
				SwapOrLp::Lp => b"LP_INTENT_ID",
			},
			|storage| {
				let intent_id: IntentId = storage.unwrap_or_default();
				let _ = storage.insert(intent_id + 1);
				intent_id
			},
		);
		(
			intent_id,
			match asset.into() {
				ForeignChain::Ethereum => ForeignChainAddress::Eth([intent_id as u8; 20]),
				ForeignChain::Polkadot => ForeignChainAddress::Dot([intent_id as u8; 32]),
				ForeignChain::Bitcoin => todo!("Bitcoin address"),
			},
		)
	}

	pub fn get_liquidity_intents() -> Vec<LpIntent<C, T>> {
		<Self as MockPalletStorage>::get_value(b"LP_INGRESS_INTENTS").unwrap_or_default()
	}

	pub fn get_swap_intents() -> Vec<SwapIntent<C, T>> {
		<Self as MockPalletStorage>::get_value(b"SWAP_INGRESS_INTENTS").unwrap_or_default()
	}
}

impl<C: Chain, T: Chainflip> IngressApi<C> for MockIngressHandler<C, T> {
	type AccountId = <T as frame_system::Config>::AccountId;

	fn register_liquidity_ingress_intent(
		lp_account: Self::AccountId,
		ingress_asset: <C as cf_chains::Chain>::ChainAsset,
	) -> Result<(cf_primitives::IntentId, ForeignChainAddress), sp_runtime::DispatchError> {
		let (intent_id, ingress_address) = Self::get_new_intent(SwapOrLp::Lp, ingress_asset);
		<Self as MockPalletStorage>::mutate_value(b"LP_INGRESS_INTENTS", |lp_intents| {
			if lp_intents.is_none() {
				*lp_intents = Some(vec![]);
			}
			if let Some(inner) = lp_intents.as_mut() {
				inner.push(LpIntent::<C, T> {
					ingress_address: ingress_address.clone(),
					ingress_asset,
					lp_account,
				});
			}
		});
		Ok((intent_id, ingress_address))
	}

	fn register_swap_intent(
		ingress_asset: <C as Chain>::ChainAsset,
		egress_asset: cf_primitives::Asset,
		egress_address: ForeignChainAddress,
		relayer_commission_bps: BasisPoints,
		relayer_id: Self::AccountId,
		message_metadata: Option<CcmIngressMetadata>,
	) -> Result<(cf_primitives::IntentId, ForeignChainAddress), sp_runtime::DispatchError> {
		let (intent_id, ingress_address) = Self::get_new_intent(SwapOrLp::Swap, ingress_asset);
		<Self as MockPalletStorage>::mutate_value(b"SWAP_INGRESS_INTENTS", |swap_intents| {
			if swap_intents.is_none() {
				*swap_intents = Some(vec![]);
			}
			if let Some(inner) = swap_intents.as_mut() {
				inner.push(SwapIntent::<C, T> {
					ingress_address: ingress_address.clone(),
					ingress_asset,
					egress_asset,
					egress_address,
					relayer_commission_bps,
					relayer_id,
					message_metadata,
				});
			};
		});
		Ok((intent_id, ingress_address))
	}

	fn expire_intent(
		_chain: ForeignChain,
		_intent_id: IntentId,
		address: <C as cf_chains::Chain>::ChainAccount,
	) {
		<Self as MockPalletStorage>::mutate_value(
			b"SWAP_INGRESS_INTENTS",
			|storage: &mut Option<Vec<SwapIntent<C, T>>>| {
				if let Some(inner) = storage.as_mut() {
					inner.retain(|x| x.ingress_address != address.clone().into());
				}
			},
		);
		<Self as MockPalletStorage>::mutate_value(
			b"LP_INGRESS_INTENTS",
			|storage: &mut Option<Vec<LpIntent<C, T>>>| {
				if let Some(inner) = storage.as_mut() {
					inner.retain(|x| x.ingress_address != address.clone().into());
				}
			},
		);
	}
}
