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
RUN cargo build --release --package fi-ar

# Final stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from the builder
COPY --from=builder /app/target/release/fi-ar /app/fi-ar

# Copy configuration
COPY --from=builder /app/services/fi-ar/config /app/config/

# Expose port
EXPOSE 50062

# Run the binary
CMD ["/app/fi-ar"]
