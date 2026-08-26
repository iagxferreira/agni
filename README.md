# agni

Agni is a Redis-like in-memory cache server written in Kotlin.

## Requirements

- JDK 21

## Workspace

- `agni-core/` core library for store, protocol, and command logic
- `agni-server/` TCP server binary
- `agni-client/` CLI client binary
- `agni-bench/` benchmark binary

## Getting Started

```bash
./gradlew :agni-server:run --args="--config config.example.yml"
./gradlew :agni-client:run --args="PING"
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
- [BENCHMARK.md](BENCHMARK.md) performance methodology and results
- [KOTLIN_MIGRATION.md](KOTLIN_MIGRATION.md) plan for the Kotlin rewrite

## Development

Use the `Makefile` for common local commands:

```bash
make run-server
make run-client CMD="PING"
make test
make check
```

## Library Use

```kotlin
dependencies {
    implementation("dev.agni:agni-core:0.1.0")
}
```

```kotlin
import dev.agni.core.store.Store

val store = Store()
store.set("key", "value".toByteArray())
```

## Roadmap

- [x] Core commands: `PING`, `GET`, `SET`
- [ ] Remaining commands: `DEL`, `EXISTS`, `EXPIRE`, `TTL`
- [ ] TTL and background expiry cleanup
- [ ] Persistence
- [ ] Additional data types: lists, hashes, sets
