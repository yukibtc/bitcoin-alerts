// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::thread;
use std::time::Duration;

pub fn spawn<F, T>(name: &str, f: F) -> thread::JoinHandle<T>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    thread::Builder::new()
        .name(name.to_owned())
        .spawn(f)
        .unwrap()
}

pub fn sleep(seconds: u64) {
    thread::sleep(Duration::from_secs(seconds));
}

/* pub fn sleep_millis(millis: u64) {
    thread::sleep(Duration::from_millis(millis));
} */

pub fn panicking() -> bool {
    thread::panicking()
}
