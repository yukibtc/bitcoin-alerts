// Copyright (c) 2021-2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::sync::LazyLock;
use std::thread;
use std::time::Duration;

#[macro_use]
extern crate serde;

use nostr_sdk::Result;

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

static CONFIG: LazyLock<Config> = LazyLock::new(Config::from_args);
static BITCOIN_STORE: LazyLock<BitcoinStore> =
    LazyLock::new(|| BitcoinStore::open(&CONFIG.bitcoin.db_path).unwrap());
static NOTIFICATION_STORE: LazyLock<NotificationStore> =
    LazyLock::new(|| NotificationStore::open(&CONFIG.main_path.join("notification")).unwrap());

#[tokio::main]
async fn main() -> Result<()> {
    logger::init();

    Bitcoin::run();

    Dispatcher::run().await?;

    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
