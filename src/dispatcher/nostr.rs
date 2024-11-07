// Copyright (c) 2021-2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::thread;
use std::time::Duration;

use nostr_sdk::nostr::Metadata;
use nostr_sdk::{Client, Options, Result};

use crate::primitives::Target;
use crate::{CONFIG, NOTIFICATION_STORE};

pub struct Nostr;

impl Nostr {
    pub async fn run() -> Result<()> {
        let opts = Options::new()
            .connection_timeout(Some(Duration::from_secs(10)))
            .difficulty(CONFIG.nostr.pow_difficulty);
        let client: Client = Client::builder()
            .signer(CONFIG.nostr.keys.clone())
            .opts(opts)
            .build();

        for relay in CONFIG.nostr.relays.iter() {
            client.add_relay(relay).await?;
        }

        client.connect().await;

        let mut metadata = Metadata::new()
            .name(&CONFIG.nostr.name)
            .display_name(&CONFIG.nostr.display_name)
            .about(&CONFIG.nostr.description)
            .picture(CONFIG.nostr.picture.clone())
            .lud16(&CONFIG.nostr.lud16);

        if let Some(nip05) = &CONFIG.nostr.nip05 {
            metadata = metadata.nip05(nip05);
        }

        if let Err(err) = client.set_metadata(&metadata).await {
            tracing::error!("Impossible to update profile metadata: {}", err);
        }

        tokio::spawn(async move {
            tracing::info!("Nostr Dispatcher started");

            loop {
                tracing::debug!("Process pending notifications");

                let notifications =
                    match NOTIFICATION_STORE.get_notifications_by_target(Target::Nostr) {
                        Ok(result) => result,
                        Err(error) => {
                            tracing::error!(
                                "Impossible to get nostr notifications from db: {:?}",
                                error
                            );
                            tokio::time::sleep(Duration::from_secs(60)).await;
                            continue;
                        }
                    };

                if !notifications.is_empty() {
                    for (id, notification) in notifications.into_iter() {
                        match client.publish_text_note(&notification.plain_text, []).await {
                            Ok(_) => {
                                tracing::info!("Sent notification: {}", notification.plain_text);

                                match NOTIFICATION_STORE.delete_notification(id.as_str()) {
                                    Ok(_) => tracing::debug!("Notification {} deleted", id),
                                    Err(error) => tracing::error!(
                                        "Impossible to delete notification {}: {:#?}",
                                        id,
                                        error
                                    ),
                                };
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Impossible to send notification {}: {}",
                                    id,
                                    e.to_string()
                                )
                            }
                        }
                    }
                }

                tracing::debug!("Wait for new notifications");
                tokio::time::sleep(Duration::from_secs(60)).await;
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
