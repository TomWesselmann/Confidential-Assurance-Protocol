# 🔐 LkSG Proof Agent

**Offline Compliance Proof System für das deutsche Lieferkettensorgfaltspflichtengesetz (LkSG)**

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Tests](https://img.shields.io/badge/tests-53%2F53-brightgreen.svg)](./agent/src/)
[![Version](https://img.shields.io/badge/version-0.6.0-blue.svg)](./docs/SYSTEMARCHITEKTUR_v0.6.0.md)

---

## 📋 Überblick

Der **LkSG Proof Agent** ist ein Rust-basiertes CLI-Tool zur Erzeugung kryptographisch verifizierbarer Nachweise für:

- ✅ **Lieferketten-Compliance** (Supplier & UBO Commitments)
- ✅ **Sanktionsprüfungen** (EU Sanctions Lists mit Non-Membership Checks)
- ✅ **Jurisdiktions-Risikobewertung** (High-Risk Countries)
- ✅ **Zero-Knowledge Proofs** (Privacy-preserving Compliance Nachweise)
- ✅ **Proof Registry** (Lokale Verwaltung mit Timestamping)

**Kernprinzipien:**
- 🔒 **100% Offline** - Keine Netzwerkverbindungen
- 🔐 **Kryptographisch sicher** - BLAKE3, SHA3-256, Ed25519
- 📝 **Auditierbar** - SHA3-verkettete Hash-Chain für alle Operationen
- 🧩 **Modular** - Erweiterbar für echte ZK-Systeme (Halo2, Spartan, RISC0)

---

## 🚀 Schnellstart

### Installation

```bash
cd agent
cargo build --release
```

### Basis-Workflow

```bash
# 1. Commitments aus CSV-Daten generieren
cargo run -- prepare \
  --suppliers examples/suppliers.csv \
  --ubos examples/ubos.csv

# 2. Policy validieren
cargo run -- policy validate \
  --file examples/policy.lksg.v1.yml

# 3. Manifest erstellen
cargo run -- manifest build \
  --policy examples/policy.lksg.v1.yml

# 4. Zero-Knowledge Proof generieren
cargo run -- proof zk-build \
  --policy examples/policy.lksg.v1.yml \
  --manifest build/manifest.json \
  --sanctions-csv lists/eu_sanctions.csv

# 5. Proof verifizieren
cargo run -- proof zk-verify \
  --proof build/zk_proof.dat
```

---

## 🏗️ Architektur

```
┌────────────────────────────────────────────────────────┐
│                  LkSG Proof Agent v0.6.0               │
│     Offline Compliance Proof System (P0+P1+P2)         │
└────────────────────────────────────────────────────────┘

Input Layer (CSV, YAML, Keys, Lists)
    ↓
Commitment Engine (BLAKE3 Merkle Roots)
    ↓
Policy Layer (Validation, Manifest, Signing)
    ↓
Lists Layer (Sanctions, Jurisdictions)
    ↓
Zero-Knowledge Layer (SimplifiedZK → Halo2 ready)
    ↓
Registry & Timestamp Layer (Proof Management)
    ↓
Verifier (Offline Package Verification)
```

**Detaillierte Architektur:** [docs/SYSTEMARCHITEKTUR_v0.6.0.md](./docs/SYSTEMARCHITEKTUR_v0.6.0.md)

---

## 🛠️ CLI-Kommandos

### Core Commands

| Command | Beschreibung |
|---------|--------------|
| `prepare` | Generiert BLAKE3 Merkle Roots aus CSV-Daten |
| `inspect` | Zeigt Commitments-Datei an |
| `version` | Zeigt Version an |

### Policy & Manifest

| Command | Beschreibung |
|---------|--------------|
| `policy validate` | Validiert Policy-Datei (YAML/JSON) |
| `manifest build` | Erstellt Compliance-Manifest |

### Zero-Knowledge Proofs

| Command | Beschreibung |
|---------|--------------|
| `proof zk-build` | Erstellt ZK-Proof (mit optionalen Sanctions-Checks) |
| `proof zk-verify` | Verifiziert ZK-Proof offline |
| `proof bench` | Performance-Benchmarks |
| `proof export` | Exportiert Proof-Paket für Auditoren |

### Sanctions & Jurisdictions (P1)

| Command | Beschreibung |
|---------|--------------|
| `lists sanctions-root` | Generiert BLAKE3 Root aus Sanctions CSV |
| `lists jurisdictions-root` | Generiert BLAKE3 Root aus Jurisdictions CSV |

### Audit & Timestamping (P0+P2)

| Command | Beschreibung |
|---------|--------------|
| `audit tip` | Schreibt Audit-Chain-Head in Datei |
| `audit anchor` | Setzt Zeitanker im Manifest |
| `audit timestamp` | Erstellt RFC3161-Mock-Timestamp |
| `audit verify-timestamp` | Verifiziert Timestamp gegen Audit-Head |

### Registry (P2)

| Command | Beschreibung |
|---------|--------------|
| `registry add` | Fügt Proof zur lokalen Registry hinzu |
| `registry list` | Listet alle Registry-Einträge auf |
| `registry verify` | Verifiziert Proof gegen Registry |

### Signing & Verification

| Command | Beschreibung |
|---------|--------------|
| `sign keygen` | Generiert Ed25519-Keypair |
| `sign manifest` | Signiert Manifest |
| `sign verify` | Verifiziert Signatur |
| `verifier run` | Verifiziert komplettes Proof-Paket |
| `verifier extract` | Extrahiert Manifest aus Paket |
| `verifier audit` | Zeigt Audit-Trail an |

**Gesamt:** 27 CLI-Commands

---

## 📦 Module

| Modul | Zeilen | Funktion |
|-------|--------|----------|
| `io.rs` | 90 | CSV-Parsing (Suppliers, UBOs) |
| `commitment.rs` | 157 | BLAKE3 Merkle Roots + Counts |
| `audit.rs` | 145 | SHA3-256 Hash-Chain + Tip Management |
| `policy.rs` | 210 | Policy Validation + Extensions |
| `manifest.rs` | 230 | Manifest Builder + TimeAnchor |
| `sign.rs` | 140 | Ed25519 Signing & Verification |
| `proof_engine.rs` | 392 | Structured Proof Building |
| `verifier.rs` | 283 | Offline Package Verification |
| `zk_system.rs` | 420 | Zero-Knowledge Proof System |
| `lists/sanctions.rs` | 285 | Sanctions List Processing |
| `lists/jurisdictions.rs` | 275 | Jurisdictions List Processing |
| `registry.rs` | 335 | Registry & Timestamp Management |
| `main.rs` | 1467 | CLI Interface |

**Gesamt:** ~4400 Zeilen Rust Code

---

## 🧪 Tests

```bash
# Alle Tests ausführen
cargo test

# Mit Clippy prüfen
cargo clippy -- -D warnings
```

**Test-Coverage:**
- ✅ **53/53 Unit-Tests** (alle grün)
- ✅ **0 Clippy-Warnings**
- ✅ **< 0.01s Test-Zeit**

**Test-Module:**
- `io::tests` (2)
- `commitment::tests` (3)
- `audit::tests` (4)
- `policy::tests` (6)
- `manifest::tests` (3)
- `proof_engine::tests` (3)
- `verifier::tests` (3)
- `sign::tests` (3)
- `zk_system::tests` (6)
- `lists/sanctions::tests` (3)
- `lists/jurisdictions::tests` (3)
- `registry::tests` (9)

---

## 🔐 Kryptographische Primitiven

| Funktion | Algorithmus | Verwendung |
|----------|-------------|------------|
| Merkle Roots | **BLAKE3** | Commitments (Suppliers, UBOs, Sanctions, Jurisdictions) |
| Audit Chain | **SHA3-256** | Append-only Event-Log (Hash-verkettung) |
| Policy Hash | **SHA3-256** | Policy-Identifikation |
| Manifest Hash | **SHA3-256** | Proof-Verifikation |
| File Hashing | **SHA3-256** | Registry Integrity-Checks |
| Signatur | **Ed25519** | Manifest-Signierung |
| Encoding | **Base64** | Proof.dat Serialisierung |
| Timestamp Sig | **SHA3-256** | Mock-Timestamp-Verifikation |

---

## 📁 Projektstruktur

```
/TestClaude/
├── README.md                           # Diese Datei
├── MD/                                 # PRDs
│   ├── PRD_P0_QuickWins.md
│   ├── PRD_P1_Sanctions_Jurisdictions.md
│   └── PRD_P2_Timestamp_Registry.md
├── docs/
│   └── SYSTEMARCHITEKTUR_v0.6.0.md    # Vollständige Architekturdoku
├── agent/                              # Rust-Projekt
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs                    # CLI-Interface
│   │   ├── audit.rs                   # SHA3 Hash-Chain
│   │   ├── commitment.rs              # BLAKE3 Merkle Roots
│   │   ├── io.rs                      # CSV-Parsing
│   │   ├── manifest.rs                # Manifest Builder
│   │   ├── policy.rs                  # Policy Validation
│   │   ├── sign.rs                    # Ed25519 Signing
│   │   ├── proof_engine.rs            # Proof Builder
│   │   ├── proof_mock.rs              # Mock Proof (Legacy)
│   │   ├── verifier.rs                # Offline Verifier
│   │   ├── zk_system.rs               # Zero-Knowledge System
│   │   ├── registry.rs                # Registry & Timestamp
│   │   └── lists/
│   │       ├── mod.rs
│   │       ├── sanctions.rs           # Sanctions Processing
│   │       └── jurisdictions.rs       # Jurisdictions Processing
│   ├── examples/
│   │   ├── suppliers.csv
│   │   ├── ubos.csv
│   │   └── policy.lksg.v1.yml
│   ├── lists/
│   │   ├── eu_sanctions.csv           # Sanctions-Beispieldaten
│   │   └── highrisk.csv               # Jurisdictions-Beispieldaten
│   └── build/                         # Output-Verzeichnis
│       ├── commitments.json
│       ├── manifest.json
│       ├── zk_proof.dat
│       ├── zk_proof.json
│       ├── audit.head
│       ├── timestamp.tsr
│       ├── registry.json
│       └── agent.audit.jsonl          # SHA3-verketteter Audit-Trail
└── .git/
```

---

## 📊 Version History

### v0.6.0 (2025-10-29) - **Current**
- ✅ **P2: Timestamp & Registry**
  - Registry-Modul (JSON-basierte Proof-Verwaltung)
  - RFC3161-Mock-Timestamps mit Audit-Tip-Verankerung
  - CLI: `registry add/list/verify`, `audit timestamp/verify-timestamp`
  - 9 neue Tests (53 gesamt)

### v0.5.0 (2025-10-29)
- ✅ **P1: Sanctions & Jurisdictions**
  - Lists-Modul mit BLAKE3 Merkle Roots
  - ZK-Integration: `sanctions_non_membership` Constraint
  - CLI: `lists sanctions-root/jurisdictions-root`, `proof zk-build --sanctions-csv`
  - 9 neue Tests (44 gesamt)

### v0.4.1 (2025-10-29)
- ✅ **P0: Quick Wins**
  - Audit Tip Management (`audit tip`, `audit anchor`)
  - TimeAnchor im Manifest
  - Policy-Extensions (`ubo_count_min`, `require_statement_roots`)
  - Optional Roots in ZK-Statement (`sanctions_root`, `jurisdiction_root`)
  - 8 neue Tests (36 gesamt)

### v0.4.0 (Baseline)
- ✅ Tag 1-4: Core System (Commitment Engine, Policy Layer, Proof Engine, ZK System)

---

## 🎯 Roadmap

### v0.7.0 (Geplant)
- [ ] Echte TSA-Integration (RFC3161 mit DigiCert/Let's Encrypt)
- [ ] Blockchain-Anchoring (Ethereum/Solana)
- [ ] SQLite-Registry (Performance-Upgrade)

### v0.8.0 (Geplant)
- [ ] Halo2-Backend (Echtes Zero-Knowledge)
- [ ] Recursive Proofs (Nova/Proof-of-Proofs)
- [ ] Web-Verifier (WASM für Browser-Verifikation)

---

## 🔧 Development

### Build

```bash
cd agent
cargo build --release
```

### Run Tests

```bash
cargo test
```

### Lint

```bash
cargo clippy -- -D warnings
```

### Benchmarks

```bash
cargo run --release -- proof bench \
  --policy examples/policy.lksg.v1.yml \
  --manifest build/manifest.json \
  --iterations 1000
```

---

## 📚 Dokumentation

- **Systemarchitektur:** [docs/SYSTEMARCHITEKTUR_v0.6.0.md](./docs/SYSTEMARCHITEKTUR_v0.6.0.md)
- **PRDs:**
  - [MD/PRD_P0_QuickWins.md](./MD/PRD_P0_QuickWins.md)
  - [MD/PRD_P1_Sanctions_Jurisdictions.md](./MD/PRD_P1_Sanctions_Jurisdictions.md)
  - [MD/PRD_P2_Timestamp_Registry.md](./MD/PRD_P2_Timestamp_Registry.md)

---

## 🤝 Contributing

Dieses Projekt ist Teil des **Confidential Assurance Protocol (CAP)** und dient als Proof-of-Concept für privacy-preserving Compliance-Nachweise.

### Entwicklungsprinzipien

1. **Offline-First** - Keine Netzwerkverbindungen
2. **Kryptographisch sicher** - Nur etablierte Algorithmen (BLAKE3, SHA3, Ed25519)
3. **Auditierbar** - Alle Operationen protokolliert
4. **Deterministisch** - Gleiche Inputs → Gleiche Outputs
5. **Modular** - Erweiterbar für echte ZK-Backends

---

## 📄 Lizenz

© 2025 Confidential Assurance Protocol – Core Engineering
**Alle Rechte vorbehalten.**

---

## 🙏 Acknowledgments

Entwickelt mit:
- **Rust** (Edition 2021)
- **BLAKE3** (Merkle Trees)
- **SHA3** (Audit Chain)
- **Ed25519-dalek** (Digital Signatures)
- **Clap** (CLI Framework)

---

**Status:** Production-Ready for Architecture Demo & Extension
**Version:** 0.6.0
**Build:** Alle Tests grün ✅ | Clippy clean ✅
