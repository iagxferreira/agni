# Contributing to Agni

Thanks for helping improve Agni. Keep contributions small, focused, and easy to review.

## Before You Start

- Read [AGENTS.md](AGENTS.md) for repo conventions and workflow.
- Check [README.md](README.md) for the public project overview.
- Use Rust 1.97.1.
- Review [BENCHMARK.md](BENCHMARK.md) before changing performance-sensitive code.

## Community Expectations

- Be specific and constructive in review feedback.
- Focus on the code, behavior, and tradeoffs.
- Keep discussions respectful and on-topic.
- Ask before making broad changes to shared interfaces or performance-sensitive paths.

## Working Style

- Prefer one change per commit.
- Keep shared logic in `agni/` and keep binaries thin.
- Preserve the boundary between protocol, storage, client, and server code.
- Keep documentation changes separate from code changes when possible.

## Local Checks

- `make test`
- `make fmt`
- `make clippy`
- `make run-server`
- `make run-client CMD="PING"`

Use `make bench-build` for benchmark-related changes.

## Tests

- Add `#[test]` for synchronous logic and `#[tokio::test]` for async code.
- Name tests by behavior, for example `set_overwrites_existing_value`.
- Add coverage for protocol parsing, store behavior, and command execution.

## Pull Requests

- Summarize what changed and why.
- List the commands you ran.
- Include benchmark results or screenshots when relevant.
- Mention any config changes, especially if they affect `config.example.yml` or Docker use.

## Docs

- Update `README.md` for public usage changes.
- Update `BENCHMARK.md` for benchmark methodology or results.
- Update `AGENTS.md` when repository conventions change.
- Keep contribution guidance in this file instead of introducing separate policy docs unless the repo needs them.
