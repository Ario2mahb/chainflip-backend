//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{
	benchmarks, benchmarks_instance_pallet, impl_benchmark_test_suite, whitelisted_caller,
};

use cf_chains::eth::{UnsignedTransaction, U256};
use frame_support::{dispatch::UnfilteredDispatchable, traits::EnsureOrigin};
use frame_system::RawOrigin;

#[allow(unused)]
use crate::Pallet as Auction;

benchmarks_instance_pallet! {
	on_initialize {} : {}
	start_broadcast {} : {}
	// start_broadcast {
	// 	let caller: T::AccountId = whitelisted_caller();
	// 	let unsigned: SignedTransactionFor<T, I> = UnsignedTransaction {
	// 		chain_id: 42,
	// 		max_fee_per_gas: U256::from(1_000_000_000u32).into(),
	// 		gas_limit: U256::from(21_000u32).into(),
	// 		contract: [0xcf; 20].into(),
	// 		value: 0.into(),
	// 		data: b"do_something()".to_vec(),
	// 		..Default::default()
	// 	};
	// 	let call = Call::<T, I>::start_broadcast(unsigned.into());
	// 	let origin = T::EnsureWitnessed::successful_origin();
	// } : { call.dispatch_bypass_filter(origin)? }
	transaction_ready_for_transmission {} : {}
	transmission_success {} : {}
	transmission_failure {} : {}
}

// TODO: Figure out how to make this work with instantiable pallets.
//
impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test,);
