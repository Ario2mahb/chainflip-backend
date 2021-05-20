#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::{DispatchResultWithPostInfo, Dispatchable};
use sp_std::prelude::*;
use frame_support::traits::ValidatorRegistration;

/// A trait abstracting the functionality of the witnesser
pub trait Witnesser {
	/// The type of accounts that can witness.
	type AccountId;

	/// The call type of the runtime.
	type Call: Dispatchable;

	/// Witness an event. The event is represented by a call, which should be
	/// dispatched when a threshold number of witnesses have been made.
	fn witness(who: Self::AccountId, call: Self::Call) -> DispatchResultWithPostInfo;
}

pub trait EpochInfo {
	/// The id type used for the validators.
	type ValidatorId;
	/// An amount
	type Amount;
	/// The index of an epoch
	type EpochIndex;

	/// The current set of validators
	fn current_validators() -> Vec<Self::ValidatorId>;

	/// Checks if the account is currently a validator.
	fn is_validator(account: &Self::ValidatorId) -> bool;

	/// If we are in auction phase then the proposed set to validate once the auction is
	/// confirmed else an empty vector
	fn next_validators() -> Vec<Self::ValidatorId>;

	/// The amount to be used as bond, this is the minimum stake needed to get into the
	/// candidate validator set
	fn bond() -> Self::Amount;

	/// The current epoch we are in
	fn epoch_index() -> Self::EpochIndex;
}

/// Something that can provide us a list of candidates with their corresponding stakes
pub trait CandidateProvider {
	type ValidatorId;
	type Amount;

	fn get_candidates() -> Vec<(Self::ValidatorId, Self::Amount)>;
}

/// A set of validators and their stake
pub type ValidatorSet<T> = Vec<(<T as Auction>::ValidatorId, <T as Auction>::Amount)>;
/// A proposal of validators after an auction with bond amount
pub type ValidatorProposal<T> = (Vec<<T as Auction>::ValidatorId>, <T as Auction>::Amount);

#[derive(Debug, PartialEq)]
pub enum AuctionError {
	BondIsZero,
}

pub trait Auction {
	/// The id type used for the validators.
	type ValidatorId;
	/// An amount
	type Amount;
	/// A registrar to validate keys
	type Registrar: ValidatorRegistration<Self::ValidatorId>;
	/// Validate before running the auction the set of validators
	/// An empty vector is a bad bunch
	fn validate_auction(candidates: ValidatorSet<Self>) -> ValidatorSet<Self>;

	/// Run an auction with a set of validators returning the a proposed set of validators with the bond amount
	fn run_auction(candidates: ValidatorSet<Self>) -> ValidatorProposal<Self>;

	/// Complete an auction with a set of validators and accept this set and the bond for the next epoch
	fn complete_auction(proposal: &ValidatorProposal<Self>) -> Result<(), AuctionError>;
}
