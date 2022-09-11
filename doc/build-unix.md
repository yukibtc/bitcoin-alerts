# BUILD FOR UNIX

## Introduction

Before build, see [build requirements](#build-requirements) for your specific platform.

## Install Rust

If you have already installed Rust on your device you can skip to the next step, otherwise:

```
make rust
```

## Build

Compile binary from source code:

```
make
```

You will find the executable file in the `target/release` folder with name `bitcoin-alerts`.

## Build requirements

### Ubuntu & Debian

```
sudo apt install build-essential clang cmake libssl-dev pkg-config
```

### Fedora

```
sudo dnf group install "C Development Tools and Libraries" "Development Tools"
```

```
sudo dnf install clang cmake
```

### MacOS

```
brew install cmake
```