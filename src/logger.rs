// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use tracing::Level;

use crate::CONFIG;

pub fn init() {
    let level: Level = if cfg!(debug_assertions) && CONFIG.log_level != Level::TRACE {
        Level::DEBUG
    } else {
        CONFIG.log_level
    };

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(level)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
}
