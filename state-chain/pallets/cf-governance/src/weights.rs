
//! Autogenerated weights for pallet_cf_governance
//!
//! THIS FILE WAS AUTO-GENERATED USING THE CHAINFLIP BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-01-20, STEPS: `20`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `wagmi.local`, CPU: `<UNKNOWN>`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// ./target/production/chainflip-node
// benchmark
// pallet
// --pallet
// pallet_cf_governance
// --extrinsic
// *
// --output
// state-chain/pallets/cf-governance/src/weights.rs
// --execution=wasm
// --steps=20
// --repeat=10
// --template=state-chain/chainflip-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_cf_governance.
pub trait WeightInfo {
	fn propose_governance_extrinsic() -> Weight;
	fn approve() -> Weight;
	fn new_membership_set() -> Weight;
	fn call_as_sudo() -> Weight;
	fn on_initialize(b: u32, ) -> Weight;
	fn on_initialize_best_case() -> Weight;
	fn expire_proposals(b: u32, ) -> Weight;
	fn set_whitelisted_call_hash() -> Weight;
	fn submit_govkey_call() -> Weight;
}

/// Weights for pallet_cf_governance using the Substrate node and recommended hardware.
pub struct PalletWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for PalletWeight<T> {
	// Storage: Governance Members (r:1 w:0)
	// Storage: Governance ProposalIdCounter (r:1 w:1)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: Governance ExpiryTime (r:1 w:0)
	// Storage: Governance ActiveProposals (r:1 w:1)
	// Storage: Governance ExecutionPipeline (r:1 w:1)
	// Storage: Governance Proposals (r:0 w:1)
	fn propose_governance_extrinsic() -> Weight {
		// Minimum execution time: 29_000 nanoseconds.
		Weight::from_parts(30_000_000, 0)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: Governance Members (r:1 w:0)
	// Storage: Governance Proposals (r:1 w:1)
	// Storage: Governance ExecutionPipeline (r:1 w:1)
	// Storage: Governance ActiveProposals (r:1 w:1)
	fn approve() -> Weight {
		// Minimum execution time: 23_000 nanoseconds.
		Weight::from_parts(24_000_000, 0)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: Governance Members (r:0 w:1)
	fn new_membership_set() -> Weight {
		// Minimum execution time: 4_000 nanoseconds.
		Weight::from_parts(4_000_000, 0)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: System Digest (r:1 w:1)
	// Storage: unknown [0x3a636f6465] (r:0 w:1)
	fn call_as_sudo() -> Weight {
		// Minimum execution time: 14_000 nanoseconds.
		Weight::from_parts(16_000_000, 0)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Governance ActiveProposals (r:1 w:0)
	// Storage: Governance ExecutionPipeline (r:1 w:0)
	// Storage: Timestamp Now (r:1 w:0)
	/// The range of component `b` is `[1, 100]`.
	fn on_initialize(b: u32, ) -> Weight {
		// Minimum execution time: 3_000 nanoseconds.
		Weight::from_parts(6_951_927, 0)
			// Standard Error: 4_941
			.saturating_add(Weight::from_parts(394_899, 0).saturating_mul(b.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Governance ActiveProposals (r:1 w:0)
	// Storage: Governance ExecutionPipeline (r:1 w:0)
	fn on_initialize_best_case() -> Weight {
		// Minimum execution time: 2_000 nanoseconds.
		Weight::from_parts(3_000_000, 0)
			.saturating_add(T::DbWeight::get().reads(2))
	}
	// Storage: Governance ActiveProposals (r:1 w:0)
	// Storage: Governance Proposals (r:0 w:5)
	/// The range of component `b` is `[1, 100]`.
	fn expire_proposals(b: u32, ) -> Weight {
		// Minimum execution time: 2_000 nanoseconds.
		Weight::from_parts(8_763_542, 0)
			// Standard Error: 13_275
			.saturating_add(Weight::from_parts(2_998_230, 0).saturating_mul(b.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(b.into())))
	}
	// Storage: Governance GovKeyWhitelistedCallHash (r:0 w:1)
	fn set_whitelisted_call_hash() -> Weight {
		// Minimum execution time: 11_000 nanoseconds.
		Weight::from_parts(11_000_000, 0)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Governance NextGovKeyCallHashNonce (r:1 w:1)
	// Storage: Governance GovKeyWhitelistedCallHash (r:1 w:1)
	// Storage: Governance Members (r:0 w:1)
	fn submit_govkey_call() -> Weight {
		// Minimum execution time: 21_000 nanoseconds.
		Weight::from_parts(21_000_000, 0)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(3))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Governance Members (r:1 w:0)
	// Storage: Governance ProposalIdCounter (r:1 w:1)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: Governance ExpiryTime (r:1 w:0)
	// Storage: Governance ActiveProposals (r:1 w:1)
	// Storage: Governance ExecutionPipeline (r:1 w:1)
	// Storage: Governance Proposals (r:0 w:1)
	fn propose_governance_extrinsic() -> Weight {
		// Minimum execution time: 29_000 nanoseconds.
		Weight::from_parts(30_000_000, 0)
			.saturating_add(RocksDbWeight::get().reads(6))
			.saturating_add(RocksDbWeight::get().writes(4))
	}
	// Storage: Governance Members (r:1 w:0)
	// Storage: Governance Proposals (r:1 w:1)
	// Storage: Governance ExecutionPipeline (r:1 w:1)
	// Storage: Governance ActiveProposals (r:1 w:1)
	fn approve() -> Weight {
		// Minimum execution time: 23_000 nanoseconds.
		Weight::from_parts(24_000_000, 0)
			.saturating_add(RocksDbWeight::get().reads(4))
			.saturating_add(RocksDbWeight::get().writes(3))
	}
	// Storage: Governance Members (r:0 w:1)
	fn new_membership_set() -> Weight {
		// Minimum execution time: 4_000 nanoseconds.
		Weight::from_parts(4_000_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: System Digest (r:1 w:1)
	// Storage: unknown [0x3a636f6465] (r:0 w:1)
	fn call_as_sudo() -> Weight {
		// Minimum execution time: 14_000 nanoseconds.
		Weight::from_parts(16_000_000, 0)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
	// Storage: Governance ActiveProposals (r:1 w:0)
	// Storage: Governance ExecutionPipeline (r:1 w:0)
	// Storage: Timestamp Now (r:1 w:0)
	/// The range of component `b` is `[1, 100]`.
	fn on_initialize(b: u32, ) -> Weight {
		// Minimum execution time: 3_000 nanoseconds.
		Weight::from_parts(6_951_927, 0)
			// Standard Error: 4_941
			.saturating_add(Weight::from_parts(394_899, 0).saturating_mul(b.into()))
			.saturating_add(RocksDbWeight::get().reads(3))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: Governance ActiveProposals (r:1 w:0)
	// Storage: Governance ExecutionPipeline (r:1 w:0)
	fn on_initialize_best_case() -> Weight {
		// Minimum execution time: 2_000 nanoseconds.
		Weight::from_parts(3_000_000, 0)
			.saturating_add(RocksDbWeight::get().reads(2))
	}
	// Storage: Governance ActiveProposals (r:1 w:0)
	// Storage: Governance Proposals (r:0 w:5)
	/// The range of component `b` is `[1, 100]`.
	fn expire_proposals(b: u32, ) -> Weight {
		// Minimum execution time: 2_000 nanoseconds.
		Weight::from_parts(8_763_542, 0)
			// Standard Error: 13_275
			.saturating_add(Weight::from_parts(2_998_230, 0).saturating_mul(b.into()))
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes((1_u64).saturating_mul(b.into())))
	}
	// Storage: Governance GovKeyWhitelistedCallHash (r:0 w:1)
	fn set_whitelisted_call_hash() -> Weight {
		// Minimum execution time: 11_000 nanoseconds.
		Weight::from_parts(11_000_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: Governance NextGovKeyCallHashNonce (r:1 w:1)
	// Storage: Governance GovKeyWhitelistedCallHash (r:1 w:1)
	// Storage: Governance Members (r:0 w:1)
	fn submit_govkey_call() -> Weight {
		// Minimum execution time: 21_000 nanoseconds.
		Weight::from_parts(21_000_000, 0)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(3))
	}
}
