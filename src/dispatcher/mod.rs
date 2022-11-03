// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use anyhow::Result;
use bpns_common::thread;

mod matrix;
mod nostr;
mod ntfy;

use self::matrix::Matrix;
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

        if CONFIG.matrix.enabled {
            Matrix::run().await?;
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
