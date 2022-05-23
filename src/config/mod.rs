// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::net::{SocketAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};

use clap::Parser;
use dirs::home_dir;

pub mod model;

use model::{Bitcoin, ConfigFile, Matrix};

pub use model::Config;

fn default_dir() -> PathBuf {
    let home: PathBuf = home_dir().unwrap_or_else(|| {
        log::error!("Unknown home directory");
        std::process::exit(1)
    });
    home.join(".bitcoin_alerts")
}

fn default_config_file() -> PathBuf {
    let mut default = default_dir().join("config");
    default.set_extension("toml");
    default
}

fn str_to_socketaddr(address: &str, what: &str) -> SocketAddr {
    address
        .to_socket_addrs()
        .unwrap_or_else(|_| panic!("unable to resolve {} address", what))
        .collect::<Vec<_>>()
        .pop()
        .unwrap()
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, parse(from_os_str))]
    config_file: Option<PathBuf>,
}

impl Config {
    pub fn from_args() -> Self {
        let args: Args = Args::parse();

        let config_file_path: PathBuf = match args.config_file {
            Some(path) => path,
            None => default_config_file(),
        };

        let config_file: ConfigFile = match Self::read_config_file(&config_file_path) {
            Ok(data) => data,
            Err(error) => {
                log::error!("Impossible to read config file at {:?}", config_file_path);
                panic!("{}", error);
            }
        };

        let main_path: PathBuf = match config_file.main_path {
            Some(path) => path,
            None => default_dir(),
        };

        let bitcoin_rpc_addr: SocketAddr = match config_file.bitcoin.rpc_addr {
            Some(addr) => addr,
            None => str_to_socketaddr("127.0.0.1:8332", "Bitcoin RPC"),
        };

        let config = Self {
            db_path: main_path.join("db"),
            bitcoin: Bitcoin {
                rpc_addr: bitcoin_rpc_addr,
                rpc_username: config_file.bitcoin.rpc_username,
                rpc_password: config_file.bitcoin.rpc_password,
            },
            matrix: Matrix {
                state_path: main_path.join("matrix/state"),
                homeserver_url: config_file.matrix.homeserver_url,
                proxy: config_file.matrix.proxy,
                user_id: config_file.matrix.user_id,
                password: config_file.matrix.password,
                admins: config_file.matrix.admins,
            },
        };

        log::info!("{:?}", config);

        config
    }

    fn read_config_file(path: &Path) -> std::io::Result<ConfigFile> {
        let content = std::fs::read_to_string(&path)?;
        Ok(toml::from_str(&content)?)
    }
}
