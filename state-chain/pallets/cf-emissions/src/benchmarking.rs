//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_support::traits::{OnInitialize, OnRuntimeUpgrade};
use frame_system::RawOrigin;

const MINT_INTERVAL: u32 = 100;

#[allow(unused)]
use crate::Pallet as Emissions;

benchmarks! {
	// Benchmark for the backup validator extrinsic
	update_backup_validator_emission_inflation {
		let b in 1 .. 1_000;
	}: _(RawOrigin::Root, b.into())
	update_validator_emission_inflation {
		let b in 1 .. 1_000;
	}: _(RawOrigin::Root, b.into())
	no_rewards_minted {

	} : {
		Emissions::<T>::on_initialize((5 as u32).into());
	}
	// Benchmark for the rewards minted case in the on init hook
	rewards_minted {
	}: {
		Emissions::<T>::on_initialize((MINT_INTERVAL).into());
	}
	update_mint_interval {
	}: _(RawOrigin::Root, (50 as u32).into())
	verify {
		 let mint_interval = Pallet::<T>::mint_interval();
		 assert_eq!(mint_interval, (50 as u32).into());
	}
	// Benchmark for the runtime migration v1
	on_runtime_upgrade_v1 {
		releases::V0.put::<Pallet<T>>();
	} : {
		Emissions::<T>::on_runtime_upgrade();
	} verify {
		assert_eq!(MintInterval::<T>::get(), 100u32.into());
	}
	// Benchmark for a runtime upgrade in which we do nothing
	on_runtime_upgrade {
		releases::V1.put::<Pallet<T>>();
	} : {
		Emissions::<T>::on_runtime_upgrade();
	}
}

impl_benchmark_test_suite!(
	Pallet,
	crate::mock::new_test_ext(Default::default(), Default::default()),
	crate::mock::Test,
);
