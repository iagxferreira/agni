# agni

Agni is a Redis-like in-memory cache server written in Rust, and a benchmark project for comparing language implementations of the same core idea.

The goal is twofold:

1. build a small but serious in-memory cache server with clear operational boundaries
2. use the same project shape to compare Rust, Go, and Kotlin as implementation choices

The repository keeps that history intact. Every implementation lives in `main`'s
own history rather than on separate branches, so a single clone carries all of it:

- `main` tracks the Rust implementation and the current benchmark-focused line
- the Go implementation is preserved at tag `snapshot/go`
- the Kotlin implementation is preserved at tag `snapshot/kotlin`

Check either out with `git checkout snapshot/go` to get that language's full tree.

That makes the project useful both as a cache server and as a portfolio piece for showing how the same workload behaves across languages, runtimes, and trade-offs.

## Requirements

- Rust 1.97.1

## Workspace

- `agni/` core library for store, protocol, and command logic
- `agni-server/` TCP server binary
- `agni-client/` CLI client binary
- `agni-bench/` benchmark binary

## Scope

Agni is intentionally small in scope. The current focus is:

- a TCP cache server with a simple request/response protocol
- in-memory storage with predictable behavior
- client and benchmark tools that exercise the same wire protocol as the server
- performance measurement that is explicit about methodology and comparable across implementations

What is intentionally out of scope for now:

- replication and clustering
- persistence and recovery
- multi-node coordination
- advanced Redis module compatibility

## Current Direction

`main` is the Rust line. It exists to show how far we can push a small cache server with a strong async/runtime story, good tests, and honest benchmarks.

The older Go and Kotlin histories are preserved as reference points, not deleted. That way the repository can answer a useful interview question: what changes when the same cache workload is implemented in different languages?

## Getting Started

```bash
cargo run -p agni-server -- --config config.example.yml
cargo run -p agni-client -- PING
```

## Docker

```bash
docker build -t agni .
docker run -p 6379:6379 agni
```

For custom config, mount a file at `/etc/agni/config.yml`. Inside the container, `host` must be `0.0.0.0`.

## Documentation

- [AGENTS.md](AGENTS.md) repo conventions and agent workflow
- [CONTRIBUTING.md](CONTRIBUTING.md) how to contribute
- [CHANGELOG.md](CHANGELOG.md) shipped changes
- [BENCHMARK.md](BENCHMARK.md) performance methodology, results, and comparison notes

## Development

Use the `Makefile` for common local commands:

```bash
make run-server
make run-client CMD="PING"
make test
make clippy
```

## Library Use

```toml
[dependencies]
agni = "0.1"
```

```rust
use agni::store::Store;

let store = Store::new();
store.set("key".to_string(), b"value".to_vec());
```

## Roadmap

- [x] Core commands: `PING`, `GET`, `SET`
- [ ] Response and protocol hardening
- [ ] TTL and background expiry cleanup
- [ ] Better memory limits and abuse controls
- [ ] Observability and operational polish
- [ ] Benchmarks for steady-state and cold-start behavior
- [ ] Additional data types: lists, hashes, sets
