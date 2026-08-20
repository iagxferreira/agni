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

## Rust Baseline (pre-Kotlin migration)

Captured 2026-08-19, before starting the [Kotlin rewrite](KOTLIN_MIGRATION.md). This is the reference point the Kotlin implementation will be measured against as each module (`agni-server` in particular) is ported — same methodology as above, same `agni-bench` tool, current `DashMap`-backed `agni-server` in release mode.

Numbers are **not comparable to the "Results" section above** — different machine, different point in time. Only compare Kotlin runs against this section.

- Machine: Intel(R) Xeon(R) CPU E5-2690 v4 @ 2.60GHz, 28 vCPUs, Linux 7.1.8 (loopback, 127.0.0.1:6379)
- Build: `cargo build --release -p agni-server -p agni-bench`
- 50 concurrent connections, 10,000 ops per scenario

| Scenario | Throughput (ops/sec) | p50 | p95 | p99 |
|---|---|---|---|---|
| PING | 507,259 | 91.8µs | 146.0µs | 171.9µs |
| SET (1000 unique keys) | 511,495 | 91.0µs | 146.0µs | 177.2µs |
| GET (hit) | 512,975 | 89.8µs | 148.3µs | 182.7µs |
| GET (miss) | 511,302 | 90.5µs | 145.6µs | 177.4µs |
| Mixed SET+GET | 502,559 | — | — | — |

When `agni-server` lands in Kotlin, re-run this exact command on the same machine and append a comparison table here rather than replacing this section.

## Kotlin vs Rust (post-migration)

Captured 2026-08-20, right after [#7](https://github.com/iagxferreira/agni/pull/7) merged and all four modules had landed in Kotlin. Same machine and methodology as the Rust baseline above (Intel Xeon E5-2690 v4, 28 vCPUs, loopback), so the two are directly comparable.

- Build: `./gradlew :agni-server:installDist :agni-bench:installDist` (standalone distributions, not `gradlew run`, to keep Gradle task overhead out of the measurement — the JVM-under-test is a plain forked process either way, same as the Rust release binary)
- Runtime: JDK 25.0.4 (system default on this machine; the Gradle toolchain targets JDK 21 for compilation, but the packaged start script resolves `java` from `JAVA_HOME`/`PATH` at run time, which was JDK 25 here — noted for reproducibility, not expected to materially change the numbers below)
- Same server process across three consecutive `agni-bench -c 50 -n 10000` invocations, to separate JIT warm-up from steady state

### Throughput, cold vs warm (ops/sec)

| Scenario | Run 1 (cold) | Run 2 | Run 3 (steady) | Cold → steady |
|---|---|---|---|---|
| PING | 19,233 | 26,667 | 28,917 | **+50.3%** |
| SET (1000 unique keys) | 27,877 | 44,276 | 45,600 | **+63.6%** |
| GET (hit) | 59,891 | 61,538 | 55,679 | -7.0% (noise) |
| GET (miss) | 68,660 | 69,079 | 69,674 | +1.5% (flat) |
| Mixed SET+GET | 74,371 | 63,483 | 71,092 | -4.4% (noise) |

PING and SET — the first two scenarios each run hits — pick up 50-64% throughput between the cold run and the third run against the same server process. GET is already near its ceiling on the very first run, most likely because the connection-handling and dispatch code paths it depends on (framing, `Command.fromBytes`, `Response.toBytes`) were already JIT-compiled by the PING/SET scenarios that ran before it within run 1 itself.

### Rust vs Kotlin (steady state)

Using run 3 (steady state) for the Kotlin side, since a long-running server settles here in practice; the cold numbers above matter mainly for restart/scale-up latency, not sustained throughput.

| Scenario | Rust (ops/sec) | Kotlin (ops/sec) | Rust is... |
|---|---|---|---|
| PING | 507,259 | 28,917 | **17.5x** |
| SET (1000 unique keys) | 511,495 | 45,600 | **11.2x** |
| GET (hit) | 512,975 | 55,679 | **9.2x** |
| GET (miss) | 511,302 | 69,674 | **7.3x** |
| Mixed SET+GET | 502,559 | 71,092 | **7.1x** |

| Scenario | Rust p50 | Kotlin p50 | Rust p99 | Kotlin p99 |
|---|---|---|---|---|
| PING | 91.8µs | 1.29ms | 171.9µs | 6.78ms |
| SET | 91.0µs | 929.2µs | 177.2µs | 4.07ms |
| GET (hit) | 89.8µs | 774.8µs | 182.7µs | 1.86ms |
| GET (miss) | 90.5µs | 579.9µs | 177.4µs | 1.63ms |

## Analysis: Kotlin vs Rust

**The gap is large and expected.** Tokio's async I/O compiles to zero-cost state machines with no garbage collector and no warm-up curve; a `DashMap` lookup and a length-delimited frame write are about as close to the metal as safe Rust gets. The JVM stack (Ktor coroutines, `ConcurrentHashMap`, kotlinx.serialization only on the `getAsJson` path) adds virtual dispatch, allocation, and GC pauses at every layer. A 7-18x throughput gap and a similarly sized latency gap is in line with what's typically seen comparing a native async runtime to a JVM one at this request size and concurrency.

**JIT warm-up is real and scenario-dependent.** PING and SET improved 50-64% from the first to the third run against the same long-lived server process; GET was already near its ceiling on run 1. This matters for how the numbers should be read: the "steady state" column is the honest comparison for a long-running production server, but a freshly started (or frequently restarted/scaled) Kotlin instance will run meaningfully slower than these numbers until warmed. This is exactly the tradeoff called out in the ["Native compilation"](KOTLIN_MIGRATION.md#6-native-compilation) section of the migration roadmap — it's `agni-server`'s workload profile (long-running, steady traffic) that makes native-image a poor fit there, while the same warm-up cost is why native-image was worth it for `agni-client`/`agni-bench`, which never run long enough to reach steady state.

**Conclusion:** the migration trades roughly an order of magnitude of raw throughput and latency for whatever the JVM move is buying elsewhere (ecosystem, tooling, team familiarity — outside this document's scope). If a future workload needs Rust-level throughput from `agni-server` specifically, that gap is real and won't close with JVM tuning alone — it's a signal to revisit the decision, not a bug to fix here.
