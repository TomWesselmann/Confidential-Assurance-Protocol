# Confidential Assurance Protocol (CAP)

**Minimal Local Agent - Offline Compliance Proof System for Supply Chain Due Diligence**

> ⚠️ **Minimal Local Agent Version (v0.12.0)** - Diese Version fokussiert auf lokale/offline Funktionalität.
> Server-Komponenten (REST API, WebUI, Monitoring Stack) wurden entfernt.

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/TomWesselmann/Confidential-Assurance-Protocol)
[![Version](https://img.shields.io/badge/version-0.12.0--minimal-blue)](https://github.com/TomWesselmann/Confidential-Assurance-Protocol/releases)
[![License](https://img.shields.io/badge/license-All%20Rights%20Reserved-red)](LICENSE)
[![Desktop](https://img.shields.io/badge/desktop-Tauri%202.0-24C8DB?logo=tauri)](src-tauri/)

---

## 📖 Overview

The **Confidential Assurance Protocol (CAP)** is a cryptographic compliance proof system designed for the German Supply Chain Due Diligence Act (Lieferkettensorgfaltspflichtengesetz - LkSG). It enables companies to prove compliance with supply chain regulations **without revealing sensitive business data**.

### Key Features (Minimal Local Agent)

✅ **Desktop App (Tauri 2.0)** - Offline-capable application with 6-step workflow
✅ **CLI Tool** - Vollständiges Command-Line Interface für Automatisierung
✅ **Zero-Knowledge Proofs** - Prove compliance without disclosing raw data (SimplifiedZK)
✅ **Cryptographic Commitments** - BLAKE3 Merkle roots + SHA3-256 audit trails
✅ **Policy Engine** - Flexible YAML-based compliance rules (v2 with linting)
✅ **Audit Trail** - Immutable SHA3-256 hash chain for all operations (V1.0 format)
✅ **Key Management** - Ed25519 signing with key rotation and attestation
✅ **Bundle V2 Format** - Standardisiertes Proof-Package-Format
✅ **Registry** - JSON or SQLite backend für Proof-Verwaltung
✅ **BLOB Store** - Content-Addressable Storage für Dateien
✅ **Test Coverage** - 100% test pass rate with 556 tests passing

### Entfernte Features (siehe Full Version)

❌ REST API Server (cap-verifier-api)
❌ Web UI (React Frontend)
❌ Monitoring Stack (Prometheus, Grafana, Loki, Jaeger)
❌ TLS/mTLS Support
❌ Docker & Kubernetes Deployment
❌ WASM Loader

---

## 🚀 Quick Start

### Prerequisites

- **Rust** 1.70+ (für CLI und Desktop App Build)
- **Node.js** 18+ (für Desktop App Frontend)
- **Git** (zum Klonen des Repositories)

### 1. Clone Repository

```bash
git clone https://github.com/TomWesselmann/Confidential-Assurance-Protocol.git
cd Confidential-Assurance-Protocol/agent
```

### 2. Run CAP Agent

**Option A: Desktop App** (Empfohlen für Endanwender)
```bash
cd src-tauri
cargo tauri build
# macOS:
open target/release/bundle/macos/CAP\ Desktop\ Proofer.app
# Windows:
.\target\release\bundle\msi\CAP_Desktop_Proofer.msi
# Linux:
./target/release/bundle/appimage/cap-desktop-proofer.AppImage
```

**Option B: CLI** (Für Automatisierung)
```bash
cd agent
cargo build --release
./target/release/cap-agent --help
```

### 3. Verify Installation

```bash
# CLI Version check
./target/release/cap-agent --version

# Expected output:
# cap-agent 0.12.0
```

---

## 📚 Documentation

### Primary Documentation
- **[DESKTOP_APP_ARCHITEKTUR.md](docs/project/DESKTOP_APP_ARCHITEKTUR.md)** - Desktop App Architektur (Tauri 2.0)
- **[GETTING_STARTED.md](agent/docs/GETTING_STARTED.md)** - Beginner-friendly Quick Start
- **[REFACTORING_GUIDE.md](docs/project/REFACTORING_GUIDE.md)** - CLI Refactoring Guide (abgeschlossen)

### Zukünftige Features
- **[SAP_Adapter_Pilot_E2E.md](docs/project/SAP_Adapter_Pilot_E2E.md)** - SAP S/4HANA Integration (geplant)

---

## 🏗️ Architecture (Minimal Local Agent)

```
┌─────────────────────────────────────────────────────────────────┐
│              CAP Minimal Local Agent v0.12.0                    │
├─────────────────────────────────────────────────────────────────┤
│  Desktop App Layer (Tauri 2.0)                                  │
│  ├─ 6-Step Proofer Workflow                                    │
│  │   (Import → Commit → Policy → Manifest → Proof → Export)    │
│  ├─ Verifier Mode (Offline bundle verification)                │
│  ├─ Audit Mode (Timeline with SHA3-256 hash chain)            │
│  └─ IPC Commands (Tauri invoke/emit pattern)                   │
├─────────────────────────────────────────────────────────────────┤
│  CLI Layer                                                      │
│  ├─ cap-agent prepare (Import suppliers/UBOs)                  │
│  ├─ cap-agent manifest build (Build compliance manifest)       │
│  ├─ cap-agent proof build (Generate proof)                     │
│  ├─ cap-agent proof export (Export proof package)              │
│  ├─ cap-agent policy validate/lint/compile                     │
│  └─ cap-agent inspect (Bundle inspection)                      │
├─────────────────────────────────────────────────────────────────┤
│  Core Processing Layer                                         │
│  ├─ Commitment Engine (BLAKE3 Merkle Roots)                    │
│  ├─ Policy Engine (YAML-based Rules v2 with linting)          │
│  ├─ Proof Engine (SimplifiedZK - Mock Backend)                 │
│  ├─ Verifier Core (I/O-free, portable)                         │
│  └─ Audit Trail (SHA3-256 Hash Chain, V1.0 Format)             │
├─────────────────────────────────────────────────────────────────┤
│  Storage Layer (Local)                                          │
│  ├─ Registry (JSON or SQLite)                                  │
│  ├─ BLOB Store (Content-Addressable Storage)                   │
│  └─ Key Store (Ed25519 with KID rotation)                      │
└─────────────────────────────────────────────────────────────────┘
```

**Key Principles:**
- **Privacy by Design** - Zero-knowledge proofs keep data confidential
- **Offline-First** - Vollständig lokale Verarbeitung ohne Server
- **Audit-First** - Every action is logged in immutable hash chain
- **Portable** - CLI und Desktop App für alle Plattformen

---

## 💡 Usage Examples

### Desktop App: 6-Step Proofer Workflow (Empfohlen)

1. **Import** - Lieferanten- und UBO-Daten importieren (CSV/JSON)
2. **Commitments** - BLAKE3 Merkle Roots generieren
3. **Policy** - Compliance-Regeln auswählen/konfigurieren
4. **Manifest** - Compliance-Manifest erstellen
5. **Proof** - Zero-Knowledge Proof generieren
6. **Export** - Proof-Package als ZIP exportieren

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

### CLI: Verify Proof

```bash
# Verify a proof bundle
cap-agent verify --bundle build/cap-proof.zip

# Inspect bundle contents
cap-agent inspect --bundle build/cap-proof.zip
```

### CLI: Policy Management

```bash
# Validate policy YAML
cap-agent policy validate --policy examples/policy.lksg.v1.yml

# Lint policy for best practices
cap-agent policy lint --policy examples/policy.lksg.v1.yml

# Compile policy to binary format
cap-agent policy compile --policy examples/policy.lksg.v1.yml --out build/policy.bin
```

---

## 🔐 Security

### Cryptography
- **BLAKE3** - Merkle roots (commitment engine)
- **SHA3-256** - Hash chain audit log
- **Ed25519** - Digital signatures with KID-based rotation
- **Zero-Knowledge Proofs** - SimplifiedZK (Mock Backend)

### Audit & Compliance
- **Immutable Audit Trail** - SHA3-256 hash chain
- **Key Rotation** - Automated KID-based key management
- **DSGVO-compliant** - Privacy by design
- **Bundle V2 Format** - Standardisierte Proof-Packages mit Integritätsprüfung

### Security Audits
- ✅ **cargo audit** in CI/CD pipeline
- 🔄 **External audit** planned (2026)

---

## 🛠️ Development

### Build from Source

```bash
cd agent
cargo build --release
```

### Run Tests

```bash
# All tests (457 tests - 78.4% coverage)
cargo test

# Integration tests only
cargo test --test '*'

# Specific module
cargo test crypto::

# Coverage report (requires cargo-tarpaulin)
cargo tarpaulin --all-features --workspace --timeout 120 --out Html
```

**Test Results:**
- Total Tests: 556 passing ✅ (0 failures)
- Test Breakdown:
  - Library Unit Tests: 385 passing
  - Binary Unit Tests: 164 passing
  - Integration Tests: 42 test suites passing
  - Doc Tests: 7 passing
- Test Coverage: Bundle v2, Dual-Anchor, Hash Validation, Registry, SQLite, Policy Store
- Performance: All benchmarks passing

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

## 📦 Distribution

### Desktop App Build

```bash
cd src-tauri
cargo tauri build

# Outputs:
# macOS: target/release/bundle/macos/CAP Desktop Proofer.app
# Windows: target/release/bundle/msi/CAP_Desktop_Proofer.msi
# Linux: target/release/bundle/appimage/cap-desktop-proofer.AppImage
```

### CLI Binary

```bash
cd agent
cargo build --release

# Binary: target/release/cap-agent
# Copy to /usr/local/bin for system-wide access
```

---

## 🗺️ Roadmap

### ✅ Completed (Minimal Local Agent v0.12.0)
- CLI & Core Features (Commitment, Policy, Proof, Verifier)
- Desktop App (Tauri 2.0) mit 6-Step Workflow
- Key Management with KID Rotation
- Registry (JSON + SQLite)
- BLOB Store (Content-Addressable Storage)
- Bundle V2 Format mit Integritätsprüfung
- Policy Engine v2 mit Linting
- Audit Trail (SHA3-256 Hash Chain)
- SimplifiedZK Proof Backend
- All Tests Passing (556 tests)

### ❌ Entfernt in Minimal Version
- REST API Server (cap-verifier-api)
- Web UI (React Frontend)
- Monitoring Stack (Prometheus, Grafana, Loki, Jaeger)
- TLS/mTLS Support
- Docker & Kubernetes Deployment
- WASM Loader

### 📅 Geplant (Future Full Version)
- REST API Server wieder hinzufügen
- Web UI wieder hinzufügen
- Halo2-based Zero-Knowledge Proofs
- SAP Integration (OData v4)
- HSM Integration (PKCS#11)

**Full Details:** See [ROADMAP_MVP_PRODUCTION.md](docs/project/ROADMAP_MVP_PRODUCTION.md)

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
- Desktop App powered by [Tauri 2.0](https://tauri.app/)
- Development assisted by [Claude Code](https://claude.com/claude-code) (Anthropic)

---

## 📞 Support & Contact

- **Documentation**: See [DESKTOP_APP_ARCHITEKTUR.md](docs/project/DESKTOP_APP_ARCHITEKTUR.md)
- **Issues**: [GitHub Issues](https://github.com/TomWesselmann/Confidential-Assurance-Protocol/issues)
- **Discussions**: [GitHub Discussions](https://github.com/TomWesselmann/Confidential-Assurance-Protocol/discussions)

---

**Project Status:** Minimal Local Agent (Offline-First)
**Current Version:** v0.12.0-minimal
**Last Updated:** December 11, 2025

**Key Metrics:**
- Tests: 556/556 passing (0 failures)
- Interfaces: Desktop App (Tauri 2.0) + CLI
- Proof Backend: SimplifiedZK (Mock)
- Security Features: Path traversal prevention, cycle detection, TOCTOU mitigation
