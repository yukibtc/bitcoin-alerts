// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::thread;

use nostr_sdk::Result;

mod nostr;
mod ntfy;

use self::nostr::Nostr;
use self::ntfy::Ntfy;

use crate::CONFIG;

pub struct Dispatcher;

impl Dispatcher {
    pub async fn run() -> Result<()> {
        if CONFIG.ntfy.enabled {
            Ntfy::run().await?;
        }

        if CONFIG.nostr.enabled {
            Nostr::run().await?;
        }

        Ok(())
    }
}

impl Drop for Dispatcher {
    fn drop(&mut self) {
        if thread::panicking() {
            std::process::exit(0x1);
        }
    }
}
