all: build

build:
	cargo build --release

check:
	cargo check

rust:
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

precommit:
	cargo fmt && cargo clippy

clean:
	cargo clean

loc:
	@echo "--- Counting lines of .rs files in 'src' (LOC):" && find src/ -type f -name "*.rs" -exec cat {} \; | wc -l
