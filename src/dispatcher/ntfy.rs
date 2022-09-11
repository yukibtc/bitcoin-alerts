// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use bpns_common::thread;
use ntfy::{Client, Payload, Priority};

use crate::primitives::Target;
use crate::{CONFIG, STORE};

pub struct Ntfy;

impl Ntfy {
    pub fn run() {
        thread::spawn("ntfy", {
            log::info!("Ntfy Dispatcher started");

            let proxy: Option<&str> = match &CONFIG.ntfy.proxy {
                Some(proxy) => Some(proxy.as_str()),
                None => None,
            };

            let client = Client::new(&CONFIG.ntfy.url, proxy).unwrap();

            move || loop {
                log::debug!("Process pending notifications");

                let notifications = match STORE.get_notifications_by_target(Target::Ntfy) {
                    Ok(result) => result,
                    Err(error) => {
                        log::error!("Impossible to get ntfy notifications from db: {:?}", error);
                        thread::sleep(60);
                        continue;
                    }
                };

                for (id, notification) in notifications.into_iter() {
                    let payload = Payload {
                        topic: CONFIG.ntfy.topic.clone(),
                        message: notification.plain_text.clone(),
                        priority: Priority::Default,
                        title: Some(String::from("Bitcoin Alerts")),
                    };

                    match client.publish(&payload) {
                        Ok(_) => {
                            log::info!("Sent notification: {}", notification.plain_text);

                            match STORE.delete_notification(id.as_str()) {
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
