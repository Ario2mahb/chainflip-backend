//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::dispatch::UnfilteredDispatchable;
use frame_system::Origin;
use frame_system::RawOrigin;
use sp_std::{boxed::Box, vec, vec::Vec};

#[allow(unused)]
use crate::Pallet;

benchmarks! {

	staked {
		let balance: T::Balance = T::Balance::from(100 as u32);
		let eth_addr: EthereumAddress = [42u8; 20];
		let tx_hash: pallet::EthTransactionHash = [211u8; 32];
		let caller: T::AccountId = whitelisted_caller();

		let call = Call::<T>::staked(caller, balance, eth_addr, tx_hash);
		let origin = T::EnsureWitnessed::successful_origin();

	}: { call.dispatch_bypass_filter(origin)? }

	claim {
		let balance_to_claim: T::Balance = T::Balance::from(50 as u32);
		let balance_to_stake: T::Balance = T::Balance::from(100 as u32);
		let tx_hash: pallet::EthTransactionHash = [211u8; 32];
		let eth_addr: EthereumAddress = [42u8; 20];

		let caller: T::AccountId = whitelisted_caller();
		let origin = T::EnsureWitnessed::successful_origin();

		// Stake some funds to claim
		let stake_call = Call::<T>::staked(caller.clone(), balance_to_stake, eth_addr, tx_hash);
		stake_call.dispatch_bypass_filter(origin)?;

	} :_(RawOrigin::Signed(caller.clone()), balance_to_claim, eth_addr)

	claim_all {
		let eth_addr: EthereumAddress = [42u8; 20];
		let caller: T::AccountId = whitelisted_caller();

	}:_(RawOrigin::Signed(caller), eth_addr)

	// Ends in success case - could be that the error case is more expensive?
	claimed {
		let balance_to_claim: T::Balance = T::Balance::from(100 as u32);
		let balance_to_stake: T::Balance = T::Balance::from(100 as u32);
		let tx_hash: pallet::EthTransactionHash = [211u8; 32];
		let eth_addr: EthereumAddress = [42u8; 20];

		let caller: T::AccountId = whitelisted_caller();
		let origin = T::EnsureWitnessed::successful_origin();

		// Stake some funds to claim
		let stake_call = Call::<T>::staked(caller.clone(), balance_to_stake, eth_addr, tx_hash);
		stake_call.dispatch_bypass_filter(origin.clone())?;

		// Push a claim
		let claimable = T::Flip::claimable_balance(&caller);
		Pallet::<T>::do_claim(&caller, claimable, eth_addr)?;

		let call = Call::<T>::claimed(caller.clone(), balance_to_claim, tx_hash);

	}: { call.dispatch_bypass_filter(origin.clone())? }

	post_claim_signature {
		let sig: U256 = U256::zero();
		let tx_hash: pallet::EthTransactionHash = [211u8; 32];
		let eth_addr: EthereumAddress = [42u8; 20];
		let balance_to_stake: T::Balance = T::Balance::from(100 as u32);

		let caller: T::AccountId = whitelisted_caller();
		let origin = T::EnsureWitnessed::successful_origin();

		// Stake some funds to claim
		let stake_call = Call::<T>::staked(caller.clone(), balance_to_stake, eth_addr, tx_hash);
		stake_call.dispatch_bypass_filter(origin.clone())?;

		// Push a claim
		let claimable = T::Flip::claimable_balance(&caller);
		Pallet::<T>::do_claim(&caller, claimable, eth_addr)?;

		let msg_hash = PendingClaims::<T>::get(caller.clone()).unwrap().msg_hash.unwrap();
		let call = Call::<T>::post_claim_signature(caller, msg_hash, sig);
	}: { call.dispatch_bypass_filter(origin)? }

	retire_account {
		let caller: T::AccountId = whitelisted_caller();
		AccountRetired::<T>::insert(caller.clone(), false);

	}:_(RawOrigin::Signed(caller))

	activate_account {
		let caller: T::AccountId = whitelisted_caller();
		AccountRetired::<T>::insert(caller.clone(), true);

	}:_(RawOrigin::Signed(caller))
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test,);
