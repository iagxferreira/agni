# Repository Guidelines

Agni is a Kotlin/Gradle workspace for an in-memory cache server, client, and benchmark tools. Keep changes small, testable, and scoped to the owning module.

JDK 21 is the minimum supported toolchain for the workspace (pinned via `jvmToolchain(21)` in each module; Gradle auto-provisions it if it isn't installed locally).

## Project Structure

- `agni-core/` is the core library with store, protocol, and config logic.
- `agni-server/` is the TCP server binary.
- `agni-client/` is the CLI client binary.
- `agni-bench/` is the benchmarking binary.
- `README.md` stays high level. `BENCHMARK.md` records performance results.

Prefer moving shared logic into `agni-core/` and keeping binaries thin.

## Build And Test

- `make test` runs the workspace tests.
- `make check` compiles the workspace without running tests.
- `make run-server` starts the server locally.
- `make run-client CMD="PING"` sends a command to a running server.

Use distributions for benchmark work:

- `make bench-build`

No formatter or linter (ktlint/detekt) is wired into the build yet — don't reference `make fmt`/`make clippy`-style targets until one is actually added.

## Style And Boundaries

Use standard Kotlin formatting: 4-space indentation, `camelCase` for functions and properties, `PascalCase` for types, and `UPPER_SNAKE_CASE` for constants. Prefer explicit, testable functions over shared mutable state.

Keep boundaries clear:

- the server handles networking and I/O
- the client handles CLI input and output
- the core module (`agni-core`) owns cache behavior and protocol types

## Testing

Use JUnit5 `@Test` for synchronous logic. Name tests by behavior, using backtick-quoted names, such as `` `set overwrites existing value`() ``.

For coroutine code that does real I/O (sockets, file access), prefer plain `runBlocking` over `kotlinx-coroutines-test`'s `runTest`: mixing `runTest`'s virtual-time scheduler with real work launched on `Dispatchers.IO` causes spurious instant timeouts, since the scheduler auto-advances virtual time when it doesn't see pending work on it. See `agni-server`'s `ServerTest` for the pattern.

Focus coverage on protocol parsing, store behavior, command execution, and client/server integration points.

## Documentation

- Update `README.md` when public usage changes.
- Update `CHANGELOG.md` when shipped behavior changes.
- Update `CONTRIBUTING.md` when contribution workflow changes.
- Update `BENCHMARK.md` when benchmark methodology or results change.
- Update `AGENTS.md` when workflow or project conventions change.

## Commits

Use concise imperative commit messages, for example `feat: add ttl command`. Group code, docs, and benchmark changes separately when possible.
