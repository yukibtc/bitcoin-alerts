// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde;

mod bitcoin;
mod bot;
mod config;
mod db;
mod logger;
mod util;

use bitcoin::Bitcoin;
use bot::Bot;
use config::Config;
use db::DBStore;

lazy_static! {
    pub(crate) static ref CONFIG: Config = Config::from_args();
    pub(crate) static ref STORE: DBStore = DBStore::open(&CONFIG.db_path).unwrap();
}

#[tokio::main]
async fn main() {
    logger::init();

    Bitcoin::run();

    Bot::run().await.unwrap();
}
