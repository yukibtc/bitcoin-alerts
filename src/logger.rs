// Copyright (c) 2021-2024 Yuki Kishimoto
// Distributed under the MIT software license

use tracing::Level;

use crate::config::Config;

pub fn init(config: &Config) {
    let level: Level = if cfg!(debug_assertions) && config.log_level != Level::TRACE {
        Level::DEBUG
    } else {
        config.log_level
    };

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(level)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
}
