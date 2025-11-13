# CAP Proof System - Systemarchitektur v0.10

**Stand:** 2025-11-04
**Version:** 0.10.0 (Key Management Integration)
**Status:** Development - Production Ready Core Features

---

## Inhaltsverzeichnis

1. [Übersicht](#übersicht)
2. [Architektur-Diagramm](#architektur-diagramm)
3. [Module & Komponenten](#module--komponenten)
4. [Datenmodell](#datenmodell)
5. [CLI-Befehle](#cli-befehle)
6. [Persistenz & Storage](#persistenz--storage)
7. [Kryptographie](#kryptographie)
8. [Test-Coverage](#test-coverage)
9. [Performance](#performance)
10. [Implementierungsstatus](#implementierungsstatus)
11. [Nächste Schritte](#nächste-schritte)

---

## Übersicht

Das **CAP Proof System** (Confidential Assurance Protocol) ist ein Rust-basiertes CLI-Tool für die Erzeugung, Verwaltung und Verifikation von kryptographischen Nachweisen im Kontext des deutschen Lieferkettensorgfaltspflichtengesetzes (LkSG).

### Design-Prinzipien

- **Offline-First**: Keine Netzwerkabhängigkeiten für Core-Funktionen
- **Deterministisch**: Gleiche Inputs → Gleiche Outputs
- **Portable**: I/O-freie Verifikationskerne für WASM/zkVM-Integration
- **Backward-Compatible**: Alle neuen Features sind optional
- **Audit-Ready**: Vollständige Hash-Chain für alle Operationen

### Technologie-Stack

| Komponente | Technologie | Version |
|------------|-------------|---------|
| Sprache | Rust | Edition 2021 |
| CLI Framework | clap | v4.5 |
| Hashing | blake3 + sha3 | v1.5 + v0.10 |
| Signaturen | ed25519-dalek | v2.1 |
| Datenbank | rusqlite (SQLite) | v0.31 (bundled) |
| WASM Runtime | wasmtime | v27.0 |
| Serialisierung | serde + serde_json | Latest |
| Schema-Validierung | jsonschema | v0.17 (Draft 2020-12) |

---

## Architektur-Diagramm

```
┌─────────────────────────────────────────────────────────────────┐
│                    CAP Proof System v0.10                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │                    Input Layer                          │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────┐ │  │
│  │  │ CSV Data │  │ Policy   │  │ Keys     │  │ Proofs │ │  │
│  │  │ (S + U)  │  │ (YAML)   │  │(Ed25519) │  │ (WASM) │ │  │
│  │  └────┬─────┘  └────┬─────┘  └────┬─────┘  └───┬────┘ │  │
│  └───────┼─────────────┼─────────────┼────────────┼──────┘  │
│          │             │             │            │          │
│  ┌───────▼─────────────▼─────────────▼────────────▼──────┐  │
│  │              Commitment Engine (Tag 1)                │  │
│  │  - BLAKE3 Merkle Roots (Supplier, UBO, Company)     │  │
│  │  - SHA3-256 Hash Chain Audit Log                     │  │
│  │  - Deterministic Commitment Generation               │  │
│  └───────┬───────────────────────────────────────────────┘  │
│          │                                                    │
│  ┌───────▼───────────────────────────────────────────────┐  │
│  │              Policy Layer (Tag 2)                     │  │
│  │  - Policy Validation (YAML/JSON)                      │  │
│  │  - Manifest Builder (manifest.v1.0)                   │  │
│  │  - Ed25519 Signing                                    │  │
│  │  - Dual-Anchor Timestamp (Private + Public)          │  │
│  └───────┬───────────────────────────────────────────────┘  │
│          │                                                    │
│  ┌───────▼───────────────────────────────────────────────┐  │
│  │              Proof Engine (Tag 3)                     │  │
│  │  - ZK Backend Abstraction (Mock/zkVM/Halo2)          │  │
│  │  - Constraint Verification                            │  │
│  │  - CAPZ Container Format (v2)                         │  │
│  │  - Proof Serialization (Base64)                       │  │
│  └───────┬───────────────────────────────────────────────┘  │
│          │                                                    │
│  ┌───────▼───────────────────────────────────────────────┐  │
│  │           BLOB Store (v0.9)                           │  │
│  │  - Content-Addressable Storage (BLAKE3)               │  │
│  │  - SQLite Backend with Deduplication                  │  │
│  │  - Reference Counting & Garbage Collection            │  │
│  │  - Media Type Support (manifest/proof/wasm/abi)       │  │
│  └───────┬───────────────────────────────────────────────┘  │
│          │                                                    │
│  ┌───────▼───────────────────────────────────────────────┐  │
│  │           Key Management (v0.10)                      │  │
│  │  - KID Derivation (blake3(pubkey)[0:16])              │  │
│  │  - Key Metadata Schema (cap-key.v1)                   │  │
│  │  - Key Store (keys/, archive/, trusted/)              │  │
│  │  - Rotation Chain of Trust                            │  │
│  └───────┬───────────────────────────────────────────────┘  │
│          │                                                    │
│  ┌───────▼───────────────────────────────────────────────┐  │
│  │           Registry (Pluggable Backend)                │  │
│  │  - JSON Store (Backward Compatible)                   │  │
│  │  - SQLite Store (WAL Mode, Concurrent)                │  │
│  │  - Entry Signing with KID                             │  │
│  │  - Self-Verification Status Tracking                  │  │
│  │  - BLOB Reference Management                          │  │
│  └───────┬───────────────────────────────────────────────┘  │
│          │                                                    │
│  ┌───────▼───────────────────────────────────────────────┐  │
│  │           Verifier (Multi-Layer)                      │  │
│  │  - Verifier Core (I/O-free, portable)                 │  │
│  │  - Package Verifier (File-based)                      │  │
│  │  - WASM Sandbox Verifier (Bundle v2)                  │  │
│  │  - Legacy Key Verification                            │  │
│  └───────┬───────────────────────────────────────────────┘  │
│          │                                                    │
│  ┌───────▼───────────────────────────────────────────────┐  │
│  │           Output Layer                                │  │
│  │  - CAP Proof Packages (v1/v2)                         │  │
│  │  - Verification Reports (JSON)                        │  │
│  │  - Audit Logs (JSONL)                                 │  │
│  │  - Performance Metrics                                │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Module & Komponenten

### Core Library (`src/lib.rs`)

Exportiert folgende Module für Testing, Benchmarking und externe Nutzung:

```rust
pub mod blob_store;   // v0.9 - Content-Addressable Storage
pub mod crypto;       // v0.9 - Unified Crypto API
pub mod keys;         // v0.10 - Key Management & KID System
pub mod proof;        // v0.3 - Proof Generation & CAPZ Format
pub mod registry;     // v0.8 - Pluggable Registry Backend
pub mod verifier;     // v0.9 - Portable Verification Core
pub mod wasm;         // v0.9 - WASM Sandbox & Loader
```

### Binary Modules (`src/main.rs`)

```rust
mod audit;              // Hash-Chain Audit Log
mod commitment;         // Merkle Root Computation
mod io;                 // CSV Import
mod keys;               // Key Management (CLI + Core)
mod lists;              // Sanctions & Jurisdictions
mod manifest;           // Manifest Builder
mod package_verifier;   // File-based Package Verification
mod policy;             // Policy Validation
mod proof_engine;       // ZK-Ready Proof Engine
mod proof_mock;         // Mock Proof (Legacy)
mod registry;           // Registry Store Implementations
mod sign;               // Ed25519 Signing
mod zk_system;          // ZK Backend Abstraction
```

---

## Datenmodell

### Registry Entry Schema (v0.10)

```rust
pub struct RegistryEntry {
    // Core Fields
    pub id: String,
    pub manifest_hash: String,      // SHA3-256
    pub proof_hash: String,          // SHA3-256
    pub timestamp_file: Option<String>,
    pub registered_at: String,       // RFC3339

    // Signature Fields (v0.8)
    pub signature: Option<String>,   // Ed25519 (base64)
    pub public_key: Option<String>,  // Ed25519 (base64)

    // BLOB Store Fields (v0.9)
    pub blob_manifest: Option<String>,   // BLAKE3 hash
    pub blob_proof: Option<String>,      // BLAKE3 hash
    pub blob_wasm: Option<String>,       // BLAKE3 hash
    pub blob_abi: Option<String>,        // SHA3-256 hash

    // Self-Verification Fields (v0.9)
    pub selfverify_status: Option<String>,    // "ok", "fail", "unknown"
    pub selfverify_at: Option<String>,        // RFC3339
    pub verifier_name: Option<String>,
    pub verifier_version: Option<String>,

    // Key Management Fields (v0.10)
    pub kid: Option<String>,                  // 32 hex chars (16 bytes)
    pub signature_scheme: Option<String>,     // "ed25519"
}
```

### Key Metadata Schema (cap-key.v1)

```rust
pub struct KeyMetadata {
    pub schema: String,              // "cap-key.v1"
    pub kid: String,                 // Derived from public key
    pub owner: String,               // Organization name
    pub created_at: String,          // RFC3339
    pub valid_from: String,          // RFC3339
    pub valid_to: String,            // RFC3339
    pub algorithm: String,           // "ed25519"
    pub status: String,              // "active", "retired", "revoked"
    pub usage: Vec<String>,          // ["signing", "registry", ...]
    pub public_key: String,          // Base64-encoded
    pub fingerprint: String,         // SHA-256 fingerprint
    pub comment: Option<String>,
}
```

### BLOB Metadata Schema

```rust
pub struct BlobMetadata {
    pub blob_id: String,    // BLAKE3 hash (0x-prefixed, 64 hex)
    pub size: usize,
    pub media_type: String, // MIME type
    pub refcount: i64,      // Reference count for GC
}
```

### Manifest Schema (manifest.v1.0)

```json
{
  "version": "manifest.v1.0",
  "created_at": "2025-10-30T12:00:00Z",
  "supplier_root": "0x...",
  "ubo_root": "0x...",
  "company_commitment_root": "0x...",
  "policy": {
    "name": "LkSG Demo Policy",
    "version": "lksg.v1",
    "hash": "0x..."
  },
  "audit": {
    "tail_digest": "0x...",
    "events_count": 42
  },
  "time_anchor": {
    "kind": "tsa",
    "reference": "./tsa/test.tsr",
    "audit_tip_hex": "0x...",
    "private": {
      "audit_tip_hex": "0x...",
      "created_at": "2025-10-30T12:05:00Z"
    },
    "public": {
      "chain": "ethereum",
      "txid": "0xabc123...",
      "digest": "0x...",
      "created_at": "2025-10-30T12:10:00Z"
    }
  },
  "signatures": []
}
```

---

## CLI-Befehle

### Commitment Engine (Tag 1)

```bash
# Commitment-Berechnung
cap-agent prepare --suppliers examples/suppliers.csv --ubos examples/ubos.csv

# Commitment-Anzeige
cap-agent inspect --file build/commitments.json
```

### Policy Layer (Tag 2)

```bash
# Policy-Validierung
cap-agent policy validate --file examples/policy.lksg.v1.yml

# Manifest-Erstellung
cap-agent manifest build --policy examples/policy.lksg.v1.yml

# Manifest-Schema-Validierung
cap-agent manifest validate --file build/manifest.json

# Manifest-Verifikation (Offline)
cap-agent manifest verify \
  --manifest build/manifest.json \
  --proof build/proof.dat \
  --registry build/registry.json \
  [--timestamp build/timestamp.tsr] \
  [--out build/verification.report.json]
```

### Signing (Tag 2)

```bash
# Schlüsselerzeugung
cap-agent sign keygen --dir keys

# Manifest-Signierung
cap-agent sign manifest \
  --manifest-in build/manifest.json \
  --key keys/company.ed25519 \
  --out build/signature.json

# Signatur-Verifikation
cap-agent sign verify \
  --signature build/signature.json \
  --key keys/company.pub
```

### Proof Engine (Tag 3)

```bash
# Proof-Erstellung
cap-agent proof build \
  --manifest build/manifest.json \
  --policy examples/policy.lksg.v1.yml

# Proof-Verifikation
cap-agent proof verify \
  --proof build/proof.dat \
  --manifest build/manifest.json

# Standardisiertes CAP Proof-Paket-Export (v1.0)
cap-agent proof export \
  --manifest build/manifest.json \
  --proof build/zk_proof.dat \
  [--timestamp build/timestamp.tsr] \
  [--registry build/registry.json] \
  [--out build/cap-proof] \
  [--force]
```

### Package Verifier (Tag 3)

```bash
# Proof-Paket-Verifikation
cap-agent verifier run --package build/proof_package

# Manifest-Extraktion
cap-agent verifier extract --package build/proof_package

# Audit-Trail-Anzeige
cap-agent verifier audit --package build/proof_package
```

### Registry (v0.8)

```bash
# Proof zur Registry hinzufügen
cap-agent registry add \
  --manifest build/manifest.json \
  --proof build/proof.dat \
  [--timestamp build/timestamp.tsr] \
  [--signing-key keys/company.ed25519] \
  [--backend sqlite] \
  [--registry build/registry.sqlite]

# Registry-Einträge auflisten
cap-agent registry list \
  [--backend sqlite] \
  [--registry build/registry.sqlite]

# Proof gegen Registry verifizieren
cap-agent registry verify \
  --manifest build/manifest.json \
  --proof build/proof.dat \
  [--backend sqlite] \
  [--registry build/registry.sqlite]

# Registry-Migration
cap-agent registry migrate \
  --from json --input build/registry.json \
  --to sqlite --output build/registry.sqlite
```

### Dual-Anchor System (v0.9)

```bash
# Private Anchor setzen
cap-agent audit set-private-anchor \
  --manifest build/manifest.json \
  --audit-tip 0x83a8779d... \
  [--created-at "2025-10-30T10:00:00Z"]

# Public Anchor setzen
cap-agent audit set-public-anchor \
  --manifest build/manifest.json \
  --chain ethereum \
  --txid 0xabc123... \
  --digest 0x1234567890... \
  [--created-at "2025-10-30T10:00:00Z"]

# Dual-Anchor-Konsistenz verifizieren
cap-agent audit verify-anchor \
  --manifest build/manifest.json \
  [--out build/anchor_verification.json]
```

### Bundle v2 (WASM-Verifier + Loader)

```bash
# Bundle erstellen
cap-agent bundle-v2 \
  --manifest build/manifest.json \
  --proof build/proof.capz \
  [--verifier-wasm build/verifier.wasm] \
  --out build/cap-proof-v2 \
  [--zip] \
  [--force]

# Bundle verifizieren
cap-agent verify-bundle \
  --bundle build/cap-proof-v2 \
  [--out build/verification.report.json]
```

### Key Management (v0.10) - **IN DEVELOPMENT**

```bash
# Schlüsselerzeugung mit Metadata
cap-agent keygen \
  --owner company \
  --algo ed25519 \
  --out keys/company.v1.json

# Schlüsselliste
cap-agent keys list

# Schlüssel-Details
cap-agent keys show --kid b3f42c9d7e6a45a1

# Schlüssel-Rotation
cap-agent keys rotate \
  --current company.v1.json \
  --new company.v2.json

# Schlüssel-Attestierung
cap-agent keys attest \
  --signer company.v1.json \
  --subject company.v2.json

# Schlüssel-Archivierung
cap-agent keys archive --kid b3f42c9d7e6a45a1
```

### Batch Operations (v0.11) - **NOT IMPLEMENTED**

```bash
# Batch-Add
cap-agent registry batch-add \
  --manifest-dir ./proofs/manifests \
  --proof-dir ./proofs/data \
  --threads 8 \
  --limit 10000

# Batch-Verify
cap-agent registry batch-verify \
  --registry ./build/registry.sqlite \
  --threads 8 \
  --limit 10000

# Benchmark
cap-agent bench run --suite scale10k
cap-agent bench report --input bench/results/scale10k.json
```

---

## Persistenz & Storage

### File Structure

```
project/
├── build/
│   ├── agent.audit.jsonl           # Hash-Chain Audit Log
│   ├── commitments.json            # Merkle Roots
│   ├── manifest.json               # Compliance Manifest
│   ├── proof.dat                   # Base64-encoded Proof
│   ├── proof.json                  # Human-readable Proof
│   ├── signature.json              # Ed25519 Signature
│   ├── registry.json               # JSON Registry
│   ├── registry.sqlite             # SQLite Registry
│   ├── registry.sqlite-wal         # WAL file
│   ├── blobs.sqlite                # BLOB Store
│   └── cap-proof/                  # Exported Package
│       ├── manifest.json
│       ├── proof.dat
│       ├── timestamp.tsr
│       ├── registry.json
│       ├── verification.report.json
│       ├── README.txt
│       └── _meta.json
│
├── keys/
│   ├── company.ed25519             # Private Key
│   ├── company.pub                 # Public Key
│   ├── company.v1.json             # Key Metadata
│   ├── archive/                    # Archived Keys
│   │   ├── company.v0.json
│   │   └── auditor.v0.json
│   └── trusted/                    # Trusted Third-Party Keys
│       ├── auditor.pub
│       └── tsa.pub
│
├── examples/
│   ├── suppliers.csv
│   ├── ubos.csv
│   └── policy.lksg.v1.yml
│
└── docs/
    ├── manifest.schema.json        # JSON Schema Draft 2020-12
    └── bundle_v2_spec.md
```

### SQLite Schemas

**Registry (registry_entries)**
```sql
CREATE TABLE IF NOT EXISTS registry_entries (
    id TEXT PRIMARY KEY,
    manifest_hash TEXT NOT NULL,
    proof_hash TEXT NOT NULL,
    timestamp_file TEXT,
    registered_at TEXT NOT NULL,
    signature TEXT,
    public_key TEXT,
    -- BLOB fields (v0.9)
    blob_manifest TEXT,
    blob_proof TEXT,
    blob_wasm TEXT,
    blob_abi TEXT,
    -- Self-verification fields (v0.9)
    selfverify_status TEXT,
    selfverify_at TEXT,
    verifier_name TEXT,
    verifier_version TEXT,
    -- Key management fields (v0.10)
    kid TEXT,
    signature_scheme TEXT
);

CREATE INDEX IF NOT EXISTS idx_registry_hashes
    ON registry_entries (manifest_hash, proof_hash);
```

**BLOB Store (blobs)**
```sql
CREATE TABLE IF NOT EXISTS blobs (
    blob_id TEXT PRIMARY KEY,
    size INTEGER NOT NULL,
    media_type TEXT NOT NULL,
    data BLOB NOT NULL,
    refcount INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_blobs_refcount
    ON blobs(refcount);
```

### SQLite Optimization Pragmas

```sql
PRAGMA journal_mode = WAL;        -- Write-Ahead Logging
PRAGMA synchronous = NORMAL;      -- Balance safety/performance
PRAGMA cache_size = -40000;       -- 40 MB cache
PRAGMA mmap_size = 268435456;     -- 256 MB memory map
```

---

## Kryptographie

### Hash-Funktionen

| Verwendung | Algorithmus | Output-Format | Beispiel |
|------------|-------------|---------------|----------|
| Merkle Roots | BLAKE3 | 0x-präfixiert, 64 hex | `0xabc123...` |
| Audit Chain | SHA3-256 | 0x-präfixiert, 64 hex | `0xdef456...` |
| Policy Hash | SHA3-256 | 0x-präfixiert, 64 hex | `0x789abc...` |
| BLOB IDs | BLAKE3 | 0x-präfixiert, 64 hex | `0x123def...` |
| KID | BLAKE3[0:16] | 32 hex chars (no prefix) | `b3f42c9d7e6a45a1` |

### Signaturen

| Typ | Algorithmus | Key Size | Signature Size |
|-----|-------------|----------|----------------|
| Registry Signing | Ed25519 | 32 bytes | 64 bytes |
| Manifest Signing | Ed25519 | 32 bytes | 64 bytes |
| Key Attestation | Ed25519 | 32 bytes | 64 bytes |

### Encoding

| Datentyp | Encoding | Verwendung |
|----------|----------|------------|
| Proofs | Base64 | proof.dat |
| Signatures | Base64 | Registry, Manifest |
| Public Keys | Base64 | Registry, Key Metadata |
| Hashes | Hex (lowercase) | Alle Hash-Werte |

---

## Test-Coverage

### Unit Tests

| Modul | Tests | Status |
|-------|-------|--------|
| crypto | 11 | ✅ Passing |
| verifier::core | 6 | ✅ Passing |
| registry (lib) | 13 | ✅ Passing |
| keys | 6 | ✅ Passing |
| blob_store | 6 | ✅ Passing |
| **Total Library** | **54** | **✅ All Passing** |

| Modul (Binary) | Tests | Status |
|----------------|-------|--------|
| io | 2 | ✅ Passing |
| commitment | 3 | ✅ Passing |
| audit | 4 | ✅ Passing |
| policy | 7 | ✅ Passing |
| manifest | 3 | ✅ Passing |
| proof_mock | 3 | ✅ Passing |
| proof_engine | 3 | ✅ Passing |
| package_verifier | 3 | ✅ Passing |
| sign | 3 | ✅ Passing |
| registry (binary) | 9 | ✅ Passing |
| zk_system | 6 | ✅ Passing |
| lists | 4 | ✅ Passing |
| **Total Binary** | **56** | **✅ All Passing** |

### Integration Tests

| Test Suite | Tests | Status |
|------------|-------|--------|
| test_bundle_v2 | 6 | ✅ Passing |
| test_dual_anchor | 4 | ✅ Passing |
| test_hash_validation | 3 | ✅ Passing |
| test_registry_sqlite | 5 | ✅ Passing (sequential) |
| test_timestamp_provider | 3 | ✅ Passing |
| test_verify_bundle | 6 | ✅ Passing |
| test_zip_creation | 3 | ✅ Passing |
| test_zk_backend | 3 | ✅ Passing |
| **Total Integration** | **33** | **✅ All Passing** |

**Gesamt: 143 Tests, alle bestehend** ✅

### Known Issues

- **Audit Log Concurrency**: Bei parallelen Tests kann es zu korrupten Audit-Log-Einträgen kommen (mehrere JSON-Objekte auf einer Zeile). Workaround: Tests sequentiell ausführen oder separaten Audit-Log pro Test verwenden.

---

## Performance

### Current Benchmarks (Criterion.rs)

**Registry Operations (1000 Entries):**

| Operation | JSON | SQLite | Winner |
|-----------|------|--------|--------|
| Insert | 110.7 ms | 27.1 ms | ✅ SQLite (4× schneller) |
| Load | 320 µs | 1.19 ms | ✅ JSON (3.7× schneller) |
| Find | 428 µs | 9.5 µs | ✅ SQLite (45× schneller) |
| List | 533 µs | 1.29 ms | ✅ JSON (2.4× schneller) |

**Empfehlung:**
- SQLite für Workloads mit vielen Writes/Searches (Production)
- JSON für einfache Setups (<100 Entries)

### Target Performance (v0.11 - Batch Ops)

| Metric | Target | Hardware |
|--------|--------|----------|
| Insert 10k Entries | < 600s (≈17ms/entry) | 8-Core CPU |
| Verify 10k Entries | < 900s | 8-Core CPU |
| Disk Footprint | < 1.5 GB | SSD |
| Memory Peak | < 2 GB | RAM |
| Determinism | Variance < 0.5% | Reproducible |

---

## Implementierungsstatus

### ✅ Vollständig Implementiert (Production Ready)

| Feature | Version | Status |
|---------|---------|--------|
| Commitment Engine | v0.1 | ✅ |
| Policy Validation | v0.2 | ✅ |
| Manifest Builder | v0.2 | ✅ |
| Ed25519 Signing | v0.2 | ✅ |
| Proof Engine (Mock) | v0.3 | ✅ |
| Package Verifier | v0.3 | ✅ |
| JSON Registry | v0.3 | ✅ |
| Manifest Schema Validation | v0.4 | ✅ |
| Standardized Proof Export | v0.5 | ✅ |
| SQLite Registry Backend | v0.6 | ✅ |
| ZK Backend Abstraction | v0.7 | ✅ |
| Registry Entry Signing | v0.8 | ✅ |
| Verifier Core Refactor | v0.9 | ✅ |
| Crypto Namespace | v0.9 | ✅ |
| Dual-Anchor Timestamp | v0.9 | ✅ |
| BLOB Store | v0.9 | ✅ |
| Bundle v2 (WASM Verifier) | v0.9 | ✅ |
| CAPZ Container Format | v0.9 | ✅ |

### 🟡 Teilweise Implementiert (In Development)

| Feature | Version | Status | Completion |
|---------|---------|--------|------------|
| Key Management | v0.10 | 🟡 In Progress | ~60% |
| └─ KID Derivation | v0.10 | ✅ Done | 100% |
| └─ KeyMetadata Schema | v0.10 | ✅ Done | 100% |
| └─ KeyStore | v0.10 | ✅ Done | 100% |
| └─ Registry Integration | v0.10 | ✅ Done | 100% |
| └─ sign_entry() with KID | v0.10 | ✅ Done | 100% |
| └─ CLI Commands | v0.10 | ❌ Pending | 0% |
| └─ Key Rotation | v0.10 | ❌ Pending | 0% |
| └─ Key Attestation | v0.10 | ❌ Pending | 0% |

### ❌ Nicht Implementiert (Geplant)

| Feature | Version | Priority | Complexity |
|---------|---------|----------|------------|
| BLOB CLI Commands | v0.10 | High | Medium |
| Self-Verify Execution | v0.10 | High | High |
| Registry Exec Command | v0.10 | High | High |
| Key Management CLI | v0.10 | High | Medium |
| Batch Operations | v0.11 | Medium | High |
| Scale Benchmarks (10k) | v0.11 | Medium | Medium |
| Metrics Collector | v0.11 | Low | Medium |
| HSM/TPM Support | v1.0 | Low | Very High |
| Real RFC3161 TSA | v1.0 | Low | High |
| Real ZK Backend (zkVM) | v2.0 | Low | Very High |

---

## Nächste Schritte

### Kurzfristig (v0.10 Completion)

1. **Key Management CLI** (Priority: High)
   - [ ] `keygen` Command
   - [ ] `keys list` Command
   - [ ] `keys show` Command
   - [ ] `keys rotate` Command
   - [ ] `keys attest` Command
   - [ ] `keys archive` Command
   - [ ] Integration Tests

2. **BLOB Store CLI** (Priority: High)
   - [ ] `blob put` Command
   - [ ] `blob get` Command
   - [ ] `blob gc` Command
   - [ ] `blob list` Command

3. **Self-Verification** (Priority: High)
   - [ ] `registry add --selfverify` Flag
   - [ ] Sandboxed WASM Execution
   - [ ] `registry exec --id` Command

### Mittelfristig (v0.11 - Scale & Batch)

4. **Batch Operations** (Priority: Medium)
   - [ ] `registry batch-add` Command
   - [ ] `registry batch-verify` Command
   - [ ] Parallel Processing (Thread Pool)
   - [ ] Resume-Index for Crash Recovery

5. **Scale Benchmarks** (Priority: Medium)
   - [ ] Benchmark-Suite Setup
   - [ ] 10k Entry Tests
   - [ ] Metrics Collector
   - [ ] Performance Reports (JSON + MD)

### Langfristig (v1.0+)

6. **Production Hardening**
   - [ ] HSM/TPM Integration
   - [ ] Real RFC3161 TSA Support
   - [ ] Smartcard Signing (PKCS#11)
   - [ ] Remote Signer Service (gRPC)

7. **ZK Integration**
   - [ ] RISC Zero Backend
   - [ ] Halo2 Backend
   - [ ] Public Sanctions List Integration
   - [ ] ZK-Verifier CLI

---

## Dokumentation

### Verfügbare Dokumente

- **CLAUDE.md**: Vollständige System-Dokumentation mit Entwicklungshistorie
- **SYSTEMARCHITEKTUR.md**: Diese Datei - Architektur-Übersicht
- **docs/manifest.schema.json**: JSON Schema Draft 2020-12
- **docs/bundle_v2_spec.md**: Bundle v2 Spezifikation
- **Registry_BLOB_Sandbox.md**: BLOB Store & Sandbox Spezifikation (Desktop)
- **Key_Management_KID_Rotation.md**: Key Management Spezifikation (Desktop)
- **Scale_Benches_Batch_Ops.md**: Batch Operations Spezifikation (Desktop)

### Code-Dokumentation

Alle öffentlichen APIs sind mit Rust-Docstrings dokumentiert:

```bash
# Generate documentation
cargo doc --no-deps --open
```

---

## Entwicklungsrichtlinien

### Git Workflow

- **Feature Branches**: Nicht verwendet (Single Developer)
- **Commits**: Aussagekräftige Messages mit Emoji-Präfix
- **Tags**: Semantic Versioning (v0.10.0)

### Code Style

- **Linting**: `cargo clippy -- -D warnings`
- **Formatting**: `cargo fmt`
- **Tests**: Mindestens 80% Coverage für neue Features
- **Dokumentation**: Deutsche Kommentare, englische Code

### Test-Strategie

1. **Unit Tests**: Für jede öffentliche Funktion
2. **Integration Tests**: Für End-to-End Workflows
3. **Benchmarks**: Für Performance-kritische Operationen
4. **Property Tests**: Für Determinismus-Checks

---

## Lizenz & Copyright

**Projekt:** Confidential Assurance Protocol – Core Engineering
**Copyright:** © 2025
**Alle Rechte vorbehalten.**

---

**Dokumentation erstellt:** 2025-11-04
**Letzte Aktualisierung:** 2025-11-04
**Version:** v0.10.0 (Key Management Integration)
**Autor:** Tom Wesselmann mit Claude Code (Anthropic)
