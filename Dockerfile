# Multi-stage build for minimal runtime image
# Stage 1: Build
FROM rust:1.93-slim AS builder

# Install build dependencies (including C++ for tokenizers/esaxx-rs)
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    g++ \
    cmake \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build the API server in release mode
RUN cargo build --release --package redact-api

# Stage 2: Minimal runtime using distroless
# gcr.io/distroless/cc contains only:
# - glibc
# - libgcc
# - ca-certificates
# No shell, no package manager, no OS utilities = smaller attack surface
FROM gcr.io/distroless/cc-debian12

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/redact-api /usr/local/bin/redact-api

# Expose API port
EXPOSE 8080

# Run as non-root (distroless includes nonroot user with UID 65532)
USER nonroot:nonroot

# Run the API server
ENTRYPOINT ["/usr/local/bin/redact-api"]
