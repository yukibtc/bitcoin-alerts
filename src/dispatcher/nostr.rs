// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use anyhow::Result;
use bpns_common::thread;
use nostr_sdk::nostr::Metadata;
use nostr_sdk::Client;
use url::Url;

use crate::primitives::Target;
use crate::{CONFIG, NOTIFICATION_STORE};

pub struct Nostr;

impl Nostr {
    pub async fn run() -> Result<()> {
        let mut client: Client = Client::new(&CONFIG.nostr.keys);

        for relay in CONFIG.nostr.relays.iter() {
            if let Err(err) = client.add_relay(relay.as_str(), None) {
                log::error!("Impossible to add relay: {}", err);
            }
        }

        let _ = client.connect().await;

        #[cfg(not(debug_assertions))]
        let metadata = Metadata::new()
            .name("bitcoin_alerts")
            .display_name("Bitcoin Alerts")
            .about("Hashrate, supply, blocks until halving, difficulty adjustment and more.")
            .picture(Url::from_str(
                "https://avatars.githubusercontent.com/u/13464320",
            )?);

        #[cfg(debug_assertions)]
        let metadata = Metadata::new()
            .name("test_alerts")
            .display_name("Test Alerts")
            .about("Description")
            .picture(Url::from_str(
                "http://mymodernmet.com/wp/wp-content/uploads/2017/03/gabrielius-khiterer-stray-cats-11.jpg",
            )?);

        if let Err(err) = client.update_profile(metadata).await {
            log::error!("Impossible to update profile metadata: {}", err);
        }

        tokio::spawn(async move {
            log::info!("Nostr Dispatcher started");

            loop {
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
                    let result = if CONFIG.nostr.pow_enabled {
                        client
                            .publish_pow_text_note(
                                &notification.plain_text,
                                &[],
                                CONFIG.nostr.pow_difficulty,
                            )
                            .await
                    } else {
                        client
                            .publish_text_note(&notification.plain_text, &[])
                            .await
                    };

                    match result {
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
                    }
                }

                log::debug!("Wait for new notifications");
                thread::sleep(120);
            }
        });

        Ok(())
    }
}

impl Drop for Nostr {
    fn drop(&mut self) {
        if thread::panicking() {
            std::process::exit(0x1);
        }
    }
}
