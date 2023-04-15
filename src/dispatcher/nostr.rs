// Copyright (c) 2021-2023 Yuki Kishimoto
// Distributed under the MIT software license

use anyhow::Result;
use bpns_common::thread;
use nostr_sdk::nostr::Metadata;
use nostr_sdk::{Client, Options};

use crate::primitives::Target;
use crate::{CONFIG, NOTIFICATION_STORE};

pub struct Nostr;

impl Nostr {
    pub async fn run() -> Result<()> {
        let opts = Options::new().difficulty(CONFIG.nostr.pow_difficulty);
        let client: Client = Client::with_opts(&CONFIG.nostr.keys, opts);

        for url in CONFIG.nostr.relays.iter() {
            client.add_relay(url.as_str(), None).await?;
        }

        client.connect().await;

        let metadata = Metadata::new()
            .name(&CONFIG.nostr.name)
            .display_name(&CONFIG.nostr.display_name)
            .about(&CONFIG.nostr.description)
            .picture(CONFIG.nostr.picture.clone())
            .lud16(&CONFIG.nostr.lud16);

        if let Err(err) = client.set_metadata(metadata).await {
            log::error!("Impossible to update profile metadata: {}", err);
        }

        if let Err(e) = client.disconnect().await {
            log::error!("Impossible to disconnect relays: {}", e);
        }

        tokio::spawn(async move {
            log::info!("Nostr Dispatcher started");

            thread::sleep(30);

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

                if !notifications.is_empty() {
                    client.connect().await;

                    for (id, notification) in notifications.into_iter() {
                        match client
                            .publish_text_note(&notification.plain_text, &[])
                            .await
                        {
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
                            Err(e) => {
                                log::error!(
                                    "Impossible to send notification {}: {}",
                                    id,
                                    e.to_string()
                                )
                            }
                        }
                    }

                    if let Err(e) = client.disconnect().await {
                        log::error!("Impossible to disconnect relays: {}", e);
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
