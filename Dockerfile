# ── Build stage ──────────────────────────────────────────────────────────────
FROM eclipse-temurin:21-jdk-alpine AS builder

WORKDIR /app

# Copy the Gradle wrapper and build files first, to cache dependency downloads
COPY gradlew build.gradle.kts settings.gradle.kts ./
COPY gradle gradle
COPY agni-core/build.gradle.kts agni-core/build.gradle.kts
COPY agni-server/build.gradle.kts agni-server/build.gradle.kts
COPY agni-client/build.gradle.kts agni-client/build.gradle.kts
COPY agni-bench/build.gradle.kts agni-bench/build.gradle.kts

# Warm the dependency cache. No source is present yet, so this just resolves
# and downloads jars; whether the (sourceless) build itself succeeds doesn't
# matter here.
RUN ./gradlew --no-daemon :agni-server:installDist || true

# Copy real source and build the server distribution
COPY agni-core/src agni-core/src
COPY agni-server/src agni-server/src

RUN ./gradlew --no-daemon :agni-server:installDist

# ── Runtime stage ─────────────────────────────────────────────────────────────
FROM eclipse-temurin:21-jre-alpine

COPY --from=builder /app/agni-server/build/install/agni-server /opt/agni-server
COPY config.docker.yml /etc/agni/config.yml

EXPOSE 6379

ENTRYPOINT ["/opt/agni-server/bin/agni-server", "--config", "/etc/agni/config.yml"]
