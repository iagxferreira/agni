# Changelog

All notable changes to Agni are tracked here.

## Unreleased

### Added

- Workspace split into `agni/`, `agni-server/`, `agni-client/`, and `agni-bench/`.
- Redis-style command handling for `PING`, `GET`, and `SET`.
- Structured logging with `tracing`.
- Example and Docker config files for local and containerized runs.
- Docker support for building and running the server.
- Benchmark tooling for persistent-connection load tests.
- DashMap-backed store implementation with benchmark results documented in `BENCHMARK.md`.

### Changed

- Documentation was reorganized into `README.md`, `AGENTS.md`, and `CONTRIBUTING.md`.
- Workspace guidance now points to `BENCHMARK.md` for performance methodology and results.
- The Go and Kotlin lines were merged into `main`, so the repository is now a
  single branch carrying the full project lineage. Their trees stay addressable
  as the tags `snapshot/go`, `snapshot/kotlin`, and `snapshot/rust-pre-merge`;
  the `go-main` and `kotlin-main` branches were removed.

### Fixed

- `agni-bench` no longer panics when `-n` is smaller than `-c`. `percentile()`
  underflowed a `usize` computing `len() - 1` on an empty latency set, which
  happened whenever `ops_per_task` truncated to 0.
- `agni-bench` no longer panics on `-c 0`, which divided by zero computing
  `ops_per_task` before any worker started. Both inputs are now rejected up
  front with an error message.

