// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;
use std::net::SocketAddr;
use std::path::PathBuf;

use bitcoincore_rpc::Auth;

pub struct Bitcoin {
    pub rpc_addr: SocketAddr,
    pub rpc_auth: Auth,
}

#[derive(Deserialize)]
pub struct ConfigFileBitcoin {
    pub rpc_addr: Option<SocketAddr>,
    pub rpc_username: String,
    pub rpc_password: String,
}

pub struct Ntfy {
    pub enabled: bool,
    pub url: String,
    pub topic: String,
    pub proxy: Option<String>,
}

#[derive(Deserialize)]
pub struct ConfigFileNtfy {
    pub enabled: Option<bool>,
    pub url: Option<String>,
    pub topic: Option<String>,
    pub proxy: Option<String>,
}

pub struct Matrix {
    pub state_path: PathBuf,
    pub homeserver_url: String,
    pub proxy: Option<String>,
    pub user_id: String,
    pub password: String,
    pub admins: Vec<String>,
}

#[derive(Deserialize)]
pub struct ConfigFileMatrix {
    pub homeserver_url: String,
    pub proxy: Option<String>,
    pub user_id: String,
    pub password: String,
    pub admins: Vec<String>,
}

#[derive(Debug)]
pub struct Config {
    pub db_path: PathBuf,
    pub bitcoin: Bitcoin,
    pub ntfy: Ntfy,
    pub matrix: Matrix,
}

#[derive(Deserialize)]
pub struct ConfigFile {
    pub main_path: Option<PathBuf>,
    pub bitcoin: ConfigFileBitcoin,
    pub ntfy: ConfigFileNtfy,
    pub matrix: ConfigFileMatrix,
}

impl fmt::Debug for Bitcoin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ rpc_addr: {:?} }}", self.rpc_addr)
    }
}

impl fmt::Debug for Ntfy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ url: {:?}, topic: {}, proxy: {:?} }}",
            self.url, self.topic, self.proxy
        )
    }
}

impl fmt::Debug for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ state_path: {:?}, homeserver_url: {}, proxy: {:?}, user_id: {}, admins: {:?} }}",
            self.state_path, self.homeserver_url, self.proxy, self.user_id, self.admins
        )
    }
}
