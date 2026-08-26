# ── Build stage ──────────────────────────────────────────────────────────────
FROM rust:1-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests first to cache dependency compilation
COPY Cargo.toml Cargo.lock ./
COPY agni/Cargo.toml agni/Cargo.toml
COPY agni-server/Cargo.toml agni-server/Cargo.toml
COPY agni-client/Cargo.toml agni-client/Cargo.toml
COPY agni-bench/Cargo.toml agni-bench/Cargo.toml

# Create dummy lib/main files so cargo can compile dependencies
RUN mkdir -p agni/src agni-server/src agni-client/src agni-bench/src \
    && echo "pub fn placeholder() {}" > agni/src/lib.rs \
    && echo "fn main() {}" > agni-server/src/main.rs \
    && echo "fn main() {}" > agni-client/src/main.rs \
    && echo "fn main() {}" > agni-bench/src/main.rs

RUN cargo build --release -p agni-server

# Copy real source and rebuild only agni-server
COPY agni/src agni/src
COPY agni-server/src agni-server/src

RUN touch agni/src/lib.rs agni-server/src/main.rs \
    && cargo build --release -p agni-server

# ── Runtime stage ─────────────────────────────────────────────────────────────
FROM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /app/target/release/agni-server /usr/local/bin/agni-server
COPY config.docker.yml /etc/agni/config.yml

EXPOSE 6379

ENTRYPOINT ["agni-server", "--config", "/etc/agni/config.yml"]
