// Copyright (c) 2021-2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::sync::Arc;
use std::time::Duration;

use bitcoincore_rpc::json::{
    GetBlockchainInfoResult, GetMiningInfoResult, GetNetworkInfoResult, GetTxOutSetInfoResult,
    HashOrHeight,
};
use bitcoincore_rpc::{Auth, Client, RpcApi};
use tokio::{task, time};

use nostr_sdk::Result;

use crate::config::Config;

pub struct RpcClient {
    client: Arc<Client>,
}

impl RpcClient {
    pub fn new(config: &Config) -> Self {
        let url: String = format!("http://{}", config.bitcoin.rpc_addr);
        let auth: Auth = Auth::UserPass(
            config.bitcoin.rpc_username.clone(),
            config.bitcoin.rpc_password.clone(),
        );
        Self {
            client: Arc::new(Client::new(&url, auth).unwrap()),
        }
    }

    #[inline]
    async fn interact<F, R>(&self, f: F, timeout: Duration) -> Result<R>
    where
        F: FnOnce(Arc<Client>) -> R + Send + 'static,
        R: Send + 'static,
    {
        let client = self.client.clone();
        Ok(time::timeout(timeout, task::spawn_blocking(move || f(client))).await??)
    }

    #[inline]
    pub async fn get_blockchain_info(&self, timeout: Duration) -> Result<GetBlockchainInfoResult> {
        Ok(self
            .interact(move |client| client.get_blockchain_info(), timeout)
            .await??)
    }

    #[inline]
    pub async fn get_network_info(&self, timeout: Duration) -> Result<GetNetworkInfoResult> {
        Ok(self
            .interact(move |client| client.get_network_info(), timeout)
            .await??)
    }

    #[inline]
    pub async fn get_block_count(&self, timeout: Duration) -> Result<u64> {
        Ok(self
            .interact(move |client| client.get_block_count(), timeout)
            .await??)
    }

    #[inline]
    pub async fn get_mining_info(&self, timeout: Duration) -> Result<GetMiningInfoResult> {
        Ok(self
            .interact(move |client| client.get_mining_info(), timeout)
            .await??)
    }

    #[inline]
    pub async fn get_tx_out_set_info(
        &self,
        timeout: Duration,
        height: u64,
    ) -> Result<GetTxOutSetInfoResult> {
        let height = HashOrHeight::Height(height);
        Ok(self
            .interact(
                move |client| client.get_tx_out_set_info(None, Some(height), None),
                timeout,
            )
            .await??)
    }
}
