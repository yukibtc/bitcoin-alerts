// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use bpns_common::thread;
use ntfy::{Dispatcher, Payload};

use crate::primitives::Target;
use crate::{CONFIG, NOTIFICATION_STORE};

pub struct Ntfy;

impl Ntfy {
    pub fn run() {
        thread::spawn("ntfy", {
            log::info!("Ntfy Dispatcher started");

            let proxy: Option<&str> = match &CONFIG.ntfy.proxy {
                Some(proxy) => Some(proxy.as_str()),
                None => None,
            };

            let dispatcher = Dispatcher::new(&CONFIG.ntfy.url, proxy).unwrap();

            move || loop {
                log::debug!("Process pending notifications");

                let notifications = match NOTIFICATION_STORE
                    .get_notifications_by_target(Target::Ntfy)
                {
                    Ok(result) => result,
                    Err(error) => {
                        log::error!("Impossible to get ntfy notifications from db: {:?}", error);
                        thread::sleep(60);
                        continue;
                    }
                };

                for (id, notification) in notifications.into_iter() {
                    let payload = Payload::new(&CONFIG.ntfy.topic, &notification.plain_text)
                        .title("Bitcoin Alerts");

                    match dispatcher.send(&payload) {
                        Ok(_) => {
                            log::info!("Sent notification: {}", notification.plain_text);

                            match NOTIFICATION_STORE.delete_notification(id.as_str()) {
                                Ok(_) => log::debug!("Notification {} deleted", id),
                                Err(error) => log::error!(
                                    "Impossible to delete notification {}: {:#?}",
                                    id,
                                    error
                                ),
                            };
                        }
                        Err(err) => {
                            log::error!("Impossible to send notification {}: {:?}", id, err)
                        }
                    };
                }

                log::debug!("Wait for new notifications");
                thread::sleep(30);
            }
        });
    }
}

impl Drop for Ntfy {
    fn drop(&mut self) {
        if thread::panicking() {
            std::process::exit(0x1);
        }
    }
}
