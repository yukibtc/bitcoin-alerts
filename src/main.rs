// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::thread;
use std::time::Duration;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde;

use anyhow::Result;

mod bitcoin;
mod config;
mod db;
mod dispatcher;
mod logger;
mod primitives;
mod util;

use self::bitcoin::Bitcoin;
use self::config::Config;
use self::db::{BitcoinStore, NotificationStore};
use self::dispatcher::Dispatcher;

lazy_static! {
    pub static ref CONFIG: Config = Config::from_args();
    pub static ref BITCOIN_STORE: BitcoinStore =
        BitcoinStore::open(&CONFIG.bitcoin.db_path).unwrap();
    pub static ref NOTIFICATION_STORE: NotificationStore =
        NotificationStore::open(&CONFIG.main_path.join("notification")).unwrap();
}

#[tokio::main]
async fn main() -> Result<()> {
    logger::init();

    Bitcoin::run();

    Dispatcher::run().await?;

    loop {
        thread::sleep(Duration::from_secs(60));
    }
}
