// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use bpns_rocksdb::{BoundColumnFamily, Error, Store};

use crate::primitives::Target;
use crate::util;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub access_token: String,
    pub device_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Notification {
    pub target: Target,
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

    fn session_cf(&self) -> Arc<BoundColumnFamily> {
        self.db.cf_handle(SESSION_CF)
    }

    fn subscription_cf(&self) -> Arc<BoundColumnFamily> {
        self.db.cf_handle(SUBSCRIPTION_CF)
    }

    fn network_cf(&self) -> Arc<BoundColumnFamily> {
        self.db.cf_handle(NETWORK_CF)
    }

    fn notification_cf(&self) -> Arc<BoundColumnFamily> {
        self.db.cf_handle(NOTIFICATION_CF)
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

        self.db.put_serialized(self.session_cf(), user_id, &value)
    }

    pub fn session_exist(&self, user_id: &str) -> bool {
        self.db.get(self.session_cf(), user_id).is_ok()
    }

    pub fn get_session(&self, user_id: &str) -> Result<Session, Error> {
        self.db.get_deserialized(self.session_cf(), user_id)
    }

    pub fn create_subscription(&self, room_id: &str) -> Result<(), Error> {
        self.db.put(self.subscription_cf(), room_id, "true")
    }

    pub fn subscription_exist(&self, room_id: &str) -> bool {
        self.db.get(self.subscription_cf(), room_id).is_ok()
    }

    pub fn delete_subscription(&self, room_id: &str) -> Result<(), Error> {
        self.db.delete(self.subscription_cf(), room_id)
    }

    pub fn get_subscriptions(&self) -> Result<Vec<String>, Error> {
        let collection = self
            .db
            .iterator_str_serialized::<bool>(self.subscription_cf())?;

        Ok(collection.keys().cloned().collect())
    }

    pub fn create_notification(
        &self,
        target: Target,
        plain_text: &str,
        html: &str,
    ) -> Result<(), Error> {
        let key: &str = &util::sha512(format!("{}:{}:{}", target, plain_text, html))[..32];
        let value: Notification = Notification {
            target,
            plain_text: plain_text.into(),
            html: html.into(),
        };

        self.db.put_serialized(self.notification_cf(), key, &value)
    }

    /* pub fn get_all_notifications(&self) -> Result<HashMap<String, Notification>, Error> {
        self.db
            .iterator_str_serialized::<Notification>(self.notification_cf())
    } */

    pub fn get_notifications_by_target(
        &self,
        target: Target,
    ) -> Result<HashMap<String, Notification>, Error> {
        let mut notification: HashMap<String, Notification> = HashMap::new();
        let collection = self
            .db
            .iterator_str_serialized::<Notification>(self.notification_cf())?;

        collection.into_iter().for_each(|(key, value)| {
            if value.target == target {
                notification.insert(key, value);
            }
        });

        Ok(notification)
    }

    pub fn delete_notification(&self, notification_id: &str) -> Result<(), Error> {
        self.db.delete(self.notification_cf(), notification_id)
    }

    pub fn get_last_processed_block(&self) -> Result<u64, Error> {
        let cf = self.network_cf();
        match self.db.get(cf, "last_processed_block") {
            Ok(result) => match util::bytes_to_number::<u64>(result) {
                Some(num) => Ok(num),
                None => Err(Error::FailedToDeserialize),
            },
            Err(error) => Err(error),
        }
    }

    pub fn set_last_processed_block(&self, block_height: u64) -> Result<(), Error> {
        self.db.put(
            self.network_cf(),
            "last_processed_block",
            block_height.to_string(),
        )
    }

    pub fn get_last_difficulty(&self) -> Result<f64, Error> {
        match self.db.get(self.network_cf(), "last_difficulty") {
            Ok(result) => match util::bytes_to_number::<f64>(result) {
                Some(num) => Ok(num),
                None => Err(Error::FailedToDeserialize),
            },
            Err(error) => Err(error),
        }
    }

    pub fn set_last_difficculty(&self, difficulty: f64) -> Result<(), Error> {
        self.db
            .put(self.network_cf(), "last_difficulty", difficulty.to_string())
    }

    pub fn get_last_supply(&self) -> Result<f64, Error> {
        match self.db.get(self.network_cf(), "last_supply") {
            Ok(result) => match util::bytes_to_number::<f64>(result) {
                Some(num) => Ok(num),
                None => Err(Error::FailedToDeserialize),
            },
            Err(error) => Err(error),
        }
    }

    pub fn set_last_supply(&self, hashrate: f64) -> Result<(), Error> {
        self.db
            .put(self.network_cf(), "last_supply", hashrate.to_string())
    }

    pub fn get_last_hashrate_ath(&self) -> Result<f64, Error> {
        match self.db.get(self.network_cf(), "last_hashrate_ath") {
            Ok(result) => match util::bytes_to_number::<f64>(result) {
                Some(num) => Ok(num),
                None => Err(Error::FailedToDeserialize),
            },
            Err(error) => Err(error),
        }
    }

    pub fn set_last_hashrate_ath(&self, hashrate: f64) -> Result<(), Error> {
        self.db
            .put(self.network_cf(), "last_hashrate_ath", hashrate.to_string())
    }
}

impl Drop for DBStore {
    fn drop(&mut self) {
        log::trace!("Closing Database");
    }
}
