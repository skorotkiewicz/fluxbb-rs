# syntax=docker/dockerfile:1.7
# Builder
FROM rust:1.93-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo install dioxus-cli --version 0.7.6 --locked

RUN dx --version
RUN rustup target add wasm32-unknown-unknown

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    dx build --release && \
    mkdir -p /app/dist && \
    cp /app/target/dx/fluxbb-rs/release/web/server /app/dist && \
    cp -r /app/target/dx/fluxbb-rs/release/web/public /app/dist

# Runtime
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y \
    ca-certificates \
    wget \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd -r app && useradd -r -g app app

WORKDIR /app
COPY --from=builder /app/dist/server ./server
COPY --from=builder /app/dist/public ./public
COPY --from=builder /app/db/schema.sql ./db/schema.sql
RUN chown -R app:app /app
USER app
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=5 \
    CMD wget -qO- http://127.0.0.1:8080/api/health || exit 1
CMD ["./server"]
