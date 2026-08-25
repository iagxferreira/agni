# agni

Agni is a Redis-like in-memory cache server written in Rust, and a benchmark project for comparing language implementations of the same core idea.

The goal is twofold:

1. build a small but serious in-memory cache server with clear operational boundaries
2. use the same project shape to compare Rust, Go, and Kotlin as implementation choices

The repository keeps that history intact:

- `main` tracks the Rust implementation and the current benchmark-focused line
- `go-main` preserves the Go baseline
- `kotlin-main` preserves the Kotlin benchmark snapshot

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

## Benchmark Snapshot

Captured on 2026-08-25 on local loopback (`127.0.0.1:6379`) with 50 concurrent connections and 10,000 operations per scenario. The Go run used `go-main` through `mise x go@1.27.0`, and the Rust run used the release binaries from `main`. Both used the same persistent-connection workload shape.

### Throughput

| Scenario | Go (ops/sec) | Rust (ops/sec) | Rust is... |
|---|---|---|---|
| PING | 262,976 | 478,315 | 1.8x faster |
| SET (1000 unique keys) | 193,132 | 466,359 | 2.4x faster |
| GET (hit) | 260,325 | 470,338 | 1.8x faster |
| GET (miss) | 270,682 | 466,458 | 1.7x faster |
| Mixed SET+GET | 201,805 | 451,237 | 2.2x faster |

### Latency

| Scenario | Go p50 | Rust p50 | Go p95 | Rust p95 | Go p99 | Rust p99 |
|---|---|---|---|---|---|---|
| PING | 138.64µs | 95.602µs | 397.289µs | 151.968µs | 613.28µs | 193.405µs |
| SET | 117.383µs | 98.243µs | 780.748µs | 162.034µs | 1.256833ms | 197.647µs |
| GET (hit) | 135.439µs | 98.593µs | 408.492µs | 159.154µs | 689.82µs | 194.81µs |
| GET (miss) | 132.644µs | 98.399µs | 375.776µs | 162.379µs | 648.086µs | 197.545µs |

The mixed SET+GET scenario is throughput-only because the benchmark tool reports percentiles for the primary scenarios above.

Rust leads this snapshot across every measured scenario, especially on writes and tail latency. That makes `main` the right line to harden and optimize further, while `go-main` stays as the comparison baseline.

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
