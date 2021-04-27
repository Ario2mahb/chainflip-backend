use codec::Decode;

/// Should only be one of these in the final PR, this is to try them out
pub mod sc_observer;

// types for the pallets the client is reading things for

// I don't think we'll actually use this
pub mod transactions;

/// Staking pallet support for substrate-subxt
pub mod staking;

/// The state chain runtime client type definitions
pub mod runtime;

// Copied from the Subxt crate... not sure why I needed to copy it but I did
// TODO: Check this is actually required
pub trait Event<T>: Decode {
    /// Module name.
    const MODULE: &'static str;
    /// Event name.
    const EVENT: &'static str;
}
