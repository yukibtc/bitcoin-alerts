// Copyright (c) 2021-2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::Duration;

use tokio::time;

mod constants;
mod processor;
mod rpc;

use self::constants::DEFATL_RPC_TIMEOUT;
use self::processor::Processor;
pub use self::rpc::RpcClient;
use crate::config::Config;
use crate::db::{BitcoinStore, NotificationStore};

pub async fn run(
    config: Config,
    rpc: RpcClient,
    bitcoin_store: BitcoinStore,
    notification_store: NotificationStore,
) {
    loop {
        let blockchain_info = match rpc.get_blockchain_info(DEFATL_RPC_TIMEOUT).await {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("Get blockchain info: {e} - retrying in 60 sec");
                time::sleep(Duration::from_secs(60)).await;
                continue;
            }
        };

        let network_info = match rpc.get_network_info(DEFATL_RPC_TIMEOUT).await {
            Ok(data) => data,
            Err(error) => {
                tracing::error!("Get network info: {:?} - retrying in 60 sec", error);
                time::sleep(Duration::from_secs(60)).await;
                continue;
            }
        };

        if network_info.version < 22_00_00 {
            tracing::error!("This application requires Bitcoin Core 22.0+");
            panic!("Bitcoin Core version incompatible");
        }

        if !network_info.network_active {
            tracing::error!("This application requires active Bitcoin P2P network.");
            panic!("P2P network not enabled");
        }

        let left_blocks: u64 = blockchain_info.headers - blockchain_info.blocks;

        if left_blocks == 0 {
            break;
        }

        tracing::info!(
            "Waiting to download {} blocks{}",
            left_blocks,
            if blockchain_info.initial_block_download {
                " (IBD)"
            } else {
                ""
            }
        );

        time::sleep(Duration::from_secs(60)).await;
    }

    Processor::new(config, rpc, bitcoin_store, notification_store)
        .run()
        .await;
}
