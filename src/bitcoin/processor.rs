// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::convert::From;
use std::time::Instant;

use bitcoin::network::constants::Network;
use bitcoincore_rpc::bitcoincore_rpc_json::{GetMiningInfoResult, GetTxOutSetInfoResult};
use bitcoincore_rpc::RpcApi;
use bpns_common::thread;

use crate::bitcoin::RPC;
use crate::primitives::Target;
use crate::{util, BITCOIN_STORE, CONFIG, NOTIFICATION_STORE};

#[derive(Debug)]
pub enum Error {
    Db(bpns_rocksdb::Error),
    Rpc(bitcoincore_rpc::Error),
}

pub struct Processor;

impl Processor {
    pub fn run() {
        thread::spawn("block_processor", {
            log::info!("Bitcoin Block Processor started");

            let mut delay = 30; // Delay seconds

            move || loop {
                let block_height: u64 = match RPC.get_block_count() {
                    Ok(n) => {
                        log::debug!("Current block is {}", n);
                        n
                    }
                    Err(err) => {
                        log::error!("Get block height: {:?}", err);
                        thread::sleep(120);
                        continue;
                    }
                };

                let last_processed_block: u64 = match BITCOIN_STORE.get_last_processed_block() {
                    Ok(value) => value,
                    Err(_) => {
                        let _ = BITCOIN_STORE.set_last_processed_block(block_height);
                        block_height
                    }
                };

                log::debug!("Last processed block is {}", last_processed_block);

                if block_height <= last_processed_block {
                    log::debug!("Wait for new block");
                    thread::sleep(120);
                    continue;
                }

                let next_block_to_process: u64 = last_processed_block + 1;
                let start = Instant::now();
                match Self::process_block(next_block_to_process) {
                    Ok(_) => {
                        delay = 30;

                        let elapsed_time = start.elapsed().as_millis();
                        log::trace!(
                            "Block {} processed in {} ms",
                            next_block_to_process,
                            elapsed_time
                        );
                        let _ = BITCOIN_STORE.set_last_processed_block(next_block_to_process);
                    }
                    Err(err) => {
                        if delay > 3600 {
                            log::error!("Impossible to process block: {:?}", err);
                            std::process::exit(0x1);
                        }

                        log::error!("Process block: {:?} - retrying in {} sec", err, delay);

                        thread::sleep(delay);

                        delay *= 2;
                    }
                };
            }
        });
    }

    fn process_block(block_height: u64) -> Result<(), Error> {
        let mining_info: GetMiningInfoResult = RPC.get_mining_info()?;

        Self::halving(block_height)?;
        Self::difficulty_adjustment(block_height, &mining_info)?;
        Self::supply(block_height)?;
        Self::hashrate(&mining_info)?;
        Self::block(block_height)?;
        Ok(())
    }

    fn halving(block_height: u64) -> Result<(), Error> {
        if block_height % 210_000 == 0 {
            let current_block_reward: f64 = 50.0 / f64::powf(2.0, (block_height / 210_000) as f64);

            let plain_text: String = format!(
                "🎉 The Halving has arrived ({}/32) 🎉",
                block_height / 210_000
            );
            Self::queue_notification(plain_text.clone(), plain_text)?;

            let plain_text: String =
                format!("New block reward: {} BTC", current_block_reward / 2.0);
            Self::queue_notification(plain_text.clone(), plain_text)?;
        } else {
            let missing_blocks: u64 = (block_height / 210_000 + 1) * 210_000 - block_height;

            if missing_blocks <= 144
                || (missing_blocks <= 4320 && missing_blocks % 144 == 0)
                || (missing_blocks <= 51840 && missing_blocks % 4320 == 0)
                || missing_blocks == 105000
                || block_height % (6 * 24 * 30 * 2) == 0
            {
                let plain_text: String = format!(
                    "🔥 {} blocks to the next Halving 🔥",
                    util::format_number(missing_blocks as usize)
                );

                Self::queue_notification(plain_text.clone(), plain_text)?;
            }
        }

        Ok(())
    }

    fn difficulty_adjustment(block_height: u64, mining_info: &GetMiningInfoResult) -> Result<(), Error> {
        if block_height % 2016 == 0 {
            let difficulty: f64 = mining_info.difficulty / u64::pow(10, 12) as f64;

            let last_difficulty: f64 = match BITCOIN_STORE.get_last_difficulty() {
                Ok(value) => value,
                Err(_) => {
                    BITCOIN_STORE.set_last_difficculty(difficulty)?;
                    difficulty
                }
            };

            let change: f64 = (difficulty - last_difficulty) / last_difficulty * 100.0;

            let plain_text: String =
                format!("⛏️ Difficulty adj: {:.2} T ({:.2}%) ⛏️", difficulty, change);

            Self::queue_notification(plain_text.clone(), plain_text)?;
            BITCOIN_STORE.set_last_difficculty(difficulty)?;
        }

        Ok(())
    }

    fn supply(block_height: u64) -> Result<(), Error> {
        let current_halving: u64 = block_height / 210_000;
        let current_reward: f64 = 50.0 / f64::powf(2.0, current_halving as f64);

        if block_height % 50_000 == 0 || BITCOIN_STORE.get_last_supply().is_err() {
            let txoutset_info: GetTxOutSetInfoResult = RPC.get_tx_out_set_info(None, None, None)?;
            let mut total_supply: f64 = txoutset_info.total_amount.to_btc();

            log::debug!(
                "txoutset_info.height: {} - block_height: {}",
                txoutset_info.height,
                block_height
            );

            if txoutset_info.height != block_height {
                total_supply -= (txoutset_info.height - block_height) as f64 * current_reward;
            }

            let _ = BITCOIN_STORE.set_last_supply(total_supply);
        } else if let Ok(last_supply) = BITCOIN_STORE.get_last_supply() {
            let _ = BITCOIN_STORE.set_last_supply(last_supply + current_reward);
        }

        if let Ok(last_supply) = BITCOIN_STORE.get_last_supply() {
            log::debug!("Total supply: {} BTC", last_supply);

            if last_supply >= 19_000_000.0 && last_supply - current_reward < 19_000_000.0 {
                let plain_text: String = format!(
                    "🎊 The supply has just reached {} BTC 🎊",
                    util::format_number(19_000_000)
                );

                Self::queue_notification(plain_text.clone(), plain_text)?;
            }
        }

        Ok(())
    }

    fn hashrate(mining_info: &GetMiningInfoResult) -> Result<(), Error> {
        let current_hashrate: f64 = mining_info.network_hash_ps / u64::pow(10, 18) as f64; // Hashrate in EH/s

        let last_hashrate_ath: f64 = match BITCOIN_STORE.get_last_hashrate_ath() {
            Ok(value) => value,
            Err(_) => {
                BITCOIN_STORE.set_last_hashrate_ath(current_hashrate)?;
                current_hashrate
            }
        };

        if current_hashrate > last_hashrate_ath {
            let plain_text: String =
                format!("🎉  New hashrate ATH: {:.2} EH/s 🎉", current_hashrate);

            Self::queue_notification(plain_text.clone(), plain_text)?;
            BITCOIN_STORE.set_last_hashrate_ath(current_hashrate)?;
        }

        Ok(())
    }

    fn block(block_height: u64) -> Result<(), Error> {
        let block_alerts: Vec<u64> = vec![800_000, 900_000, 1_000_000];

        if block_alerts.contains(&block_height) {
            let plain_text: String = format!(
                "⛓️ Reached block {} ⛓️",
                util::format_number(block_height as usize)
            );
            Self::queue_notification(plain_text.clone(), plain_text)?;
        }

        if CONFIG.bitcoin.network == Network::Regtest {
            let plain_text: String = format!(
                "⛓️ Reached block {} ⛓️",
                util::format_number(block_height as usize)
            );
            Self::queue_notification(plain_text.clone(), plain_text)?;
        }

        Ok(())
    }

    fn queue_notification(plain_text: String, html: String) -> Result<(), Error> {
        if CONFIG.ntfy.enabled {
            Self::queue_notification_with_target(Target::Ntfy, plain_text.clone(), html.clone())?;
        }

        if CONFIG.matrix.enabled {
            Self::queue_notification_with_target(Target::Matrix, plain_text, html)?;
        }

        Ok(())
    }

    fn queue_notification_with_target(
        target: Target,
        plain_text: String,
        html: String,
    ) -> Result<(), Error> {
        match NOTIFICATION_STORE.create_notification(
            target.clone(),
            plain_text.as_str(),
            html.as_str(),
        ) {
            Ok(_) => log::info!("Queued a new notification for {}", target),
            Err(err) => log::error!("Impossible to queue notification for {}: {:?}", target, err),
        };

        Ok(())
    }
}

impl Drop for Processor {
    fn drop(&mut self) {
        if thread::panicking() {
            std::process::exit(0x1);
        }
    }
}

impl From<bpns_rocksdb::Error> for Error {
    fn from(err: bpns_rocksdb::Error) -> Self {
        Error::Db(err)
    }
}

impl From<bitcoincore_rpc::Error> for Error {
    fn from(err: bitcoincore_rpc::Error) -> Self {
        Error::Rpc(err)
    }
}
