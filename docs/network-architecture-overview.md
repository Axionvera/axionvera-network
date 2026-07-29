# Axionvera Network Architecture Overview

This document describes how the Axionvera repository is actually laid out today: what the
network node does, how peer and protocol modules are wired, what role consensus plays, and
how configuration and diagnostics flow through the system.

It is written to let a new contributor work safely. Where the code and the existing
documentation disagree, this document follows the code and says so explicitly.

**Verified against commit `ce4d008` (2026-07-29).** Every claim below is anchored to a file
and, where useful, a line number. If you are reading this much later, re-check the anchors
before trusting them.

---

## Build Status

**The `axionvera-network-node` crate does not compile.** This is the single most important
thing to know before touching anything under `network-node/`.

The repository contains a committed `cargo check` log,
[`cargo-check-full.log`](../cargo-check-full.log), whose final line reads:

```
error: could not compile `axionvera-network-node` (lib) due to 129 previous errors; 48 warnings emitted
```

That log was produced on a different machine (its paths begin with
`/Users/boufdaddy/Documents/web3-projects/…`, see `cargo-check-full.log:1`) at an unrecorded
commit. Because a stale log proves nothing on its own, the failures below were re-verified
by hand against the current tree.

### Verified against current source

| Error from the log | Current location | Status |
| --- | --- | --- |
| `no method named to_rfc3331 found for struct chrono::DateTime` | [`network-node/src/enhanced_server.rs:601`](../network-node/src/enhanced_server.rs) | Still present — typo for `to_rfc3339` |
| `no field state_trie on type AppState` | [`enhanced_server.rs:665`](../network-node/src/enhanced_server.rs) reads `state.state_trie`; `AppState` is defined at `enhanced_server.rs:389-397` and has no such field | Still present |
| `no field p2p_manager on type AppState` | [`enhanced_server.rs:682`](../network-node/src/enhanced_server.rs) reads `state.p2p_manager`; not a field of `AppState` | Still present |

### Found during this review, not in the log

The log covers the library target only. The binary target has an independent failure:

[`network-node/src/main.rs`](../network-node/src/main.rs) calls four telemetry functions —
`telemetry::init_jaeger_tracing` (`main.rs:59`), `telemetry::init_xray_tracing`
(`main.rs:63`), `telemetry::init_tracing` (`main.rs:67`), and `telemetry::shutdown_tracer`
(`main.rs:99` and `main.rs:108`). **None of the four exist.**
[`network-node/src/telemetry.rs`](../network-node/src/telemetry.rs) is 25 lines long and
defines exactly three functions: `init_logging` (`telemetry.rs:5`),
`extract_traceparent_grpc` (`telemetry.rs:13`), and `inject_traceparent_grpc`
(`telemetry.rs:21`).

This is consistent with `opentelemetry`, `opentelemetry_sdk`, and `tracing-opentelemetry`
being declared in [`network-node/Cargo.toml:52-54`](../network-node/Cargo.toml) but appearing
in no `.rs` file under `network-node/src/`. The OTLP/Jaeger/X-Ray tracing described by the
config enum (`config.rs:95-100`) has no implementation behind it.

### Where the errors are concentrated

Counted from the `-->` location lines in `cargo-check-full.log`:

| File | Error locations |
| --- | --- |
| `network-node/src/grpc/gateway_service.rs` | 39 |
| `network-node/src/horizon_client.rs` | 22 |
| `network-node/src/grpc/server.rs` | 22 |
| `network-node/src/enhanced_server.rs` | 22 |
| `network-node/src/stellar_service.rs` | 15 |
| `network-node/src/error.rs` | 15 |
| `network-node/src/database.rs` | 14 |
| `network-node/src/p2p.rs` | 12 |
| `network-node/src/crypto.rs` | 8 |
| `network-node/src/consensus.rs` | 7 |

### Why CI does not catch this

CI never builds the node. The workflows in `.github/workflows/` are `e2e-tests.yml`,
`protocol-integration-tests.yml`, and `trigger-auto-merge.yml`. The only Rust command they
run is `npm run build:contracts` (`e2e-tests.yml:36,87,130`;
`protocol-integration-tests.yml:42,89`), which resolves to:

```
cargo build -p axionvera-vault-contract --target wasm32-unknown-unknown --release
```

(see [`package.json:6`](../package.json)). The `axionvera-network-node` package is not part
of any job, and neither is `cargo build --workspace`.

### What this means for you

- Contract work (`contracts/vault-contract` and its dependency tree) builds and tests
  normally. `npm run build:contracts` and `cargo test -p axionvera-vault-contract` are the
  supported paths.
- Node work requires fixing the build first. Treat every `network-node/src/` description in
  this document as *"this is what the source says it intends to do"*, not *"this runs"*.
- Do not assume a green CI run means your `network-node/` change compiles.

---

## Status Legend

Every component below carries one of four labels.

| Label | Meaning |
| --- | --- |
| **Implemented** | Code exists, is compiled into the crate, and is reachable from a real entry point. Subject to the build status above. |
| **Documented, not verified** | Described in existing repo documentation or infrastructure files, but not confirmable from the Rust/TypeScript sources reviewed here. |
| **Present, not wired** | The code exists and may even compile, but nothing constructs, calls, or depends on it. Dead on arrival. |
| **Absent** | Claimed somewhere in the repo, but does not exist in the code. |

---

## System Shape

Axionvera is a Soroban (Stellar) vault protocol plus a Rust service that fronts it. The
honest one-line description of the topology is:

> A set of independent gateway nodes, each a client of Stellar's Horizon and Soroban RPC
> endpoints. Ledger state and finality come from Stellar. Nodes do not form a network with
> each other.

```
   Clients / dApps
         |
         |  HTTP :8080 (Axum)  ·  gRPC :50051 (tonic)  ·  gRPC-Web gateway :8081
         v
  +--------------------------------------------+
  |            axionvera-network-node          |
  |                                            |
  |  HTTP server ...... enhanced_server.rs     |
  |  gRPC services .... grpc/                  |
  |  Event indexer .... indexer.rs  (5s poll)  |
  |  Signing .......... signing.rs, aws_kms_*  |
  |  State trie ....... state_trie.rs (RocksDB)|
  |  Peer registry .... p2p.rs   [in-memory]   |
  |  Consensus engine . consensus.rs [unwired] |
  +--------------------------------------------+
         |                        |
         | sqlx / Postgres        | HTTPS JSON-RPC + Horizon REST
         v                        v
    PostgreSQL              Soroban RPC / Horizon
    Redis (rate limit)             |
                                   v
                          Vault contract (WASM)
```

Ports come from [`network-node/src/config.rs:166`](../network-node/src/config.rs) (`BIND_ADDRESS`,
default `0.0.0.0:8080`) and `config.rs:251-254` (`GRPC_BIND_ADDRESS` default `0.0.0.0:50051`,
`GATEWAY_BIND_ADDRESS` default `0.0.0.0:8081`). The Prometheus exporter binds `0.0.0.0:9090`
in [`main.rs:25-29`](../network-node/src/main.rs).

There is no peer-to-peer port. See [Peer and Protocol Modules](#peer-and-protocol-modules).

---

## Node Runtime Responsibilities

Source of truth: [`network-node/src/lib.rs`](../network-node/src/lib.rs).

The crate declares 26 modules at `lib.rs:20-46`. The runtime is assembled as a single
`NetworkNode` struct (`lib.rs:49-65`), built by `NetworkNode::new` (`lib.rs:69`), started by
`NetworkNode::start` (`lib.rs:204`), and torn down by `NetworkNode::shutdown` (`lib.rs:275`).

### Startup sequence

`NetworkNode::new` constructs, in order (`lib.rs:73-182`):

| Step | Component | Line |
| --- | --- | --- |
| 1 | `ErrorMiddleware` — centralized error mapping | `lib.rs:73` |
| 2 | `MetricsCollector` | `lib.rs:76` |
| 3 | Postgres `ConnectionPool` | `lib.rs:79-81` |
| 4 | `StateTrie` on RocksDB at `./data/state_trie` | `lib.rs:84-86` |
| 5 | `P2PManager` | `lib.rs:90` |
| 6 | `SigningService` (AWS KMS or local key) | `lib.rs:94-115` |
| 7 | `HorizonClient` (multi-provider) | `lib.rs:118` |
| 8 | `StellarService` | `lib.rs:125` |
| 9 | `SorobanRpcClient` | `lib.rs:129-131` |
| 10 | `SorobanService` | `lib.rs:135-137` |
| 11 | `EventIndexer`, 5-second poll interval | `lib.rs:141-147` |
| 12 | `ChainParameterRegistry` from genesis file or development default | `lib.rs:153-157` |
| 13 | `EnhancedHttpServer` | `lib.rs:160-169` |
| 14 | `GrpcServer` | `lib.rs:172-179` |
| 15 | `ShutdownHandler` | `lib.rs:182` |

`NetworkNode::start` then spawns the HTTP server (`lib.rs:211`), the gRPC server
(`lib.rs:214-222`), P2P maintenance (`lib.rs:225`), the Horizon health checker (`lib.rs:228`),
and the event indexer (`lib.rs:234-238`), before optionally bootstrapping from
`BOOTSTRAP_PEER` (`lib.rs:242-247`) and blocking on a `tokio::select!` over all handles plus
the shutdown token (`lib.rs:252-269`).

### Responsibilities, by area

**HTTP surface — Implemented.**
[`enhanced_server.rs:109-137`](../network-node/src/enhanced_server.rs) builds an Axum router
with 13 routes and five middleware layers: error handling, connection limiting, rate
limiting, connection tracking, and `TraceLayer`. Routes are listed under
[Diagnostics](#diagnostics-surface).

**gRPC surface — Implemented.** Services are generated from `proto/` by
[`network-node/build.rs`](../network-node/build.rs), which compiles `vault.proto`,
`gateway.proto`, and `network.proto`. Implementations live in `network-node/src/grpc/`
(`grpc/mod.rs:13-19`). Declared services:

| Service | Definition | Implementation |
| --- | --- | --- |
| `NetworkService` | `proto/network.proto:9` | `grpc/network_service.rs` |
| `VaultService` | `proto/network.proto:39`, `proto/vault.proto:7` | `grpc/vault_service.rs` |
| `P2PService` | `proto/network.proto:44` | `grpc/p2p_service.rs` |
| `HealthService` | `proto/network.proto:53` | `grpc/health_service.rs` |
| `ServiceRegistry` | `proto/network.proto:60` | `grpc/service_registry_service.rs` |
| `GatewayService` | `proto/gateway.proto:10` | `grpc/gateway_service.rs` |

`NetworkService` (`proto/network.proto:11-35`) is the main protocol surface: `Deposit`,
`Withdraw`, `DistributeRewards`, `ClaimRewards`, `GetBalance`, `GetRewards`,
`GetContractState`, `GetNetworkStatus`, `GetNodeInfo`, `GetTransaction`,
`GetTransactionHistory`, `ParameterUpgrade`, `GetChainParameters`,
`ListPendingParameterUpgrades`, `GetTVL`.

**Chain interaction — Implemented.** Writes go out over Soroban JSON-RPC via
`soroban_rpc_client.rs` and `soroban_service.rs`. Reads and account/ledger queries go through
`horizon_client.rs` and `stellar_service.rs`, which use `reqwest` with retry middleware.
`HorizonClient` supports multiple providers with priority ordering and a health checker
(`config.rs:67-92`).

**Indexing — Implemented.** `indexer.rs` polls Soroban for contract events every 5 seconds
(`lib.rs:146`) for the address in `VAULT_CONTRACT_ADDRESS` and persists them to Postgres.

**Persistence — Implemented.** [`database.rs`](../network-node/src/database.rs) uses `sqlx`
with a `PgPool` (`database.rs:3`, `database.rs:25-31`) and creates its schema on startup:
`indexer_state` (`database.rs:51`), `events` (`database.rs:77`), and `deposits`
(`database.rs:124`).

**Local state — Implemented.** `state_trie.rs` wraps RocksDB (the only consumer of the
`rocksdb` dependency).

**Signing — Implemented.** `signing.rs` defines a `SignerFactory` with an AWS KMS backend
(`aws_kms_signer.rs`) and a local Ed25519 key backend. Selected by environment, see
[Configuration Flow](#configuration-flow).

**Graceful shutdown — Implemented.** `shutdown.rs` plus `NetworkNode::shutdown`
(`lib.rs:275-322`): stop accepting connections, drain within a grace period
(`SHUTDOWN_GRACE_PERIOD`, default 30s), wait for in-flight DB work (`lib.rs:325-352`), close
pools, stop the server.

**Distributed tracing — Absent.** See [Build Status](#build-status). The config surface
exists; the implementation does not.

### Files that are never compiled

These files exist under `network-node/src/` but are declared in no `mod` statement anywhere in
the crate. They are not part of the build and cannot be relied on:

- `amazon_eks.rs`
- `governance.rs`
- `openapi.rs`
- `profiling.rs`
- `server.rs`
- `tls_utils.rs`

Note the trap: `grpc/server.rs` **is** used (declared at `grpc/mod.rs:17`, consumed at
`lib.rs:53` and `lib.rs:172`). The top-level `src/server.rs` is a different, orphaned file.

---

## Peer and Protocol Modules

**Short answer: "peer" is real as an API surface and as a data structure. It is not real as a
network.** Nodes never talk to each other.

### What exists — Implemented

[`network-node/src/p2p.rs`](../network-node/src/p2p.rs) (263 lines, declared at `lib.rs:36`)
contains a genuine Kademlia routing table:

- `KademliaRoutingTable` with 256 k-buckets (`p2p.rs:21-33`)
- XOR distance metric (`p2p.rs:36-42`) and bucket indexing (`p2p.rs:45-53`)
- K-bucket capacity of 20 (`p2p.rs:60`)
- `find_closest` returning the K nearest peers (`p2p.rs:70-85`)
- `P2PManager` tracking connected peers in a `HashMap` (`p2p.rs:88-101`)

It is genuinely instantiated and started: `lib.rs:90` constructs it, `lib.rs:225` starts its
maintenance loop, `lib.rs:242-247` bootstraps from `BOOTSTRAP_PEER` when set. It is passed to
both the HTTP server (`lib.rs:166`) and the gRPC server (`lib.rs:175`).

The gRPC `P2PService` (`proto/network.proto:44-49`, implemented in `grpc/p2p_service.rs`) is
a real, served surface: `ConnectToPeer`, `DisconnectFromPeer`, `GetPeerList`,
`BroadcastMessage`, `SyncChain`.

### What does not exist — Absent

**There is no peer transport.** The crate opens exactly three listening sockets, and none of
them is a peer port:

| Site | Purpose |
| --- | --- |
| `enhanced_server.rs:172` | Axum HTTP server |
| `soroban_rpc_client.rs:283` | test-only mock listener |
| `grpc/server.rs` | tonic gRPC server |

Configured bind addresses are HTTP, gRPC, and gateway only (`config.rs:166`, `config.rs:251-254`).

**The peer operations are stubs.** Each of the following logs its intent and returns success
without doing anything:

| Function | Location | Behaviour |
| --- | --- | --- |
| `bootstrap` | `p2p.rs:129-134` | Logs, returns `Ok(())`. The FIND_NODE steps are comments. |
| `handle_ping` | `p2p.rs:138-142` | Logs, returns `Ok(())`. Routing-table update is a comment. |
| `start_maintenance` | `p2p.rs:105-125` | Ticks every 30s and logs `"Pinging peers..."`. The ping/evict steps are comments; the cloned `routing_table` is unused, which the build log flags as `warning: unused variable: routing_table --> p2p.rs:106`. |
| `connect_to_peer` | `p2p.rs:146-176` | Inserts a `PeerInfo` with `latency_ms: 50, // Mock latency` (`p2p.rs:162`) and a random session id. No socket is opened. |
| `broadcast_message` | `p2p.rs:208-262` | Counts which target peers are in the map and returns the count. Sends nothing. |
| `sync_chain` | `grpc/p2p_service.rs:168-198` | `// TODO: Implement actual chain synchronization` (`p2p_service.rs:178`), then returns fabricated blocks built with `fastrand`. |

### How to read this

The `p2p` module is best understood as an in-memory peer *registry* with a correct Kademlia
distance implementation sitting behind it, exposed over gRPC so that operators and tests can
poke at it. Nothing populates it from the network, and nothing is transmitted between nodes.

The only remote parties this node actually communicates with are Stellar Horizon and Soroban
RPC — both plain HTTPS clients. If you are looking for the "protocol" in the blockchain
sense, it is Stellar's, and this repository consumes it rather than implementing it.

---

## Consensus Module Responsibilities

The issue asks for consensus responsibilities *"where applicable."* Both halves of the answer
matter here, because the naive readings in either direction are wrong.

### A consensus module exists — Present, not wired

[`network-node/src/consensus.rs`](../network-node/src/consensus.rs) is 605 lines, declared at
`lib.rs:25`, and implements a quorum-voting engine that is real code, not a placeholder:

- `Proposal` with id, proposer, payload, creation/expiry timestamps, required and current
  vote counts, and status (`consensus.rs:19-71`)
- `VoteType` — `Approve` / `Reject` / `Abstain` (`consensus.rs:11-16`)
- `Vote` carrying a signature and an optional W3C trace context (`consensus.rs:74-105`)
- `ConsensusEngine` (`consensus.rs:108-116`) with proposal and vote maps behind `RwLock`s and
  two unbounded broadcast channels
- `create_proposal` (`consensus.rs:147`), `vote` (`consensus.rs:181`) with duplicate-vote
  rejection (`consensus.rs:227`), `process_vote` for inbound votes (`consensus.rs:285`) with
  its own duplicate check (`consensus.rs:326`), `process_proposal` (`consensus.rs:370`)
- `finalize_proposal` on quorum, resolving by simple majority of approve over reject
  (`consensus.rs:400-450`)
- Expiry sweep and a 60-second background maintenance task (`consensus.rs:454`,
  `consensus.rs:543`)
- `ConsensusStats` for observability (`consensus.rs:596-605`)

### It is not connected to anything

Searching the whole repository for `ConsensusEngine` returns exactly two hits, both inside
`consensus.rs` itself: the struct definition (`consensus.rs:108`) and its `impl` block
(`consensus.rs:118`). It is never constructed.

The `NetworkNode` struct (`lib.rs:49-65`) has fields for `p2p_manager`, `state_trie`,
`soroban_service`, and eleven others. **There is no consensus field.** Nothing in
`NetworkNode::new` or `NetworkNode::start` touches the module.

All engine state is held in `HashMap`s in process memory (`consensus.rs:110-111`). It is
never persisted and, given the absence of peer transport described above, never exchanged.
The `vote_sender` and `proposal_sender` channels (`consensus.rs:112-113`) hand their receivers
back to the caller at construction time (`consensus.rs:129-130`) — and since there is no
caller, nothing ever drains them.

### Where agreement actually comes from — Implemented

Transaction ordering and finality are Stellar's, not Axionvera's:

- State-changing calls are submitted as Soroban transactions through
  `soroban_rpc_client.rs` / `soroban_service.rs` over HTTPS JSON-RPC. The default endpoint is
  `https://soroban-testnet.stellar.org:443` (`config.rs:30-32`).
- Confirmed state is read back by the event indexer polling Soroban every 5 seconds
  (`lib.rs:141-147`).
- The authoritative record is the vault contract's on-chain storage. See
  [`docs/contract-storage.md`](contract-storage.md) and
  [`../contracts/vault-contract/src/storage.rs`](../contracts/vault-contract/src/storage.rs).

Governance-style decisions that do exist are on-chain and contract-mediated: admin
authorization in [`../contracts/auth`](../contracts/auth), the delegation layer described in
[`../ARCHITECTURE.md`](../ARCHITECTURE.md), and chain parameter upgrades exposed through
`NetworkService.ParameterUpgrade` (`proto/network.proto:30`) backed by
`network-node/src/chain_params.rs`.

**Summary for contributors:** a self-contained consensus skeleton is checked in, it plays no
part in execution, and Axionvera's safety properties derive entirely from Stellar. Do not
plan work on the assumption that `consensus.rs` is load-bearing, and do not delete it on the
assumption that it is worthless — decide deliberately.

---

## Configuration Flow

Two distinct things in this repository are called "config." They are unrelated, and confusing
them is easy.

### 1. Node configuration — Implemented

[`network-node/src/config.rs`](../network-node/src/config.rs) defines `NetworkConfig`
(`config.rs:103-133`) and populates it in `NetworkConfig::from_env` (`config.rs:163-288`),
called once from `main.rs:19` before anything else starts.

**It is environment-variable only.** There is no config-file loading, no CLI parsing, and no
layered override. Note that the `config = "0.13"` crate (`network-node/Cargo.toml:25`) and
`clap` (`Cargo.toml:26`) are both declared as dependencies and used in no file under
`network-node/src/`.

| Variable | Default | Line |
| --- | --- | --- |
| `BIND_ADDRESS` | `0.0.0.0:8080` | `config.rs:165-166` |
| `GRPC_BIND_ADDRESS` | `0.0.0.0:50051` | `config.rs:251-252` |
| `GATEWAY_BIND_ADDRESS` | `0.0.0.0:8081` | `config.rs:253-254` |
| `DATABASE_URL` | `sqlite::memory:` | `config.rs:168-169` |
| `SHUTDOWN_GRACE_PERIOD` | `30` (seconds) | `config.rs:171-176` |
| `LOG_LEVEL` | `info` | `config.rs:178` |
| `NODE_ID` | generated `node-<uuid-prefix>` | `config.rs:180-189` |
| `TRACING_ENABLED` | `true` | `config.rs:191-194` |
| `TRACING_EXPORTER` | `otlp` (`jaeger` / `xray` / `none`) | `config.rs:196-203` |
| `CACHE_TTL_SECONDS` | `3600` | `config.rs:205-208` |
| `GENESIS_CONFIG_PATH` | unset | `config.rs:210` |
| `HORIZON_CONFIG` | JSON blob; falls back to default providers on parse error | `config.rs:212-219` |
| `SOROBAN_CONFIG` | JSON blob; same fallback | `config.rs:221-228` |
| `VAULT_CONTRACT_ADDRESS` | `CCDRM2F5H7...` *(placeholder)* | `config.rs:230-231` |
| `AWS_KMS_KEY_ID`, `AWS_REGION`, `AWS_PROFILE` | selects the KMS signer | `config.rs:233-240` |
| `LOCAL_SIGNER_KEY_PATH` | selects the local signer | `config.rs:241-245` |
| `BOOTSTRAP_PEER` | unset | `config.rs:259` |
| `TLS_CERT_PATH`, `TLS_KEY_PATH`, `TLS_CLIENT_CA_PATH` | unset | `config.rs:260-262` |
| `TLS_REQUIRE_CLIENT_AUTH` | `true` | `config.rs:263-266` |
| `ENABLE_GATEWAY` | `true` | `config.rs:267-270` |
| `ENABLE_REFLECTION` | `true` | `config.rs:271-274` |
| `LOG_DIR` | `logs` (read directly in `main.rs:130`) | `main.rs:130` |

Two behaviours worth knowing before you debug a misconfiguration:

- `DATABASE_URL` defaults to `sqlite::memory:`, but the pool is built with `PgPoolOptions`
  and only supports Postgres (`database.rs:3`, `database.rs:25`). The default value cannot
  work.
- `VAULT_CONTRACT_ADDRESS` defaults to the literal placeholder string `"CCDRM2F5H7..."`
  (`config.rs:231`, comment says `// Placeholder`). The indexer will be pointed at a
  nonexistent contract unless you set it.
- Invalid `HORIZON_CONFIG` / `SOROBAN_CONFIG` JSON is logged at `warn` and silently replaced
  with defaults (`config.rs:213-215`, `config.rs:222-224`) rather than failing startup.

**Genesis / chain parameters — Implemented.** When `GENESIS_CONFIG_PATH` is set,
`ChainParameterRegistry::from_genesis_file` loads it (`lib.rs:153-155`); otherwise a
development default is used (`lib.rs:156`). A sample lives at
[`../config/genesis.example.json`](../config/genesis.example.json). The registry is passed to
the gRPC server (`lib.rs:178`) and surfaced through `GetChainParameters` and
`ListPendingParameterUpgrades` (`proto/network.proto:31-32`).

### 2. The on-chain config contract — Present, not wired

[`../contracts/config`](../contracts/config) is a **separate Soroban contract**, unrelated to
node configuration. It is a complete, well-formed implementation: 303 lines in
`contracts/config/src/lib.rs` defining `ConfigContract` with `initialize`, `get_config`,
range-validated protocol parameters, and its own `errors`, `events`, `storage`, `types`, and
`test` modules.

**Nothing compiles it.** It is absent from the workspace `members` list
([`../Cargo.toml:2-24`](../Cargo.toml)), and searching the repository for `axionvera-config`
or `axionvera_config` outside its own directory returns zero results. See
[Workspace Membership](#workspace-membership-and-uncompiled-crates).

---

## Diagnostics Surface

### HTTP endpoints — Implemented

From the router at [`enhanced_server.rs:109-137`](../network-node/src/enhanced_server.rs):

| Endpoint | Method | Line |
| --- | --- | --- |
| `/health` | GET | `enhanced_server.rs:110` |
| `/ready` | GET | `enhanced_server.rs:111` |
| `/metrics` | GET | `enhanced_server.rs:112` |
| `/error-stats` | GET | `enhanced_server.rs:113` |
| `/circuit-breaker-status` | GET | `enhanced_server.rs:114-117` |
| `/health/liveness` | GET | `enhanced_server.rs:118` |
| `/health/readiness` | GET | `enhanced_server.rs:119` |
| `/stellar/account/:account_id` | GET | `enhanced_server.rs:120-123` |
| `/stellar/ledger/latest` | GET | `enhanced_server.rs:124-127` |
| `/stellar/providers/status` | GET | `enhanced_server.rs:128-131` |
| `/stellar/providers/switch` | POST | `enhanced_server.rs:132-135` |
| `/admin/memory/dump` | POST | `enhanced_server.rs:136` |
| `/admin/memory/stats` | GET | `enhanced_server.rs:137` |

The `/admin/memory/*` routes are backed by `memory_profiling.rs` and the `profiling` cargo
feature (`network-node/Cargo.toml:78-79`), which gates `tikv-jemalloc-ctl`. See
[`../MEMORY_PROFILING_GUIDE.md`](../MEMORY_PROFILING_GUIDE.md).

Note: `/health` and `/health/liveness` (and `/ready` vs `/health/readiness`) are separate
handlers registered under separate paths. If you are wiring Kubernetes probes, check
[`../k8s`](../k8s) for which pair is actually referenced rather than assuming.

### Prometheus metrics — Implemented

A second HTTP listener is installed on `0.0.0.0:9090` by `PrometheusBuilder` in
[`main.rs:25-29`](../network-node/src/main.rs), independent of the Axum server. Metric
descriptions are registered at `main.rs:32-51`:

| Metric | Type |
| --- | --- |
| `grpc_requests_total` | counter |
| `grpc_request_duration_seconds` | histogram |
| `soroban_rpc_errors_total` | counter |
| `soroban_rpc_failovers_total` | counter |
| `indexer_last_ledger_processed` | gauge |

Additional metrics are emitted by `MetricsCollector`
([`metrics.rs`](../network-node/src/metrics.rs)), which keeps atomic counters alongside the
`metrics` macros: `http_requests_total` (`metrics.rs:33`), `active_connections`
(`metrics.rs:39`), `errors_total` (`metrics.rs:45`), `bytes_sent_total` (`metrics.rs:51`),
`bytes_received_total` (`metrics.rs:57`), `request_duration_seconds` (`metrics.rs:62`),
`axionvera_pending_transactions_total` and `axionvera_transaction_queue_depth`
(`metrics.rs:68-69`).

### gRPC health — Implemented

`HealthServiceImpl` ([`grpc/health_service.rs:14`](../network-node/src/grpc/health_service.rs))
serves `Check` and streaming `Watch` (`proto/network.proto:53-55`). It probes the database
through `ConnectionPool::health_check` (`health_service.rs:24-29`) and downgrades overall
status to `NotServing` when the database is unreachable (`health_service.rs:52-54`).

Caveat: memory and CPU figures in the health details are hardcoded. `health_service.rs:56-57`
returns `"45%"` under the comment `// Check memory usage (mock implementation)`, repeated at
`health_service.rs:138`. Do not alert on them.

### Logging — Implemented

`init_basic_logging` (`main.rs:113-159`) sets up structured JSON logging to both stdout and a
daily-rotating file under `LOG_DIR` (default `logs/`), with thread ids, file, line, and
RFC-3339 UTC timestamps. Filtering honours `RUST_ENV`-style directives via `EnvFilter`, with a
fallback of `axionvera_network_node=<LOG_LEVEL>` (`main.rs:151-154`).

This is the only logging path that works. The three OpenTelemetry branches at `main.rs:57-68`
call functions that do not exist — see [Build Status](#build-status).

### Rate limiting — Implemented, with a caveat

[`rate_limiter.rs`](../network-node/src/rate_limiter.rs) supports a Redis backend with an
in-memory fallback (`rate_limiter.rs:13-16`), selecting Redis when a URL is supplied and the
connection succeeds, and warning-and-degrading otherwise (`rate_limiter.rs:25-40`).

Caveat: `rate_limiter.rs:9` gates the `redis::AsyncCommands` import behind
`#[cfg(feature = "redis-backend")]`, but `network-node/Cargo.toml:78-79` declares only a
`profiling` feature. **`redis-backend` does not exist**, so that `cfg` is permanently false.

---

## Workspace Membership and Uncompiled Crates

This section exists because the directory listing of `contracts/` is misleading: 34
directories are present, and 13 of them are built by nothing.

### How membership works here

The root [`Cargo.toml`](../Cargo.toml) is a **virtual manifest** — `[workspace]` with no
`[package]`. This is why no application dependencies appear in it; the node's dependencies all
live in [`network-node/Cargo.toml`](../network-node/Cargo.toml).

`Cargo.toml:2-24` lists 21 explicit members. Cargo additionally treats any `path` dependency
located inside the workspace directory as an implicit member, so the real member set is larger
than the list suggests.

### Implicit members — Implemented

These four are absent from `members` but are still built, because explicit members depend on
them by path:

| Crate | Pulled in by |
| --- | --- |
| `contracts/fees` | `contracts/vault-contract/Cargo.toml:17` |
| `contracts/security` | `contracts/vault-contract/Cargo.toml:20`, `tests/security/Cargo.toml:10`, `tests/risk/Cargo.toml:15` |
| `contracts/risk` | `contracts/vault-contract/Cargo.toml:21`, `tests/risk/Cargo.toml:9` |
| `contracts/snapshots` | `contracts/core/Cargo.toml:16` |

### Genuinely orphaned crates — Present, not wired

No member depends on these, directly or transitively. `cargo build --workspace` skips them in
silence:

`assets`, `capabilities`, `config`, `context`, `lifecycle`, `metrics`, `monitoring`, `policy`,
`registry`, `replay`, `scheduler`, `treasury`, `upgrades`

`contracts/capabilities` is a special case: its only referent is
`tests/capabilities/Cargo.toml:9` — and `tests/capabilities` is itself absent from `members`.
Two orphans pointing at each other.

**They cannot be built standalone either.** Almost all of them declare
`soroban-sdk = { workspace = true }`, which requires inheriting from the parent workspace.
Since they are neither members nor excluded, running `cargo build` inside one of those
directories fails with:

```
error: current package believes it's in a workspace when it's not
```

Three are additionally broken at the manifest level:

| Crate | Problem |
| --- | --- |
| `contracts/metrics` | `Cargo.toml` is **0 bytes** — an entirely empty file — while `src/lib.rs`, `src/test.rs`, and two test snapshots exist |
| `contracts/upgrades` | `license` and `publish` keys appear *after* the `[dependencies]` header, so TOML parses them as dependency entries. Also declares an empty `[workspace]` while using `soroban-sdk = { workspace = true }`, leaving nothing to inherit |
| `contracts/treasury` | Same empty-`[workspace]` conflict with `soroban-sdk = { workspace = true }` |

The `tests/` tree has the same problem. `tests/auth`, `tests/security`, and `tests/risk` are
members; `tests/capabilities` and `tests/invariants` are Rust crates that are not. And
`tests/metrics/metrics_test.rs`, `tests/storage/versioning_test.rs`, and
`tests/upgrades/validator_test.rs` are loose `.rs` files belonging to no crate at all.

### What actually builds

The dependency closure rooted at `contracts/vault-contract` — `auth`, `events`, `accounting`,
`core`, `fees`, `interfaces`, `storage`, `security`, `risk`, and transitively `state`,
`resources`, `snapshots` — is the part of the contract layer that is compiled and tested by
`npm run build:contracts` and `cargo test -p axionvera-vault-contract`.

---

## Known Gaps

Collected in one place, with the caveats that most affect day-to-day work.

**Build**
- `axionvera-network-node` does not compile (lib and bin). See [Build Status](#build-status).
- CI does not build or test the node at all.

**Test code that has drifted from the source**
- `signing.rs:464-465` declares `#[cfg(test)] mod signing_tests;` from inside `src/signing.rs`,
  which Rust resolves to `src/signing/signing_tests.rs`. That directory does not exist — the
  file is at `src/signing_tests.rs`. A `#[path]` attribute is missing.
- `lib.rs:396-421` constructs a `NetworkConfig` without the `tls_client_ca_path` and
  `tls_require_client_auth` fields, both of which the struct requires (`config.rs:115`,
  `config.rs:117`).
- `lib.rs:426` calls `node.clone()` on a `NetworkNode` that does not derive `Clone`.

**Declared dependencies with no consumer**

Checked across `network-node/src/`: `clap` (`Cargo.toml:26`), `config` (`Cargo.toml:25`),
`stellar_horizon` (`Cargo.toml:61`), `opentelemetry` / `opentelemetry_sdk` /
`tracing-opentelemetry` (`Cargo.toml:52-54`). The `slog` family (`Cargo.toml:27-29`) is
likewise unused alongside `tracing`.

**Claims in `../ARCHITECTURE.md` that the code contradicts**

`ARCHITECTURE.md` is not purely aspirational — Axum, PostgreSQL, Redis, and circuit breakers
are all genuinely present. These specific details are wrong:

| Claim | Location | Reality |
| --- | --- | --- |
| Connection pooling via `bb8` or `deadpool` | `ARCHITECTURE.md:70` | Neither is a dependency. Pooling is `sqlx::PgPoolOptions` (`database.rs:25-31`) |
| Driver is `tokio-postgres` | `ARCHITECTURE.md:122` | Not a dependency. The driver is `sqlx` with the `postgres` feature (`network-node/Cargo.toml:66-71`) |
| Tables `requests_log` and `user_cache` | `ARCHITECTURE.md:126-138` | Neither exists. The schema is `indexer_state`, `events`, `deposits` (`database.rs:51`, `:77`, `:124`) |
| Circuit breaker trips after >10 failures/minute | `ARCHITECTURE.md:75` | The configurable threshold defaults to `3` (`config.rs:88`) |
| ALB, AWS VPC, background worker node | `ARCHITECTURE.md:15-22, 51-52` | *Documented, not verified* — nothing in `network-node/src/` corresponds to these. Check [`../terraform`](../terraform) and [`../k8s`](../k8s) directly |

The em-dash rendering in `ARCHITECTURE.md` is **not** a file problem: the file is valid UTF-8
with no BOM and correctly encoded `U+2014` characters. If it looks like `â€"` in your
terminal, that is your console code page.

**Broken links in `architecture.md`**

[`docs/architecture.md`](architecture.md) lines 12, 13, 14, 15, and 48 contain absolute
`file:///Users/boufdaddy/…` paths from a contributor's local machine. Line 73 points at
`../../ARCHITECTURE.md`, which is one level too high from `docs/`. Those files are correctly
reachable as:

- [`../contracts/vault-contract/src/lib.rs`](../contracts/vault-contract/src/lib.rs)
- [`../contracts/vault-contract/src/storage.rs`](../contracts/vault-contract/src/storage.rs)
- [`../contracts/vault-contract/src/events.rs`](../contracts/vault-contract/src/events.rs)
- [`../contracts/vault-contract/src/errors.rs`](../contracts/vault-contract/src/errors.rs)
- [`../ARCHITECTURE.md`](../ARCHITECTURE.md)

---

## Working Safely In This Repository

- **Contracts are the solid ground.** `contracts/vault-contract` and its dependency closure
  build, test, and run in CI. Start there.
- **Check workspace membership before you start.** If your crate is not in
  [`../Cargo.toml:2-24`](../Cargo.toml) and no member depends on it by path, nothing you write
  there will be compiled or tested. Add it to `members` first.
- **Do not trust a green CI run for node changes.** No workflow compiles `network-node`.
- **Grep before you extend.** Several substantial modules — `consensus.rs`, `contracts/config`,
  the orphaned `src/*.rs` files — look load-bearing and are not. Confirm a module has a caller
  before building on it.
- **Prefer code over prose when they disagree,** including this document. Re-verify the
  anchors above against the current tree.

---

## Related Documentation

| Document | Scope |
| --- | --- |
| [`../README.md`](../README.md) | Project entry point and quickstart |
| [`../ARCHITECTURE.md`](../ARCHITECTURE.md) | Network topology, upgradeability, delegation layer — see caveats above |
| [`architecture.md`](architecture.md) | Vault contract internals: storage design, reward accounting |
| [`contract-spec.md`](contract-spec.md) | Contract interface specification |
| [`contract-storage.md`](contract-storage.md) | On-chain storage walkthrough |
| [`EVENTS.md`](EVENTS.md) | Event catalogue |
| [`CROSS_CONTRACT.md`](CROSS_CONTRACT.md) | Cross-contract call patterns |
| [`ORCHESTRATOR.md`](ORCHESTRATOR.md) | Orchestrator contract |
| [`grpc-implementation.md`](grpc-implementation.md) | gRPC service details |
| [`horizon-fallback.md`](horizon-fallback.md) | Horizon multi-provider failover |
| [`MTLS_IMPLEMENTATION.md`](MTLS_IMPLEMENTATION.md), [`MTLS_ROTATION.md`](MTLS_ROTATION.md) | Transport security |
| [`architecture/dependency-graph.md`](architecture/dependency-graph.md) | Crate dependency graph |
| [`architecture/upgradeability-framework.md`](architecture/upgradeability-framework.md) | Upgrade framework |
| [`../AWS_KMS_INTEGRATION.md`](../AWS_KMS_INTEGRATION.md) | KMS signer setup |
| [`../MEMORY_PROFILING_GUIDE.md`](../MEMORY_PROFILING_GUIDE.md) | Memory profiling endpoints |
| [`../CONTRIBUTING.md`](../CONTRIBUTING.md) | Contribution workflow |
