// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use bitcoin_rpc::{BlockchainInfo, Client, NetworkInfo};
use bpns_common::thread;

mod processor;

use processor::Processor;

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
        thread::spawn("bitcoin", {
            move || {
                loop {
                    let blockchain_info: BlockchainInfo = match RPC.get_blockchain_info() {
                        Ok(data) => data,
                        Err(error) => {
                            log::error!("Get blockchain info: {:?} - retrying in 60 sec", error);
                            thread::sleep(60);
                            continue;
                        }
                    };
                    let network_info: NetworkInfo = match RPC.get_network_info() {
                        Ok(data) => data,
                        Err(error) => {
                            log::error!("Get network info: {:?} - retrying in 60 sec", error);
                            thread::sleep(60);
                            continue;
                        }
                    };

                    if network_info.version < 22_00_00 {
                        log::error!("This application requires Bitcoin Core 22.0+");
                        panic!("Bitcoin Core version incompatible");
                    }

                    if !network_info.network_active {
                        log::error!("This application requires active Bitcoin P2P network.");
                        panic!("P2P network not enabled");
                    }

                    let left_blocks: u64 = blockchain_info.headers - blockchain_info.blocks;

                    if left_blocks == 0 {
                        break;
                    }

                    log::info!(
                        "Waiting to download {} blocks{}",
                        left_blocks,
                        if blockchain_info.initial_block_download {
                            " (IBD)"
                        } else {
                            ""
                        }
                    );
                    thread::sleep(120);
                }

                Processor::run();

                Ok(())
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
