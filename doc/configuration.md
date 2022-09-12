# CONFIGURATION

## Config file

Copy `config-example.toml` file (it's in doc folder), rename to `config.toml`, edit with your settings and then move to `~/.bitcoin_alerts/config.toml`.

## Bitcoin

You must set RPC credentials in your `bitcoin.conf` file. Also, add `coinstatsindex=1` to reduce call time of `gettxoutsetinfo` request.