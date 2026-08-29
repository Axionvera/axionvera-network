# Maintainer Handoff Guide: Testnet Deployment

This guide establishes a secure, contributor-safe process for preparing and executing testnet deployments for the Axionvera Network protocol and Soroban smart contracts.

---

## 1. Security & Responsibility Boundary

To protect network security, maintain private key custody, and ensure reproducible deployments, responsibilities are strictly separated between contributors and maintainers:

| Responsibility Area | Contributor Scope (Safe) | Maintainer Scope (Privileged) |
| :--- | :--- | :--- |
| **Code & Contracts** | Build contracts, write unit/integration tests, optimize WASM. | Review and approve code, perform final audit verification. |
| **Configuration** | Provide mock/placeholder `.env.example` templates and schemas. | Configure live testnet RPC endpoints and funded deployment keys. |
| **Secrets & Keys** | **Never touch or commit private keys, mnemonics, or live secrets.** | Manage deployment keypairs and network accounts securely. |
| **Deployment** | Test against local/mocked environments (`cargo test`). | Execute actual contract deployment to public Testnet. |
| **Artifacts** | Define JSON schemas and validate mock deployment outputs. | Sign and commit canonical deployment artifact JSON files. |

---

## 2. Contributor Preparation Workflow (Safe)

Contributors must verify that all artifacts, schemas, and contract checks pass locally before handing off the release:

1. **Local Build & Quality Checks:**
   Verify that the workspace compiles cleanly without warnings:
   ```bash
   cargo fmt --check
   cargo check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test