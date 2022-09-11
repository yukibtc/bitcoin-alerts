// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::Instant;

use matrix_sdk::{
    config::SyncSettings,
    room::Room,
    ruma::{
        events::room::message::{
            MessageType, OriginalSyncRoomMessageEvent, RoomMessageEventContent,
            TextMessageEventContent,
        },
        RoomId, UserId,
    },
    store::{CryptoStore, StateStore},
    Client, ClientBuilder, Session,
};
use tokio::time::{sleep, Duration};
use tokio_stream::StreamExt;

mod autojoin;

use crate::{CONFIG, STORE};

pub struct Bot;

#[derive(Debug)]
pub enum Error {
    Db(bpns_rocksdb::Error),
    Matrix(matrix_sdk::Error),
    MatrixClientBuilder(matrix_sdk::ClientBuildError),
    MatrixStore(matrix_sdk::StoreError),
    MatrixCryptoStore(matrix_sdk::store::OpenStoreError),
}

impl Bot {
    pub async fn run() -> Result<(), Error> {
        let homeserver_url: &str = CONFIG.matrix.homeserver_url.as_str();
        let user_id: &str = CONFIG.matrix.user_id.as_str();
        let password: &str = CONFIG.matrix.password.as_str();

        let user_id_boxed = Box::<UserId>::try_from(user_id).unwrap();
        let state_store = StateStore::open_with_path(&CONFIG.matrix.state_path)?;
        let crypto_store = CryptoStore::open_with_passphrase(&CONFIG.matrix.state_path, None)?;

        let mut client_builder: ClientBuilder = Client::builder()
            .homeserver_url(homeserver_url)
            .crypto_store(Box::new(crypto_store))
            .state_store(Box::new(state_store));

        if let Some(proxy) = &CONFIG.matrix.proxy {
            client_builder = client_builder.proxy(proxy);
        }

        let client: Client = client_builder.build().await?;

        log::debug!("Checking session...");

        if STORE.session_exist(user_id) {
            let session_store = STORE.get_session(user_id)?;

            let session = Session {
                access_token: session_store.access_token,
                user_id: user_id_boxed,
                device_id: session_store.device_id.into(),
            };

            client.restore_login(session).await?;

            log::debug!("Session restored from database");
        } else {
            log::debug!("Session not found into database");
            log::debug!("Login with credentials...");
            let username = user_id_boxed.localpart();
            client.login(username, password, None, None).await?;

            log::debug!("Getting session data...");

            if let Some(session) = client.session().await {
                log::debug!("Saving session data into database...");
                STORE.create_session(user_id, &session.access_token, session.device_id.as_ref())?;

                log::debug!("Session saved to database");
            } else {
                log::error!("Impossible to get and save session");
                log::warn!("The bot can continue to work without saving the session but if you are using an encrypted room, on the next restart, the bot will not be able to read the messages");
            }
        }

        client
            .account()
            .set_display_name(Some("Bitcoin Alerts"))
            .await?;

        log::info!("Matrix Bot started");

        client
            .register_event_handler(autojoin::on_stripped_state_member)
            .await
            .register_event_handler(
                move |event: OriginalSyncRoomMessageEvent, room: Room| async move {
                    if let Err(error) = Self::on_room_message(event, &room).await {
                        if let Room::Joined(room) = room {
                            let _ = room
                                .send(
                                    RoomMessageEventContent::text_plain(format!("{:?}", error)),
                                    None,
                                )
                                .await;
                        }
                    }
                },
            )
            .await;

        Self::process_pending_notifications(client.clone());

        let settings = SyncSettings::default().full_state(true);
        client.sync(settings).await;

        Ok(())
    }

    async fn on_room_message(
        event: OriginalSyncRoomMessageEvent,
        room: &Room,
    ) -> Result<(), Error> {
        if *event.sender.clone() == CONFIG.matrix.user_id {
            return Ok(());
        }

        if let Room::Joined(room) = room {
            let msg_body = match event.content.msgtype {
                MessageType::Text(TextMessageEventContent { body, .. }) => body,
                _ => return Ok(()),
            };

            log::debug!("Message received: {}", msg_body);

            let start = Instant::now();

            let user_id: &str = event.sender.as_str();
            let room_id: &str = room.room_id().as_str();

            if !CONFIG.matrix.admins.contains(&user_id.to_string()) {
                log::warn!("{} unathorized", user_id);
                let _ = room.redact(&event.event_id, None, None).await;
                return Ok(());
            }

            let msg_splitted: Vec<&str> = msg_body.split(' ').collect();
            let command: &str = msg_splitted[0];

            let mut msg_content: &str = "";

            match command {
                "!subscribe" => {
                    if !STORE.subscription_exist(room_id) {
                        STORE.create_subscription(room_id)?;
                        msg_content = "Subscribed";
                    } else {
                        msg_content = "This account is already subscribed";
                    }
                }
                "!unsubscribe" => {
                    if STORE.subscription_exist(room_id) {
                        STORE.delete_subscription(room_id)?;
                        msg_content = "Unsibscribed";
                    } else {
                        msg_content = "This account is not subscribed";
                    }
                }
                "!help" => {
                    let mut msg = String::new();
                    msg.push_str("!subscribe - Subscribe to alerts\n");
                    msg.push_str("!unsubscribe - Unsubscribe from alerts\n");
                    msg.push_str("!help - Help");

                    let content = RoomMessageEventContent::text_plain(msg);
                    room.send(content, None).await?;
                }
                _ => {
                    msg_content = "Invalid command";
                }
            };

            if !msg_content.is_empty() {
                let content = RoomMessageEventContent::text_plain(msg_content);
                room.send(content, None).await?;
            }

            log::trace!(
                "{} command processed in {} ms",
                command,
                start.elapsed().as_millis()
            );
        }

        Ok(())
    }

    fn process_pending_notifications(bot: Client) {
        tokio::spawn(async move {
            loop {
                log::debug!("Process pending notifications");

                let mut notifications = match STORE.get_notifications() {
                    Ok(result) => tokio_stream::iter(result),
                    Err(error) => {
                        log::error!("Impossible to get notifications from db: {:?}", error);
                        sleep(Duration::from_secs(60)).await;
                        continue;
                    }
                };

                let mut subscriptions = match STORE.get_subscriptions() {
                    Ok(result) => tokio_stream::iter(result),
                    Err(error) => {
                        log::error!("Impossible to get subscriptions from db: {:?}", error);
                        sleep(Duration::from_secs(60)).await;
                        continue;
                    }
                };

                while let Some(notification) = notifications.next().await {
                    let mut notification_sent: bool = false;

                    while let Some(subscription) = subscriptions.next().await {
                        let notification_copy = notification.clone();
                        let plain_text: String = notification_copy.1.plain_text;
                        let html: String = notification_copy.1.html;

                        let room_id = Box::<RoomId>::try_from(subscription.as_str()).unwrap();
                        let room = bot.get_joined_room(&room_id).unwrap();

                        let content =
                            RoomMessageEventContent::text_html(plain_text.clone(), html.clone());

                        match room.send(content, None).await {
                            Ok(_) => {
                                log::info!("Sent notification: {}", plain_text);
                                notification_sent = true;
                            }
                            Err(_) => log::error!("Impossible to send notification"),
                        };

                        sleep(Duration::from_millis(100)).await;
                    }

                    let notification_id: String = notification.0;

                    if notification_sent {
                        match STORE.delete_notification(notification_id.as_str()) {
                            Ok(_) => log::debug!("Notification {} deleted", notification_id),
                            Err(error) => log::error!(
                                "Impossible to delete notification {}: {:#?}",
                                notification_id,
                                error
                            ),
                        };
                    }
                }

                log::debug!("Wait for new notifications");
                sleep(Duration::from_secs(30)).await;
            }
        });
    }
}

impl From<bpns_rocksdb::Error> for Error {
    fn from(err: bpns_rocksdb::Error) -> Self {
        Error::Db(err)
    }
}

impl From<matrix_sdk::Error> for Error {
    fn from(err: matrix_sdk::Error) -> Self {
        Error::Matrix(err)
    }
}

impl From<matrix_sdk::ClientBuildError> for Error {
    fn from(err: matrix_sdk::ClientBuildError) -> Self {
        Error::MatrixClientBuilder(err)
    }
}

impl From<matrix_sdk::StoreError> for Error {
    fn from(err: matrix_sdk::StoreError) -> Self {
        Error::MatrixStore(err)
    }
}

impl From<matrix_sdk::store::OpenStoreError> for Error {
    fn from(err: matrix_sdk::store::OpenStoreError) -> Self {
        Error::MatrixCryptoStore(err)
    }
}
