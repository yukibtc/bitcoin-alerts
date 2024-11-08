// Copyright (c) 2021-2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;

use bitcoin::network::Network;
use clap::Parser;
use dirs::home_dir;
use nostr_sdk::{Keys, Url};
use ntfy::Auth;
use tracing::Level;

pub mod model;

pub use self::model::Config;
use self::model::{Bitcoin, ConfigFile, Nostr, Ntfy};

fn default_dir() -> PathBuf {
    let home: PathBuf = home_dir().unwrap_or_else(|| {
        tracing::error!("Unknown home directory");
        std::process::exit(1)
    });
    home.join(".bitcoin_alerts")
}

fn default_config_file() -> PathBuf {
    let mut default = default_dir().join("config");
    default.set_extension("toml");
    default
}

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    config_file: Option<PathBuf>,
}

impl Config {
    pub fn from_args() -> Self {
        let args: Args = Args::parse();

        // Read and parse config file
        let config_file_path: PathBuf = args.config_file.unwrap_or_else(default_config_file);
        let config_content = std::fs::read_to_string(config_file_path).unwrap();
        let config_file: ConfigFile = toml::from_str(&config_content).unwrap();

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
            Network::Signet => 38332,
            _ => 18443,
        };

        let folder: &str = match network {
            Network::Bitcoin => "bitcoin",
            Network::Testnet => "testnet",
            Network::Signet => "signet",
            _ => "regtest",
        };

        let main_path: PathBuf = config_file
            .main_path
            .unwrap_or_else(default_dir)
            .join(folder);

        let log_level: Level = match config_file.log_level {
            Some(log_level) => Level::from_str(log_level.as_str()).unwrap_or(Level::INFO),
            None => Level::INFO,
        };

        let ntfy_auth: Option<Auth> = if let Some(username) = config_file.ntfy.username {
            config_file
                .ntfy
                .password
                .map(|password| Auth::credentials(username, password))
        } else {
            None
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
                auth: ntfy_auth,
                proxy: config_file.ntfy.proxy,
            },
            nostr: Nostr {
                enabled: config_file.nostr.enabled.unwrap_or(false),
                keys: config_file.nostr.secret_key.map(Keys::new),
                name: config_file.nostr.name.unwrap_or_else(|| String::from("bitcoin_alerts")),
                display_name: config_file.nostr.display_name.unwrap_or_else(|| String::from("Bitcoin Alerts")),
                description: config_file.nostr.description.unwrap_or_else(|| String::from("Hashrate, supply, blocks until halving, difficulty adjustment and more.\n\nBuilt with https://crates.io/crates/nostr-sdk 🦀")),
                picture: config_file.nostr.picture.unwrap_or_else(|| Url::parse("https://avatars.githubusercontent.com/u/13464320").expect("Invalid url")),
                nip05: config_file.nostr.nip05,
                lud16: config_file.nostr.lud16.unwrap_or_else(|| String::from("yuki@getalby.com")),
                relays: config_file.nostr.relays,
                pow_difficulty: config_file.nostr.pow_difficulty.unwrap_or(0),
            },
        };

        println!("{config:?}");

        config
    }
}
