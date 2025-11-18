# Confidential Assurance Protocol (CAP)

**Production-Ready Compliance Proof System for Supply Chain Due Diligence**

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/TomWesselmann/Confidential-Assurance-Protocol)
[![Version](https://img.shields.io/badge/version-0.11.0-blue)](https://github.com/TomWesselmann/Confidential-Assurance-Protocol/releases)
[![License](https://img.shields.io/badge/license-All%20Rights%20Reserved-red)](LICENSE)
[![Docker](https://img.shields.io/badge/docker-ready-2496ED?logo=docker)](agent/DOCKER_DEPLOYMENT.md)
[![Monitoring](https://img.shields.io/badge/monitoring-production--ready-success)](agent/monitoring/)

---

## 📖 Overview

The **Confidential Assurance Protocol (CAP)** is a cryptographic compliance proof system designed for the German Supply Chain Due Diligence Act (Lieferkettensorgfaltspflichtengesetz - LkSG). It enables companies to prove compliance with supply chain regulations **without revealing sensitive business data**.

### Key Features

✅ **Zero-Knowledge Proofs** - Prove compliance without disclosing raw data
✅ **Cryptographic Commitments** - BLAKE3 Merkle roots + SHA3-256 audit trails
✅ **Policy Engine** - Flexible YAML-based compliance rules
✅ **REST API** - OAuth2-secured endpoints for system integration
✅ **Web UI** - React-based interface for proof upload and verification (v0.11.0)
✅ **Production Monitoring** - Full observability stack (Prometheus, Grafana, Loki, Jaeger)
✅ **Audit Trail** - Immutable SHA3-256 hash chain for all operations
✅ **Key Management** - Ed25519 signing with key rotation and attestation
✅ **Docker & Kubernetes** - Production-ready deployment configs

---

## 🚀 Quick Start

### Prerequisites

- **Docker** (for containerized deployment)
- **Rust** 1.70+ (for local development)
- **Git** (to clone the repository)

### 1. Clone Repository

```bash
git clone https://github.com/TomWesselmann/Confidential-Assurance-Protocol.git
cd Confidential-Assurance-Protocol/agent
```

### 2. Start Monitoring Stack (Optional but Recommended)

```bash
cd monitoring
docker compose up -d
./test-monitoring.sh
```

**Access:**
- Grafana: http://localhost:3000 (admin/admin)
- Prometheus: http://localhost:9090
- Jaeger: http://localhost:16686

### 3. Run CAP Agent

**Option A: Using Docker** (Recommended)
```bash
docker pull ghcr.io/tomwesselmann/cap-agent:v0.11.0-alpine
docker run --rm -p 8080:8080 ghcr.io/tomwesselmann/cap-agent:v0.11.0-alpine
```

**Option B: From Source**
```bash
cd agent
cargo build --release
./target/release/cap-agent --help
```

### 4. Verify Installation

```bash
# Health check
curl http://localhost:8080/healthz

# Expected output:
# {"status":"OK","version":"0.1.0","build_hash":null}
```

---

## 📚 Documentation

### Primary Documentation
- **[CLAUDE.md](agent/CLAUDE.md)** - Complete technical documentation (all features, architecture, APIs, examples)
- **[DOCKER_DEPLOYMENT.md](agent/DOCKER_DEPLOYMENT.md)** - Docker & WebUI deployment guide
- **[GETTING_STARTED.md](agent/docs/GETTING_STARTED.md)** - Beginner-friendly Quick Start
- **[WEBUI_BACKEND_STATUS.md](agent/WEBUI_BACKEND_STATUS.md)** - WebUI integration status

### Deployment & Operations
- **[Monitoring README](agent/monitoring/README.md)** - Observability stack guide (Prometheus, Grafana, Loki, Jaeger)
- **[Kubernetes Deployment](agent/kubernetes/README.md)** - K8s deployment configs
- **[SLO README](agent/monitoring/slo/README.md)** - SLO/SLI monitoring and error budgets

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    CAP System v0.11.0                           │
├─────────────────────────────────────────────────────────────────┤
│  REST API Layer (Axum)                                         │
│  ├─ OAuth2 Middleware (JWT RS256)                              │
│  ├─ /healthz, /readyz (Public)                                 │
│  ├─ /verify (Protected)                                        │
│  └─ /policy/* (Protected)                                      │
├─────────────────────────────────────────────────────────────────┤
│  Core Processing Layer                                         │
│  ├─ Commitment Engine (BLAKE3 Merkle Roots)                    │
│  ├─ Policy Engine (YAML-based Rules)                           │
│  ├─ Proof Engine (ZK-Ready, currently SimplifiedZK)            │
│  ├─ Verifier Core (I/O-free, portable)                         │
│  └─ Audit Trail (SHA3-256 Hash Chain)                          │
├─────────────────────────────────────────────────────────────────┤
│  Storage Layer                                                  │
│  ├─ Registry (JSON or SQLite)                                  │
│  ├─ BLOB Store (Content-Addressable Storage)                   │
│  └─ Key Store (Ed25519 with KID rotation)                      │
├─────────────────────────────────────────────────────────────────┤
│  Observability Layer (Week 2)                                  │
│  ├─ Metrics (Prometheus)                                       │
│  ├─ Logs (Loki + Promtail)                                     │
│  ├─ Traces (Jaeger)                                            │
│  └─ Dashboards (Grafana: Main + SLO)                           │
└─────────────────────────────────────────────────────────────────┘
```

**Key Principles:**
- **Privacy by Design** - Zero-knowledge proofs keep data confidential
- **Defense in Depth** - Multiple security layers (Crypto, TLS, OAuth2)
- **Audit-First** - Every action is logged in immutable hash chain
- **Production-Ready** - Full monitoring, health checks, graceful shutdown

---

## 💡 Usage Examples

### Web UI: Upload & Verify Proof (Easiest)

```bash
# 1. Start Backend API
cd agent
cargo run --bin cap-verifier-api &

# 2. Start WebUI
cd webui
npm install
npm run dev

# 3. Compile Policy
curl -X POST http://localhost:8080/policy/v2/compile \
  -H "Authorization: Bearer admin-tom" \
  -H "Content-Type: application/json" \
  -d @examples/policy_v2_request.json

# 4. Open Browser
open http://localhost:5173

# 5. Upload proof package ZIP and click "Proof Verifizieren"
```

**Features:**
- Drag & Drop proof package upload
- Visual manifest display
- One-click verification
- Detailed results with status badges

### CLI: Generate Proof

```bash
# 1. Import supplier and UBO data
cap-agent prepare \
  --suppliers examples/suppliers.csv \
  --ubos examples/ubos.csv

# 2. Build compliance manifest
cap-agent manifest build \
  --policy examples/policy.lksg.v1.yml

# 3. Generate proof
cap-agent proof build \
  --manifest build/manifest.json \
  --policy examples/policy.lksg.v1.yml

# 4. Export proof package
cap-agent proof export \
  --manifest build/manifest.json \
  --proof build/proof.dat \
  --out build/cap-proof
```

### REST API: Verify Proof

```bash
# Generate JWT token (for testing)
TOKEN=$(cargo run --example generate_mock_token 2>&1 | grep "^eyJ" | head -1)

# Compile policy
curl -X POST http://localhost:8080/policy/compile \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d @examples/policy_request.json

# Verify proof
curl -X POST http://localhost:8080/verify \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d @examples/verify_request.json
```

---

## 🔐 Security

### Authentication
- **OAuth2 Client Credentials Flow** with JWT (RS256)
- **TLS/mTLS** support for production deployments
- **Scope-based authorization** for fine-grained access control

### Cryptography
- **BLAKE3** - Merkle roots (commitment engine)
- **SHA3-256** - Hash chain audit log
- **Ed25519** - Digital signatures
- **Zero-Knowledge Proofs** - SimplifiedZK (Halo2 in development)

### Audit & Compliance
- **Immutable Audit Trail** - SHA3-256 hash chain
- **Key Rotation** - Automated KID-based key management
- **DSGVO-compliant** - Privacy by design

### Security Audits
- ✅ **cargo audit** in CI/CD pipeline
- 🔄 **External audit** planned (Q1 2026)

---

## 📊 Monitoring & Observability (Week 2)

**Production-Ready Monitoring Stack:**
- **8 Services**: Prometheus, Grafana, Loki, Promtail, Jaeger, Node Exporter, cAdvisor
- **2 Dashboards**: Main Dashboard (13 panels) + SLO Dashboard (17 panels)
- **4 SLOs**: Availability (99.9%), Error Rate (<0.1%), Auth Success (99.95%), Cache Hit (>70%)
- **11 Alert Rules**: Critical (3), Warning (4), Info (2), SLO-based (1)
- **Full Correlation**: Logs ↔ Traces ↔ Metrics

**Quick Start:**
```bash
cd agent/monitoring
docker compose up -d
```

**Details:** [monitoring/README.md](agent/monitoring/README.md)

---

## 🛠️ Development

### Build from Source

```bash
cd agent
cargo build --release
```

### Run Tests

```bash
# All tests (146 tests)
cargo test

# Integration tests only
cargo test --test '*'

# Specific module
cargo test crypto::
```

### Code Quality

```bash
# Linting
cargo clippy -- -D warnings

# Formatting
cargo fmt --check

# Security audit
cargo audit
```

### Benchmarks

```bash
# Registry performance
cargo bench --bench registry_bench

# Policy compilation
cargo bench --bench compile_bench
```

---

## 📦 Deployment

### Docker

```bash
# Build production image
docker build -f Dockerfile.optimized -t cap-agent:v0.11.0-alpine .

# Run container
docker run -d -p 8080:8080 \
  --name cap-verifier-api \
  cap-agent:v0.11.0-alpine
```

**Guide:** [DOCKER_DEPLOYMENT.md](agent/DOCKER_DEPLOYMENT.md)

### Kubernetes

```bash
# Apply K8s manifests
kubectl apply -k agent/kubernetes/

# Check deployment
kubectl get pods -l app=cap-verifier-api
```

**Guide:** [kubernetes/README.md](agent/kubernetes/README.md)

### Docker Compose (Monitoring)

```bash
cd agent/monitoring
docker compose up -d
```

---

## 🗺️ Roadmap

### ✅ Completed (v0.11.0)
- CLI & Core Features (Commitment, Policy, Proof, Verifier)
- REST API with OAuth2 (JWT RS256)
- TLS/mTLS Support
- Key Management with KID Rotation
- Registry (JSON + SQLite)
- BLOB Store (Content-Addressable Storage)
- **Web UI** (React + TypeScript + Vite) - Upload & Verification ✨
- Production Monitoring Stack (Prometheus, Grafana, Loki, Jaeger)
- Docker & Kubernetes Deployment
- Comprehensive Documentation

### 🔄 In Progress (Week 3-6)
- **Week 3-4**: Halo2 ZK-Proofs Implementation
- **Week 5**: SAP Adapter (OData v4)
- **Week 6**: Web UI Enhancements (CSV Import, Multi-Policy Support)
- **Security Audit**: External mini-audit

### 📅 Planned (MVP v1.0 - Dec 31, 2025)
- Halo2-based Zero-Knowledge Proofs
- SAP Integration (automated data import)
- Web UI Full Features (CSV Import, Multi-Policy, Signature Verification)
- Security Audit completed
- Production Deployment (Docker + K8s)

### 🚀 Future (v2.0 - 2026)
- Multi-Tenancy
- HSM Integration (PKCS#11)
- Blockchain Time Anchoring
- Additional ERP Integrations (Oracle, Dynamics)
- SOC 2 & ISO 27001 Certification

**Full Details:** See [CLAUDE.md](agent/CLAUDE.md) for complete roadmap and implementation timeline

---

## 🤝 Contributing

Contributions are welcome! Please read our contributing guidelines before submitting PRs.

### Development Workflow

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Commit Convention

We use [Conventional Commits](https://www.conventionalcommits.org/):
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `refactor:` - Code refactoring
- `test:` - Test changes
- `chore:` - Build/tooling changes

---

## 📄 License

**All Rights Reserved**

Copyright © 2025 Tom Wesselmann

This project is proprietary software. Unauthorized copying, distribution, or modification is prohibited.

For licensing inquiries, please contact: [contact information]

---

## 🙏 Acknowledgments

- Built with ❤️ using [Rust](https://www.rust-lang.org/)
- Monitoring stack inspired by [Google SRE Workbook](https://sre.google/workbook/table-of-contents/)
- Development assisted by [Claude Code](https://claude.com/claude-code) (Anthropic)

---

## 📞 Support & Contact

- **Documentation**: See [CLAUDE.md](agent/CLAUDE.md) and [DOCKER_DEPLOYMENT.md](agent/DOCKER_DEPLOYMENT.md)
- **Issues**: [GitHub Issues](https://github.com/TomWesselmann/Confidential-Assurance-Protocol/issues)
- **Discussions**: [GitHub Discussions](https://github.com/TomWesselmann/Confidential-Assurance-Protocol/discussions)

---

**Project Status:** ✅ Production-Ready (Phase 1+2 completed)
**Current Version:** v0.11.0
**Last Updated:** November 18, 2025
**MVP Target:** December 31, 2025
