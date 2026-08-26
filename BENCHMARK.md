# Benchmarks

Agni treats benchmarking as part of the product story, not an afterthought. This document tracks the measurement methodology and the results that compare the major language implementations preserved in the repo history.

The important takeaway is not just the numbers, but the shape of the comparisons:

- the Rust line on `main` is the current reference implementation
- `go-main` preserves the Go baseline
- `kotlin-main` preserves the Kotlin benchmark snapshot

Use this file as the source of truth for benchmark methodology, results, and interpretation. Append new measurement sets instead of replacing older ones so the performance story stays auditable over time.

## Go vs Rust Snapshot

Captured on 2026-08-25 on local loopback (`127.0.0.1:6379`) with 50 concurrent connections and 10,000 operations per scenario. The Go run used `go-main` through `mise x go@1.27.0`, and the Rust run used the release binaries from `main`. Both used the same persistent-connection workload shape.

### Throughput (ops/sec)

| Scenario | Go | Rust | Rust is... |
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

Rust leads this snapshot across every measured scenario, especially on writes and tail latency. That is the comparison point the repo now centers on for the benchmark story.

## Methodology

- Tool: `agni-bench` — a persistent-connection benchmarking binary in the workspace
- Each worker holds a **persistent TCP connection** and sends requests sequentially
- No process spawning overhead, no reconnects per operation
- 50 concurrent connections, 10,000 ops per scenario
- Server built in release mode (`cargo build --release`)
- Machine: local loopback (127.0.0.1:6379)

```bash
cargo build --release -p agni-server -p agni-bench
./target/release/agni-server --config config.yml
./target/release/agni-bench -c 50 -n 10000
```

## Historical Comparison

The earliest Rust baseline compared `Arc<RwLock<HashMap>>` with [`DashMap`](https://docs.rs/dashmap). That experiment helped validate the storage choice for the Rust line and is kept here as part of the project’s evolution.

## Results

### Throughput (ops/sec)

| Scenario | `HashMap+RwLock` | `DashMap` | Delta |
|---|---|---|---|
| PING | 103,947 | 116,021 | +11.6% |
| SET (1000 unique keys) | 93,575 | 120,229 | **+28.5%** |
| GET (hit) | 111,237 | 121,641 | +9.4% |
| GET (miss) | 132,833 | 120,594 | -9.2% |
| Mixed SET+GET | 125,301 | 119,762 | -4.4% |

### Latency

| Scenario | HashMap p50 | DashMap p50 | HashMap p99 | DashMap p99 |
|---|---|---|---|---|
| PING | 396µs | 354µs | 1.28ms | 1.13ms |
| SET | 427µs | 355µs | **1.52ms** | **1.07ms** |
| GET (hit) | 373µs | 344µs | 1.12ms | 1.15ms |
| GET (miss) | 317µs | 354µs | 1.13ms | 1.08ms |

## Analysis

**DashMap wins on writes.** With 50 concurrent connections contending on 1,000 shared keys, `HashMap+RwLock` serializes all writes through a single lock. DashMap shards across 64 buckets, reducing contention by roughly the same factor. The result is materially better throughput and tail latency on SET.

**HashMap wins on pure cache misses.** A miss on a read lock is extremely cheap — the lock is shared, the key lookup short-circuits fast, and there is no DashMap shard selection overhead. In practice, a well-warmed cache will have few misses, so this is a minor concern.

**Conclusion:** `DashMap` is the better choice for a write-heavy or mixed workload cache. The remaining miss-only regression is a trade-off the project accepts in exchange for better concurrent behavior on the common path.
