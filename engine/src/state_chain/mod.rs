/// Reads events from state chain
pub mod sc_observer;

/// Submits events to state chain
pub mod sc_broadcaster;

/// The state chain runtime client type definitions
pub mod runtime;

mod sc_event;

/// Contains the pallets supported by the CFE
pub mod pallets;
