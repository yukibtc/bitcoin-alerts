# Bitcoin Alerts

## Description

Bitcoin Alerts Bot for Matrix

This project is in an ALPHA state.

## Requirements

- [Rust (1.57.0+)](https://rustup.rs/)
- [Matrix account](https://matrix.org)

### Linux

If you are using linux, you must also install the following packages:

```
sudo apt update
sudo apt install clang cmake build-essential pkg-config libssl-dev
```

## Build

```
cargo build --release
```

You will find the executable file in the `target/release` folder with name `bitcoin-alerts`.

## Configuration

### Config file

Copy `docs/config-example.toml` file, rename to `config.toml`, edit with your settings and then move to `~/.bitcoin_alerts/config.toml` for Linux and MacOS or `C:\Users\YOUR_USERNAME\.bitcoin_alerts\config.toml` for Windows.

## Execution

To run BPNS, execute `bitcoin-alerts` file in `target/release` folder.