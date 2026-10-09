# syntax=docker/dockerfile:1.6
FROM rust:1.97.0-bookworm AS builder
WORKDIR /usr/src/app

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY crates/flow ./crates/flow
COPY crates/shared ./crates/shared
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY bins/server/Cargo.toml bins/server/Cargo.toml

RUN mkdir -p bins/server/src \
    && echo 'fn main() {}' > bins/server/src/main.rs
RUN cargo fetch

COPY bins/server ./bins/server
RUN cargo build --release --no-default-features

FROM debian:bookworm-slim
WORKDIR /usr/src/app

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/server ./server

ENV RUST_LOG=info
ENTRYPOINT ["./server"]
