// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::convert::From;
use std::time::Instant;

use bitcoin_rpc::{MiningInfo, RpcError, TxOutSetInfo};

use crate::{
    bitcoin::RPC,
    common::{db, thread},
    util, STORE,
};

#[derive(Debug)]
pub enum Error {
    Db(db::Error),
    Rpc(RpcError),
}

pub struct Processor;

impl Processor {
    pub fn run() {
        thread::spawn("block_processor", {
            log::info!("Bitcoin Block Processor started");
            move || loop {
                let block_height: u32 = match RPC.get_block_count() {
                    Ok(data) => {
                        log::debug!("Current block is {}", data);
                        data
                    }
                    Err(error) => {
                        log::error!("Get block height: {:?}", error);
                        thread::sleep(60);
                        continue;
                    }
                };

                let last_processed_block: u32 = match STORE.get_last_processed_block() {
                    Ok(value) => value,
                    Err(_) => {
                        let _ = STORE.set_last_processed_block(block_height);
                        block_height
                    }
                };

                log::debug!("Last processed block is {}", last_processed_block);

                if block_height <= last_processed_block {
                    log::debug!("Wait for new block");
                    thread::sleep(120);
                    continue;
                }

                let next_block_to_process: u32 = last_processed_block + 1;
                let start = Instant::now();
                match Self::process_block(next_block_to_process) {
                    Ok(_) => {
                        let elapsed_time = start.elapsed().as_millis();
                        log::trace!(
                            "Block {} processed in {} ms",
                            next_block_to_process,
                            elapsed_time
                        );
                        let _ = STORE.set_last_processed_block(next_block_to_process);
                    }
                    Err(error) => {
                        log::error!("Process block: {:?} - retrying in 60 sec", error);
                        thread::sleep(60);
                    }
                };
            }
        });
    }

    fn process_block(block_height: u32) -> Result<(), Error> {
        Self::halving(block_height)?;
        Self::difficulty_adjustment(block_height)?;
        Self::supply(block_height)?;
        Self::hashrate()?;
        Ok(())
    }

    fn halving(block_height: u32) -> Result<(), Error> {
        if block_height % 210_000 == 0 {
            let current_block_reward: f64 = 50.0 / u32::pow(2, block_height / 210_000) as f64;

            let plain_text: String = format!(
                "🎉 The Halving has arrived ({}/32) 🎉",
                block_height / 210_000
            );
            Self::queue_notification(plain_text.clone(), plain_text)?;

            let plain_text: String =
                format!("New block reward: {} BTC", current_block_reward / 2.0);
            Self::queue_notification(plain_text.clone(), plain_text)?;
        } else {
            let missing_blocks: u32 = (block_height / 210_000 + 1) * 210_000 - block_height;

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

    fn difficulty_adjustment(block_height: u32) -> Result<(), Error> {
        if block_height % 2016 == 0 {
            let mining_info: MiningInfo = RPC.get_mining_info()?;

            let difficulty: f64 = mining_info.difficulty / u64::pow(10, 12) as f64;

            let last_difficulty: f64 = match STORE.get_last_difficulty() {
                Ok(value) => value,
                Err(_) => {
                    STORE.set_last_difficculty(difficulty)?;
                    difficulty
                }
            };

            let change: f64 = (difficulty - last_difficulty) / last_difficulty * 100.0;

            let plain_text: String =
                format!("⛏️ Difficulty adj: {:.2} T ({:.2}%) ⛏️", difficulty, change);

            Self::queue_notification(plain_text.clone(), plain_text)?;
            STORE.set_last_difficculty(difficulty)?;
        }

        Ok(())
    }

    fn supply(block_height: u32) -> Result<(), Error> {
        let current_halving: u32 = block_height / 210_000;
        let current_reward: f64 = 50.0 / u32::pow(2, current_halving) as f64;

        if block_height % 50_000 == 0 || STORE.get_last_supply().is_err() {
            let txoutset_info: TxOutSetInfo = RPC.get_txoutset_info()?;
            let mut total_supply = txoutset_info.total_amount;

            log::debug!(
                "txoutset_info.height: {} - block_height: {}",
                txoutset_info.height,
                block_height
            );

            if txoutset_info.height != block_height {
                total_supply = txoutset_info.total_amount
                    - (txoutset_info.height - block_height) as f64 * current_reward
            }

            let _ = STORE.set_last_supply(total_supply);
        } else if let Ok(last_supply) = STORE.get_last_supply() {
            let _ = STORE.set_last_supply(last_supply + current_reward);
        }

        if let Ok(last_supply) = STORE.get_last_supply() {
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

    fn hashrate() -> Result<(), Error> {
        let mining_info: MiningInfo = RPC.get_mining_info()?;
        let current_hashrate: f64 = mining_info.networkhashps / u64::pow(10, 18) as f64; // Hashrate in EH/s

        let last_hashrate_ath: f64 = match STORE.get_last_hashrate_ath() {
            Ok(value) => value,
            Err(_) => {
                STORE.set_last_hashrate_ath(current_hashrate)?;
                current_hashrate
            }
        };

        if current_hashrate > last_hashrate_ath {
            let plain_text: String =
                format!("🎉  New hashrate ATH: {:.2} EH/s 🎉", current_hashrate);

            Self::queue_notification(plain_text.clone(), plain_text)?;
            STORE.set_last_hashrate_ath(current_hashrate)?;
        }

        Ok(())
    }

    fn queue_notification(plain_text: String, html: String) -> Result<(), Error> {
        match STORE.create_notification(plain_text.as_str(), html.as_str()) {
            Ok(_) => log::info!("Queued a new notification"),
            Err(error) => log::error!("Impossible to queue notification: {:#?}", error),
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

impl From<db::Error> for Error {
    fn from(err: db::Error) -> Self {
        Error::Db(err)
    }
}

impl From<RpcError> for Error {
    fn from(err: RpcError) -> Self {
        Error::Rpc(err)
    }
}
