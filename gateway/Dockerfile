# Build stage
FROM rust:1.93-slim AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    cmake \
    & rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the entire workspace
COPY . .

# Build the specific service
RUN cargo build --release --package gateway

# Final stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    & rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from the builder
COPY --from=builder /app/target/release/gateway /app/gateway

# Copy configuration (if any)
# Gateway configuration might be in its own directory
COPY --from=builder /app/gateway/config /app/config/

# Expose ports
EXPOSE 8080

# Run the binary
CMD ["/app/gateway"]
