use std::{collections::HashMap, fmt::Display, time::Duration};

use crate::{
    logging::COMPONENT_KEY,
    p2p::{AccountId, P2PMessage, P2PMessageCommand},
    signing::{
        client::{KeyId, MultisigInstruction, SigningInfo},
        crypto::{
            BigInt, ECPoint, ECScalar, KeyGenBroadcastMessage1, LegacySignature, VerifiableSS, FE,
            GE,
        },
        db::KeyDB,
        MessageHash, MessageInfo,
    },
};

use slog::o;
use tokio::sync::mpsc::UnboundedSender;

use super::{
    common::KeygenResultInfo, frost::SigningDataWrapped, key_store::KeyStore,
    keygen_manager::KeygenManager, signing_manager::SigningManager,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchnorrSignature {
    /// Scalar component
    pub s: [u8; 32],
    /// Point component (commitment)
    pub r: secp256k1::PublicKey,
}

impl From<LegacySignature> for SchnorrSignature {
    fn from(sig: LegacySignature) -> Self {
        let s: [u8; 32] = sig.sigma.get_element().as_ref().clone();
        let r = sig.v.get_element();
        SchnorrSignature { s, r }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MultisigMessage {
    KeyGenMessage(KeyGenMessageWrapped),
    SigningMessage(SigningDataWrapped),
}

use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Broadcast1 {
    pub bc1: KeyGenBroadcastMessage1,
    pub blind: BigInt,
    pub y_i: GE,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Secret2 {
    pub vss: VerifiableSS<GE>,
    pub secret_share: FE,
}

impl From<Secret2> for KeygenData {
    fn from(sec2: Secret2) -> Self {
        KeygenData::Secret2(sec2)
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KeyGenMessageWrapped {
    pub key_id: KeyId,
    pub message: KeygenData,
}

impl KeyGenMessageWrapped {
    pub fn new<M>(key_id: KeyId, m: M) -> Self
    where
        M: Into<KeygenData>,
    {
        KeyGenMessageWrapped {
            key_id,
            message: m.into(),
        }
    }
}

impl From<KeyGenMessageWrapped> for MultisigMessage {
    fn from(wrapped: KeyGenMessageWrapped) -> Self {
        MultisigMessage::KeyGenMessage(wrapped)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum KeygenData {
    Broadcast1(Broadcast1),
    Secret2(Secret2),
}

impl From<Broadcast1> for KeygenData {
    fn from(bc1: Broadcast1) -> Self {
        KeygenData::Broadcast1(bc1)
    }
}

impl Display for KeygenData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            KeygenData::Broadcast1(_) => write!(f, "Keygen::Broadcast"),
            KeygenData::Secret2(_) => write!(f, "Keygen::Secret"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Error {
    Unauthorised,
    Timeout,
    Invalid,
}

pub type CeremonyOutcomeResult<Output> = Result<Output, (Error, Vec<AccountId>)>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CeremonyOutcome<Id, Output> {
    pub ceremony_id: Id,
    pub result: CeremonyOutcomeResult<Output>,
}
impl<Id, Output> CeremonyOutcome<Id, Output> {
    pub fn success(ceremony_id: Id, output: Output) -> Self {
        Self {
            ceremony_id,
            result: Ok(output),
        }
    }
    pub fn unauthorised(ceremony_id: Id, bad_validators: Vec<AccountId>) -> Self {
        Self {
            ceremony_id,
            result: Err((Error::Unauthorised, bad_validators)),
        }
    }
    pub fn timeout(ceremony_id: Id, bad_validators: Vec<AccountId>) -> Self {
        Self {
            ceremony_id,
            result: Err((Error::Timeout, bad_validators)),
        }
    }
    pub fn invalid(ceremony_id: Id, bad_validators: Vec<AccountId>) -> Self {
        Self {
            ceremony_id,
            result: Err((Error::Invalid, bad_validators)),
        }
    }
}

/// The final result of a keygen ceremony
pub type KeygenOutcome = CeremonyOutcome<KeyId, secp256k1::PublicKey>;
/// The final result of a Signing ceremony
pub type SigningOutcome = CeremonyOutcome<MessageInfo, SchnorrSignature>;

#[derive(Debug, PartialEq)]
pub enum InnerEvent {
    P2PMessageCommand(P2PMessageCommand),
    SigningResult(SigningOutcome),
    KeygenResult(KeygenOutcome),
}

#[derive(Clone)]
pub struct MultisigClient<S>
where
    S: KeyDB,
{
    my_account_id: AccountId,
    key_store: KeyStore<S>,
    keygen: KeygenManager,
    pub signing_manager: SigningManager,
    event_sender: UnboundedSender<InnerEvent>,
    /// Requests awaiting a key
    pending_requests_to_sign: HashMap<KeyId, Vec<(MessageHash, SigningInfo)>>,
    logger: slog::Logger,
}

impl<S> MultisigClient<S>
where
    S: KeyDB,
{
    pub fn new(
        my_account_id: AccountId,
        db: S,
        event_sender: UnboundedSender<InnerEvent>,
        phase_timeout: Duration,
        logger: &slog::Logger,
    ) -> Self {
        MultisigClient {
            my_account_id: my_account_id.clone(),
            key_store: KeyStore::new(db),
            keygen: KeygenManager::new(
                my_account_id.clone(),
                event_sender.clone(),
                phase_timeout.clone(),
                logger,
            ),
            signing_manager: SigningManager::new(my_account_id, event_sender.clone(), logger),
            pending_requests_to_sign: Default::default(),
            event_sender,
            logger: logger.new(o!(COMPONENT_KEY => "MultisigClient")),
        }
    }

    #[cfg(test)]
    pub fn get_keygen(&self) -> &KeygenManager {
        &self.keygen
    }

    #[cfg(test)]
    pub fn get_key(&self, key_id: KeyId) -> Option<&KeygenResultInfo> {
        self.key_store.get_key(key_id)
    }

    #[cfg(test)]
    pub fn get_db(&self) -> &S {
        self.key_store.get_db()
    }

    #[cfg(test)]
    pub fn get_my_account_id(&self) -> AccountId {
        self.my_account_id.clone()
    }

    /// Change the time we wait until deleting all unresolved states
    #[cfg(test)]
    pub fn set_timeout(&mut self, phase_timeout: Duration) {
        self.keygen.set_timeout(phase_timeout);
        self.signing_manager.set_timeout(phase_timeout);
    }

    /// Clean up expired states
    pub fn cleanup(&mut self) {
        self.keygen.cleanup();
        self.signing_manager.cleanup();
    }

    fn add_pending(&mut self, data: MessageHash, sign_info: SigningInfo) {
        slog::debug!(
            self.logger,
            "[{}] Delaying a request to sign",
            self.my_account_id
        );

        // TODO: check for duplicates?

        let entry = self
            .pending_requests_to_sign
            .entry(sign_info.id)
            .or_default();

        entry.push((data, sign_info));
    }

    /// Process `instruction` issued internally (i.e. from SC or another local module)
    pub fn process_multisig_instruction(&mut self, instruction: MultisigInstruction) {
        match instruction {
            MultisigInstruction::KeyGen(keygen_info) => {
                // For now disable generating a new key when we already have one

                // TODO: print ceremony id
                slog::debug!(
                    self.logger,
                    "[{}] Received keygen instruction",
                    self.my_account_id
                );

                self.keygen.on_keygen_request(keygen_info);
            }
            MultisigInstruction::Sign(hash, sign_info) => {
                // TODO: print ceremony id
                slog::debug!(
                    self.logger,
                    "[{}] Received a request to sign",
                    self.my_account_id
                );
                let key_id = sign_info.id;

                match self.key_store.get_key(key_id) {
                    Some(key) => {
                        self.signing_manager
                            .on_request_to_sign(hash, key.clone(), sign_info);
                    }
                    None => {
                        // The key is not ready, delay until either it is ready or timeout
                        self.add_pending(hash, sign_info);
                    }
                }
            }
        }
    }

    /// Process requests to sign that required `key_id`
    fn process_pending(&mut self, key_id: KeyId, key_info: KeygenResultInfo) {
        if let Some(reqs) = self.pending_requests_to_sign.remove(&key_id) {
            slog::debug!(
                self.logger,
                "Processing pending requests to sign, count: {}",
                reqs.len()
            );
            for (data, info) in reqs {
                self.signing_manager
                    .on_request_to_sign(data, key_info.clone(), info)
            }
        }
    }

    fn on_key_generated(&mut self, key_id: KeyId, key_info: KeygenResultInfo) {
        self.key_store.set_key(key_id, key_info.clone());
        self.process_pending(key_id, key_info.clone());

        // NOTE: we only notify the SC after we have successfully saved the key

        if let Err(err) = self
            .event_sender
            .send(InnerEvent::KeygenResult(KeygenOutcome::success(
                key_id,
                key_info.key.get_public_key().get_element(),
            )))
        {
            slog::error!(
                self.logger,
                "Could not sent KeygenOutcome::Success: {}",
                err
            );
        }
    }

    /// Process message from another validator
    pub fn process_p2p_mq_message(&mut self, msg: P2PMessage) {
        let P2PMessage { sender_id, data } = msg;
        let msg: Result<MultisigMessage, _> = bincode::deserialize(&data);

        match msg {
            Ok(MultisigMessage::KeyGenMessage(msg)) => {
                // NOTE: we should be able to process Keygen messages
                // even when we are "signing"... (for example, if we want to
                // generate a new key)

                let key_id = msg.key_id;

                if let Some(key) = self.keygen.process_keygen_message(sender_id, msg) {
                    self.on_key_generated(key_id, key);
                    // NOTE: we could already delete the state here, but it is
                    // not necessary as it will be deleted by "cleanup"
                }
            }
            Ok(MultisigMessage::SigningMessage(msg)) => {
                // NOTE: we should be able to process Signing messages
                // even when we are generating a new key (for example,
                // we should be able to receive phase1 messages before we've
                // finalized the signing key locally)
                self.signing_manager.process_signing_data(sender_id, msg);
            }
            Err(_) => {
                slog::warn!(self.logger, "Cannot parse multisig message, discarding");
            }
        }
    }
}
