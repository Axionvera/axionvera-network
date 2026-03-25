# Multi-stage build for Soroban smart contract
# Stage 1: Build stage with full Rust toolchain
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./
COPY contracts/vault-contract/ ./contracts/vault-contract/

# Build the contract for wasm32-unknown-unknown target
RUN rustup target add wasm32-unknown-unknown && \
    cargo build --release --target wasm32-unknown-unknown -p axionvera-vault-contract

# Stage 2: Minimal runtime stage with distroless image
FROM gcr.io/distroless/cc-debian12:latest

# Create non-root user for security
# Note: distroless images don't include useradd, so we use a different approach
# The distroless cc image already runs as non-root by default

# Copy the built WASM contract
COPY --from=builder /app/target/wasm32-unknown-unknown/release/axionvera_vault_contract.wasm /app/contract.wasm

# Set working directory
WORKDIR /app

# Expose contract metadata (no network ports needed for smart contract)
# The contract is deployed to Stellar network, not served locally

# Health check - verify contract file exists and is readable
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/bin/sh", "-c", "test -f /app/contract.wasm && test -r /app/contract.wasm"]

# Default command - display contract info
CMD ["/bin/sh", "-c", "ls -la /app/contract.wasm && file /app/contract.wasm"]
