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
RUN cargo build --release --package cs-cb

# Final stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from the builder
COPY --from=builder /app/target/release/cs-cb /app/cs-cb

# Copy configuration
COPY --from=builder /app/services/cs-cb/config /app/config/

# Expose port
EXPOSE 50130

# Run the binary
CMD ["/app/cs-cb"]
