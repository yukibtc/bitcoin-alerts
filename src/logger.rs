// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use env_logger::{Builder, Env};
use log::Level;

use crate::CONFIG;

pub fn init() {
    let log_level: Level = if cfg!(debug_assertions) && CONFIG.log_level != Level::Trace {
        Level::Debug
    } else {
        CONFIG.log_level
    };

    Builder::from_env(Env::default().default_filter_or(log_level.to_string())).init();
}
