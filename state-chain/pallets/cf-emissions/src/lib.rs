#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("../README.md")]
#![doc = include_str!("../../cf-doc-head.md")]

use cf_chains::UpdateFlipSupply;
use cf_traits::{Broadcaster, EthEnvironmentProvider, ReplayProtectionProvider};
use frame_support::dispatch::Weight;
use frame_system::pallet_prelude::BlockNumberFor;
pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod migrations;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub const PALLET_VERSION: StorageVersion = StorageVersion::new(2);
pub const SUPPLY_UPDATE_INTERVAL_DEFAULT: u64 = 100;

use cf_traits::{BlockEmissions, EpochTransitionHandler, Issuance, RewardsDistribution};
use frame_support::traits::{Get, Imbalance, OnRuntimeUpgrade, StorageVersion};
use sp_arithmetic::traits::UniqueSaturatedFrom;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, CheckedDiv, UniqueSaturatedInto, Zero},
	SaturatedConversion,
};

pub mod weights;
pub use weights::WeightInfo;

type BasisPoints = u32;

#[frame_support::pallet]
pub mod pallet {

	use super::*;
	use cf_chains::ChainAbi;
	use cf_traits::SystemStateInfo;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::OriginFor;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	#[pallet::disable_frame_system_supertrait_check]
	pub trait Config: cf_traits::Chainflip {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The host chain to which we broadcast supply updates.
		///
		/// In practice this is always [Ethereum] but making this configurable simplifies
		/// testing.
		type HostChain: ChainAbi;

		/// The Flip token denomination.
		type FlipBalance: Member
			+ Parameter
			+ MaxEncodedLen
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ AtLeast32BitUnsigned
			+ UniqueSaturatedFrom<Self::BlockNumber>;

		/// An imbalance type representing freshly minted, unallocated funds.
		type Surplus: Imbalance<Self::FlipBalance>;

		/// An implementation of the [Issuance] trait.
		type Issuance: Issuance<
			Balance = Self::FlipBalance,
			AccountId = Self::AccountId,
			Surplus = Self::Surplus,
		>;

		/// An implementation of `RewardsDistribution` defining how to distribute the emissions.
		type RewardsDistribution: RewardsDistribution<
			Balance = Self::FlipBalance,
			Issuance = Self::Issuance,
		>;

		/// An outgoing api call that supports UpdateFlipSupply.
		type ApiCall: UpdateFlipSupply<Self::HostChain>;

		/// Transaction broadcaster for the host chain.
		type Broadcaster: Broadcaster<Self::HostChain, ApiCall = Self::ApiCall>;

		/// Blocks per day.
		#[pallet::constant]
		type BlocksPerDay: Get<Self::BlockNumber>;

		/// Something that can provide a nonce for the threshold signature.
		type ReplayProtectionProvider: ReplayProtectionProvider<Self::HostChain>;

		/// Something that can provide the stake manager address.
		type EthEnvironmentProvider: EthEnvironmentProvider;

		/// Benchmark stuff
		type WeightInfo: WeightInfo;

		/// For governance checks.
		type EnsureGovernance: EnsureOrigin<Self::Origin>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(PALLET_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn last_supply_update_block)]
	/// The block number at which we last updated supply to the Eth Chain.
	pub type LastSupplyUpdateBlock<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn current_authority_emission_per_block)]
	/// The amount of Flip we mint to validators per block.
	pub type CurrentAuthorityEmissionPerBlock<T: Config> =
		StorageValue<_, T::FlipBalance, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn backup_node_emission_per_block)]
	/// The amount of Flip we mint to backup nodes per block.
	pub type BackupNodeEmissionPerBlock<T: Config> = StorageValue<_, T::FlipBalance, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn current_authority_emission_inflation)]
	/// Annual inflation set aside for current authorities, expressed as basis points ie. hundredths
	/// of a percent.
	pub(super) type CurrentAuthorityEmissionInflation<T: Config> =
		StorageValue<_, BasisPoints, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn backup_node_emission_inflation)]
	/// Annual inflation set aside for *backup* nodes, expressed as basis points ie. hundredths
	/// of a percent.
	pub(super) type BackupNodeEmissionInflation<T: Config> =
		StorageValue<_, BasisPoints, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn supply_update_interval)]
	/// Mint interval in blocks
	pub(super) type SupplyUpdateInterval<T: Config> =
		StorageValue<_, BlockNumberFor<T>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Supply Update has been Broadcasted [block_number]
		SupplyUpdateBroadcasted(BlockNumberFor<T>),
		/// Current authority inflation emission has been updated \[new\]
		CurrentAuthorityInflationEmissionsUpdated(BasisPoints),
		/// Backup node inflation emission has been updated \[new\]
		BackupNodeInflationEmissionsUpdated(BasisPoints),
		/// SupplyUpdateInterval has been updated [block_number]
		SupplyUpdateIntervalUpdated(BlockNumberFor<T>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Emissions calculation resulted in overflow.
		Overflow,
		/// Invalid percentage
		InvalidPercentage,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_runtime_upgrade() -> Weight {
			migrations::PalletMigration::<T>::on_runtime_upgrade()
		}

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<(), &'static str> {
			migrations::PalletMigration::<T>::pre_upgrade()
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade() -> Result<(), &'static str> {
			migrations::PalletMigration::<T>::post_upgrade()
		}
		fn on_initialize(current_block: BlockNumberFor<T>) -> Weight {
			Self::mint_rewards_for_block();
			if Self::should_update_supply_at(current_block) {
				if T::SystemState::ensure_no_maintenance().is_ok() {
					Self::broadcast_update_total_supply(
						T::Issuance::total_issuance(),
						current_block,
					);
					Self::deposit_event(Event::SupplyUpdateBroadcasted(current_block));
					// Update this pallet's state.
					LastSupplyUpdateBlock::<T>::set(current_block);
				} else {
					log::info!("System maintenance: skipping supply update broadcast.");
				}
			}
			T::WeightInfo::rewards_minted()
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Updates the emission rate to Validators.
		///
		/// Can only be called by the root origin.
		///
		/// ## Events
		///
		/// - [CurrentAuthorityInflationEmissionsUpdated](Event::
		///   CurrentAuthorityInflationEmissionsUpdated)
		///
		/// ## Errors
		///
		/// - [BadOrigin](frame_support::error::BadOrigin)
		#[pallet::weight(T::WeightInfo::update_current_authority_emission_inflation())]
		pub fn update_current_authority_emission_inflation(
			origin: OriginFor<T>,
			inflation: BasisPoints,
		) -> DispatchResultWithPostInfo {
			T::EnsureGovernance::ensure_origin(origin)?;
			CurrentAuthorityEmissionInflation::<T>::set(inflation);
			Self::deposit_event(Event::<T>::CurrentAuthorityInflationEmissionsUpdated(inflation));
			Ok(().into())
		}

		/// Updates the emission rate to Backup nodes.
		///
		/// ## Events
		///
		/// - [BackupNodeInflationEmissionsUpdated](Event:: BackupNodeInflationEmissionsUpdated)
		///
		/// ## Errors
		///
		/// - [BadOrigin](frame_support::error::BadOrigin)
		#[pallet::weight(T::WeightInfo::update_backup_node_emission_inflation())]
		pub fn update_backup_node_emission_inflation(
			origin: OriginFor<T>,
			inflation: BasisPoints,
		) -> DispatchResultWithPostInfo {
			T::EnsureGovernance::ensure_origin(origin)?;
			BackupNodeEmissionInflation::<T>::set(inflation);
			Self::deposit_event(Event::<T>::BackupNodeInflationEmissionsUpdated(inflation));
			Ok(().into())
		}

		/// Updates the Supply Update interval.
		///
		/// ## Events
		///
		/// - [SupplyUpdateIntervalUpdated](Event:: SupplyUpdateIntervalUpdated)
		///
		/// ## Errors
		///
		/// - [BadOrigin](frame_support::error::BadOrigin)
		#[pallet::weight(T::WeightInfo::update_supply_update_interval())]
		pub fn update_supply_update_interval(
			origin: OriginFor<T>,
			value: BlockNumberFor<T>,
		) -> DispatchResultWithPostInfo {
			T::EnsureGovernance::ensure_origin(origin)?;
			SupplyUpdateInterval::<T>::put(value);
			Self::deposit_event(Event::<T>::SupplyUpdateIntervalUpdated(value));
			Ok(().into())
		}
	}

	#[pallet::genesis_config]
	#[cfg_attr(feature = "std", derive(Default))]
	pub struct GenesisConfig {
		pub current_authority_emission_inflation: BasisPoints,
		pub backup_node_emission_inflation: BasisPoints,
	}

	/// At genesis we need to set the inflation rates for active and passive validators.
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			CurrentAuthorityEmissionInflation::<T>::put(self.current_authority_emission_inflation);
			BackupNodeEmissionInflation::<T>::put(self.backup_node_emission_inflation);
			SupplyUpdateInterval::<T>::put(T::BlockNumber::from(
				SUPPLY_UPDATE_INTERVAL_DEFAULT as u32,
			));
			<Pallet<T> as BlockEmissions>::calculate_block_emissions();
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Determines if we should broadcast supply update at block number `block_number`.
	fn should_update_supply_at(block_number: T::BlockNumber) -> bool {
		let supply_update_interval = SupplyUpdateInterval::<T>::get();
		let blocks_elapsed = block_number - LastSupplyUpdateBlock::<T>::get();
		Self::should_update_supply(blocks_elapsed, supply_update_interval)
	}

	/// Checks if we should broadcast supply update.
	fn should_update_supply(
		blocks_elapsed_since_last_supply_update: T::BlockNumber,
		supply_update_interval: T::BlockNumber,
	) -> bool {
		blocks_elapsed_since_last_supply_update >= supply_update_interval
	}

	/// Updates the total supply on the ETH blockchain
	fn broadcast_update_total_supply(total_supply: T::FlipBalance, block_number: T::BlockNumber) {
		// Emit a threshold signature request.
		// TODO: See if we can replace an old request if there is one.
		T::Broadcaster::threshold_sign_and_broadcast(T::ApiCall::new_unsigned(
			T::ReplayProtectionProvider::replay_protection(),
			total_supply.unique_saturated_into(),
			block_number.saturated_into(),
			&T::EthEnvironmentProvider::stake_manager_address(),
		));
	}

	/// Mints and distributes block author rewards via [RewardsDistribution].
	fn mint_rewards_for_block() {
		// Mint and Delegate the distribution.
		T::RewardsDistribution::distribute();
	}
}

impl<T: Config> BlockEmissions for Pallet<T> {
	type Balance = T::FlipBalance;

	fn update_authority_block_emission(emission: Self::Balance) {
		CurrentAuthorityEmissionPerBlock::<T>::put(emission);
	}

	fn update_backup_node_block_emission(emission: Self::Balance) {
		BackupNodeEmissionPerBlock::<T>::put(emission);
	}

	fn calculate_block_emissions() {
		fn inflation_to_block_reward<T: Config>(inflation: BasisPoints) -> T::FlipBalance {
			const DAYS_IN_YEAR: u32 = 365;

			((T::Issuance::total_issuance() * inflation.into()) /
				10_000u32.into() / DAYS_IN_YEAR.into())
			.checked_div(&T::FlipBalance::unique_saturated_from(T::BlocksPerDay::get()))
			.unwrap_or_else(|| {
				log::error!("blocks per day should be greater than zero");
				Zero::zero()
			})
		}

		Self::update_authority_block_emission(inflation_to_block_reward::<T>(
			CurrentAuthorityEmissionInflation::<T>::get(),
		));

		Self::update_backup_node_block_emission(inflation_to_block_reward::<T>(
			BackupNodeEmissionInflation::<T>::get(),
		));
	}
}

impl<T: Config> EpochTransitionHandler for Pallet<T> {
	type ValidatorId = <T as frame_system::Config>::AccountId;

	fn on_new_epoch(_epoch_authorities: &[Self::ValidatorId]) {
		// Calculate block emissions on every epoch
		Self::calculate_block_emissions();
	}
}
