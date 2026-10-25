fmt:
	cargo fmt -- --check

clippy:
	cargo clippy -- -D warnings

test:
	cargo test

check: fmt clippy test

build:
	cargo build

build-release:
	cargo build --release

run:
	cargo run

package:
	cargo package --allow-dirty --list
