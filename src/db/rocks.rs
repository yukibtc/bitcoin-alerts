// Copyright (c) 2021-2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::sync::Arc;

pub use rocksdb::{
    BoundColumnFamily, ColumnFamilyDescriptor, DBCompactionStyle, DBCompressionType,
};
use serde::de::DeserializeOwned;
use serde::Serialize;

#[derive(Debug)]
pub enum Error {
    RocksDb(rocksdb::Error),
    FailedToPut,
    FailedToGet,
    FailedToDelete,
    FailedToDeserialize,
    FailedToSerialize,
    ValueNotFound,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RocksDb(e) => write!(f, "{e}"),
            Self::FailedToPut => write!(f, "failed to put data"),
            Self::FailedToGet => write!(f, "failed to get data"),
            Self::FailedToDelete => write!(f, "failed to delete data"),
            Self::FailedToDeserialize => write!(f, "failed to deserialize data"),
            Self::FailedToSerialize => write!(f, "failed to serialize data"),
            Self::ValueNotFound => write!(f, "value not found"),
        }
    }
}

impl From<rocksdb::Error> for Error {
    fn from(err: rocksdb::Error) -> Self {
        Error::RocksDb(err)
    }
}

fn default_opts() -> rocksdb::Options {
    let mut opts = rocksdb::Options::default();
    opts.set_keep_log_file_num(10);
    opts.set_max_open_files(100);
    opts.set_compaction_style(DBCompactionStyle::Level);
    opts.set_compression_type(DBCompressionType::Zstd);
    opts.set_target_file_size_base(256 << 20);
    opts.set_write_buffer_size(256 << 20);
    opts.set_enable_write_thread_adaptive_yield(true);
    opts.set_disable_auto_compactions(true); // for initial bulk load
    opts
}

#[derive(Clone)]
pub struct Store {
    db: Arc<rocksdb::DB>,
}

impl Store {
    pub fn open(path: &Path, column_families: &[&str]) -> Result<Self, Error> {
        tracing::debug!("Opening {}", path.display());
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
            Ok(live_files) => tracing::info!(
                "{}: {} SST files, {} GB, {} Grows",
                path.display(),
                live_files.len(),
                live_files.iter().map(|f| f.size).sum::<usize>() as f64 / 1e9,
                live_files.iter().map(|f| f.num_entries).sum::<u64>() as f64 / 1e9
            ),
            Err(_) => tracing::warn!("Impossible to get live files"),
        };
        Ok(Self { db: Arc::new(db) })
    }

    fn create_cf_descriptors(column_families: &[&str]) -> Vec<ColumnFamilyDescriptor> {
        column_families
            .iter()
            .map(|&name| ColumnFamilyDescriptor::new(name, default_opts()))
            .collect()
    }

    pub fn cf_handle(&self, name: &str) -> Arc<BoundColumnFamily> {
        self.db
            .cf_handle(name)
            .unwrap_or_else(|| panic!("missing {}_CF", name.to_uppercase()))
    }

    pub fn serialize<T>(&self, data: T) -> Result<Vec<u8>, Error>
    where
        T: Serialize + fmt::Debug,
    {
        match serde_json::to_string(&data) {
            Ok(serialized) => Ok(serialized.into_bytes()),
            Err(_) => Err(Error::FailedToSerialize),
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

    pub fn put<K, V>(&self, cf: Arc<BoundColumnFamily>, key: K, value: V) -> Result<(), Error>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        match self.db.put_cf(&cf, key, value) {
            Ok(_) => Ok(()),
            Err(error) => {
                tracing::error!("Impossible to put value in database: {}", error);
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
        V: Serialize + fmt::Debug,
    {
        self.put(cf, key, self.serialize(value)?)
    }

    pub fn iterator(
        &self,
        cf: &Arc<BoundColumnFamily>,
    ) -> Result<HashMap<Vec<u8>, Vec<u8>>, Error> {
        let mut collection = HashMap::new();
        let mut iter = self.db.raw_iterator_cf(cf);
        iter.seek_to_first();
        while iter.valid() {
            if let Some(key) = iter.key() {
                if let Some(value) = iter.value() {
                    collection.insert(key.to_vec(), value.to_vec());
                };
            };
            iter.next();
        }
        Ok(collection)
    }
    pub fn iterator_str_serialized<V>(
        &self,
        cf: Arc<BoundColumnFamily>,
    ) -> Result<HashMap<String, V>, Error>
    where
        V: DeserializeOwned,
    {
        let mut collection = HashMap::new();
        for (key, value) in self.iterator(&cf)?.iter() {
            match String::from_utf8(key.to_vec()) {
                Ok(key) => {
                    match self.deserialize::<V>(value.to_vec()) {
                        Ok(value) => {
                            collection.insert(key, value);
                        }
                        Err(error) => {
                            tracing::error!("Failed to deserialize value: {:?}", error);
                            let _ = self.delete(&cf, key);
                        }
                    };
                }
                Err(error) => tracing::error!("Failed to deserialize key: {:?}", error),
            };
        }
        Ok(collection)
    }

    pub fn delete<K>(&self, cf: &Arc<BoundColumnFamily>, key: K) -> Result<(), Error>
    where
        K: AsRef<[u8]>,
    {
        match self.db.delete_cf(cf, key) {
            Ok(_) => Ok(()),
            Err(error) => {
                tracing::error!("Impossible to delete key from database: {}", error);
                Err(Error::FailedToDelete)
            }
        }
    }
}
