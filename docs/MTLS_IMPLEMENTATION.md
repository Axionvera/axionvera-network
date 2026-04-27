# mTLS Implementation and Operator Guide

This document explains the mutual TLS (mTLS) enforcement added in branch `feature/mtls-enforce`, environment variables to configure it, verification steps, and CA rotation guidance.

## Summary of changes

- gRPC server (`network-node/src/grpc/server.rs`) now configures TLS using `rustls` and will require client certificates when `TLS_CLIENT_CA_PATH` is provided and `TLS_REQUIRE_CLIENT_AUTH=true`.
- HTTP server (`network-node/src/enhanced_server.rs`) now performs transport-level TLS handshakes using `tokio-rustls`. Failed TLS handshakes are dropped and logged before any application parsing occurs.
- `scripts/rotate_ca.sh` is present and can fetch a new CA bundle and atomically replace the local CA file. Use it to automate CA rotation.
- New dependencies added to `network-node/Cargo.toml`: `rustls`, `tokio-rustls`, `tonic-rustls`, `rustls-pemfile`, `tokio-stream`, `tokio-stream`.

## Environment variables

- `TLS_CERT_PATH` - path to the PEM file with the server certificate chain (one or more `BEGIN CERTIFICATE` entries).
- `TLS_KEY_PATH` - path to the PEM file containing the server private key (PKCS8 or RSA PEM supported).
- `TLS_CLIENT_CA_PATH` - optional path to a PEM file containing one or more CA certificates used to validate client certificates.
- `TLS_REQUIRE_CLIENT_AUTH` - `true`/`false`. When true and `TLS_CLIENT_CA_PATH` is set, the server requires client certificates (mTLS). Defaults to `true` when unspecified.

## Behavioral guarantees

- TLS negotiation happens at the transport layer. Any connection that fails the TLS handshake (including missing or invalid client certs when required) is dropped immediately and does not reach application-layer parsing.
- When `TLS_CLIENT_CA_PATH` is omitted, the server falls back to one-way TLS (server-only cert).
- When `TLS_REQUIRE_CLIENT_AUTH=false` but `TLS_CLIENT_CA_PATH` is provided, the server will accept connections without client certs but will still validate any presented client certs against the provided CA.

## Operator steps: running locally

1. Install Rust toolchain and dependencies (if not already):

```bash
# Install rustup then toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable

cd network-node
cargo build -p axionvera-network-node
```

2. Start the node with mTLS enabled (example):

```bash
TLS_CERT_PATH=/path/to/server.cert.pem \
TLS_KEY_PATH=/path/to/server.key.pem \
TLS_CLIENT_CA_PATH=/path/to/ca.cert.pem \
TLS_REQUIRE_CLIENT_AUTH=true \
cargo run -p axionvera-network-node
```

3. Verify behavior:

- Try to connect without a client certificate (e.g., `curl` without client cert for HTTP or a gRPC client without cert). The connection should fail at TLS handshake.
- Connect with a client certificate signed by the CA in `TLS_CLIENT_CA_PATH`. Connection should succeed.

## CA rotation

There is an operator helper script at `scripts/rotate_ca.sh` which will fetch a new CA bundle and atomically replace the destination file. Example usage:

```bash
# Fetch new CA bundle and replace local file
scripts/rotate_ca.sh https://example.com/new-ca.pem /etc/axionvera/ca.cert.pem "systemctl restart axionvera-node"
```

Notes:
- Use an orchestration step or systemd unit that restarts the service gracefully after CA rotation.
- The script performs a basic validation that the fetched file contains a `BEGIN CERTIFICATE` header before replacing.

## Testing & CI suggestions

- Add integration tests that spawn the server in a test harness with temporary certs and exercise:
  - Successful handshake with valid client cert
  - Failed handshake when client cert absent or invalid
  - Behavior when `TLS_REQUIRE_CLIENT_AUTH=false`

- Add a CI job that builds `network-node` and runs the TLS integration tests.

## Notes / Limitations

- I could not run `cargo build` or execute tests in this environment (Rust toolchain not available here). Please run `cargo build` and paste any CI/build errors here and I will fix them.
- There is no separate inbound P2P validator TCP listener implemented; if you operate a dedicated validator listener, I can integrate the same rustls handshake pattern for it.

## Next actions I can take for you

- Add integration tests for TLS handshake behavior.
- Integrate mTLS into any other TCP listeners you specify (e.g., a P2P inbound listener).
- Prepare a PR description and open a draft PR (if you provide remote access or push permissions), or prepare a ready-to-open PR file you can paste into GitHub.

