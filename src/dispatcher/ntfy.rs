// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::Duration;

use anyhow::Result;
use ntfy::{Dispatcher, Payload};

use crate::primitives::Target;
use crate::{CONFIG, NOTIFICATION_STORE};

pub struct Ntfy;

impl Ntfy {
    pub async fn run() -> Result<()> {
        let proxy: Option<&str> = match &CONFIG.ntfy.proxy {
            Some(proxy) => Some(proxy.as_str()),
            None => None,
        };

        let dispatcher = Dispatcher::new(&CONFIG.ntfy.url, proxy)?;

        tokio::spawn(async move {
            log::info!("Ntfy Dispatcher started");

            loop {
                log::debug!("Process pending notifications");

                let notifications = match NOTIFICATION_STORE
                    .get_notifications_by_target(Target::Ntfy)
                {
                    Ok(result) => result,
                    Err(error) => {
                        log::error!("Impossible to get ntfy notifications from db: {:?}", error);
                        tokio::time::sleep(Duration::from_secs(60)).await;
                        continue;
                    }
                };

                for (id, notification) in notifications.into_iter() {
                    let payload = Payload::new(&CONFIG.ntfy.topic, &notification.plain_text)
                        .title("Bitcoin Alerts");

                    match dispatcher.send(&payload).await {
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
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        });

        Ok(())
    }
}
