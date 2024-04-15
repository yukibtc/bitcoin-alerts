// Copyright (c) 2021-2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::thread;
use std::time::Duration;

use bitcoin_rpc::{BlockchainInfo, Client, NetworkInfo};

mod processor;

use self::processor::Processor;
use crate::CONFIG;

lazy_static! {
    pub static ref RPC: Client = Client::new(
        &format!("http://{}", CONFIG.bitcoin.rpc_addr),
        &CONFIG.bitcoin.rpc_username,
        &CONFIG.bitcoin.rpc_password
    );
}

pub struct Bitcoin;

impl Bitcoin {
    pub fn run() {
        thread::spawn({
            move || {
                loop {
                    let blockchain_info: BlockchainInfo = match RPC.get_blockchain_info() {
                        Ok(data) => data,
                        Err(error) => {
                            tracing::error!(
                                "Get blockchain info: {:?} - retrying in 60 sec",
                                error
                            );
                            thread::sleep(Duration::from_secs(60));
                            continue;
                        }
                    };
                    let network_info: NetworkInfo = match RPC.get_network_info() {
                        Ok(data) => data,
                        Err(error) => {
                            tracing::error!("Get network info: {:?} - retrying in 60 sec", error);
                            thread::sleep(Duration::from_secs(60));
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
                    thread::sleep(Duration::from_secs(60));
                }

                Processor::run();
            }
        });
    }
}

impl Drop for Bitcoin {
    fn drop(&mut self) {
        if thread::panicking() {
            std::process::exit(0x1);
        }
    }
}
