use std::time::Duration;

use crate as pallet_cf_witness_api;
use cf_traits::{
	impl_mock_ensure_witnessed_for_origin, impl_mock_stake_transfer,
	impl_mock_witnesser_for_account_and_call_types, AuctionConfirmation,
};
use frame_support::{parameter_types, traits::ValidatorRegistration};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		Auction: pallet_cf_auction::{Module, Call, Event<T>, Config},
		Staking: pallet_cf_staking::{Module, Call, Event<T>, Config<T>},
		WitnessApi: pallet_cf_witness_api::{Module, Call},
	}
);

impl_mock_witnesser_for_account_and_call_types!(u64, Call);
impl_mock_ensure_witnessed_for_origin!(Origin);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
	pub const MinClaimTTL: Duration = Duration::from_millis(100);
	pub const ClaimTTL: Duration = Duration::from_millis(1000);
	pub const MinAuctionSize: u32 = 2;
}

impl system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
}

impl_mock_stake_transfer!(u64, u128);

impl pallet_cf_staking::Config for Test {
	type Event = Event;
	type Balance = u128;
	type Flip = MockStakeTransfer;
	type Nonce = u64;
	type EnsureWitnessed = MockEnsureWitnessed;
	type EpochInfo = cf_traits::mocks::epoch_info::Mock;
	type TimeSource = cf_traits::mocks::time_source::Mock;
	type MinClaimTTL = MinClaimTTL;
	type ClaimTTL = ClaimTTL;
}

pub struct MockAuctionTraits;

impl cf_traits::BidderProvider for MockAuctionTraits {
	type ValidatorId = u64;
	type Amount = u128;

	fn get_bidders() -> Vec<(Self::ValidatorId, Self::Amount)> {
		vec![]
	}
}

impl ValidatorRegistration<u64> for MockAuctionTraits {
	fn is_registered(_id: &u64) -> bool {
		true
	}
}

impl AuctionConfirmation for MockAuctionTraits {
	fn awaiting_confirmation() -> bool {
		true
	}
	fn set_awaiting_confirmation(_waiting: bool) {
		unimplemented!()
	}
}

impl pallet_cf_auction::Config for Test {
	type Event = Event;
	type Amount = u128;
	type ValidatorId = u64;
	type BidderProvider = MockAuctionTraits;
	type Registrar = MockAuctionTraits;
	type AuctionIndex = u32;
	type MinAuctionSize = MinAuctionSize;
	type Confirmation = MockAuctionTraits;
	type EnsureWitnessed = MockEnsureWitnessed;
}

impl pallet_cf_witness_api::Config for Test {
	type Call = Call;
	type Witnesser = MockWitnesser;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap()
		.into()
}