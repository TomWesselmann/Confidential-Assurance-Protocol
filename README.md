# 🔐 Confidential Assurance Protocol (CAP)

**Production-Grade Compliance Proof System with Zero-Knowledge Privacy**

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Build](https://github.com/TomWesselmann/Confidential-Assurance-Protocol/actions/workflows/security.yml/badge.svg)](https://github.com/TomWesselmann/Confidential-Assurance-Protocol/actions)
[![Tests](https://img.shields.io/badge/tests-146%2F146-brightgreen.svg)](./agent/src/)
[![Version](https://img.shields.io/badge/version-0.11.0-blue.svg)](./agent/CLAUDE.md)
[![Docker](https://img.shields.io/badge/docker-ready-blue.svg)](./agent/Dockerfile)
[![License](https://img.shields.io/badge/license-Proprietary-red.svg)](#license)

---

## 📋 Overview

The **Confidential Assurance Protocol (CAP)** is a comprehensive Rust-based system for generating, managing, and verifying cryptographic compliance proofs with zero-knowledge privacy guarantees. Originally developed for the German Supply Chain Due Diligence Act (LkSG), CAP provides enterprise-grade tools for:

- ✅ **Supply Chain Compliance** - Cryptographic commitments for supplier & UBO data
- ✅ **Privacy-Preserving Proofs** - Zero-knowledge proofs for sensitive compliance checks
- ✅ **REST API** - OAuth2-secured HTTP API for proof verification
- ✅ **Observability** - Prometheus metrics + Grafana dashboards
- ✅ **Production-Ready** - Docker/Kubernetes deployment with CI/CD

---

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+ (`rustup`)
- Docker (optional, for containerized deployment)
- Kubernetes cluster (optional, for production deployment)

### Installation

```bash
git clone https://github.com/TomWesselmann/Confidential-Assurance-Protocol.git
cd Confidential-Assurance-Protocol/agent
cargo build --release
```

### Basic Workflow (CLI)

```bash
# 1. Generate commitments from CSV data
cargo run --release -- prepare \
  --suppliers examples/suppliers.csv \
  --ubos examples/ubos.csv

# 2. Validate policy
cargo run --release -- policy validate \
  --file examples/lksg_v1.policy.yml

# 3. Build manifest
cargo run --release -- manifest build \
  --policy examples/lksg_v1.policy.yml

# 4. Create proof
cargo run --release -- proof build \
  --manifest build/manifest.json \
  --policy examples/lksg_v1.policy.yml

# 5. Verify proof
cargo run --release -- proof verify \
  --proof build/proof.dat \
  --manifest build/manifest.json
```

### REST API Server

```bash
# Start REST API (OAuth2 + Prometheus)
cargo run --release --bin cap-verifier-api

# API available at http://localhost:8080
# Metrics at http://localhost:8080/metrics
```

See [API Documentation](#rest-api) for endpoint details.

---

## 🏗️ Architecture

```
┌──────────────────────────────────────────────────────────────────────┐
│               Confidential Assurance Protocol v0.11.0                │
│         Production-Grade Compliance Proof System (Phase 1)           │
└──────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────┐
│                         REST API Layer (v0.11.0)                     │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │  OAuth2 Middleware (JWT RS256) + Prometheus Metrics           │ │
│  │  Endpoints: /verify, /policy/*, /healthz, /readyz, /metrics   │ │
│  └────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────┘
                               ↓
┌──────────────────────────────────────────────────────────────────────┐
│                         Core Processing Layer                        │
│                                                                      │
│  Input Layer:   CSV Data → BLAKE3 Merkle Roots                     │
│  Policy Layer:  YAML/JSON → Policy Validation + Compilation         │
│  Proof Layer:   ZK Proofs (Mock → Production-Ready)                │
│  Registry:      SQLite + BLOB Store (Content-Addressable)          │
│  Crypto:        BLAKE3, SHA3-256, Ed25519 + KID Rotation           │
└──────────────────────────────────────────────────────────────────────┘
                               ↓
┌──────────────────────────────────────────────────────────────────────┐
│                         Output Layer                                 │
│  Proof Packages:  Offline-verifiable proof bundles (CAPZ v2)       │
│  REST Responses:  JSON API responses                                │
│  Metrics:         Prometheus text format                            │
│  Audit Trail:     SHA3-256 hash-chained event log (JSONL)          │
└──────────────────────────────────────────────────────────────────────┘
```

**Detailed Architecture:** [agent/CLAUDE.md](./agent/CLAUDE.md)

---

## 🛠️ Components

### Core Agent (Rust)

| Component | Description | Status |
|-----------|-------------|--------|
| **Commitment Engine** | BLAKE3 Merkle roots for supplier/UBO data | ✅ Complete |
| **Policy Compiler** | YAML/JSON policy validation + IR generation | ✅ Complete |
| **Proof Engine** | Zero-knowledge proof generation & verification | ✅ Complete |
| **Registry** | SQLite-backed proof registry with BLOB store | ✅ Complete |
| **Key Management** | Ed25519 signing with KID-based key rotation | ✅ Complete |
| **Verifier Core** | Portable, I/O-free proof verification | ✅ Complete |

### REST API (Axum)

| Endpoint | Method | Auth | Description |
|----------|--------|------|-------------|
| `/healthz` | GET | None | Health check |
| `/readyz` | GET | None | Readiness check |
| `/metrics` | GET | None | Prometheus metrics |
| `/verify` | POST | OAuth2 | Verify proof against policy |
| `/policy/compile` | POST | OAuth2 | Compile policy to IR |
| `/policy/:id` | GET | OAuth2 | Retrieve compiled policy |

**Security:** JWT RS256 validation + audience/issuer/scope checks

### Infrastructure

| Component | Description | Status |
|-----------|-------------|--------|
| **Docker** | Multi-stage build with security scanning | ✅ Complete |
| **Kubernetes** | Helm charts + manifests (dev/stage/prod) | ✅ Complete |
| **CI/CD** | GitHub Actions (build, test, security, SBOM) | ✅ Complete |
| **Monitoring** | Prometheus metrics + Grafana dashboards | ✅ Complete |

---

## 📊 Features

### Phase 1 (✅ 100% Complete)

- [x] **TLS/mTLS** - Secure communication (via Kubernetes Ingress)
- [x] **Prometheus Metrics** - Request rate, duration, auth failures, cache hit ratio
- [x] **Docker/K8s Deployment** - Production-ready containerization
- [x] **SBOM + Security Scanning** - CycloneDX SBOM + cargo-audit integration

### Core Features (✅ Stable)

- [x] **Offline-First** - No network dependencies, fully deterministic
- [x] **Cryptographic Security** - BLAKE3, SHA3-256, Ed25519
- [x] **Audit Trail** - SHA3-chained hash log (append-only)
- [x] **Key Rotation** - KID-based key management with chain-of-trust
- [x] **Policy Validation** - Schema-based policy enforcement
- [x] **Proof Packaging** - Standardized CAPZ v2 container format
- [x] **BLOB Storage** - Content-addressable storage with garbage collection
- [x] **Registry** - SQLite backend with transaction support

---

## 🧪 Testing

### Run Tests

```bash
cd agent
cargo test                      # All tests
cargo test --lib                # Library tests only
cargo test --test <name>        # Specific integration test
cargo clippy -- -D warnings     # Lint check
```

### Test Coverage

- ✅ **146/146 Tests Passing** (100%)
- ✅ **0 Clippy Warnings**
- ✅ **57 Library Unit Tests**
- ✅ **65 Binary Unit Tests**
- ✅ **24 Integration Tests**

### Test Categories

| Category | Tests | Description |
|----------|-------|-------------|
| Crypto | 11 | SHA3, BLAKE3, Ed25519, Hex encoding |
| Verifier Core | 6 | Statement extraction, verification logic |
| Registry | 13 | CRUD operations, timestamps, entry signing |
| Key Management | 9 | KID derivation, metadata, key store ops |
| BLOB Store | 6 | Storage, metadata, garbage collection |
| Policy | 7 | Validation, YAML loading, constraints |
| Proof Engine | 3 | Proof generation, verification, serialization |
| Integration | 24 | Bundle creation, dual-anchor, registry migration |

---

## 🔐 Security

### Cryptographic Primitives

| Function | Algorithm | Usage |
|----------|-----------|-------|
| Merkle Roots | BLAKE3 | Supplier/UBO/Sanctions commitments |
| Audit Chain | SHA3-256 | Append-only event log |
| Policy Hash | SHA3-256 | Policy identification |
| Signatures | Ed25519 | Manifest signing + registry entries |
| Encoding | Base64 | Proof serialization (CAPZ format) |

### Security Features

- ✅ **OAuth2 Client Credentials Flow** - JWT RS256 token validation
- ✅ **Scope-Based Authorization** - Fine-grained access control
- ✅ **Key Rotation** - KID-based key management with attestation
- ✅ **Audit Trail** - Immutable SHA3-chained event log
- ✅ **SBOM Generation** - CycloneDX format for supply chain security
- ✅ **Dependency Scanning** - Automated vulnerability checks (cargo-audit)

---

## 🚢 Deployment

### Docker

```bash
cd agent
docker build -t cap-agent:0.11.0 .
docker run -p 8080:8080 cap-agent:0.11.0
```

### Kubernetes (Helm)

```bash
cd agent/helm/cap-verifier
helm install cap-verifier . \
  -f values-prod.yaml \
  --namespace cap-system \
  --create-namespace
```

### Configuration

See [agent/DEPLOYMENT.md](./agent/DEPLOYMENT.md) for:
- Environment variables
- TLS/mTLS configuration
- OAuth2 setup
- Prometheus/Grafana integration
- Production checklist

---

## 📈 Monitoring

### Prometheus Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `cap_verifier_requests_total` | Counter | Total requests (by result: ok/fail/warn) |
| `cap_auth_token_validation_failures_total` | Counter | Authentication failures |
| `cap_cache_hit_ratio` | Gauge | Cache effectiveness (0.0-1.0) |
| `cap_verifier_request_duration_seconds` | Histogram | Request latency (p50/p95/p99) |

### Grafana Dashboard

Pre-built dashboard available at [agent/grafana-dashboard.json](./agent/grafana-dashboard.json)

Includes:
- Request rate by result (time series)
- Authentication failures (stat)
- Success rate % (gauge)
- Request duration percentiles (time series)
- Cache hit ratio (gauge)

---

## 🌍 REST API

### Authentication

All protected endpoints require OAuth2 Bearer token (JWT RS256):

```bash
TOKEN="your-jwt-token"
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/verify \
  -d '{"policy_id": "...", "context": {...}}'
```

### Generate Mock Token (Development)

```bash
cargo run --example generate_mock_token
```

### Example Request

```bash
# Compile Policy
curl -X POST http://localhost:8080/policy/compile \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "policy": {
      "version": "lksg.v1",
      "name": "Test Policy",
      "constraints": {
        "require_at_least_one_ubo": true,
        "supplier_count_max": 10
      }
    }
  }'

# Verify Proof
curl -X POST http://localhost:8080/verify \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d @verify-request.json
```

Full API spec: [agent/openapi/openapi.yaml](./agent/openapi/openapi.yaml)

---

## 📚 Documentation

### Core Documentation

- **Main Documentation:** [agent/CLAUDE.md](./agent/CLAUDE.md)
- **Deployment Guide:** [agent/DEPLOYMENT.md](./agent/DEPLOYMENT.md)
- **System Architecture:** [Systemarchitekur/SYSTEMARCHITEKTUR_v0.8.0.md](./Systemarchitekur/SYSTEMARCHITEKTUR_v0.8.0.md)

### Design Documents (PRDs)

Located in [MD/](./MD/) directory:
- [PRD_REST_Verifier_API.md](./MD/PRD_REST_Verifier_API.md)
- [PRD_Docker_K8s_Container_CAP_Verifier.md](./MD/PRD_Docker_K8s_Container_CAP_Verifier.md)
- [Key_Management_KID_Rotation.md](./MD/Key_Management_KID_Rotation.md)
- [BLOB_Store_CLI_PRD.md](./MD/BLOB_Store_CLI_PRD.md)
- [PRD_Policy_Compiler_v1.md](./MD/PRD_Policy_Compiler_v1.md)

### Runbooks

- [Backup & Restore](./agent/docs/runbook_restore.md)
- [Key Rotation](./agent/docs/runbook_rotation.md)
- [Week 5 Runbooks](./agent/docs/Week5_Runbooks.md)

---

## 🎯 Roadmap

### Phase 2 (In Planning)

- [ ] **SAP S/4HANA Adapter** - Direct integration with SAP systems
- [ ] **Policy Compiler Finalization** - Advanced policy features
- [ ] **Adaptive Proof Orchestrator** - Dynamic proof selection

### Future (v0.12.0+)

- [ ] **Real Zero-Knowledge** - Halo2/RISC Zero integration
- [ ] **Blockchain Anchoring** - Ethereum/Hedera timestamp anchoring
- [ ] **Web Verifier** - WASM-based browser verification
- [ ] **Multi-Signature Support** - Chain-of-trust for registry entries

---

## 🛠️ Development

### Prerequisites

```bash
rustup update
cargo install cargo-audit cargo-cyclonedx
```

### Build

```bash
cargo build --release
```

### Lint

```bash
cargo clippy -- -D warnings
```

### Benchmarks

```bash
cargo bench --bench registry_bench
```

### Documentation

```bash
cargo doc --open
```

---

## 📄 License

**© 2025 Confidential Assurance Protocol – Core Engineering**

**All Rights Reserved.**

This software is proprietary and confidential. Unauthorized copying, distribution, or use is strictly prohibited.

---

## 🙏 Acknowledgments

Built with:
- **Rust** (Edition 2021)
- **Axum** (Web framework)
- **Tokio** (Async runtime)
- **BLAKE3** (Merkle trees)
- **SHA3** (Audit chain)
- **Ed25519-dalek** (Digital signatures)
- **SQLite/rusqlite** (Registry backend)
- **Prometheus** (Metrics)
- **Docker** (Containerization)
- **Kubernetes/Helm** (Orchestration)

---

## 📞 Support

- **Issues:** https://github.com/TomWesselmann/Confidential-Assurance-Protocol/issues
- **Documentation:** [agent/CLAUDE.md](./agent/CLAUDE.md)

---

**Status:** Phase 1 Complete (Production-Ready) | Phase 2 Planning
**Version:** 0.11.0
**Build:** ✅ 146/146 Tests Passing | ✅ 0 Clippy Warnings | ✅ Docker Ready
**CI/CD:** ✅ GitHub Actions | ✅ Security Scanning | ✅ SBOM Generation
