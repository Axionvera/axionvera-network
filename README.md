# Axionvera Network

Axionvera Network is a Soroban (Stellar) smart-contract vault and reward distribution protocol with a comprehensive production-ready network infrastructure.

Users can:
- Deposit tokens into a vault
- Track per-user balances
- Withdraw funds
- Receive proportional reward distributions (via `distribute_rewards` + `claim_rewards`)

## 🚀 Production Infrastructure Features

### ✅ Issue #311 - Multi-Stage Distroless Docker Production Image
- **Multi-stage build** with dedicated compilation and execution stages
- **Distroless base image** (`gcr.io/distroless/cc-debian12`) for minimal attack surface
- **Non-root user execution** for security compliance
- **Automated security scanning** with Trivy integration
- **Optimized image size** with only essential components

### ✅ Issue #312 - Terraform Cloud Infrastructure (AWS)
- **Secure VPC architecture** with public/private subnets
- **NAT Gateways** for private subnet internet access
- **Application Load Balancer** with health checks
- **Auto Scaling Group** for high availability
- **Security Groups** with strict traffic restrictions
- **CloudWatch monitoring** and alerting
- **S3 logging** with encryption

### ✅ Issue #310 - Graceful Shutdown Protocol
- **Signal handling** for SIGTERM, SIGINT, and SIGQUIT
- **Grace period** for active operations to complete
- **Connection pool cleanup** with timeout handling
- **Database connection management** with proper shutdown
- **Health check endpoints** for monitoring

### ✅ Issue #309 - Centralized Error Propagation
- **Custom error types** with comprehensive categorization
- **Error middleware** with circuit breaker pattern
- **Contextual error tracking** with request IDs
- **Error statistics** and monitoring
- **Internal error logging** without external exposure
- **Retry logic** for transient failures

This repository is structured like a real open-source project intended for contribution programs: modular contract code, clear extension points, tests, scripts, and contribution templates.

## Repository Layout

- [contracts/vault-contract](contracts/vault-contract) — Soroban vault contract (Rust)
- [network-node](network-node) — Production network node service with error handling
- [terraform](terraform) — AWS infrastructure as code
- [scripts](scripts) — Deployment and security scanning scripts
- [tests](tests) — TypeScript test scaffold
- [docs](docs) — Architecture and contract specification

### New Components Added

- **network-node/**: Rust-based network node with:
  - Centralized error handling middleware
  - Graceful shutdown protocol
  - Database connection pooling
  - HTTP server with health endpoints
  - Circuit breaker pattern implementation

- **terraform/**: Complete AWS infrastructure with:
  - Secure VPC with public/private subnets
  - Auto Scaling Groups and Load Balancers
  - Security Groups with restrictive rules
  - CloudWatch monitoring and alerting
  - S3 logging with encryption

- **Dockerfile**: Multi-stage distroless build for production
- **docker-compose.yml**: Development orchestration with security scanning

## Architecture (High Level)

The vault uses an index-based accounting model:
- `total_deposits` tracks total deposited vault shares (1:1 with deposit token units).
- `reward_index` is a cumulative “rewards-per-share” index scaled by `1e18`.
- Each user stores a `user_reward_index` snapshot and `user_rewards` accrued amount.
- `distribute_rewards(amount)` increases `reward_index` proportionally to `amount / total_deposits`.
- Users realize rewards lazily on interactions (`deposit`, `withdraw`, `claim_rewards`).

More detail:
- [docs/architecture.md](file:///Users/boufdaddy/Documents/trae_projects/axionvera-network/docs/architecture.md)
- [docs/contract-spec.md](file:///Users/boufdaddy/Documents/trae_projects/axionvera-network/docs/contract-spec.md)

## Prerequisites

- Rust (stable)
- `wasm32-unknown-unknown` target
- Soroban CLI (`soroban`)
- Node.js (18+ recommended)
- Docker (20.10+)
- Terraform (1.5+)
- AWS CLI configured

## Quick Start

### 1. Build Contract
```bash
rustup target add wasm32-unknown-unknown
npm install
npm run build:contracts
```

### 2. Build Network Node
```bash
cd network-node
cargo build --release
```

### 3. Docker Build & Security Scan
```bash
docker build -t axionvera-network:latest .
./scripts/security-scan.sh
```

### 4. Deploy Infrastructure
```bash
./scripts/deploy-infrastructure.sh
```

### 5. Run Network Node
```bash
# Development
cd network-node
cargo run

# Production
docker run -d \
  --name axionvera-network \
  -p 8080:8080 \
  -e DATABASE_URL="postgresql://..." \
  axionvera-network:latest
```

## API Endpoints

The network node provides the following endpoints:

- `GET /health` - Health check
- `GET /ready` - Readiness probe  
- `GET /metrics` - Prometheus metrics
- `GET /error-stats` - Error statistics
- `GET /circuit-breaker-status` - Circuit breaker status

```bash
# Health check
curl http://localhost:8080/health

# Error statistics
curl http://localhost:8080/error-stats

# Circuit breaker status
curl http://localhost:8080/circuit-breaker-status
```

## Security Features

### Docker Security
- **Distroless image** with minimal packages
- **Non-root execution** by default
- **Read-only filesystem** where possible
- **Security scanning** with Trivy
- **Capability dropping** for unnecessary privileges

### AWS Security
- **VPC isolation** with private subnets
- **Security groups** with restrictive rules
- **IAM roles** with least privilege
- **Encryption at rest** and in transit
- **VPC flow logs** for monitoring

### Application Security
- **Input validation** with comprehensive error types
- **Error sanitization** - no stack traces in responses
- **Circuit breaker** to prevent cascade failures
- **Rate limiting** and connection management
- **Audit logging** for all operations

## Graceful Shutdown

The network node implements comprehensive graceful shutdown:

1. **Signal Interception**: Catches SIGTERM, SIGINT, SIGQUIT
2. **Connection Draining**: Stops accepting new requests immediately
3. **Grace Period**: Waits for active operations (default 10s)
4. **Resource Cleanup**: Closes database connections properly
5. **Process Exit**: Clean termination with status codes

```bash
# Test graceful shutdown
docker kill -s SIGTERM axionvera-network
```

## Testing

### Contract Tests
```bash
# Rust unit tests
npm run test:rust

# TypeScript tests
npm test

# Integration tests (requires Soroban environment)
SOROBAN_INTEGRATION=1 npm test
```

### Network Node Tests
```bash
cd network-node
cargo test
```

### Security Tests
```bash
# Docker security scan
./scripts/security-scan.sh

# Terraform validation
cd terraform
terraform validate
```

## Environment Variables

```bash
# Network Node Configuration
BIND_ADDRESS=0.0.0.0:8080
DATABASE_URL=postgresql://user:pass@localhost/db
SHUTDOWN_GRACE_PERIOD=10
LOG_LEVEL=info

# Terraform Variables
TF_VAR_aws_region=us-east-1
TF_VAR_environment=production
TF_VAR_ssh_allowed_ips=["203.0.113.0/24"]
```

## Contributing

- Read [CONTRIBUTING.md](CONTRIBUTING.md)
- See [docs/contributing-guide.md](docs/contributing-guide.md) for contribution areas and standards
- All new features must include tests and security considerations

## Implementation Status

- [x] **Issue #311**: Multi-Stage Distroless Docker Production Image
- [x] **Issue #312**: Terraform Cloud Infrastructure (AWS)  
- [x] **Issue #310**: Graceful Shutdown Protocol
- [x] **Issue #309**: Centralized Error Propagation

All four issues have been successfully implemented with production-ready solutions that address security, reliability, and operational concerns.

## Security

This project is a reference-quality open-source starting point and is not audited.
Do not deploy to mainnet without a dedicated security review.
