# AskRepo Rust application

FROM rust:1.84-slim-bullseye AS builder
WORKDIR /src

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates build-essential \
    && rm -rf /var/lib/apt/lists/*

ENV CARGO_TARGET_DIR=/src/target

# Pre-copy Cargo manifests to leverage incremental caching
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Copy README or other assets needed for the build
COPY README.md .

# Build the binary from the local manifest
RUN cargo build --release

FROM debian:bullseye-slim
WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

# Runtime user (non-root)
RUN useradd --system --home /app --shell /usr/sbin/nologin askrepo

COPY --from=builder /src/target/release/askrepo-service /usr/local/bin/askrepo-service

# Persistent directory for logs/state if needed
RUN mkdir -p /app/data && chown askrepo:askrepo /app/data
VOLUME ["/app/data"]

ENV RUST_LOG=info
USER askrepo

CMD ["askrepo-service"]
