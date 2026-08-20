SHELL := /bin/sh

.PHONY: help run-server run-client check test build bench-build

help:
	@printf '%s\n' \
		'agni targets:' \
		'  make run-server  - run the TCP server with config.example.yml' \
		'  make run-client  - run the CLI client, pass CMD="PING"' \
		'  make check       - compile the workspace without running tests' \
		'  make test        - run the workspace tests' \
		'  make build       - build the full workspace' \
		'  make bench-build - build the server and bench distributions'

run-server:
	./gradlew :agni-server:run --args="--config config.example.yml"

run-client:
	./gradlew :agni-client:run --args="$(CMD)"

check:
	./gradlew classes testClasses

test:
	./gradlew test

build:
	./gradlew build

bench-build:
	./gradlew :agni-server:installDist :agni-bench:installDist
