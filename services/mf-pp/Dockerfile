# Build stage
FROM rust:1.83-slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    cmake \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
RUN cargo build --release --package mf-pp

# Final stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/mf-pp /app/mf-pp
COPY --from=builder /app/services/mf-pp/config /app/config/

EXPOSE 50094
CMD ["/app/mf-pp"]
