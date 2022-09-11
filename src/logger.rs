// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use env_logger::{Builder, Env};

pub fn init() {
    let mut log_level = "info";

    if cfg!(debug_assertions) {
        log_level = "debug";
    }

    Builder::from_env(Env::default().default_filter_or(log_level.to_string())).init();
}
