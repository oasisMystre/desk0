# syntax=docker/dockerfile:1.6
FROM rust:1.97.0-bookworm AS rust
WORKDIR /usr/src/app

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/flow ./crates/flow
COPY crates/shared ./crates/shared
COPY bins/server/Cargo.toml bins/server/Cargo.toml

RUN mkdir -p bins/server/src \
    && echo 'fn main() {}' > bins/server/src/main.rs
RUN cargo build --release --no-default-features


FROM oven/bun:1.4.2 AS bun
WORKDIR /usr/src/app

ENV DOCKER_BUILDKIT=1
ENV NODE_ENV="production"
ENV PATH="/root/.bun/bin:$PATH"

FROM bun AS templates

RUN bun add -g turbo@2.10.5
COPY . .
RUN turbo prune @workspace/templates --docker

COPY out/json/. ./
RUN bun install

COPY out/full/. ./
RUN bun turbo run build --filter=@workspace/templates

FROM rust AS builder

COPY bins/server ./bins/server
COPY --from=templates packages/templates/dist bins/server/templates

RUN touch bins/server/src/main.rs && \
    cargo build --release --no-default-features

FROM debian:bookworm-slim
WORKDIR /usr/src/app

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/desk0-server ./desk0-server

ENV RUST_LOG=info
ENTRYPOINT ["./desk0-server"]
