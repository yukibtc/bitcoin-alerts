// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use bitcoin::network::constants::Network;
use clap::Parser;
use dirs::home_dir;
use log::Level;
use nostr_sdk::nostr::key::{FromBech32, Keys};
use secp256k1::SecretKey;

pub mod model;

pub use self::model::Config;
use self::model::{Bitcoin, ConfigFile, Matrix, Nostr, Ntfy};

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

        let network: Network = match config_file.bitcoin.network {
            Some(network_str) => match Network::from_str(network_str.as_str()) {
                Ok(network) => network,
                Err(_) => panic!("Invalid bitcoin network selected in config file."),
            },
            None => Network::Bitcoin,
        };

        let default_bitcoin_rpc_port: u16 = match network {
            Network::Bitcoin => 8332,
            Network::Testnet => 18332,
            Network::Regtest => 18443,
            Network::Signet => 38332,
        };

        let folder: &str = match network {
            Network::Bitcoin => "bitcoin",
            Network::Testnet => "testnet",
            Network::Regtest => "regtest",
            Network::Signet => "signet",
        };

        let main_path: PathBuf = config_file
            .main_path
            .unwrap_or_else(default_dir)
            .join(folder);

        let log_level: Level = match config_file.log_level {
            Some(log_level) => Level::from_str(log_level.as_str()).unwrap_or(Level::Info),
            None => Level::Info,
        };

        let keys: Keys = match Keys::from_bech32(&config_file.nostr.secret_key) {
            Ok(keys) => keys,
            Err(_) => match SecretKey::from_str(&config_file.nostr.secret_key) {
                Ok(secret_key) => Keys::new(secret_key),
                Err(_) => panic!("Invalid secret key"),
            },
        };

        let config = Self {
            main_path: main_path.clone(),
            log_level,
            bitcoin: Bitcoin {
                network,
                rpc_addr: config_file.bitcoin.rpc_addr.unwrap_or_else(|| {
                    SocketAddr::new(
                        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                        default_bitcoin_rpc_port,
                    )
                }),
                rpc_username: config_file.bitcoin.rpc_username,
                rpc_password: config_file.bitcoin.rpc_password,
                db_path: main_path.join("chainstate"),
            },
            ntfy: Ntfy {
                enabled: config_file.ntfy.enabled.unwrap_or(false),
                url: config_file.ntfy.url.unwrap_or_default(),
                topic: config_file
                    .ntfy
                    .topic
                    .unwrap_or_else(|| String::from("bitcoin_alerts")),
                proxy: config_file.ntfy.proxy,
            },
            nostr: Nostr {
                enabled: config_file.nostr.enabled.unwrap_or(false),
                keys,
                relays: config_file.nostr.relays,
                pow_enabled: config_file.nostr.pow_enabled.unwrap_or(false),
                pow_difficulty: config_file.nostr.pow_difficulty.unwrap_or(20),
            },
            matrix: Matrix {
                enabled: config_file.matrix.enabled.unwrap_or(false),
                homeserver_url: config_file.matrix.homeserver_url.unwrap_or_default(),
                proxy: config_file.matrix.proxy,
                user_id: config_file.matrix.user_id.unwrap_or_default(),
                password: config_file.matrix.password.unwrap_or_default(),
                admins: config_file.matrix.admins.unwrap_or_default(),
                db_path: main_path.join("matrix/db"),
                state_path: main_path.join("matrix/state"),
            },
        };

        println!("{:?}", config);

        config
    }

    fn read_config_file(path: &Path) -> std::io::Result<ConfigFile> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }
}
