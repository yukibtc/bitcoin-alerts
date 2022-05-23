// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use matrix_sdk::{
    room::Room,
    ruma::{
        events::{room::member::RoomMemberEventContent, StrippedStateEvent},
        UserId,
    },
    Client,
};
use tokio::time::{sleep, Duration};

pub(crate) async fn on_stripped_state_member(
    room_member: StrippedStateEvent<RoomMemberEventContent>,
    client: Client,
    room: Room,
) {
    let user_id_boxed: Box<UserId> = match client.user_id().await {
        Some(value) => value,
        None => {
            log::warn!("No user_id found");
            return;
        }
    };

    if room_member.state_key != *user_id_boxed {
        return;
    }

    tokio::spawn(async move {
        if let Room::Invited(room) = room {
            log::info!("Autojoining room {}", room.room_id());
            let mut delay = 2;

            log::debug!("Starting autojoin room loop");

            loop {
                match room.accept_invitation().await {
                    Ok(_) => {
                        log::info!("Successfully joined room {}", room.room_id());
                        break;
                    }
                    Err(err) => {
                        log::error!(
                            "Failed to join room {} ({:?}), retrying in {}s",
                            room.room_id(),
                            err,
                            delay
                        );

                        sleep(Duration::from_secs(delay)).await;
                        delay *= 2;

                        if delay > 3600 {
                            log::error!("Can't join room {} ({:?})", room.room_id(), err);
                            break;
                        }
                    }
                }
            }

            log::debug!("Out of autojoin room loop");
        }
    });
}
