// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::path::Path;
use std::sync::Arc;

use bpns_rocksdb::{BoundColumnFamily, Error, Store};

use crate::util;

pub struct BitcoinStore {
    pub db: Store,
}

const NETWORK_CF: &str = "network";

const COLUMN_FAMILIES: &[&str] = &[NETWORK_CF];

impl BitcoinStore {
    pub fn open(path: &Path) -> Result<Self, Error> {
        Ok(Self {
            db: Store::open(path, COLUMN_FAMILIES)?,
        })
    }

    fn network_cf(&self) -> Arc<BoundColumnFamily> {
        self.db.cf_handle(NETWORK_CF)
    }

    pub fn get_last_processed_block(&self) -> Result<u64, Error> {
        let cf = self.network_cf();
        match self.db.get(cf, "last_processed_block") {
            Ok(result) => match util::bytes_to_number::<u64>(result) {
                Some(num) => Ok(num),
                None => Err(Error::FailedToDeserialize),
            },
            Err(error) => Err(error),
        }
    }

    pub fn set_last_processed_block(&self, block_height: u64) -> Result<(), Error> {
        self.db.put(
            self.network_cf(),
            "last_processed_block",
            block_height.to_string(),
        )
    }

    pub fn get_last_difficulty(&self) -> Result<f64, Error> {
        match self.db.get(self.network_cf(), "last_difficulty") {
            Ok(result) => match util::bytes_to_number::<f64>(result) {
                Some(num) => Ok(num),
                None => Err(Error::FailedToDeserialize),
            },
            Err(error) => Err(error),
        }
    }

    pub fn set_last_difficculty(&self, difficulty: f64) -> Result<(), Error> {
        self.db
            .put(self.network_cf(), "last_difficulty", difficulty.to_string())
    }

    pub fn get_last_supply(&self) -> Result<f64, Error> {
        match self.db.get(self.network_cf(), "last_supply") {
            Ok(result) => match util::bytes_to_number::<f64>(result) {
                Some(num) => Ok(num),
                None => Err(Error::FailedToDeserialize),
            },
            Err(error) => Err(error),
        }
    }

    pub fn set_last_supply(&self, hashrate: f64) -> Result<(), Error> {
        self.db
            .put(self.network_cf(), "last_supply", hashrate.to_string())
    }

    pub fn get_last_hashrate_ath(&self) -> Result<f64, Error> {
        match self.db.get(self.network_cf(), "last_hashrate_ath") {
            Ok(result) => match util::bytes_to_number::<f64>(result) {
                Some(num) => Ok(num),
                None => Err(Error::FailedToDeserialize),
            },
            Err(error) => Err(error),
        }
    }

    pub fn set_last_hashrate_ath(&self, hashrate: f64) -> Result<(), Error> {
        self.db
            .put(self.network_cf(), "last_hashrate_ath", hashrate.to_string())
    }
}

impl Drop for BitcoinStore {
    fn drop(&mut self) {
        tracing::trace!("Closing Database");
    }
}
