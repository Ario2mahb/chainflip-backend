use crate::{
    common::{Liquidity, LiquidityProvider, PoolCoin},
    local_store::SideChainBlock,
};
use chainflip_common::types::{chain::*, UUIDv4};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use vault_node::VaultNodeInterface;

mod api;
mod block_poller;

use api::API;

/// The quoter database
pub mod database;

/// The vault node api consumer
pub mod vault_node;

/// The config
pub mod config;

/// Test utils
pub mod test_utils;

/// Quoter
pub struct Quoter {}

impl Quoter {
    /// Run the Quoter logic.
    ///
    /// # Blocking
    ///
    /// This will block the thread it is run on.
    pub fn run<V, D>(
        addr: impl Into<SocketAddr>,
        vault_node_api: Arc<V>,
        database: Arc<Mutex<D>>,
    ) -> Result<(), String>
    where
        V: VaultNodeInterface + Send + Sync + 'static,
        D: BlockProcessor + StateProvider + Send + 'static,
    {
        let poller = block_poller::BlockPoller::new(vault_node_api.clone(), database.clone());
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(poller.sync())?;

        // Start loops
        let poller_thread =
            std::thread::spawn(move || poller.poll(std::time::Duration::from_secs(1)));

        API::serve(addr, vault_node_api.clone(), database.clone());

        poller_thread
            .join()
            .map_err(|_| "An error occurred while polling".to_owned())?;

        Ok(())
    }
}

/// A trait for processing side chain blocks received from the vault node.
pub trait BlockProcessor {
    /// Get the block number that was last processed.
    fn get_last_processed_block_number(&self) -> Option<u32>;

    /// Process a list of blocks
    fn process_blocks(&mut self, blocks: &[SideChainBlock]) -> Result<(), String>;
}
/// A trait for providing quoter state
pub trait StateProvider: LiquidityProvider {
    /// Get all swap quotes
    fn get_swap_quotes(&self) -> Vec<SwapQuote>;
    /// Get swap quote with the given id
    fn get_swap_quote(&self, id: UUIDv4) -> Option<SwapQuote>;
    /// Get all deposit quotes
    fn get_deposit_quotes(&self) -> Vec<DepositQuote>;
    /// Get deposit quote with the given id
    fn get_deposit_quote(&self, id: UUIDv4) -> Option<DepositQuote>;
    /// Get all witnesses
    fn get_witnesses(&self) -> Vec<Witness>;
    /// Get all outputs
    fn get_outputs(&self) -> Vec<Output>;
    /// Get all output sents
    fn get_output_sents(&self) -> Vec<OutputSent>;
    /// Get all deposits
    fn get_deposits(&self) -> Vec<Deposit>;
    /// Get all withdraw requests
    fn get_withdraw_requests(&self) -> Vec<WithdrawRequest>;
    /// Get all withdraws
    fn get_withdraws(&self) -> Vec<Withdraw>;
    /// Get the pools
    fn get_pools(&self) -> HashMap<PoolCoin, Liquidity>;
}
