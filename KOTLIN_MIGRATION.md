# Kotlin Migration Roadmap

This document tracks the plan to rewrite Agni from Rust to Kotlin. It is a
living plan: update it as decisions change or phases complete.

## Decisions

- **Concurrency model:** Kotlin coroutines + Ktor networking (`ktor-network`).
  Closest analog to the current Tokio design — async TCP sockets with suspend
  functions, one lightweight coroutine per connection instead of
  `tokio::spawn`.
- **Migration strategy:** big-bang rewrite in this repo. Cargo workspace
  members are replaced by Gradle modules of the same name, ported one module
  at a time. The Rust implementation stays in place and green until each
  Kotlin module reaches parity, then the corresponding crate is deleted.
- **Workflow:** each phase below lands as its own worktree branch and PR,
  reviewed and merged before the next phase starts.

## 0. Project scaffolding

Replace the Cargo workspace with a Gradle multi-module build (Kotlin DSL),
mirroring the existing crate boundaries so the migration can go
module-by-module:

```
agni/            → agni-core/   (Gradle module)
agni-server/     → agni-server/
agni-client/     → agni-client/
agni-bench/      → agni-bench/
```

Root `build.gradle.kts` + `settings.gradle.kts` wire the four modules, with a
shared version catalog (`gradle/libs.versions.toml`) for dependency
versions — the Kotlin equivalent of the root `Cargo.toml` workspace members.

## 1. Library mapping

| Rust | Kotlin/JVM equivalent |
|---|---|
| `tokio` (async runtime) | `kotlinx-coroutines-core` |
| `tokio::net::TcpStream/TcpListener` | `io.ktor.network.sockets` (`aSocket(SelectorManager).tcp()`) |
| `tokio_util::codec::LengthDelimitedCodec` | Hand-rolled: `ByteWriteChannel.writeInt(len)` + bytes; `ByteReadChannel.readInt()` + `readFully` |
| `dashmap::DashMap` | `java.util.concurrent.ConcurrentHashMap` |
| `serde` / `serde_json` | `kotlinx.serialization` (`kotlinx-serialization-json`) |
| `serde_yaml` (config) | Jackson (`jackson-dataformat-yaml` + `jackson-module-kotlin`) |
| `clap` (CLI) | `com.github.ajalt.clikt:clikt` |
| `tracing` / `tracing-subscriber` | `kotlin-logging` + `logback-classic` |
| `uuid` crate | `java.util.UUID` (stdlib, no dependency needed) |
| `base64` crate | `kotlin.io.encoding.Base64` (stdlib since Kotlin 1.8) |
| `#[test]` / `#[tokio::test]` | JUnit5 + `kotlinx-coroutines-test` (`runTest`) |

## 2. Port order

Port module by module; keep the Rust crate running until each Kotlin module
is green.

### `agni-core` (first — no I/O dependencies, easiest to validate in isolation)

- `Entry` — data class with `id: UUID`, `key: String`, `value: ByteArray`;
  custom `kotlinx.serialization` serializer that base64-encodes `value`
  (mirrors `serialize_value` in `entry.rs`).
- `Store` — wraps `ConcurrentHashMap<String, Entry>`; same five methods
  (`set`, `get`, `delete`, `getAsJson`). Port as-is — the current Rust
  `Store` has no TTL/expiry either, don't add it early.
- `protocol` — `Command` sealed class (`Ping`, `Healthcheck`, `Get`, `Set`,
  `Unknown`) with a `Command.fromBytes(ByteArray)` factory replicating the
  `splitn(3, ' ')` parsing; `Response` sealed class with `toBytes()`.
- `Config` — data class + `Config.fromFile(path)` using Jackson YAML;
  `ConfigError` sealed class for IO/parse failure.
- Port the existing `#[test]` cases 1:1 (store tests, entry JSON test) as the
  first correctness gate.

### `agni-server`

- `Server` class: `aSocket(SelectorManager(Dispatchers.IO)).tcp().bind(host, port)`.
- `run()`: `for (socket in serverSocket.accept())` loop, `launch { handleConnection(...) }`
  per connection — direct analog of `tokio::spawn`.
- `connection.handle(...)`: read length-prefixed frame → `Command.fromBytes`
  → dispatch against `Store` → write length-prefixed `Response.toBytes()`.
  Keep the **exact same wire format** (4-byte length prefix + plaintext
  command) so client/server stay interoperable mid-migration.
- CLI parsing (`--config`) via clikt; same `warn!`/`info!` log lines via
  kotlin-logging.

### `agni-client`

Clikt CLI, one Ktor socket connection, send joined command string, print
response — thin, ports quickly.

### `agni-bench`

Coroutine `async`/`awaitAll` instead of `tokio::spawn`+`JoinHandle`; keep the
same scenario structure (PING, SET, GET hit/miss, mixed) and percentile math
(`sort()` + index) so before/after throughput numbers in `BENCHMARK.md` stay
comparable.

## 3. Build/deploy plumbing

- `Dockerfile`: replace the Rust multi-stage build with a Gradle build stage
  and a slim JRE runtime stage (e.g. `eclipse-temurin:21-jre-alpine`). Keep
  the same `config.docker.yml` mount convention.
- `Makefile`: swap `cargo run/test/clippy/fmt` targets for
  `./gradlew run/test`, `ktlint`/`detekt` in place of `clippy`/`fmt`.
- `rust-toolchain.toml` → replaced by a Gradle wrapper and a documented JDK
  version (21 LTS, best coroutines/Ktor support).

## 4. Validation gate before deleting Rust

- Kotlin unit tests pass with parity to the existing Rust test names and
  behaviors (`AGENTS.md` convention: name tests by behavior — keep doing
  that).
- Integration test: spin up `agni-server`, hit it with `agni-client` for a
  PING/GET/SET round trip.
- Run `agni-bench` against the Kotlin server, compare p50/p95/p99 and
  throughput against the numbers already in `BENCHMARK.md`. Flag any
  material regression vs Tokio — that's the main technical risk of this
  rewrite.

## 5. Docs to update once the rewrite lands

- `README.md` — Rust 1.97.1 requirement → JDK version, `cargo run` examples
  → `./gradlew run` examples.
- `AGENTS.md` — style section: 4-space/snake_case Rust conventions → Kotlin
  conventions (ktlint), build/test commands.
- `CONTRIBUTING.md` — toolchain setup instructions.
- `CHANGELOG.md` — entry for the rewrite once merged.
