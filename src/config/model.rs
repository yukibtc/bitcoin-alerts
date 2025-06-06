// Copyright (c) 2021-2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::collections::HashSet;
use std::fmt;
use std::net::SocketAddr;
use std::path::PathBuf;

use bitcoin::network::Network;
use nostr_sdk::{Keys, RelayUrl, SecretKey, Url};
use ntfy::Auth;
use tracing::Level;

#[derive(Clone)]
pub struct Bitcoin {
    pub network: Network,
    pub rpc_addr: SocketAddr,
    pub rpc_username: String,
    pub rpc_password: String,
    pub db_path: PathBuf,
}

#[derive(Deserialize)]
pub struct ConfigFileBitcoin {
    pub network: Option<String>,
    pub rpc_addr: Option<SocketAddr>,
    pub rpc_username: String,
    pub rpc_password: String,
}

#[derive(Clone)]
pub struct Ntfy {
    pub enabled: bool,
    pub url: String,
    pub topic: String,
    pub auth: Option<Auth>,
    pub proxy: Option<String>,
}

#[derive(Deserialize)]
pub struct ConfigFileNtfy {
    pub enabled: Option<bool>,
    pub url: Option<String>,
    pub topic: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub proxy: Option<String>,
}

#[derive(Clone)]
pub struct Nostr {
    pub enabled: bool,
    pub keys: Option<Keys>,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub picture: Url,
    pub nip05: Option<String>,
    pub lud16: String,
    pub relays: HashSet<RelayUrl>,
    pub pow_difficulty: u8,
}

#[derive(Deserialize)]
pub struct ConfigFileNostr {
    pub enabled: Option<bool>,
    pub secret_key: Option<SecretKey>,
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub picture: Option<Url>,
    pub nip05: Option<String>,
    pub lud16: Option<String>,
    pub relays: Option<HashSet<RelayUrl>>,
    pub pow_difficulty: Option<u8>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub main_path: PathBuf,
    pub log_level: Level,
    pub bitcoin: Bitcoin,
    pub ntfy: Ntfy,
    pub nostr: Nostr,
}

#[derive(Deserialize)]
pub struct ConfigFile {
    pub main_path: Option<PathBuf>,
    pub log_level: Option<String>,
    pub bitcoin: ConfigFileBitcoin,
    pub ntfy: ConfigFileNtfy,
    pub nostr: ConfigFileNostr,
}

impl fmt::Debug for Bitcoin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ network: {}, rpc_addr: {:?}, rpc_username: {} }}",
            self.network, self.rpc_addr, self.rpc_username
        )
    }
}

impl fmt::Debug for Ntfy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ enabled: {}, url: {:?}, topic: {}, credentials: {}, proxy: {:?} }}",
            self.enabled,
            self.url,
            self.topic,
            self.auth.is_some(),
            self.proxy
        )
    }
}

impl fmt::Debug for Nostr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ enabled: {}, relays: [{}], pow_difficulty: {} }}",
            self.enabled,
            self.relays
                .iter()
                .map(|u| u.as_str())
                .collect::<Vec<_>>()
                .join(","),
            self.pow_difficulty
        )
    }
}
