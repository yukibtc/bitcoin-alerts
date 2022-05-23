// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use rocksdb::{BoundColumnFamily, ColumnFamilyDescriptor, DBCompactionStyle, DBCompressionType};
use serde::{de::DeserializeOwned, Serialize};

#[derive(Clone)]
pub struct Store {
    db: Arc<rocksdb::DB>,
}

#[derive(PartialEq, Debug)]
pub enum Error {
    FailedToPut,
    FailedToGet,
    FailedToDelete,
    FailedToDeserialize,
    FailedToSerialize,
    ValueNotFound,
}

fn default_opts() -> rocksdb::Options {
    let mut opts = rocksdb::Options::default();
    opts.set_max_open_files(100);
    opts.set_compaction_style(DBCompactionStyle::Level);
    opts.set_compression_type(DBCompressionType::Zstd);
    opts.set_target_file_size_base(256 << 20);
    opts.set_write_buffer_size(256 << 20);
    opts.set_enable_write_thread_adaptive_yield(true);
    opts
}

impl Store {
    fn create_cf_descriptors(column_families: &[&str]) -> Vec<ColumnFamilyDescriptor> {
        column_families
            .iter()
            .map(|&name| ColumnFamilyDescriptor::new(name, default_opts()))
            .collect()
    }

    pub fn open(path: &Path, column_families: &[&str]) -> Result<Self, Error> {
        let mut db_opts = default_opts();
        db_opts.create_if_missing(true);
        db_opts.create_missing_column_families(true);

        let db = match rocksdb::DB::open_cf_descriptors(
            &db_opts,
            path,
            Self::create_cf_descriptors(column_families),
        ) {
            Ok(data) => data,
            Err(error) => panic!("{:?}", error),
        };

        match db.live_files() {
            Ok(live_files) => log::info!(
                "Database: {} SST files, {} GB, {} Grows",
                live_files.len(),
                live_files.iter().map(|f| f.size).sum::<usize>() as f64 / 1e9,
                live_files.iter().map(|f| f.num_entries).sum::<u64>() as f64 / 1e9
            ),
            Err(_) => log::warn!("Impossible to get live files"),
        };

        let store = Self { db: Arc::new(db) };
        Ok(store)
    }

    pub fn cf_handle(&self, name: &str) -> Arc<BoundColumnFamily> {
        self.db
            .cf_handle(name)
            .unwrap_or_else(|| panic!("missing {}_CF", name.to_uppercase()))
    }

    pub fn get<K>(&self, cf: Arc<BoundColumnFamily>, key: K) -> Result<Vec<u8>, Error>
    where
        K: AsRef<[u8]>,
    {
        match self.db.get_pinned_cf(&cf, key) {
            Ok(Some(value)) => Ok(value.to_vec()),
            Ok(None) => Err(Error::ValueNotFound),
            Err(_) => Err(Error::FailedToGet),
        }
    }

    pub fn deserialize<T>(&self, data: Vec<u8>) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        match serde_json::from_slice::<T>(&data) {
            Ok(u) => Ok(u),
            Err(_) => Err(Error::FailedToDeserialize),
        }
    }

    pub fn get_serialized<K, V>(&self, cf: Arc<BoundColumnFamily>, key: K) -> Result<V, Error>
    where
        K: AsRef<[u8]>,
        V: DeserializeOwned,
    {
        match self.db.get_cf(&cf, key) {
            Ok(opt) => match opt {
                Some(found) => self.deserialize::<V>(found),
                None => Err(Error::ValueNotFound),
            },
            Err(_) => Err(Error::FailedToGet),
        }
    }

    pub fn put<K, V>(&self, cf: Arc<BoundColumnFamily>, key: K, value: V) -> Result<(), Error>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        match self.db.put_cf(&cf, key, value) {
            Ok(_) => Ok(()),
            Err(error) => {
                log::error!("Impossible to put value in database: {}", error);
                Err(Error::FailedToPut)
            }
        }
    }

    pub fn put_serialized<K, V>(
        &self,
        cf: Arc<BoundColumnFamily>,
        key: K,
        value: &V,
    ) -> Result<(), Error>
    where
        K: AsRef<[u8]>,
        V: Serialize + std::fmt::Debug,
    {
        match serde_json::to_string(&value) {
            Ok(serialized) => self.put(cf, key, serialized.into_bytes()),
            Err(_) => Err(Error::FailedToSerialize),
        }
    }

    pub fn iterator_serialized<T>(
        &self,
        cf: Arc<BoundColumnFamily>,
    ) -> Result<HashMap<String, T>, Error>
    where
        T: DeserializeOwned,
    {
        let mut collection = HashMap::new();
        let mut iter = self.db.raw_iterator_cf(&cf);

        iter.seek_to_first();
        while iter.valid() {
            if let Some(key_bytes) = iter.key() {
                match String::from_utf8(key_bytes.to_vec()) {
                    Ok(key) => {
                        if let Some(value_bytes) = iter.value() {
                            match self.deserialize::<T>(value_bytes.to_vec()) {
                                Ok(value) => {
                                    collection.insert(key, value);
                                }
                                Err(error) => {
                                    log::error!("Failed to deserialize value: {:#?}", error);
                                    let _ = self.delete(cf.clone(), key.as_str());
                                }
                            };
                        };
                    }
                    Err(error) => log::error!("Failed to convert key to string: {:#?}", error),
                };
            };
            iter.next();
        }

        Ok(collection)
    }

    pub fn delete<K>(&self, cf: Arc<BoundColumnFamily>, key: K) -> Result<(), Error>
    where
        K: AsRef<[u8]>,
    {
        match self.db.delete_cf(&cf, key) {
            Ok(_) => Ok(()),
            Err(error) => {
                log::error!("Impossible to delete key from database: {}", error);
                Err(Error::FailedToDelete)
            }
        }
    }

    /* pub fn flush(&self) {
        match self.db.flush() {
            Ok(_) => log::info!("Database flushed"),
            Err(error) => log::error!("Impossible to flush database: {}", error),
        };
    } */
}

impl Drop for Store {
    fn drop(&mut self) {
        log::trace!("Closing Database");
    }
}
