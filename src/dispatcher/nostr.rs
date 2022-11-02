// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use bpns_common::thread;
use nostr_sdk::base::{Event, Keys};
use nostr_sdk::Client;

use crate::primitives::Target;
use crate::{CONFIG, NOTIFICATION_STORE};

pub struct Nostr;

impl Nostr {
    pub fn run() {
        thread::spawn("nostr", {
            log::info!("Nostr Dispatcher started");

            let my_keys: &Keys = &CONFIG.nostr.keys;

            let client: Client = Client::new(my_keys, None);

            for relay in CONFIG.nostr.relays.iter() {
                if let Err(err) = client.add_relay(relay.as_str()) {
                    log::error!("Impossible to add relay: {}", err);
                }
            }

            client.connect_and_keep_alive();

            #[cfg(not(debug_assertions))]
            match Event::set_metadata(
                my_keys,
                "bitcoin_alerts",
                "Bitcoin Alerts",
                Some("Hashrate, supply, blocks until halving, difficulty adjustment and more."),
                Some("https://avatars.githubusercontent.com/u/13464320"),
            ) {
                Ok(event) => client.send_event(event),
                Err(err) => log::error!("Impossible to set metadata: {}", err),
            };

            move || loop {
                log::debug!("Process pending notifications");

                let notifications = match NOTIFICATION_STORE
                    .get_notifications_by_target(Target::Nostr)
                {
                    Ok(result) => result,
                    Err(error) => {
                        log::error!("Impossible to get nostr notifications from db: {:?}", error);
                        thread::sleep(60);
                        continue;
                    }
                };

                for (id, notification) in notifications.into_iter() {
                    match Event::new_pow_textnote(&notification.plain_text, my_keys, &[], 16) {
                        Ok(event) => {
                            client.send_event(event);

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
                    }
                }

                log::debug!("Wait for new notifications");
                thread::sleep(30);
            }
        });
    }
}

impl Drop for Nostr {
    fn drop(&mut self) {
        if thread::panicking() {
            std::process::exit(0x1);
        }
    }
}
