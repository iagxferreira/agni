SHELL := /bin/sh

.PHONY: help run-server run-client check test build bench-build docker-build

help:
	@printf '%s\n' \
		'agni targets:' \
		'  make run-server   - run the TCP server with config.example.yml' \
		'  make run-client   - run the CLI client, pass CMD="PING"' \
		'  make check        - vet and build without running tests' \
		'  make test         - run the test suite (race detector)' \
		'  make build        - build all binaries into bin/' \
		'  make bench-build  - build the server and bench binaries into bin/' \
		'  make docker-build - build the agni-server Docker image'

run-server:
	go run ./cmd/agni-server --config config.example.yml

run-client:
	go run ./cmd/agni-client $(CMD)

check:
	go vet ./...
	go build ./...

test:
	go test -race ./...

build:
	mkdir -p bin
	go build -o bin/agni-server ./cmd/agni-server
	go build -o bin/agni-client ./cmd/agni-client
	go build -o bin/agni-bench ./cmd/agni-bench

bench-build:
	mkdir -p bin
	go build -o bin/agni-server ./cmd/agni-server
	go build -o bin/agni-bench ./cmd/agni-bench

docker-build:
	docker build -t agni -f Dockerfile .
