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

