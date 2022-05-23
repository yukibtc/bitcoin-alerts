// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::io::Write;

use env_logger::{fmt::Style, fmt::Timestamp, Builder, Env};

pub fn init() {
    let pkg_name: String = env!("CARGO_PKG_NAME").replace('-', "_");

    let mut default_filter = format!("{}=info", pkg_name);

    if cfg!(debug_assertions) {
        default_filter = format!("{}=debug", pkg_name);
    }

    Builder::from_env(Env::default().default_filter_or(default_filter))
        .format(|buf, record| {
            let level: log::Level = record.level();

            let timestamp: Timestamp = buf.timestamp();
            let level_style: Style = buf.default_level_style(level);

            writeln!(
                buf,
                "[{} {}  {}] {}",
                timestamp,
                level_style.value(level),
                record.target(),
                record.args()
            )
        })
        .init();
}
