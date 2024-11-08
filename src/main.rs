// Copyright (c) 2021-2024 Yuki Kishimoto
// Distributed under the MIT software license

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

use self::bitcoin::RpcClient;
use self::config::Config;
use self::db::{BitcoinStore, NotificationStore};

#[tokio::main]
async fn main() -> Result<()> {
    // Get config
    let config = Config::from_args();

    // Init logger
    logger::init(&config);

    let rpc = RpcClient::new(&config);
    let bitcoin_store = BitcoinStore::open(&config.bitcoin.db_path).unwrap();
    let notification_store =
        NotificationStore::open(&config.main_path.join("notification")).unwrap();

    tokio::select! {
        _ = bitcoin::run(config.clone(), rpc, bitcoin_store, notification_store.clone()) => {
            println!("Bitcoin processor exited");
        }
        _ = dispatcher::run(config, &notification_store) => {
            println!("Dispatcher exited");
        }
    }

    Ok(())
}
