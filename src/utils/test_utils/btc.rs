use crate::{common::WalletAddress, vault::blockchain_connection::btc::*};
use bitcoin::Network;
use bitcoin::Transaction;
use bitcoin::Txid;
use btc_spv::{AddressUnspentResponse, BtcUTXO};
use std::{
    collections::{HashMap, VecDeque},
    sync::Mutex,
};

use std::sync::Arc;

type Blocks = VecDeque<Vec<Transaction>>;

// ======================= TEST BITCOIN CORE CLIENT ==============================
/// A bitcoin core client for testing
#[derive(Clone)]
pub struct TestBitcoinClient {
    blocks: Arc<Mutex<Blocks>>,
}

pub struct TestBitcoinSendClient {
    send_handler:
        Option<Box<dyn Fn(&SendTransaction) -> Result<Txid, String> + Send + Sync + 'static>>,
}

#[async_trait]
impl IBitcoinSend for TestBitcoinSendClient {
    async fn send(&self, tx: &SendTransaction) -> Result<Txid, String> {
        if let Some(function) = &self.send_handler {
            return function(tx);
        }
        Err("Not handled".to_owned())
    }
}

impl TestBitcoinSendClient {
    /// Create a new test bitcoin sender only client - for output processing
    pub fn new() -> Self {
        TestBitcoinSendClient { send_handler: None }
    }

    /// Set the handler for send
    pub fn set_send_handler<F>(&mut self, function: F)
    where
        F: 'static,
        F: Fn(&SendTransaction) -> Result<Txid, String> + Send + Sync,
    {
        self.send_handler = Some(Box::new(function));
    }
}

impl TestBitcoinClient {
    /// Create a new test bitcoin sender only client - for output processing
    pub fn new() -> Self {
        TestBitcoinClient {
            blocks: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
    /// Add a block to the client
    pub fn add_block(&self, transactions: Vec<Transaction>) {
        self.blocks.lock().unwrap().push_back(transactions)
    }
}

#[async_trait]
impl BitcoinClient for TestBitcoinClient {
    async fn get_latest_block_number(&self) -> Result<u64, String> {
        Ok(0)
    }

    async fn get_transactions(&self, _block_number: u64) -> Option<Vec<Transaction>> {
        self.blocks.lock().unwrap().pop_front()
    }

    fn get_network_type(&self) -> Network {
        Network::Testnet
    }
}

// ======================= TEST SPV CLIENT ==============================
/// An bitcoin SPV client for testing
pub struct TestBitcoinSPVClient {
    map_utxos: Mutex<HashMap<String, Vec<BtcUTXO>>>,
}

impl TestBitcoinSPVClient {
    /// Create a new test Bitcoin SPV client
    pub fn new() -> Self {
        TestBitcoinSPVClient {
            map_utxos: Mutex::new(HashMap::new()),
        }
    }

    /// Add a utxo to the client
    pub fn add_utxos_for_address(&self, address: String, utxos: Vec<BtcUTXO>) {
        self.map_utxos.lock().unwrap().insert(address, utxos);
    }
}

#[async_trait]
impl BitcoinSPVClient for TestBitcoinSPVClient {
    async fn get_address_unspent(
        &self,
        address: &WalletAddress,
    ) -> Result<AddressUnspentResponse, String> {
        let utxos_for_address = self
            .map_utxos
            .lock()
            .unwrap()
            .get(&address.to_string())
            .unwrap_or(&vec![])
            .clone();
        Ok(AddressUnspentResponse(utxos_for_address))
    }

    async fn get_estimated_fee(
        &self,
        send_tx: &SendTransaction,
        fee_method: btc_spv::FeeMethod,
        fee_level: u32,
    ) -> Result<u64, String> {
        Ok(1000)
    }
}
