# ── Build stage ──────────────────────────────────────────────────────────────
FROM golang:1.27-alpine AS builder

WORKDIR /app

# Copy module files first, to cache dependency downloads
COPY go.mod go.sum ./
RUN go mod download

COPY cmd cmd
COPY internal internal
COPY store store
COPY protocol protocol
COPY config config

RUN CGO_ENABLED=0 go build -o /out/agni-server ./cmd/agni-server

# ── Runtime stage ─────────────────────────────────────────────────────────────
# A statically-linked Go binary needs nothing from userspace, so scratch is
# enough — no JRE or OS packages required.
FROM scratch

COPY --from=builder /out/agni-server /agni-server
COPY config.docker.yml /etc/agni/config.yml

EXPOSE 6379

ENTRYPOINT ["/agni-server", "--config", "/etc/agni/config.yml"]
