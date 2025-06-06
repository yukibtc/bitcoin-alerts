// Copyright (c) 2021-2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::{Duration, Instant};

use bitcoin::network::Network;
use bitcoincore_rpc::json::GetMiningInfoResult;
use nostr_sdk::Result;
use tokio::time;

use super::constants::{BLOCK_ALERTS, DEFATL_RPC_TIMEOUT};
use super::rpc::RpcClient;
use crate::config::Config;
use crate::db::{BitcoinStore, NotificationStore};
use crate::primitives::Target;
use crate::util;

pub struct Processor {
    config: Config,
    rpc: RpcClient,
    bitcoin_store: BitcoinStore,
    notification_store: NotificationStore,
}

impl Processor {
    pub fn new(
        config: Config,
        rpc: RpcClient,
        bitcoin_store: BitcoinStore,
        notification_store: NotificationStore,
    ) -> Self {
        Self {
            config,
            rpc,
            bitcoin_store,
            notification_store,
        }
    }

    pub async fn run(&self) {
        tracing::info!("Bitcoin Processor started");

        let mut delay = 30; // Delay seconds

        loop {
            let block_height: u64 = match self.rpc.get_block_count(DEFATL_RPC_TIMEOUT).await {
                Ok(height) => {
                    tracing::debug!("Current block is {height}");
                    height
                }
                Err(e) => {
                    tracing::error!("Get block height: {e}");
                    time::sleep(Duration::from_secs(60)).await;
                    continue;
                }
            };

            let last_processed_block: u64 = match self.bitcoin_store.get_last_processed_block() {
                Ok(value) => value,
                Err(_) => {
                    let _ = self.bitcoin_store.set_last_processed_block(block_height);
                    block_height
                }
            };

            tracing::debug!("Last processed block is {}", last_processed_block);

            if block_height <= last_processed_block {
                tracing::debug!("Wait for new block");
                time::sleep(Duration::from_secs(60)).await;
                continue;
            }

            let next_block_to_process: u64 = last_processed_block + 1;
            let start = Instant::now();
            match self.process_block(next_block_to_process).await {
                Ok(_) => {
                    delay = 30;

                    let elapsed_time = start.elapsed().as_millis();
                    tracing::trace!(
                        "Block {} processed in {} ms",
                        next_block_to_process,
                        elapsed_time
                    );
                    let _ = self
                        .bitcoin_store
                        .set_last_processed_block(next_block_to_process);
                }
                Err(e) => {
                    if delay > 3600 {
                        tracing::error!("Impossible to process block: {e}");
                        std::process::exit(0x1);
                    }

                    tracing::error!("Process block: {e} - retrying in {delay} secs");

                    time::sleep(Duration::from_secs(delay)).await;

                    delay *= 2;
                }
            };
        }
    }

    async fn process_block(&self, block_height: u64) -> Result<()> {
        let mining_info = self.rpc.get_mining_info(DEFATL_RPC_TIMEOUT).await?;

        self.halving(block_height)?;
        self.difficulty_adjustment(block_height, &mining_info)?;
        //self.supply(block_height).await?;
        self.hashrate(&mining_info)?;
        self.block(block_height)?;
        Ok(())
    }

    fn halving(&self, block_height: u64) -> Result<()> {
        if block_height % 210_000 == 0 {
            let halving: u64 = block_height / 210_000;

            if halving <= 32 {
                // Calc block reward
                let block_reward: f64 = 50.0 / f64::powf(2.0, (block_height / 210_000) as f64);

                // Calc epoch
                let epoch: u64 = halving + 1;

                // Send halving notification
                let plain_text: String = format!(
                    "⛏️ The Halving is here! Welcome to the {epoch}{} epoch! ⛏️",
                    match epoch {
                        1 | 21 | 31 => "st",
                        2 | 22 | 32 => "nd",
                        3 | 23 | 33 => "rd",
                        _ => "th",
                    }
                );
                self.queue_notification(plain_text)?;

                // Send block reward notification
                let plain_text: String = format!("⛏️ New block reward: {block_reward:.2} BTC ⛏️");
                self.queue_notification(plain_text)?;
            } else {
                tracing::warn!("Halving > 32 ({halving})");
            }
        } else {
            let missing_blocks: u64 = (block_height / 210_000 + 1) * 210_000 - block_height;

            if missing_blocks <= 144 * 7 // Less that a week left, notify every block
                || (missing_blocks <= 4320 && missing_blocks % 6 == 0) // Less than 1 month left, notify every hour
                || missing_blocks == 4320 // One month left
                || (missing_blocks <= 8640 && missing_blocks % 144 == 0) // Less than 2 months left, notify every day
                || missing_blocks == 8640 // 2 months left
                || (missing_blocks <= 51840 && missing_blocks % 432 == 0) // Less than one year left, notify every 3 days
                || missing_blocks == 51840 // One year left
                || (missing_blocks <= 103680 && missing_blocks % 1008 == 0) // Less than two years left, notify every week
                || missing_blocks == 105000 // Two years left
                || (missing_blocks <= 1555520 && missing_blocks % 2016 == 0) // Less than three years left, notify two weeks
                || missing_blocks == 1555520 // Three years left
                || block_height % (6 * 24 * 30 * 3) == 0
            {
                let plain_text: String = format!(
                    "🔥 {} blocks to the next Halving 🔥",
                    util::format_number(missing_blocks as usize)
                );

                self.queue_notification(plain_text)?;
            }
        }

        Ok(())
    }

    fn difficulty_adjustment(
        &self,
        block_height: u64,
        mining_info: &GetMiningInfoResult,
    ) -> Result<()> {
        if block_height % 2016 == 0 {
            let difficulty: f64 = mining_info.difficulty / u64::pow(10, 12) as f64;

            let last_difficulty: f64 = match self.bitcoin_store.get_last_difficulty() {
                Ok(value) => value,
                Err(_) => {
                    self.bitcoin_store.set_last_difficculty(difficulty)?;
                    difficulty
                }
            };

            let change: f64 = (difficulty - last_difficulty) / last_difficulty * 100.0;

            let plain_text: String =
                format!("⛏️ Difficulty adj: {difficulty:.2}T ({change:.2}%) ⛏️");

            self.queue_notification(plain_text)?;
            self.bitcoin_store.set_last_difficculty(difficulty)?;
        }

        Ok(())
    }

    // async fn supply(&self, block_height: u64) -> Result<()> {
    //     let current_halving: u64 = block_height / 210_000;
    //     let current_reward: f64 = 50.0 / f64::powf(2.0, current_halving as f64);
    //
    //     if block_height % 50_000 == 0 || self.bitcoin_store.get_last_supply().is_err() {
    //         let txoutset_info = self
    //             .rpc
    //             .get_tx_out_set_info(Duration::from_secs(240))
    //             .await?;
    //         let mut total_supply: f64 = txoutset_info.total_amount.to_btc();
    //
    //         tracing::debug!(
    //             "txoutset_info.height: {} - block_height: {}",
    //             txoutset_info.height,
    //             block_height
    //         );
    //
    //         if txoutset_info.height != block_height {
    //             total_supply -= (txoutset_info.height - block_height) as f64 * current_reward;
    //         }
    //
    //         let _ = self.bitcoin_store.set_last_supply(total_supply);
    //     } else if let Ok(last_supply) = self.bitcoin_store.get_last_supply() {
    //         let _ = self
    //             .bitcoin_store
    //             .set_last_supply(last_supply + current_reward);
    //     }
    //
    //     if let Ok(last_supply) = self.bitcoin_store.get_last_supply() {
    //         tracing::debug!("Total supply: {} BTC", last_supply);
    //
    //         for supply_alert in SUPPLY_ALERTS.iter() {
    //             if last_supply >= *supply_alert && last_supply - current_reward < *supply_alert {
    //                 let plain_text: String = format!(
    //                     "🎊 The supply has just reached {} BTC 🎊",
    //                     util::format_number(*supply_alert as usize)
    //                 );
    //
    //                 self.queue_notification(plain_text)?;
    //             }
    //         }
    //     }
    //
    //     Ok(())
    // }

    fn hashrate(&self, mining_info: &GetMiningInfoResult) -> Result<()> {
        let current_hashrate: f64 = mining_info.network_hash_ps / u64::pow(10, 18) as f64; // Hashrate in EH/s

        let last_hashrate_ath: f64 = match self.bitcoin_store.get_last_hashrate_ath() {
            Ok(value) => value,
            Err(_) => {
                self.bitcoin_store.set_last_hashrate_ath(current_hashrate)?;
                current_hashrate
            }
        };

        if current_hashrate > last_hashrate_ath {
            let plain_text: String = format!("🎉  New hashrate ATH: {current_hashrate:.2} EH/s 🎉");
            self.queue_notification(plain_text)?;
            self.bitcoin_store.set_last_hashrate_ath(current_hashrate)?;
        }

        Ok(())
    }

    fn block(&self, block_height: u64) -> Result<()> {
        if BLOCK_ALERTS.contains(&block_height) {
            let plain_text: String = format!(
                "⛓️ Reached block {} ⛓️",
                util::format_number(block_height as usize)
            );
            self.queue_notification(plain_text)?;
        }

        if self.config.bitcoin.network == Network::Regtest {
            let plain_text: String = format!(
                "⛓️ Reached block {} ⛓️",
                util::format_number(block_height as usize)
            );
            self.queue_notification(plain_text)?;
        }

        Ok(())
    }

    fn queue_notification<S>(&self, plain_text: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        let plain_text: &str = plain_text.as_ref();

        if self.config.ntfy.enabled {
            self.queue_notification_with_target(Target::Ntfy, plain_text, plain_text)?;
        }

        if self.config.nostr.enabled {
            self.queue_notification_with_target(Target::Nostr, plain_text, plain_text)?;
        }

        Ok(())
    }

    fn queue_notification_with_target(
        &self,
        target: Target,
        plain_text: &str,
        html: &str,
    ) -> Result<()> {
        match self
            .notification_store
            .create_notification(target, plain_text, html)
        {
            Ok(_) => tracing::info!("Queued a new notification for {}", target),
            Err(err) => {
                tracing::error!("Impossible to queue notification for {}: {:?}", target, err)
            }
        };

        Ok(())
    }
}
