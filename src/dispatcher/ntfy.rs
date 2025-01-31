// Copyright (c) 2021-2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::Duration;

use nostr_sdk::Result;
use ntfy::dispatcher::Async;
use ntfy::{Dispatcher, DispatcherBuilder, Payload};
use tokio::time;

use crate::config::Config;
use crate::db::NotificationStore;
use crate::primitives::Target;

pub async fn run(config: &Config, store: &NotificationStore) -> Result<()> {
    // If not enabled, infinite loop
    if !config.ntfy.enabled {
        loop {
            time::sleep(Duration::from_secs(60)).await;
        }
    }

    let mut dispatcher = DispatcherBuilder::new(&config.ntfy.url);

    if let Some(credentials) = &config.ntfy.auth {
        dispatcher = dispatcher.credentials(credentials.clone());
    }

    if let Some(proxy) = &config.ntfy.proxy {
        dispatcher = dispatcher.proxy(proxy);
    }

    let dispatcher: Dispatcher<Async> = dispatcher.build_async()?;

    tracing::info!("Ntfy Dispatcher started");

    loop {
        tracing::debug!("Process pending notifications");

        let notifications = match store.get_notifications_by_target(Target::Ntfy) {
            Ok(result) => result,
            Err(error) => {
                tracing::error!("Impossible to get ntfy notifications from db: {:?}", error);
                time::sleep(Duration::from_secs(60)).await;
                continue;
            }
        };

        for (id, notification) in notifications.into_iter() {
            let payload = Payload::new(&config.ntfy.topic)
                .message(&notification.plain_text)
                .title("Bitcoin Alerts");

            match dispatcher.send(&payload).await {
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
                Err(err) => {
                    tracing::error!("Impossible to send notification {}: {:?}", id, err)
                }
            };
        }

        tracing::debug!("Wait for new notifications");
        time::sleep(Duration::from_secs(30)).await;
    }
}
