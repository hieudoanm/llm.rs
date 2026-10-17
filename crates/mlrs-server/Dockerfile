# syntax=docker/dockerfile:1
# Build stage: compile the pure-Rust skl backend (workspace crates/skl-*).
FROM rust:1-slim AS builder

WORKDIR /app

COPY crates ./crates
RUN cargo build --release -p skl-server

# Runtime stage: minimal image with just the binary and the data directory.
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

RUN mkdir -p /var/lib/skl

COPY --from=builder /app/crates/target/release/skl-server /usr/local/bin/skl-backend

ENV SKL_DATA_DIR=/var/lib/skl
ENV SKL_LOG_LEVEL=INFO
ENV SKL_HOST=0.0.0.0
ENV SKL_PORT=8000

EXPOSE 8000

HEALTHCHECK --interval=10s --timeout=5s --start-period=15s --retries=12 \
  CMD curl -sf http://127.0.0.1:8000/healthz || exit 1

CMD ["skl-backend"]
