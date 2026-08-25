# Repository Guidelines

Agni is a Rust workspace for an in-memory cache server, client, and benchmark tools. Keep changes small, testable, and scoped to the owning crate.

Rust 1.97.1 is the minimum supported toolchain for the workspace.

## Branch Layout

- `main` tracks the Rust implementation and is the default place for current development and benchmark work.
- `go-main` preserves the Go baseline for comparison and history.
- `kotlin-main` preserves the Kotlin benchmark snapshot for comparison and history.

Treat the non-`main` branches as frozen reference points unless a change explicitly says otherwise.

## Project Structure

- `agni/` is the core library with store, protocol, and command logic.
- `agni-server/` is the TCP server binary.
- `agni-client/` is the CLI client binary.
- `agni-bench/` is the benchmarking binary.
- `README.md` stays high level. `BENCHMARK.md` records performance results.

Prefer moving shared logic into `agni/` and keeping binaries thin.

## Build And Test

- `make test` runs the workspace tests.
- `make fmt` formats the codebase.
- `make clippy` runs lint checks.
- `make run-server` starts the server locally.
- `make run-client ARGS="PING"` sends a command to a running server.

Use release builds for benchmark work:

- `make bench-build`

## Style And Boundaries

Use standard Rust formatting: 4-space indentation, `snake_case` for functions and modules, `PascalCase` for types, and `UPPER_SNAKE_CASE` for constants. Prefer explicit, testable functions over shared mutable state.

Keep boundaries clear:

- the server handles networking and I/O
- the client handles CLI input and output
- the core crate owns cache behavior and protocol types

## Testing

Use `#[test]` for synchronous logic and `#[tokio::test]` for async code. Name tests by behavior, such as `set_overwrites_existing_value`.

Focus coverage on protocol parsing, store behavior, command execution, and client/server integration points.

## Documentation

- Update `README.md` when public usage changes.
- Update `CHANGELOG.md` when shipped behavior changes.
- Update `CONTRIBUTING.md` when contribution workflow changes.
- Update `BENCHMARK.md` when benchmark methodology or results change.
- Update `AGENTS.md` when workflow or project conventions change.

## Commits

Use concise imperative commit messages, for example `feat: add ttl command`. Group code, docs, and benchmark changes separately when possible.
