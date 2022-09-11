// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use bpns_common::thread;

mod matrix;
mod ntfy;

use self::matrix::Matrix;
use self::ntfy::Ntfy;

use crate::CONFIG;

pub struct Dispatcher;

impl Dispatcher {
    pub fn run() {
        if CONFIG.ntfy.enabled {
            Ntfy::run();
        }

        if CONFIG.matrix.enabled {
            Matrix::run();
        }
    }
}

impl Drop for Dispatcher {
    fn drop(&mut self) {
        if thread::panicking() {
            std::process::exit(0x1);
        }
    }
}
