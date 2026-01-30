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
RUN cargo build --release --package mf-kb

# Final stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/mf-kb /app/mf-kb
COPY --from=builder /app/services/mf-kb/config /app/config/

EXPOSE 50093
CMD ["/app/mf-kb"]
