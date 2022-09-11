// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde;

mod bitcoin;
mod config;
mod db;
mod dispatcher;
mod logger;
mod primitives;
mod util;

use self::bitcoin::Bitcoin;
use self::config::Config;
use self::db::DBStore;
use self::dispatcher::Dispatcher;

lazy_static! {
    pub static ref CONFIG: Config = Config::from_args();
    pub static ref STORE: DBStore = DBStore::open(&CONFIG.db_path).unwrap();
}

fn main() {
    logger::init();

    Bitcoin::run();

    Dispatcher::run();

    loop {
        bpns_common::thread::sleep(120);
    }
}
