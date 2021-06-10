// Implements support for the validator module

use std::marker::PhantomData;

use codec::{Decode, Encode};
use cf_traits::AuctionRange;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use substrate_subxt::{module, system::System, Event, sp_runtime::traits::Member};

use super::{runtime::StateChainRuntime, sc_event::SCEvent};

#[module]
pub trait Validator: System {
    type EpochIndex: Member + Encode + Decode + Serialize + DeserializeOwned;
}

#[derive(Clone, Debug, Eq, PartialEq, Event, Decode, Encode, Serialize, Deserialize)]
pub struct AuctionRangeChangedEvent<V: Validator> {
    pub before: AuctionRange,
    pub now: AuctionRange,
    pub _phantom: PhantomData<V>,
}

#[derive(Clone, Debug, Eq, PartialEq, Event, Decode, Encode, Serialize, Deserialize)]
pub struct EpochDurationChangedEvent<V: Validator> {
    pub from: V::BlockNumber,
    pub to: V::BlockNumber,
}

#[derive(Clone, Debug, Eq, PartialEq, Event, Decode, Encode, Serialize, Deserialize)]
pub struct AuctionStartedEvent<V: Validator> {
    pub epoch_index: V::EpochIndex,
}

#[derive(Clone, Debug, Eq, PartialEq, Event, Decode, Encode, Serialize, Deserialize)]
pub struct AuctionConfirmedEvent<V: Validator> {
    pub epoch_index: V::EpochIndex,
}

#[derive(Clone, Debug, Eq, PartialEq, Event, Decode, Encode, Serialize, Deserialize)]
pub struct ForceRotationRequestedEvent<V: Validator> {
    pub _phantom: PhantomData<V>,
}

#[derive(Clone, Debug, Eq, PartialEq, Event, Decode, Encode, Serialize, Deserialize)]
pub struct NewEpochEvent<V: Validator> {
    pub epoch_index: V::EpochIndex,
}

/// Wrapper for all Validator events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidatorEvent<V: Validator> {
    AuctionRangeChangedEvent(AuctionRangeChangedEvent<V>),

    EpochDurationChangedEvent(EpochDurationChangedEvent<V>),

    AuctionStartedEvent(AuctionStartedEvent<V>),

    AuctionConfirmedEvent(AuctionConfirmedEvent<V>),

    ForceAuctionRequestedEvent(ForceRotationRequestedEvent<V>),

    NewEpochEvent(NewEpochEvent<V>),
}

impl From<AuctionRangeChangedEvent<StateChainRuntime>> for SCEvent {
    fn from(auction_range_changed: AuctionRangeChangedEvent<StateChainRuntime>) -> Self {
        SCEvent::ValidatorEvent(ValidatorEvent::AuctionRangeChangedEvent(
            auction_range_changed,
        ))
    }
}

impl From<EpochDurationChangedEvent<StateChainRuntime>> for SCEvent {
    fn from(epoch_duration_changed: EpochDurationChangedEvent<StateChainRuntime>) -> Self {
        SCEvent::ValidatorEvent(ValidatorEvent::EpochDurationChangedEvent(
            epoch_duration_changed,
        ))
    }
}

impl From<AuctionStartedEvent<StateChainRuntime>> for SCEvent {
    fn from(auction_started: AuctionStartedEvent<StateChainRuntime>) -> Self {
        SCEvent::ValidatorEvent(ValidatorEvent::AuctionStartedEvent(auction_started))
    }
}

impl From<AuctionConfirmedEvent<StateChainRuntime>> for SCEvent {
    fn from(auction_ended: AuctionConfirmedEvent<StateChainRuntime>) -> Self {
        SCEvent::ValidatorEvent(ValidatorEvent::AuctionConfirmedEvent(auction_ended))
    }
}

impl From<ForceRotationRequestedEvent<StateChainRuntime>> for SCEvent {
    fn from(force_rotation_requested: ForceRotationRequestedEvent<StateChainRuntime>) -> Self {
        SCEvent::ValidatorEvent(ValidatorEvent::ForceAuctionRequestedEvent(
            force_rotation_requested,
        ))
    }
}

impl From<NewEpochEvent<StateChainRuntime>> for SCEvent {
    fn from(auction_ended: NewEpochEvent<StateChainRuntime>) -> Self {
        SCEvent::ValidatorEvent(ValidatorEvent::NewEpochEvent(auction_ended))
    }
}

#[cfg(test)]
mod tests {

    use pallet_cf_validator::Config;

    use codec::Encode;
    use state_chain_runtime::Runtime as SCRuntime;

    use crate::sc_observer::runtime::StateChainRuntime;

    use super::*;

    #[test]
    fn epoch_changed_decoding() {
        let event: <SCRuntime as Config>::Event =
            pallet_cf_validator::Event::<SCRuntime>::EpochDurationChanged(4, 10).into();

        let encoded_epoch = event.encode();
        // the first 2 bytes are (module_index, event_variant_index), these can be stripped
        let encoded_epoch = encoded_epoch[2..].to_vec();

        let decoded_event =
            EpochDurationChangedEvent::<StateChainRuntime>::decode(&mut &encoded_epoch[..])
                .unwrap();

        let expecting = EpochDurationChangedEvent { from: 4, to: 10 };

        assert_eq!(decoded_event, expecting);
    }

    #[test]
    fn auction_started_decoding() {
        // AuctionStarted(EpochIndex)
        let event: <SCRuntime as Config>::Event =
            pallet_cf_auction::Event::<SCRuntime>::AuctionStarted(1).into();

        let encoded_auction_started = event.encode();
        // the first 2 bytes are (module_index, event_variant_index), these can be stripped
        let encoded_auction_started = encoded_auction_started[2..].to_vec();

        let decoded_event =
            AuctionStartedEvent::<StateChainRuntime>::decode(&mut &encoded_auction_started[..])
                .unwrap();

        let expecting = AuctionStartedEvent {
            epoch_index: 1,
        };

        assert_eq!(decoded_event, expecting);
    }

    #[test]
    fn auction_confirmed_decoding() {
        // AuctionConfirmed(EpochIndex)
        let event: <SCRuntime as Config>::Event =
            pallet_cf_auction::Event::<SCRuntime>::AuctionConfirmed(1).into();

        let encoded_auction_confirmed = event.encode();
        // the first 2 bytes are (module_index, event_variant_index), these can be stripped
        let encoded_auction_confirmed = encoded_auction_confirmed[2..].to_vec();

        let decoded_event =
            AuctionConfirmedEvent::<StateChainRuntime>::decode(&mut &encoded_auction_confirmed[..])
                .unwrap();

        let expecting = AuctionConfirmedEvent {
            epoch_index: 1,
        };

        assert_eq!(decoded_event, expecting);
    }

    #[test]
    fn new_epoch_decoding() {
        // AuctionConfirmed(EpochIndex)
        let event: <SCRuntime as Config>::Event =
            pallet_cf_validator::Event::<SCRuntime>::NewEpoch(1).into();

        let encoded_new_epoch = event.encode();
        // the first 2 bytes are (module_index, event_variant_index), these can be stripped
        let encoded_new_epoch = encoded_new_epoch[2..].to_vec();

        let decoded_event =
            NewEpochEvent::<StateChainRuntime>::decode(&mut &encoded_new_epoch[..])
                .unwrap();

        let expecting = NewEpochEvent {
            epoch_index: 1,
        };

        assert_eq!(decoded_event, expecting);
    }

    #[test]
    fn auction_ranged_changed_decoding() {
        // AuctionRangeChanged(AuctionRange, AuctionRange)
        let event: <SCRuntime as Config>::Event =
            pallet_cf_auction::Event::<SCRuntime>::AuctionRangeChanged((0, 1), (0, 2)).into();

        let encoded_auction_range_changed = event.encode();
        // the first 2 bytes are (module_index, event_variant_index), these can be stripped
        let encoded_auction_range_changed = encoded_auction_range_changed[2..].to_vec();

        let decoded_event = AuctionRangeChangedEvent::<StateChainRuntime>::decode(
            &mut &encoded_auction_range_changed[..],
        )
        .unwrap();

        let expecting = AuctionRangeChangedEvent {
            before: (0, 1),
            now: (0, 2),
            _phantom: PhantomData,
        };

        assert_eq!(decoded_event, expecting);
    }

    #[test]
    fn force_rotation_requested_decoding() {
        let event: <SCRuntime as Config>::Event =
            pallet_cf_validator::Event::<SCRuntime>::ForceRotationRequested().into();

        let encodeded_force_rotation = event.encode();
        // the first 2 bytes are (module_index, event_variant_index), these can be stripped
        let encodeded_force_rotation = encodeded_force_rotation[2..].to_vec();

        let decoded_event = ForceRotationRequestedEvent::<StateChainRuntime>::decode(
            &mut &encodeded_force_rotation[..],
        )
        .unwrap();

        let expecting = ForceRotationRequestedEvent {
            _phantom: PhantomData,
        };

        assert_eq!(decoded_event, expecting);
    }
}
