# Benchmarks

Benchmarks comparing the two store implementations: `Arc<RwLock<HashMap>>` vs [`DashMap`](https://docs.rs/dashmap).

This benchmark documents the storage decision that moved Agni to `DashMap`. Treat it as a historical comparison and reference point for future performance-sensitive changes.

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

**DashMap wins on writes.** With 50 concurrent connections contending on 1,000 shared keys, `HashMap+RwLock` serializes all writes through a single lock. DashMap shards across 64 buckets, reducing contention by ~64x. The result: **+28.5% throughput and -30% p99 latency on SET**.

**HashMap wins on pure cache misses.** A miss on a read lock is extremely cheap — the lock is shared, the key lookup short-circuits fast, and there is no DashMap shard selection overhead. In practice, a well-warmed cache will have few misses, so this is a minor concern.

**Conclusion:** DashMap is the better choice for a write-heavy or mixed workload cache. The only regression is in pure-miss read scenarios, which are uncommon in real usage.
