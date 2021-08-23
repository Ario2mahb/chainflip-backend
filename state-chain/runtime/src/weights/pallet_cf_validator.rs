//! Autogenerated weights for pallet_cf_validator
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-05-18, STEPS: [20, ], REPEAT: 10, LOW RANGE: [], HIGH RANGE: []
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 128

// Executed Command:
// target/debug/state-chain-node
// benchmark
// --chain
// dev
// --execution
// wasm
// --wasm-execution
// compiled
// --pallet
// pallet_cf_validator
// --extrinsic
// *
// --steps
// 20
// --repeat
// 10
// --raw
// --output
// ./

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for pallet_cf_validator.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_cf_validator::WeightInfo for WeightInfo<T> {
	fn set_blocks_for_epoch() -> Weight {
		(477_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn force_rotation() -> Weight {
		(401_000_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}
