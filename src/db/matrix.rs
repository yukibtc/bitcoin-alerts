// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::path::Path;
use std::sync::Arc;

use bpns_rocksdb::{BoundColumnFamily, Error, Store};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub access_token: String,
    pub device_id: String,
}

pub struct MatrixStore {
    pub db: Store,
}

const SESSION_CF: &str = "session";
const SUBSCRIPTION_CF: &str = "subscription";

const COLUMN_FAMILIES: &[&str] = &[SESSION_CF, SUBSCRIPTION_CF];

impl MatrixStore {
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
}

impl Drop for MatrixStore {
    fn drop(&mut self) {
        tracing::trace!("Closing Database");
    }
}
