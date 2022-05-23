// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Bitcoin {
    pub rpc_addr: SocketAddr,
    pub rpc_username: String,
    pub rpc_password: String,
}

#[derive(Deserialize)]
pub struct ConfigFileBitcoin {
    pub rpc_addr: Option<SocketAddr>,
    pub rpc_username: String,
    pub rpc_password: String,
}

#[derive(Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct Config {
    pub db_path: PathBuf,
    pub bitcoin: Bitcoin,
    pub matrix: Matrix,
}

#[derive(Deserialize)]
pub struct ConfigFile {
    pub main_path: Option<PathBuf>,
    pub bitcoin: ConfigFileBitcoin,
    pub matrix: ConfigFileMatrix,
}

impl fmt::Debug for Bitcoin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ rpc_addr: {:?}, rpc_username: {} }}",
            self.rpc_addr, self.rpc_username
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
