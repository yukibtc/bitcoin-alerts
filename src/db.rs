// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::collections::HashMap;
use std::path::Path;

use crate::{
    common::db::{Error, Store},
    util,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub access_token: String,
    pub device_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Notification {
    pub plain_text: String,
    pub html: String,
}

pub struct DBStore {
    pub db: Store,
}

const SESSION_CF: &str = "session";
const SUBSCRIPTION_CF: &str = "subscription";
const NETWORK_CF: &str = "network";
const NOTIFICATION_CF: &str = "notification";

const COLUMN_FAMILIES: &[&str] = &[SESSION_CF, SUBSCRIPTION_CF, NETWORK_CF, NOTIFICATION_CF];

impl DBStore {
    pub fn open(path: &Path) -> Result<Self, Error> {
        Ok(Self {
            db: Store::open(path, COLUMN_FAMILIES)?,
        })
    }

    pub fn create_session(
        &self,
        user_id: &str,
        access_token: &str,
        device_id: &str,
    ) -> Result<(), Error> {
        let value: Session = Session {
            access_token: access_token.into(),
            device_id: device_id.into(),
        };

        self.db
            .put_serialized(self.db.cf_handle(SESSION_CF), user_id, &value)
    }

    pub fn session_exist(&self, user_id: &str) -> bool {
        self.db.get(self.db.cf_handle(SESSION_CF), user_id).is_ok()
    }

    pub fn get_session(&self, user_id: &str) -> Result<Session, Error> {
        self.db
            .get_serialized(self.db.cf_handle(SESSION_CF), user_id)
    }

    pub fn create_subscription(&self, room_id: &str) -> Result<(), Error> {
        self.db
            .put(self.db.cf_handle(SUBSCRIPTION_CF), room_id, "true")
    }

    pub fn subscription_exist(&self, room_id: &str) -> bool {
        self.db
            .get(self.db.cf_handle(SUBSCRIPTION_CF), room_id)
            .is_ok()
    }

    pub fn delete_subscription(&self, room_id: &str) -> Result<(), Error> {
        self.db.delete(self.db.cf_handle(SUBSCRIPTION_CF), room_id)
    }

    pub fn get_subscriptions(&self) -> Result<Vec<String>, Error> {
        let collection = self
            .db
            .iterator_serialized::<bool>(self.db.cf_handle(SUBSCRIPTION_CF))?;

        Ok(collection.keys().cloned().collect())
    }

    pub fn create_notification(&self, plain_text: &str, html: &str) -> Result<(), Error> {
        let key: &str = &util::sha512(format!("{}:{}", plain_text, html))[..32];
        let value: Notification = Notification {
            plain_text: plain_text.into(),
            html: html.into(),
        };

        self.db
            .put_serialized(self.db.cf_handle(NOTIFICATION_CF), key, &value)
    }

    pub fn get_notifications(&self) -> Result<HashMap<String, Notification>, Error> {
        self.db
            .iterator_serialized::<Notification>(self.db.cf_handle(NOTIFICATION_CF))
    }

    pub fn delete_notification(&self, notification_id: &str) -> Result<(), Error> {
        self.db
            .delete(self.db.cf_handle(NOTIFICATION_CF), notification_id)
    }

    pub fn get_last_processed_block(&self) -> Result<u32, Error> {
        let cf = self.db.cf_handle(NETWORK_CF);
        match self.db.get(cf, "last_processed_block") {
            Ok(result) => match util::bytes_to_number::<u32>(result) {
                Some(num) => Ok(num),
                None => Err(Error::FailedToDeserialize),
            },
            Err(error) => Err(error),
        }
    }

    pub fn set_last_processed_block(&self, block_height: u32) -> Result<(), Error> {
        let cf = self.db.cf_handle(NETWORK_CF);
        self.db
            .put(cf, "last_processed_block", block_height.to_string())
    }

    pub fn get_last_difficulty(&self) -> Result<f64, Error> {
        let cf = self.db.cf_handle(NETWORK_CF);
        match self.db.get(cf, "last_difficulty") {
            Ok(result) => match util::bytes_to_number::<f64>(result) {
                Some(num) => Ok(num),
                None => Err(Error::FailedToDeserialize),
            },
            Err(error) => Err(error),
        }
    }

    pub fn set_last_difficculty(&self, difficulty: f64) -> Result<(), Error> {
        let cf = self.db.cf_handle(NETWORK_CF);
        self.db.put(cf, "last_difficulty", difficulty.to_string())
    }

    pub fn get_last_supply(&self) -> Result<f64, Error> {
        let cf = self.db.cf_handle(NETWORK_CF);
        match self.db.get(cf, "last_supply") {
            Ok(result) => match util::bytes_to_number::<f64>(result) {
                Some(num) => Ok(num),
                None => Err(Error::FailedToDeserialize),
            },
            Err(error) => Err(error),
        }
    }

    pub fn set_last_supply(&self, hashrate: f64) -> Result<(), Error> {
        let cf = self.db.cf_handle(NETWORK_CF);
        self.db.put(cf, "last_supply", hashrate.to_string())
    }

    pub fn get_last_hashrate_ath(&self) -> Result<f64, Error> {
        let cf = self.db.cf_handle(NETWORK_CF);
        match self.db.get(cf, "last_hashrate_ath") {
            Ok(result) => match util::bytes_to_number::<f64>(result) {
                Some(num) => Ok(num),
                None => Err(Error::FailedToDeserialize),
            },
            Err(error) => Err(error),
        }
    }

    pub fn set_last_hashrate_ath(&self, hashrate: f64) -> Result<(), Error> {
        let cf = self.db.cf_handle(NETWORK_CF);
        self.db.put(cf, "last_hashrate_ath", hashrate.to_string())
    }
}

impl Drop for DBStore {
    fn drop(&mut self) {
        log::trace!("Closing Database");
    }
}
