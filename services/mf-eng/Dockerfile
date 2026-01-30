# Build stage
FROM rust:1.83-slim AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    cmake \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the entire workspace
COPY . .

# Build the specific service
RUN cargo build --release --package mf-eng

# Final stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from the builder
COPY --from=builder /app/target/release/mf-eng /app/mf-eng

# Copy configuration
COPY --from=builder /app/services/mf-eng/config /app/config/

# Expose port
EXPOSE 50095

# Run the binary
CMD ["/app/mf-eng"]
