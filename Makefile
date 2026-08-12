SHELL := /bin/sh

.PHONY: help run-server run-client check test fmt clippy build-release bench-build

help:
	@printf '%s\n' \
		'agni targets:' \
		'  make run-server    - run the TCP server with config.example.yml' \
		'  make run-client    - run the CLI client, pass ARGS="PING"' \
		'  make check         - type-check the workspace' \
		'  make test          - run the workspace tests' \
		'  make fmt           - format the codebase' \
		'  make clippy        - run clippy on all targets' \
		'  make build-release - build the workspace in release mode' \
		'  make bench-build   - build the server and bench binaries in release mode'

run-server:
	cargo run -p agni-server -- --config config.example.yml

run-client:
	cargo run -p agni-client -- $(ARGS)

check:
	cargo check

test:
	cargo test

fmt:
	cargo fmt

clippy:
	cargo clippy --all-targets --all-features

build-release:
	cargo build --release

bench-build:
	cargo build --release -p agni-server -p agni-bench
