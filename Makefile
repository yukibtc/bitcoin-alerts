all: build

help:
	@echo ""
	@echo "make build             - Build executable"
	@echo "make rust              - Install rust"
	@echo "make precommit         - Execute precommit steps"
	@echo "make loc               - Count lines of code in src folder"
	@echo ""

build:
	cargo build --release

rust:
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

precommit:
	cargo fmt && cargo clippy

clean:
	cargo clean

loc:
	@echo "--- Counting lines of .rs files in 'src' (LOC):" && find src/ -type f -name "*.rs" -exec cat {} \; | wc -l
