# Confidential Assurance Protocol (CAP)

**Minimal Local Agent - Offline Compliance Proof System for Supply Chain Due Diligence**

> ⚠️ **Minimal Local Agent Version (v0.12.0)** - Diese Version fokussiert auf lokale/offline Funktionalität.
> Server-Komponenten (REST API, WebUI, Monitoring Stack) wurden entfernt.

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/TomWesselmann/Confidential-Assurance-Protocol)
[![Version](https://img.shields.io/badge/version-0.12.0--minimal-blue)](https://github.com/TomWesselmann/Confidential-Assurance-Protocol/releases)
[![License](https://img.shields.io/badge/license-All%20Rights%20Reserved-red)](LICENSE)
[![Desktop](https://img.shields.io/badge/desktop-Tauri%202.0-24C8DB?logo=tauri)](src-tauri/)

---

## Overview

The **Confidential Assurance Protocol (CAP)** is a cryptographic compliance proof system designed for the German Supply Chain Due Diligence Act (Lieferkettensorgfaltspflichtengesetz - LkSG). It enables companies to prove compliance with supply chain regulations **without revealing sensitive business data**.

### Key Features (Minimal Local Agent)

- **Desktop App (Tauri 2.0)** - Offline-capable application with 6-step workflow
- **CLI Tool** - Vollständiges Command-Line Interface für Automatisierung
- **Zero-Knowledge Proofs** - Prove compliance without disclosing raw data (SimplifiedZK)
- **Cryptographic Commitments** - BLAKE3 Merkle roots + SHA3-256 audit trails
- **Policy Engine** - Flexible YAML-based compliance rules (v2 with linting)
- **Audit Trail** - Immutable SHA3-256 hash chain for all operations (V1.0 format)
- **Key Management** - Ed25519 signing with key rotation and attestation
- **Bundle V2 Format** - Standardisiertes Proof-Package-Format
- **Registry** - JSON or SQLite backend für Proof-Verwaltung
- **BLOB Store** - Content-Addressable Storage für Dateien
- **Test Coverage** - 100% test pass rate with 556 tests passing

---

## Quick Start

### Prerequisites

- **Rust** 1.70+ (für CLI und Desktop App Build)
- **Node.js** 18+ (für Desktop App Frontend)
- **Git** (zum Klonen des Repositories)

### 1. Clone Repository

```bash
git clone https://github.com/TomWesselmann/Confidential-Assurance-Protocol.git
cd Confidential-Assurance-Protocol
```

### 2. Build & Run

**Option A: Desktop App** (Empfohlen für Endanwender)
```bash
cd src-tauri
cargo tauri build
```

**Option B: CLI** (Für Automatisierung)
```bash
cd agent
cargo build --release
./target/release/cap-agent --help
```

---

## Project Structure

```
LsKG-Agent/
├── agent/              # Rust Core Library + CLI
├── src-tauri/          # Tauri Desktop App Backend
├── tauri-frontend/     # React Frontend (TypeScript)
├── sap-adapter/        # SAP S/4HANA Integration (geplant)
├── infrastructure/     # Docker, K8s, Monitoring
└── docs/               # Projektdokumentation
    └── project/        # Architektur, Roadmaps, Guides
```

---

## Documentation

- **[Desktop App Architektur](docs/project/DESKTOP_APP_ARCHITEKTUR.md)** - Tauri 2.0 Architektur
- **[Getting Started](agent/docs/GETTING_STARTED.md)** - Beginner-friendly Quick Start
- **[Project Overview](docs/project/README.md)** - Detaillierte Projektdokumentation
- **[Roadmap](docs/project/ROADMAP_MVP_PRODUCTION.md)** - Zentrale Projekt-Roadmap

---

## Development

```bash
# Build all crates
cargo build --workspace

# Run all tests
cargo test --workspace

# Code quality
cargo clippy --workspace -- -D warnings
cargo fmt --check

# Security audit
cargo audit
```

---

## License

**All Rights Reserved**

Copyright © 2025 Tom Wesselmann

This project is proprietary software. See [LICENSE](LICENSE) for details.

---

**Project Status:** Minimal Local Agent (Offline-First)
**Current Version:** v0.12.0-minimal
**Last Updated:** December 2025
