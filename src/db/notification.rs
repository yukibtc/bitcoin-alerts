// Copyright (c) 2021-2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::db::rocks::{BoundColumnFamily, Error, Store};
use crate::primitives::Target;
use crate::util;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Notification {
    pub target: Target,
    pub plain_text: String,
    pub html: String,
}

#[derive(Clone)]
pub struct NotificationStore {
    pub db: Store,
}

const NOTIFICATION_CF: &str = "notification";

const COLUMN_FAMILIES: &[&str] = &[NOTIFICATION_CF];

impl NotificationStore {
    pub fn open(path: &Path) -> Result<Self, Error> {
        Ok(Self {
            db: Store::open(path, COLUMN_FAMILIES)?,
        })
    }

    fn notification_cf(&self) -> Arc<BoundColumnFamily> {
        self.db.cf_handle(NOTIFICATION_CF)
    }

    pub fn create_notification(
        &self,
        target: Target,
        plain_text: &str,
        html: &str,
    ) -> Result<(), Error> {
        let key: &str = &util::sha512(format!("{target}:{plain_text}:{html}"))[..32];
        let value: Notification = Notification {
            target,
            plain_text: plain_text.to_string(),
            html: html.to_string(),
        };

        self.db.put_serialized(self.notification_cf(), key, &value)
    }

    pub fn get_notifications_by_target(
        &self,
        target: Target,
    ) -> Result<HashMap<String, Notification>, Error> {
        let collection = self
            .db
            .iterator_str_serialized::<Notification>(self.notification_cf())?;
        Ok(collection
            .into_iter()
            .filter(|(_, value)| value.target == target)
            .collect())
    }

    pub fn delete_notification(&self, id: &str) -> Result<(), Error> {
        self.db.delete(&self.notification_cf(), id)
    }
}
