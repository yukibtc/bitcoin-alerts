// Copyright (c) 2021-2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::Duration;

use nostr_sdk::nostr::nips::nip01::Metadata;
use nostr_sdk::{Client, EventBuilder, Result};
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
    let client: Client = Client::builder().signer(signer).build();

    for relay_url in config.nostr.relays.iter() {
        client.add_relay(relay_url).await?;
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

    // Set NIP65 list
    let builder = EventBuilder::relay_list(config.nostr.relays.iter().cloned().map(|u| (u, None)));
    if let Err(err) = client.send_event_builder(builder).await {
        tracing::error!("Impossible to set relay list: {}", err);
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
                tracing::info!("Sending notification: {}", notification.plain_text);

                // Create builder
                let builder: EventBuilder = EventBuilder::text_note(notification.plain_text)
                    .pow(config.nostr.pow_difficulty);

                // Send event
                match client.send_event_builder(builder).await {
                    Ok(_) => match store.delete_notification(id.as_str()) {
                        Ok(_) => tracing::debug!("Notification {} deleted", id),
                        Err(error) => tracing::error!(
                            "Impossible to delete notification {}: {:#?}",
                            id,
                            error
                        ),
                    },
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
