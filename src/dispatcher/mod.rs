// Copyright (c) 2021-2024 Yuki Kishimoto
// Distributed under the MIT software license

mod nostr;
mod ntfy;

use crate::config::Config;
use crate::db::NotificationStore;

pub async fn run(config: Config, store: &NotificationStore) {
    tokio::select! {
        _ = ntfy::run(&config, store) => {
            println!("ntfy exited.");
        }
        _ = nostr::run(&config, store) => {
            println!("nostr exited.");
        }
    }
}
