// Copyright (c) 2021-2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::Duration;

use nostr_sdk::nostr::Metadata;
use nostr_sdk::{Client, Options, Result};
use tokio::time;

use crate::config::Config;
use crate::db::NotificationStore;
use crate::primitives::Target;

pub async fn run(config: &Config, store: &NotificationStore) -> Result<()> {
    // If not enabled, infinite loop
    if !config.nostr.enabled {
        loop {
            time::sleep(Duration::from_secs(60)).await;
        }
    }

    let signer = config.nostr.keys.clone().unwrap();
    let opts = Options::new()
        .connection_timeout(Some(Duration::from_secs(10)))
        .difficulty(config.nostr.pow_difficulty);
    let client: Client = Client::builder().signer(signer).opts(opts).build();

    for relay in config.nostr.relays.clone().unwrap().into_iter() {
        client.add_relay(relay).await?;
    }

    client.connect().await;

    let mut metadata = Metadata::new()
        .name(&config.nostr.name)
        .display_name(&config.nostr.display_name)
        .about(&config.nostr.description)
        .picture(config.nostr.picture.clone())
        .lud16(&config.nostr.lud16);

    if let Some(nip05) = &config.nostr.nip05 {
        metadata = metadata.nip05(nip05);
    }

    if let Err(err) = client.set_metadata(&metadata).await {
        tracing::error!("Impossible to update profile metadata: {}", err);
    }

    tracing::info!("Nostr Dispatcher started");

    loop {
        tracing::debug!("Process pending notifications");

        let notifications = match store.get_notifications_by_target(Target::Nostr) {
            Ok(result) => result,
            Err(error) => {
                tracing::error!("Impossible to get nostr notifications from db: {:?}", error);
                time::sleep(Duration::from_secs(60)).await;
                continue;
            }
        };

        if !notifications.is_empty() {
            for (id, notification) in notifications.into_iter() {
                match client.publish_text_note(&notification.plain_text, []).await {
                    Ok(_) => {
                        tracing::info!("Sent notification: {}", notification.plain_text);

                        match store.delete_notification(id.as_str()) {
                            Ok(_) => tracing::debug!("Notification {} deleted", id),
                            Err(error) => tracing::error!(
                                "Impossible to delete notification {}: {:#?}",
                                id,
                                error
                            ),
                        };
                    }
                    Err(e) => {
                        tracing::error!("Impossible to send notification {id}: {e}")
                    }
                }
            }
        }

        tracing::debug!("Wait for new notifications");
        time::sleep(Duration::from_secs(60)).await;
    }
}
