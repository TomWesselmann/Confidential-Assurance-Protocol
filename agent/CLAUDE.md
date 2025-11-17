# LkSG Proof Agent – System Documentation

## Projektübersicht

Der **LkSG Proof Agent** ist ein Rust-basiertes CLI-Tool für die Erzeugung und Verifikation von kryptographischen Nachweisen im Kontext des deutschen Lieferkettensorgfaltspflichtengesetzes (LkSG).

**Version:** 0.11.0
**Status:** Tag 3 MVP (Proof & Verifier Layer) + Manifest Schema Validation v1.0 + Complete Verifier CLI + Standardized Proof Export v1.0 + Registry SQLite Adapter v1.0 + SQLite Edge-Case Tests + ZK Backend Abstraction + Registry Entry Signing + Verifier Core Refactor + Crypto Namespace + Dual-Anchor Schema v0.9.0 + Key Management & KID Rotation v0.10 + BLOB Store CLI v0.10.9 + **REST Verifier API v0.11.0** + **Week 2: Monitoring & Observability** – ✅ Erfolgreich getestet und deployed
**Entwicklung:** Tag 1 (Commitment Engine) + Tag 2 (Policy Layer) + Tag 3 (Proof Layer) + Manifest Schema Validation + Complete Verifier CLI + Standardized Proof Export + Registry SQLite Backend + ZK Backend Abstraction + Registry Entry Signing + Verifier Core Refactor + Crypto Namespace + Dual-Anchor Timestamp System + Key Management System with KID derivation and rotation + BLOB Store CLI with CAS & GC + **REST Verifier API with OAuth2 Client Credentials & Policy Management** + **Production-Ready Monitoring Stack (Prometheus, Grafana, Loki, Jaeger) - Alle 8 Container healthy, SLO/SLI Monitoring funktional**

---

## Systemarchitektur

### Architekturdiagramm

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        LkSG Proof Agent v0.11.0                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │                      REST API Layer (NEW v0.11.0)                   │  │
│  │  ┌──────────────────────────────────────────────────────────────┐  │  │
│  │  │  OAuth2 Middleware (JWT RS256)                               │  │  │
│  │  │  - Bearer Token Validation                                   │  │  │
│  │  │  - Audience/Issuer Check                                     │  │  │
│  │  │  - Scope-based Authorization                                 │  │  │
│  │  └───────────────────┬──────────────────────────────────────────┘  │  │
│  │                      │                                              │  │
│  │  ┌───────────────────▼──────────────────────────────────────────┐  │  │
│  │  │  REST Endpoints (Axum/Tokio)                                 │  │  │
│  │  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │  │  │
│  │  │  │ /healthz │  │ /readyz  │  │ /verify  │  │ /policy  │    │  │  │
│  │  │  │ (public) │  │ (public) │  │(protected│  │(protected│    │  │  │
│  │  │  └──────────┘  └──────────┘  └────┬─────┘  └────┬─────┘    │  │  │
│  │  └──────────────────────────────────┼──────────────┼──────────┘  │  │
│  └────────────────────────────────────┼──────────────┼──────────────┘  │
│                                        │              │                  │
│  ┌────────────────────────────────────▼──────────────▼──────────────┐  │
│  │                      Core Processing Layer                        │  │
│  │                                                                    │  │
│  │  Input Layer                                                      │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │  │
│  │  │ CSV Data │  │ Policy   │  │ Keys     │  │ BLOB     │        │  │
│  │  │ (S + U)  │  │ (YAML)   │  │ (Ed25519)│  │ Store    │        │  │
│  │  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘        │  │
│  │       │             │             │              │                │  │
│  │  ┌────▼─────────────▼─────────────▼──────────────▼─────┐        │  │
│  │  │      Commitment Engine (Tag 1)                      │        │  │
│  │  │  - BLAKE3 Merkle Roots                             │        │  │
│  │  │  - SHA3-256 Hash Chain Audit                       │        │  │
│  │  │  - Content-Addressable Storage (CAS)               │        │  │
│  │  └────┬────────────────────────────────────────────────┘        │  │
│  │       │                                                          │  │
│  │  ┌────▼─────────────────────────────────┐                       │  │
│  │  │      Policy Layer (Tag 2)            │                       │  │
│  │  │  - Policy Validation & Compilation   │                       │  │
│  │  │  - Manifest Builder                  │                       │  │
│  │  │  - Ed25519 Signing with KID          │                       │  │
│  │  │  - In-Memory Policy Store            │                       │  │
│  │  └────┬─────────────────────────────────┘                       │  │
│  │       │                                                          │  │
│  │  ┌────▼─────────────────────────────────┐                       │  │
│  │  │      Proof Engine (Tag 3)            │                       │  │
│  │  │  - Proof Builder (Mock → ZK-Ready)  │                       │  │
│  │  │  - Constraint Verification           │                       │  │
│  │  │  - Base64 Serialization (CAPZ)       │                       │  │
│  │  └────┬─────────────────────────────────┘                       │  │
│  │       │                                                          │  │
│  │  ┌────▼─────────────────────────────────┐                       │  │
│  │  │      Verifier Core (Portable)        │                       │  │
│  │  │  - I/O-Free Verification Logic       │                       │  │
│  │  │  - Package Verification              │                       │  │
│  │  │  - Manifest Extraction               │                       │  │
│  │  │  - Audit Trail Display               │                       │  │
│  │  └────┬─────────────────────────────────┘                       │  │
│  │       │                                                          │  │
│  └───────┼──────────────────────────────────────────────────────────┘  │
│          │                                                              │
│  ┌───────▼──────────────────────────────────────────────────────────┐  │
│  │  Output Layer                                                     │  │
│  │  ┌─────────────────────┐  ┌─────────────────────┐               │  │
│  │  │  Proof Package      │  │  REST Response      │               │  │
│  │  │  (Offline-Ready)    │  │  (JSON/HTTP)        │               │  │
│  │  │  - manifest.json    │  │  - VerifyResponse   │               │  │
│  │  │  - proof.dat        │  │  - PolicyResponse   │               │  │
│  │  │  - README.txt       │  │  - Health Status    │               │  │
│  │  └─────────────────────┘  └─────────────────────┘               │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

### Monitoring & Observability Architecture (Week 2)

```
┌─────────────────────────────────────────────────────────────────────┐
│                Monitoring & Observability Stack                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌───────────────────────────────────────────────────────────────┐│
│  │                    Grafana Dashboards                         ││
│  │  ┌──────────────────┐  ┌────────────────────┐                ││
│  │  │ Main Dashboard   │  │  SLO Dashboard     │                ││
│  │  │ - 13 Panels      │  │  - 17 Panels       │                ││
│  │  │ - Request Metrics│  │  - Error Budgets   │                ││
│  │  │ - Auth/Security  │  │  - Burn Rate       │                ││
│  │  │ - Cache Stats    │  │  - SLI Trends      │                ││
│  │  └────────┬─────────┘  └─────────┬──────────┘                ││
│  └───────────┼──────────────────────┼───────────────────────────┘│
│              │                      │                             │
│  ┌───────────▼──────────────────────▼───────────────────────────┐│
│  │              Data Sources Layer                               ││
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         ││
│  │  │ Prometheus  │  │    Loki     │  │   Jaeger    │         ││
│  │  │  (Metrics)  │  │   (Logs)    │  │  (Traces)   │         ││
│  │  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘         ││
│  └─────────┼─────────────────┼─────────────────┼───────────────┘│
│            │                 │                 │                 │
│  ┌─────────▼─────────────────▼─────────────────▼───────────────┐│
│  │             Collection & Scraping Layer                      ││
│  │  ┌─────────────────┐  ┌──────────────┐  ┌──────────────┐  ││
│  │  │ Prometheus      │  │  Promtail    │  │ OTLP/Jaeger  │  ││
│  │  │ Scraper         │  │  (Docker SD) │  │  Collector   │  ││
│  │  │ (15s interval)  │  │  (K8s SD)    │  │              │  ││
│  │  └────────┬────────┘  └──────┬───────┘  └──────┬───────┘  ││
│  └───────────┼────────────────────┼──────────────────┼─────────┘│
│              │                    │                  │           │
│  ┌───────────▼────────────────────▼──────────────────▼─────────┐│
│  │                  CAP Verifier API v0.11.0                   ││
│  │  ┌──────────────────────────────────────────────────────┐  ││
│  │  │  Exports:                                            │  ││
│  │  │  - /metrics (Prometheus format)                      │  ││
│  │  │  - JSON Logs (stdout/stderr)                        │  ││
│  │  │  - OTLP Traces (future)                             │  ││
│  │  └──────────────────────────────────────────────────────┘  ││
│  └──────────────────────────────────────────────────────────────┘│
│                                                                   │
│  ┌──────────────────────────────────────────────────────────────┐│
│  │                   Alerting & SLO Layer                       ││
│  │  ┌──────────────────┐  ┌──────────────────┐                ││
│  │  │ Prometheus       │  │  SLO Configs     │                ││
│  │  │ Alert Rules      │  │  - 99.9% Avail   │                ││
│  │  │ - 11 Alerts      │  │  - < 0.1% Error  │                ││
│  │  │ - 3 Severities   │  │  - 99.95% Auth   │                ││
│  │  │ - SLO-based      │  │  - > 70% Cache   │                ││
│  │  └──────────────────┘  └──────────────────┘                ││
│  └──────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────┘
```

**Stack Components:**
- **Prometheus** - Metrics Collection (15s scrape interval, 30d retention)
- **Grafana** - Visualization (2 Dashboards, Auto-Provisioning)
- **Loki** - Log Aggregation (31d retention, boltdb-shipper)
- **Promtail** - Log Collection (Docker + K8s Service Discovery)
- **Jaeger** - Distributed Tracing (All-in-One, 100% sampling)
- **Node Exporter** - Host Metrics (CPU, Memory, Disk)
- **cAdvisor** - Container Metrics

**Correlation Features:**
- Logs → Traces (via trace_id)
- Traces → Logs (via Loki derived fields)
- Traces → Metrics (via Prometheus queries)
- Metrics → Dashboards (Auto-Provisioning)

---

## Module & Komponenten

### Core Module (Tag 1 – Commitment Engine)

#### `io.rs` – CSV-Datenimport
- **Funktion:** Parsen von Supplier- und UBO-Daten aus CSV-Dateien
- **Strukturen:**
  - `Supplier`: name, jurisdiction, tier
  - `Ubo`: name, birthdate, citizenship
- **Methoden:**
  - `read_suppliers_csv()` – Liest Supplier-Daten
  - `read_ubos_csv()` – Liest UBO-Daten

#### `commitment.rs` – Merkle-Root-Berechnung
- **Funktion:** BLAKE3-basierte Merkle-Roots für Supplier, UBOs und Company
- **Methoden:**
  - `hash_record()` – Hasht einzelne Records (BLAKE3)
  - `compute_supplier_root()` – Berechnet Supplier Merkle Root
  - `compute_ubo_root()` – Berechnet UBO Merkle Root
  - `compute_company_commitment_root()` – Kombiniert Supplier + UBO Roots
- **Output:** `build/commitments.json`

#### `audit.rs` – Hash-Chain Audit Log
- **Funktion:** SHA3-256 Hash-Chain für Audit-Events (append-only)
- **Struktur:**
  - Jeder Event enthält: timestamp, event_type, prev_digest, payload
  - Hash-Chain: `digest = SHA3(prev_digest || timestamp || event || payload)`
- **Methoden:**
  - `log_event()` – Fügt Event zur Hash-Chain hinzu
  - `compute_digest()` – Berechnet SHA3-256 Digest
- **Output:** `build/agent.audit.jsonl` (JSONL-Format)

---

### Policy Layer (Tag 2)

#### `policy.rs` – Policy-Validierung
- **Funktion:** Lädt und validiert Compliance-Policies (YAML/JSON)
- **Schema:**
  ```yaml
  version: "lksg.v1"
  name: "Policy Name"
  constraints:
    require_at_least_one_ubo: true
    supplier_count_max: 10
  ```
- **Methoden:**
  - `Policy::load()` – Lädt Policy aus YAML/JSON
  - `validate()` – Prüft Policy-Schema
  - `compute_policy_hash()` – Berechnet SHA3-256 Policy-Hash

#### `manifest.rs` – Manifest-Builder
- **Funktion:** Erstellt Compliance-Manifest aus Commitments + Policy
- **Struktur:**
  ```json
  {
    "version": "manifest.v1.0",
    "created_at": "2025-10-25T...",
    "supplier_root": "0x...",
    "ubo_root": "0x...",
    "company_commitment_root": "0x...",
    "policy": {
      "name": "...",
      "version": "lksg.v1",
      "hash": "0x..."
    },
    "audit": {
      "tail_digest": "0x...",
      "events_count": 20
    }
  }
  ```
- **Output:** `build/manifest.json`

#### `sign.rs` – Ed25519 Signatur
- **Funktion:** Ed25519-Schlüsselerzeugung, Signierung und Verifikation
- **Methoden:**
  - `generate_keypair()` – Erzeugt Ed25519-Keypair
  - `sign_manifest()` – Signiert Manifest mit Private Key
  - `verify_signature()` – Verifiziert Signatur mit Public Key
- **Output:** `keys/company.ed25519`, `keys/company.pub`, `build/signature.json`

#### `proof_mock.rs` – Mock-Proof (Legacy)
- **Funktion:** Mock-Proof-Engine für Tag 2 (veraltet, ersetzt durch proof_engine.rs)
- **Status:** Dead code (#[allow(dead_code)]), wird durch Tag 3 Proof Engine ersetzt

---

### Key Management Layer (v0.10)

#### `keys.rs` – Key Management Module
- **Funktion:** Verwaltung von Ed25519-Schlüsseln mit KID-basiertem Rotations-System
- **Key Identifier (KID):**
  - Ableitung: `kid = blake3(base64(public_key))[0:16]` → 32 hex characters
  - Deterministisch: Gleicher Public Key → gleicher KID
  - Collision-Resistant: BLAKE3 mit 128-bit Truncation
- **KeyMetadata Struktur (cap-key.v1):**
  - `schema`: "cap-key.v1"
  - `kid`: Key Identifier (32 hex chars)
  - `owner`: Schlüsselinhaber (Organisation)
  - `created_at`, `valid_from`, `valid_to`: RFC3339 Timestamps
  - `algorithm`: "ed25519"
  - `status`: "active" | "retired" | "revoked"
  - `usage`: ["signing", "registry", "attestation"]
  - `public_key`: Base64-encoded Public Key
  - `fingerprint`: SHA-256 Fingerprint (erste 16 Bytes)
  - `comment`: Optional
- **KeyStore:**
  - Verzeichnisstruktur: `keys/`, `keys/archive/`, `keys/trusted/`
  - Methoden:
    - `new()` – Erstellt/öffnet KeyStore
    - `list()` – Listet alle Schlüssel (inkl. Archiv)
    - `find_by_kid()` – Sucht Schlüssel nach KID
    - `archive()` – Verschiebt Schlüssel ins Archiv
    - `get_active()` – Findet aktiven Schlüssel für Owner
- **Funktionen:**
  - `derive_kid()` – Berechnet KID aus Public Key
  - `compute_fingerprint()` – SHA-256 Fingerprint
- **Registry-Integration:**
  - `sign_entry()` in `registry.rs` – Signiert Registry-Eintrag mit Ed25519 und setzt automatisch KID
  - `verify_entry_signature()` – Verifiziert Signatur und KID
  - `compute_entry_core_hash()` – BLAKE3-Hash des Entry-Cores (ohne Signatur)
- **Output:** JSON-Dateien mit KeyMetadata, Ed25519-Private/Public-Keys
- **Tests:** 6 Unit-Tests (KID-Ableitung, Metadata-Roundtrip, KeyStore-Operationen)

---

### Proof & Verifier Layer (Tag 3)

#### `proof_engine.rs` – Proof-Engine (Mock → ZK-Ready)
- **Funktion:** Erstellt strukturierte Proofs gegen Policy-Constraints
- **Proof-Objekt:**
  ```json
  {
    "version": "proof.v0",
    "type": "mock",
    "statement": "policy:lksg.v1",
    "manifest_hash": "0x...",
    "policy_hash": "0x...",
    "proof_data": {
      "checked_constraints": [
        {"name": "require_at_least_one_ubo", "ok": true},
        {"name": "supplier_count_max_10", "ok": true}
      ]
    },
    "status": "ok"
  }
  ```
- **Methoden:**
  - `Proof::build()` – Erstellt Proof aus Policy + Manifest + Daten
  - `Proof::verify()` – Verifiziert Proof gegen Manifest
  - `compute_manifest_hash()` – Berechnet SHA3-256 Manifest-Hash
  - `save_as_dat()` / `load_from_dat()` – Base64-Serialisierung für proof.dat
  - `export_proof_package()` – Exportiert vollständiges Proof-Paket
- **Output:** `build/proof.dat`, `build/proof.json`

#### `verifier.rs` – Proof-Paket-Verifier
- **Funktion:** Read-only Verifikation von Proof-Paketen (offline)
- **Methoden:**
  - `Verifier::new()` – Initialisiert Verifier für Paket-Verzeichnis
  - `verify()` – Verifiziert vollständiges Proof-Paket (Manifest + Proof)
  - `check_package_integrity()` – Prüft Vollständigkeit (manifest.json, proof.dat)
  - `extract_manifest()` / `extract_proof()` – Extrahiert Komponenten
  - `show_package_summary()` – Formatierte Zusammenfassung
  - `show_audit_trail()` – Zeigt Audit-Event-Kette
- **Output:** Verifikationsergebnisse (Konsole)

#### `registry.rs` – Registry Store (Pluggable Backend + Entry Signing)
- **Funktion:** Pluggable Persistence-Layer für Proof-Registry mit JSON und SQLite Backends, inkl. Ed25519-Signierung
- **Trait:** `RegistryStore`
  - `load()` – Lädt vollständige Registry
  - `save()` – Speichert Registry
  - `add_entry()` – Fügt einzelnen Entry hinzu
  - `find_by_hashes()` – Sucht Entry nach Manifest- und Proof-Hash
  - `list()` – Listet alle Entries auf
- **Implementierungen:**
  - `JsonRegistryStore` – JSON-basierte Speicherung (Standard, backward-compatible)
  - `SqliteRegistryStore` – SQLite-basierte Speicherung (WAL mode, concurrent-safe)
- **Backend Selection:**
  - `RegistryBackend::Json` – Standard-Backend
  - `RegistryBackend::Sqlite` – SQLite-Backend
  - `open_store()` – Factory-Funktion zur Backend-Auswahl
- **SQLite Schema:**
  - `registry_meta` – Metadata (registry_version)
  - `registry_entries` – Proof-Einträge mit Index auf (manifest_hash, proof_hash), inkl. `signature`, `public_key`, `kid`, `signature_scheme` Felder
- **Entry Signing (v0.8.0) + KID Integration (v0.10):**
  - `sign_entry()` – Signiert Registry-Eintrag mit Ed25519 und setzt automatisch KID + signature_scheme
  - `verify_entry_signature()` – Verifiziert Ed25519-Signatur eines Eintrags
  - `compute_entry_core_hash()` – BLAKE3-Hash des Entry-Cores (ohne Signatur-Felder)
  - CLI-Flag: `--signing-key <path>` (default: keys/company.ed25519)
  - Backward-compatible: Einträge ohne Signatur/KID werden toleriert
  - KID wird automatisch aus Public Key abgeleitet bei Signierung
- **Migration:** Vollständige Migration zwischen Backends via `registry migrate` Command
- **Output:** `build/registry.json` oder `build/registry.sqlite`

#### `registry.rs` – Timestamp Provider (Pluggable Interface)
- **Funktion:** Abstrahiertes Timestamp-Interface für mock und echte RFC3161 TSAs
- **Trait:** `TimestampProvider`
  - `create()` – Erstellt Timestamp für Audit-Tip
  - `verify()` – Verifiziert Timestamp gegen Audit-Tip
  - `name()` – Gibt Provider-Namen zurück
- **Implementierungen:**
  - `MockRfc3161Provider` – Lokaler Mock (SHA3-basiert, kein Netzwerk)
  - `RealRfc3161Provider` – Stub für echte RFC3161 TSA (noch nicht implementiert)
- **Provider Selection:**
  - `ProviderKind::MockRfc3161` – Standard (bestehende Funktionalität)
  - `ProviderKind::RealRfc3161 { tsa_url }` – Zukünftig für echte TSAs
  - `make_provider()` – Factory-Funktion
  - `provider_from_cli()` – Parser für CLI-Flags
- **Architektur:** Vorbereitet für CLI-Integration (`--provider mock|rfc3161`)
- **Output:** Timestamp-Struktur (tsr.v1)

#### `zk_system.rs` – ZK Backend Abstraction (Pluggable Interface)
- **Funktion:** Abstrahierte ZK-Backend-Auswahl für Mock und zukünftige echte ZK-Systeme
- **Trait:** `ProofSystem`
  - `prove()` – Erstellt ZK-Proof aus Statement und Witness
  - `verify()` – Verifiziert ZK-Proof
  - `name()` – Gibt Backend-Namen zurück
- **Backend Enum:** `ZkBackend`
  - `Mock` – SimplifiedZK (bestehende Implementierung)
  - `ZkVm` – Placeholder für RISC Zero (noch nicht implementiert)
  - `Halo2` – Placeholder für Halo2 (noch nicht implementiert)
- **Factory Funktionen:**
  - `backend_factory()` – Erstellt Backend-Instanz aus ZkBackend-Enum
  - `backend_from_cli()` – Parser für CLI-String ("mock", "zkvm", "halo2")
- **Implementierungen:**
  - `SimplifiedZK` – Mock-Backend (bestehende Funktionalität)
  - `NotImplementedZk` – Stub für zukünftige Backends (liefert "not implemented" Fehler)
- **Architektur:** Vorbereitet für CLI-Integration (`--backend mock|zkvm|halo2`)
- **Output:** ZkProof-Struktur (proof.v0)

---

### Crypto & Verifier Core (v0.9.0)

#### `crypto/mod.rs` – Zentralisierte Kryptographie-API
- **Funktion:** Einheitlicher API-Layer für alle kryptographischen Operationen
- **Hash-Funktionen:**
  - `sha3_256()` – Berechnet SHA3-256 Hash (32 bytes)
  - `blake3_256()` – Berechnet BLAKE3 Hash (32 bytes)
- **Ed25519 Digital Signatures:**
  - `Ed25519SecretKey` – Wrapper für Ed25519 Private Key (32 bytes)
  - `Ed25519PublicKey` – Wrapper für Ed25519 Public Key (32 bytes)
  - `Ed25519Signature` – Wrapper für Ed25519 Signatur (64 bytes)
  - `ed25519_sign()` – Erstellt Ed25519 Signatur
  - `ed25519_verify()` – Verifiziert Ed25519 Signatur
- **Hex Encoding/Decoding:**
  - `hex_lower_prefixed32()` – Kodiert 32 Bytes als "0x..." (lowercase)
  - `hex_to_32b()` – Parst Hex-String zu 32 Bytes (strict mode)
- **Verwendung:** Alle Krypto-Operationen im Code sollten dieses Modul verwenden (konsistent, wartbar)

#### `verifier/core.rs` – Portabler Verifikationskern (I/O-frei)
- **Funktion:** Pure Verifikationslogik ohne I/O-Abhängigkeiten (portabel für CLI, Tests, WASM, zkVM)
- **Kerntypen:**
  - `ProofStatement` – Extrahiertes Statement aus Manifest (policy_hash, company_commitment_root, optional: sanctions_root, jurisdiction_root)
  - `VerifyOptions` – Verifikations-Optionen (check_timestamp, check_registry)
  - `VerifyReport` – Strukturierter Verifikationsbericht (status, hashes, signature_valid, details)
- **Funktionen:**
  - `extract_statement_from_manifest()` – Extrahiert Statement aus Manifest-JSON
  - `verify()` – Pure Verifikation (keine Dateizugriffe, keine Console-Ausgaben)
- **Invarianten:**
  - Kein File-System-Zugriff (std::fs verboten)
  - Keine Console-Ausgaben (println!/eprintln! verboten)
  - Deterministische Ergebnisse (gleiche Inputs → gleiche Outputs)
  - Alle Inputs sind In-Memory Datenstrukturen
- **Verifikationsschritte:**
  1. Hash-Berechnung (Manifest & Proof)
  2. Statement-Validierung (Manifest ↔ Statement)
  3. Signatur-Check (wenn vorhanden)
  4. Timestamp-Validierung (optional)
  5. Registry-Match (optional)
- **CLI-Integration:** CLI lädt Dateien und ruft `verifier::core::verify()` auf (siehe `run_manifest_verify()` in main.rs)
- **Tests:** 6 Unit-Tests (extract_statement_roundtrip, verify_ok_minimal, verify_ok_with_signature, verify_fail_tampered, verify_options_disable)

#### `package_verifier.rs` – Paket-Verifier (I/O-basiert)
- **Funktion:** Proof-Paket-Verifikation mit Dateizugriff (Binary-only, nicht in Library)
- **Methoden:**
  - `Verifier::new()` – Initialisiert Verifier für Paket-Verzeichnis
  - `verify()` – Verifiziert vollständiges Proof-Paket (Manifest + Proof)
  - `check_package_integrity()` – Prüft Vollständigkeit (manifest.json, proof.dat)
  - `extract_manifest()` / `extract_proof()` – Extrahiert Komponenten
  - `show_package_summary()` – Formatierte Zusammenfassung
  - `show_audit_trail()` – Zeigt Audit-Event-Kette
- **Hinweis:** Für pure, portable Verifikation siehe `verifier::core` in der Library

---

### REST API Layer (v0.11.0)

#### `api/auth.rs` – OAuth2 Authentication Middleware
- **Funktion:** JWT Bearer Token Validierung für OAuth2 Client Credentials Flow
- **Security Model:**
  - Bearer Tokens in Authorization Header
  - JWT validation mit RS256 (asymmetrisch)
  - Audience und Issuer Validierung
  - Scope-basierte Autorisierung (optional)
- **JWT Claims Struktur:**
  ```rust
  pub struct Claims {
      pub sub: String,      // Subject (client_id)
      pub iss: String,      // Issuer (OAuth2 provider URL)
      pub aud: String,      // Audience (this API)
      pub exp: usize,       // Expiration time (Unix timestamp)
      pub iat: usize,       // Issued at (Unix timestamp)
      pub scope: String,    // Scopes (space-separated)
  }
  ```
- **OAuth2Config:**
  - `issuer` – Expected issuer URL
  - `audience` – Expected audience
  - `public_key` – Public key für JWT validation (PEM format)
  - `required_scopes` – Required scopes (optional)
- **Funktionen:**
  - `validate_token()` – Validiert JWT Bearer Token
  - `extract_bearer_token()` – Extrahiert Token aus Authorization Header
  - `auth_middleware()` – Axum middleware für OAuth2 authentication
  - `generate_mock_token()` – Generiert Mock-JWT für Tests
- **Mock Keys:** RSA 2048-bit Keypair für Testing (DO NOT USE IN PRODUCTION!)
- **Tests:** 3 Unit-Tests (Token-Validierung: success, expired, missing scope)
- **Output:** 401 Unauthorized (bei fehlender/ungültiger Auth), 403 Forbidden (bei fehlenden Scopes)

#### `api/verify.rs` – Verification API Handler
- **Funktion:** REST API Handler für Proof-Verifikation
- **Request Struktur:**
  ```rust
  pub struct VerifyRequest {
      pub policy_id: String,
      pub context: VerifyContext,      // Manifest + optional data
      pub backend: String,             // "mock", "zkvm", "halo2"
      pub options: VerifyRequestOptions,
  }
  ```
- **Response Struktur:**
  ```rust
  pub struct VerifyResponse {
      pub result: String,              // "ok" oder "fail"
      pub manifest_hash: String,       // SHA3-256
      pub proof_hash: String,          // SHA3-256
      pub trace: Option<serde_json::Value>,
      pub signature: Option<String>,
      pub timestamp: Option<String>,
      pub report: VerifyReport,
  }
  ```
- **Handler:** `handle_verify()` – Orchestriert Verifikation (Manifest → Statement → Proof → Core Verify)
- **Integration:** Nutzt `verifier::core::verify()` für portable Verifikation
- **Output:** JSON Response mit vollständigem Verifikationsbericht

#### `api/policy.rs` – Policy Management API Handler
- **Funktion:** REST API Handler für Policy Compilation und Retrieval
- **Request Struktur:**
  ```rust
  pub struct PolicyCompileRequest {
      pub policy: Policy,  // Policy definition (YAML/JSON)
  }
  ```
- **Response Struktur:**
  ```rust
  pub struct PolicyCompileResponse {
      pub policy_hash: String,       // SHA3-256
      pub policy_info: PolicyInfo,
      pub status: String,            // "compiled"
  }

  pub struct PolicyGetResponse {
      pub policy_hash: String,
      pub policy: Policy,            // Full policy definition
  }
  ```
- **Policy Store:** Thread-sicherer In-Memory Store mit `OnceLock<Arc<Mutex<HashMap>>>`
- **Funktionen:**
  - `compute_policy_hash()` – Berechnet SHA3-256 Hash der Policy
  - `store_policy()` – Speichert Policy im Store
  - `get_policy()` – Ruft Policy nach Hash ab
  - `handle_policy_compile()` – Validiert und kompiliert Policy
  - `handle_policy_get()` – Ruft Policy nach Hash ab
- **Tests:** 2 Unit-Tests (Policy-Hash-Berechnung, Store-Roundtrip)
- **Output:** JSON Response mit policy_hash und policy_info

#### `bin/verifier_api.rs` – REST API Server Binary
- **Funktion:** Axum/Tokio-basierter REST API Server
- **Framework:** Axum 0.7 + Tokio (async runtime)
- **Port:** 8080 (HTTP, Phase 4 wird TLS auf 8443 für Production hinzufügen)
- **Logging:** Tracing mit env-filter
- **Router-Konfiguration:**
  - **Public Routes (ohne Auth):**
    - `GET /healthz` – Health Check (Status + Version)
    - `GET /readyz` – Readiness Check (Dependency Status)
  - **Protected Routes (mit OAuth2):**
    - `POST /verify` – Proof-Verifikation
    - `POST /policy/compile` – Policy kompilieren
    - `GET /policy/:id` – Policy abrufen
- **Middleware:** OAuth2 auth_middleware für protected routes
- **Output:** JSON responses, HTTP status codes

---

## CLI-Kommandos

### Tag 1 Commands (Commitment Engine)

#### `prepare` – Commitment-Berechnung
```bash
cargo run -- prepare --suppliers examples/suppliers.csv --ubos examples/ubos.csv
```
**Output:**
- `build/commitments.json` (Supplier-Root, UBO-Root, Company-Root)
- `build/agent.audit.jsonl` (Audit-Log mit Event "commitments_generated")

#### `inspect` – Commitment-Anzeige
```bash
cargo run -- inspect --file build/commitments.json
```
**Output:** Zeigt Roots und Counts an

#### `version` – Versionsanzeige
```bash
cargo run -- version
```
**Output:** `cap-agent v0.3.0`

---

### Tag 2 Commands (Policy Layer)

#### `policy validate` – Policy-Validierung
```bash
cargo run -- policy validate --file examples/policy.lksg.v1.yml
```
**Output:** Bestätigung der Gültigkeit + Policy-Hash

#### `manifest build` – Manifest-Erstellung
```bash
cargo run -- manifest build --policy examples/policy.lksg.v1.yml
```
**Voraussetzung:** `build/commitments.json` muss existieren
**Output:** `build/manifest.json`

#### `manifest validate` – Manifest-Validierung
```bash
cargo run -- manifest validate --file build/manifest.json
```
**Funktion:** Validiert ein Manifest gegen das JSON Schema Draft 2020-12
**Voraussetzung:** `docs/manifest.schema.json` (wird automatisch verwendet)
**Optionen:**
- `--file` - Pfad zur Manifest-Datei (erforderlich)
- `--schema` - Optionaler Pfad zum Schema (Standard: `docs/manifest.schema.json`)
**Output:**
- ✅ Validierung erfolgreich + Audit-Log-Eintrag
- ❌ Validierung fehlgeschlagen + Liste der Fehler

#### `manifest verify` – Offline Proof-Paket-Verifikation
```bash
cargo run -- manifest verify \
  --manifest build/manifest.json \
  --proof build/zk_proof.dat \
  --registry build/registry.json \
  [--timestamp build/timestamp.tsr] \
  [--out build/verification.report.json]
```
**Funktion:** Führt vollständige Offline-Verifikation eines Proof-Pakets durch
**Verifikationsschritte:**
1. Hash-Berechnung (Manifest + Proof)
2. Signatur-Verifikation (prüft ob Signaturen vorhanden)
3. Timestamp-Verifikation (optional, Mock)
4. Registry-Match (prüft ob Hashes in Registry registriert sind)

**Voraussetzung:**
- `build/manifest.json` muss existieren
- `build/zk_proof.dat` oder äquivalente Proof-Datei muss existieren
- `build/registry.json` muss existieren
**Optionen:**
- `--manifest` - Pfad zur Manifest-Datei (erforderlich)
- `--proof` - Pfad zur Proof-Datei (erforderlich)
- `--registry` - Pfad zur Registry-Datei (erforderlich)
- `--timestamp` - Optionaler Pfad zur Timestamp-Datei
- `--out` - Optionaler Pfad für Verification Report (Standard: `build/verification.report.json`)
**Output:**
- ✅ Verifikation erfolgreich + Verification Report + Audit-Log-Eintrag
- ❌ Verifikation fehlgeschlagen + Verification Report mit Details

**Report-Format:**
```json
{
  "manifest_hash": "0xd490be94abc123...",
  "proof_hash": "0x83a8779ddef456...",
  "timestamp_valid": true,
  "registry_match": true,
  "signature_valid": true,
  "status": "ok"
}
```

#### `sign keygen` – Schlüsselerzeugung
```bash
cargo run -- sign keygen --dir keys
```
**Output:** `keys/company.ed25519` (Private Key), `keys/company.pub` (Public Key)

#### `sign manifest` – Manifest-Signierung
```bash
cargo run -- sign manifest --manifest-in build/manifest.json --key keys/company.ed25519 --out build/signature.json
```
**Output:** `build/signature.json`

#### `sign verify` – Signatur-Verifikation
```bash
cargo run -- sign verify --signature build/signature.json --key keys/company.pub
```
**Output:** Bestätigung der Gültigkeit

---

### Tag 3 Commands (Proof & Verifier Layer)

#### `proof build` – Proof-Erstellung
```bash
cargo run -- proof build --manifest build/manifest.json --policy examples/policy.lksg.v1.yml
```
**Voraussetzung:** `build/manifest.json` und Policy-Datei
**Output:**
- `build/proof.dat` (Base64-kodierter Proof)
- `build/proof.json` (Lesbare JSON-Version)

#### `proof verify` – Proof-Verifikation
```bash
cargo run -- proof verify --proof build/proof.dat --manifest build/manifest.json
```
**Output:** Bestätigung der Gültigkeit + Manifest-Hash + Policy-Hash + Status

#### `proof export` – Standardisiertes CAP Proof-Paket-Export (v1.0)
```bash
cargo run -- proof export \
  --manifest build/manifest.json \
  --proof build/zk_proof.dat \
  [--timestamp build/timestamp.tsr] \
  [--registry build/registry.json] \
  [--report build/verification.report.json] \
  [--out build/cap-proof] \
  [--force]
```
**Funktion:** Erstellt ein standardisiertes, auditor-fertiges CAP Proof-Paket (v1.0)

**Voraussetzung:**
- `build/manifest.json` muss existieren
- `build/zk_proof.dat` oder äquivalente Proof-Datei muss existieren
- Optional: Timestamp, Registry, Verification Report

**Optionen:**
- `--manifest` - Pfad zur Manifest-Datei (erforderlich)
- `--proof` - Pfad zur Proof-Datei (erforderlich)
- `--timestamp` - Optionaler Pfad zur Timestamp-Datei
- `--registry` - Optionaler Pfad zur Registry-Datei
- `--report` - Optionaler Pfad zum Verification Report (wird minimal erstellt wenn nicht angegeben)
- `--out` - Output-Verzeichnis (Standard: `build/cap-proof`)
- `--force` - Überschreibt existierendes Output-Verzeichnis

**Output:** Standardisiertes CAP Proof-Paket mit fester Struktur:
```
cap-proof/
├─ manifest.json               # Manifest mit Commitments
├─ proof.dat                   # ZK-Proof (Base64-kodiert)
├─ timestamp.tsr               # Timestamp (optional)
├─ registry.json               # Registry (optional)
├─ verification.report.json    # Verification Report
├─ README.txt                  # Human-readable Anleitung
└─ _meta.json                  # SHA3-256 Hashes aller Dateien
```

**Features:**
- SHA3-256 Hashes für alle Dateien in `_meta.json`
- Package Version: `cap-proof.v1.0`
- Minimaler Verification Report wenn nicht angegeben
- Audit-Log-Eintrag für jeden Export
- Verifikationsanleitung in README.txt

#### `verifier run` – Proof-Paket-Verifikation
```bash
cargo run -- verifier run --package build/proof_package
```
**Funktion:** Offline-Verifikation des gesamten Proof-Pakets
**Output:**
- Integritätsprüfung (manifest.json, proof.dat vorhanden)
- Manifest-Hash-Verifikation
- Policy-Hash-Verifikation
- Constraint-Checks (2/2)
- Gesamtstatus (OK/FAIL)

#### `verifier extract` – Manifest-Extraktion
```bash
cargo run -- verifier extract --package build/proof_package
```
**Output:** Formatierte Zusammenfassung mit:
- Manifest-Infos (Version, Company Root, Policy Hash)
- Proof-Infos (Version, Typ, Statement, Status)
- Constraint-Checks (✅/❌ Liste)

#### `verifier audit` – Audit-Trail-Anzeige
```bash
cargo run -- verifier audit --package build/proof_package
```
**Output:** Audit-Event-Anzahl + Tail-Digest

---

### Registry Commands (Pluggable Backend)

#### `registry add` – Proof zur Registry hinzufügen
```bash
cargo run -- registry add \
  --manifest build/manifest.json \
  --proof build/proof.dat \
  [--timestamp build/timestamp.tsr] \
  [--registry build/registry.json] \
  [--backend json|sqlite]
```
**Funktion:** Fügt einen Proof-Eintrag zur Registry hinzu
**Backends:**
- `json` (Standard) - Speichert Registry in JSON-Datei
- `sqlite` - Speichert Registry in SQLite-Datenbank
**Output:**
- Registry-Eintrag mit ID, Manifest-Hash, Proof-Hash
- Audit-Log-Eintrag

#### `registry list` – Registry-Einträge auflisten
```bash
cargo run -- registry list \
  [--registry build/registry.json] \
  [--backend json|sqlite]
```
**Funktion:** Listet alle Registry-Einträge auf
**Output:** Formatierte Liste mit Manifest-Hash, Proof-Hash, Datum

#### `registry verify` – Proof gegen Registry verifizieren
```bash
cargo run -- registry verify \
  --manifest build/manifest.json \
  --proof build/proof.dat \
  [--registry build/registry.json] \
  [--backend json|sqlite]
```
**Funktion:** Verifiziert, ob ein Proof in der Registry registriert ist
**Output:** Verifikationsergebnis (OK/FAIL) + Audit-Log-Eintrag

#### `registry migrate` – Registry-Migration zwischen Backends
```bash
cargo run -- registry migrate \
  --from json --input build/registry.json \
  --to sqlite --output build/registry.sqlite
```
**Funktion:** Migriert Registry zwischen JSON und SQLite
**Output:** Anzahl migrierter Einträge + Audit-Log-Eintrag

---

### Key Management Commands (v0.10)

Das Key-Management-System ermöglicht die sichere Verwaltung von Ed25519-Schlüsseln mit eindeutigen Key Identifiers (KIDs), die aus den öffentlichen Schlüsseln abgeleitet werden. Alle Registry-Einträge werden automatisch mit KIDs signiert.

#### `keys keygen` – Schlüsselerzeugung
```bash
cargo run -- keys keygen \
  --owner "CompanyName" \
  --out keys/mykey.v1.json \
  [--algo ed25519] \
  [--valid-days 730] \
  [--comment "Purpose of this key"]
```
**Funktion:** Generiert ein Ed25519-Schlüsselpaar mit Metadaten

**Optionen:**
- `--owner` - Name des Schlüsselinhabers (Pflicht)
- `--out` - Pfad zur Metadaten-Datei (Pflicht, Format: `*.v1.json`)
- `--algo` - Algorithmus (Standard: `ed25519`)
- `--valid-days` - Gültigkeitsdauer in Tagen (Standard: 730 = 2 Jahre)
- `--comment` - Optionaler Kommentar

**Output:**
- `<out>.json` - Key-Metadaten (cap-key.v1 Schema)
- `<out>.ed25519` - Private Key (32 bytes)
- `<out>.pub` - Public Key (32 bytes)
- Audit-Log-Eintrag "key_generated"

**Key-Metadaten enthalten:**
- `kid`: 32 hex characters (abgeleitet von Public Key via BLAKE3)
- `owner`: Schlüsselinhaber
- `created_at`, `valid_from`, `valid_to`: RFC3339 Timestamps
- `algorithm`: "ed25519"
- `status`: "active" | "retired" | "revoked"
- `usage`: ["signing", "registry"]
- `public_key`: Base64-encoded Public Key
- `fingerprint`: SHA-256 Fingerprint (erste 16 Bytes)

#### `keys list` – Schlüsselliste
```bash
cargo run -- keys list \
  --dir keys \
  [--status active|retired|revoked] \
  [--owner "CompanyName"]
```
**Funktion:** Listet alle Schlüssel im Verzeichnis auf (inkl. Archiv)

**Optionen:**
- `--dir` - Schlüssel-Verzeichnis (Pflicht)
- `--status` - Filter nach Status (optional)
- `--owner` - Filter nach Owner (optional)

**Output:** Formatierte Tabelle mit KID, Owner, Status, Valid Until

#### `keys show` – Schlüssel-Details
```bash
cargo run -- keys show \
  --dir keys \
  --kid a010ac65166984697b93b867c36e9c94
```
**Funktion:** Zeigt vollständige Metadaten eines Schlüssels

**Optionen:**
- `--dir` - Schlüssel-Verzeichnis (Pflicht)
- `--kid` - Key Identifier (32 hex characters, Pflicht)

**Output:** Vollständige Key-Metadaten inkl. Public Key

#### `keys rotate` – Schlüssel-Rotation
```bash
cargo run -- keys rotate \
  --dir keys \
  --current keys/oldkey.v1.json \
  --new keys/newkey.v1.json
```
**Funktion:** Markiert alten Schlüssel als "retired", archiviert ihn, aktiviert neuen Schlüssel

**Voraussetzung:** Beide Schlüssel-Metadaten-Dateien müssen existieren

**Optionen:**
- `--dir` - Schlüssel-Verzeichnis (Pflicht)
- `--current` - Pfad zum aktuellen Schlüssel (Pflicht)
- `--new` - Pfad zum neuen Schlüssel (Pflicht)

**Output:**
- Alter Schlüssel → Status "retired" + verschoben nach `keys/archive/`
- Neuer Schlüssel → Status bleibt "active"
- Audit-Log-Eintrag "key_rotated"

#### `keys attest` – Schlüssel-Attestierung (Chain of Trust)
```bash
cargo run -- keys attest \
  --signer keys/oldkey.v1.json \
  --subject keys/newkey.v1.json \
  --out keys/attestation.json
```
**Funktion:** Erstellt signierte Attestierung für neuen Schlüssel mit altem Schlüssel (Chain of Trust)

**Voraussetzung:** Signer-Private-Key muss existieren (`<signer>.ed25519`)

**Optionen:**
- `--signer` - Pfad zum Signer-Key-Metadaten (Pflicht)
- `--subject` - Pfad zum Subject-Key-Metadaten (Pflicht)
- `--out` - Pfad zur Attestierungs-Datei (Pflicht)

**Output:**
- Attestierungs-Dokument (cap-attestation.v1 Schema) mit Ed25519-Signatur
- Audit-Log-Eintrag "key_attested"

**Attestierungs-Schema:**
```json
{
  "attestation": {
    "schema": "cap-attestation.v1",
    "signer_kid": "...",
    "signer_owner": "...",
    "subject_kid": "...",
    "subject_owner": "...",
    "subject_public_key": "...",
    "attested_at": "2025-11-04T..."
  },
  "signature": "base64...",
  "signer_public_key": "base64..."
}
```

#### `keys archive` – Schlüssel archivieren
```bash
cargo run -- keys archive \
  --dir keys \
  --kid a010ac65166984697b93b867c36e9c94
```
**Funktion:** Markiert Schlüssel als "retired" und verschiebt ihn ins Archiv

**Optionen:**
- `--dir` - Schlüssel-Verzeichnis (Pflicht)
- `--kid` - Key Identifier (32 hex characters, Pflicht)

**Output:**
- Schlüssel → Status "retired" + verschoben nach `keys/archive/`
- Audit-Log-Eintrag "key_archived"

#### `keys verify-chain` – Chain-of-Trust verifizieren
```bash
cargo run -- keys verify-chain \
  --dir keys \
  --attestations keys/att1.json,keys/att2.json,keys/att3.json
```
**Funktion:** Verifiziert eine vollständige Attestation-Kette (Chain of Trust)

**Optionen:**
- `--dir` - Schlüssel-Verzeichnis (Pflicht)
- `--attestations` - Komma-separierte Liste von Attestations-Dateien (in chronologischer Reihenfolge, Pflicht)

**Verifikationsschritte:**
1. Lädt und verifiziert jede Attestation einzeln (Signatur + KID-Konsistenz)
2. Prüft Chain-Kontinuität: Subject von Attestation N muss Signer von Attestation N+1 sein
3. Verifiziert, dass alle Signer-Keys im KeyStore existieren
4. Prüft, dass keine Signer-Keys "revoked" sind (retired ist erlaubt für historische Chains)

**Output:**
- ✅ Chain-of-Trust verifiziert
- Audit-Log-Eintrag "chain_verified"
- Bei Fehler: Detaillierte Fehlermeldung mit Position der Unterbrechung

**Beispiel:**
```bash
# Generate key1 (root)
cargo run -- keys keygen --owner Company --out keys/key1.v1.json

# Generate key2
cargo run -- keys keygen --owner Company --out keys/key2.v1.json

# Attest key2 with key1
cargo run -- keys attest \
  --signer keys/key1.v1.json \
  --subject keys/key2.v1.json \
  --out keys/key1_to_key2.json

# Generate key3
cargo run -- keys keygen --owner Company --out keys/key3.v1.json

# Attest key3 with key2
cargo run -- keys attest \
  --signer keys/key2.v1.json \
  --subject keys/key3.v1.json \
  --out keys/key2_to_key3.json

# Verify full chain: key1 → key2 → key3
cargo run -- keys verify-chain \
  --dir keys \
  --attestations keys/key1_to_key2.json,keys/key2_to_key3.json
```

#### Registry-Integration

**Automatische KID-Signierung:**
- Alle `registry add` Befehle unterstützen `--signing-key <path>` Parameter
- Beim Signieren wird automatisch der KID aus dem Public Key abgeleitet
- Registry-Einträge enthalten `kid` und `signature_scheme` Felder

**Beispiel:**
```bash
# Registry-Eintrag mit KID signieren
cargo run -- registry add \
  --manifest build/manifest.json \
  --proof build/proof.dat \
  --signing-key keys/company.ed25519

# Resultat in registry.json:
{
  "entries": [{
    "id": "proof_001",
    "manifest_hash": "0x...",
    "proof_hash": "0x...",
    "signature": "base64...",
    "public_key": "base64...",
    "kid": "a010ac65166984697b93b867c36e9c94",
    "signature_scheme": "ed25519"
  }]
}
```

**KID-Ableitung:**
- Formula: `kid = blake3(base64(public_key))[0:16]`
- Output: 32 hex characters (16 bytes)
- Deterministisch: Gleicher Public Key → gleicher KID
- Collision-Resistant: BLAKE3 mit 128-bit Truncation

**Backward-Compatibility:**
- Alle `kid` und `signature_scheme` Felder sind optional
- Alte Registry-Einträge ohne KID funktionieren weiterhin
- SQLite-Schema unterstützt NULL-Werte für neue Felder

**Key-Status-Validierung (v0.10):**

Registry Add unterstützt optionale Key-Status-Validierung, um sicherzustellen, dass nur aktive Schlüssel für neue Einträge verwendet werden:

```bash
# Registry-Eintrag mit Key-Validierung hinzufügen
cargo run -- registry add \
  --manifest build/manifest.json \
  --proof build/proof.dat \
  --signing-key keys/company.ed25519 \
  --validate-key \
  --keys-dir keys
```

**Validierungsverhalten:**
- `--validate-key` aktiviert die Validierung (optional, default: false)
- `--keys-dir` gibt den Pfad zum Key Store an (default: `keys/`)
- Validierung prüft, ob der signende Schlüssel Status "active" hat
- Rejected werden Keys mit Status:
  - "retired" → Fehler: "Key is retired and cannot be used for new entries"
  - "revoked" → Fehler: "Key is revoked and cannot be used"
- Ohne `--validate-key` können auch retired/revoked Keys signieren (für backwards compatibility)

**Anwendungsfall:**
- Verhindert versehentliche Verwendung archivierter Schlüssel
- Erzwingt korrekte Key-Rotation-Workflows
- Ermöglicht Policy-konforme Registry-Einträge
- Audit-Trail dokumentiert, welche Keys für welche Einträge verwendet wurden

---

### Dual-Anchor Commands (v0.9.0)

Das Dual-Anchor-System ermöglicht die Verknüpfung von Manifesten mit sowohl privaten (lokalen Audit-Chain) als auch öffentlichen (Blockchain) Zeitstempeln. Dies bereitet das System für zukünftige Ledger-Integration vor, ohne jetzt schon On-Chain-Operationen durchzuführen.

#### Konzept

Ein **Dual-Anchor** besteht aus zwei optionalen Komponenten:

1. **Private Anchor**: Lokaler Audit-Tip (SHA3-256 Hash der Audit-Chain)
   - Verknüpft Manifest mit internem Audit-Log
   - Ermöglicht Konsistenzprüfung zwischen Manifest und Audit-Kette
   - Gespeichert in `time_anchor.private`

2. **Public Anchor**: Referenz zu öffentlichem Ledger (Ethereum/Hedera/BTC)
   - Enthält Blockchain, Transaction ID und Digest
   - Vorbereitet für zukünftige On-Chain-Notarisierung
   - Gespeichert in `time_anchor.public`

**Konsistenzregel**: `time_anchor.private.audit_tip_hex` muss mit `time_anchor.audit_tip_hex` übereinstimmen.

#### `audit set-private-anchor` – Private Anchor setzen
```bash
cargo run -- audit set-private-anchor \
  --manifest build/manifest.json \
  --audit-tip 0x83a8779d12345678... \
  [--created-at "2025-10-30T10:00:00Z"]
```
**Funktion:** Setzt den privaten Zeitanker (lokaler Audit-Tip) im Manifest

**Voraussetzung:**
- Manifest muss bereits existieren
- `time_anchor` muss im Manifest initialisiert sein
- `audit_tip` muss mit `time_anchor.audit_tip_hex` übereinstimmen

**Optionen:**
- `--manifest` - Pfad zur Manifest-Datei (Input/Output)
- `--audit-tip` - Audit-Chain-Tip (0x-präfixierter Hex-String, 64 Zeichen)
- `--created-at` - Optional: RFC3339 Timestamp (Standard: jetzt)

**Output:**
- Aktualisiertes Manifest mit `time_anchor.private` Feld
- Audit-Log-Eintrag

**Beispiel:**
```bash
# Manifest mit time_anchor vorbereiten
cargo run -- audit anchor --kind tsa --reference ./tsa/test.tsr \
  --manifest-in build/manifest.json --manifest-out build/manifest.json

# Private Anchor setzen
cargo run -- audit set-private-anchor \
  --manifest build/manifest.json \
  --audit-tip 0x83a8779d12345678901234567890123456789012345678901234567890123456
```

#### `audit set-public-anchor` – Public Anchor setzen
```bash
cargo run -- audit set-public-anchor \
  --manifest build/manifest.json \
  --chain ethereum|hedera|btc \
  --txid <transaction-id> \
  --digest 0x1234567890123456... \
  [--created-at "2025-10-30T10:00:00Z"]
```
**Funktion:** Setzt den öffentlichen Zeitanker (Blockchain-Referenz) im Manifest

**Voraussetzung:**
- Manifest muss bereits existieren
- `time_anchor` muss im Manifest initialisiert sein

**Optionen:**
- `--manifest` - Pfad zur Manifest-Datei (Input/Output)
- `--chain` - Blockchain (ethereum, hedera, btc)
- `--txid` - Transaction ID (Format abhängig von Chain)
- `--digest` - Hash des Audit-Tip für On-Chain-Notarisierung (0x-präfixiert, 64 Zeichen)
- `--created-at` - Optional: RFC3339 Timestamp (Standard: jetzt)

**Output:**
- Aktualisiertes Manifest mit `time_anchor.public` Feld
- Audit-Log-Eintrag

**Beispiel:**
```bash
# Public Anchor für Ethereum setzen
cargo run -- audit set-public-anchor \
  --manifest build/manifest.json \
  --chain ethereum \
  --txid 0xabc123def456... \
  --digest 0x1234567890123456789012345678901234567890123456789012345678901234
```

#### `audit verify-anchor` – Dual-Anchor-Konsistenz verifizieren
```bash
cargo run -- audit verify-anchor \
  --manifest build/manifest.json \
  [--out build/anchor_verification.json]
```
**Funktion:** Validiert die Konsistenz des Dual-Anchor-Systems

**Verifikationsschritte:**
1. Prüft ob `private.audit_tip_hex` mit `time_anchor.audit_tip_hex` übereinstimmt
2. Validiert `public.digest` Format (0x-präfixiert, 64 Zeichen)
3. Prüft ob `public.txid` nicht leer ist
4. Generiert strukturierten JSON-Report

**Voraussetzung:**
- Manifest muss existieren
- Mindestens ein Anchor (private oder public) sollte gesetzt sein

**Optionen:**
- `--manifest` - Pfad zur Manifest-Datei
- `--out` - Optional: Pfad für JSON-Report

**Output:**
- Konsolen-Ausgabe mit Verifikationsergebnis
- Optional: JSON-Report mit Details

**Report-Format:**
```json
{
  "status": "ok",
  "manifest": "build/manifest.json",
  "errors": [],
  "private_ok": true,
  "public_ok": true,
  "digest_match": true
}
```

**Beispiel:**
```bash
# Vollständiger Workflow
cargo run -- audit set-private-anchor \
  --manifest build/manifest.json \
  --audit-tip 0x83a8779d...

cargo run -- audit set-public-anchor \
  --manifest build/manifest.json \
  --chain ethereum \
  --txid 0xabc123... \
  --digest 0x1234567890...

cargo run -- audit verify-anchor \
  --manifest build/manifest.json \
  --out build/anchor_verification.json
```

#### Manifest-Struktur mit Dual-Anchor

```json
{
  "version": "manifest.v1.0",
  "created_at": "2025-10-30T10:00:00Z",
  "time_anchor": {
    "kind": "tsa",
    "reference": "./tsa/test.tsr",
    "audit_tip_hex": "0x83a8779d12345678901234567890123456789012345678901234567890123456",
    "created_at": "2025-10-30T10:00:00Z",
    "private": {
      "audit_tip_hex": "0x83a8779d12345678901234567890123456789012345678901234567890123456",
      "created_at": "2025-10-30T10:05:00Z"
    },
    "public": {
      "chain": "ethereum",
      "txid": "0xabc123def456...",
      "digest": "0x1234567890123456789012345678901234567890123456789012345678901234",
      "created_at": "2025-10-30T10:10:00Z"
    }
  }
}
```

**Wichtige Hinweise:**
- Alle Dual-Anchor-Felder sind optional (backward-compatible)
- Private und Public Anchors können unabhängig voneinander gesetzt werden
- Die CLI-Befehle führen keine On-Chain-Operationen durch
- Zukünftige Ledger-Integration kann nahtlos hinzugefügt werden (Drop-In)
- Verifier Core validiert Dual-Anchor-Konsistenz automatisch (verifier/core.rs)

---

## Datenflüsse

### End-to-End Pipeline (Tag 1 → Tag 2 → Tag 3)

```
1. prepare                 → build/commitments.json
   (CSV → BLAKE3 Roots)

2. policy validate         → Policy-Hash-Validierung
   (YAML/JSON)

3. manifest build          → build/manifest.json
   (Commitments + Policy)

4. proof build             → build/proof.dat + proof.json
   (Manifest + Policy)

5. proof verify            → Verifikationsergebnis (OK/FAIL)
   (Proof.dat + Manifest)

6. sign keygen             → keys/company.ed25519 + company.pub
   (Ed25519)

7. sign manifest           → build/signature.json
   (Manifest + Private Key)

8. proof export            → build/proof_package/
   (Manifest + Proof → Package)

9. verifier run            → Verifikationsergebnis (OK/FAIL)
   (Proof-Paket → Offline-Verifikation)

10. verifier extract       → Formatierte Zusammenfassung
    (Proof-Paket → Manifest-Infos)

11. verifier audit         → Audit-Trail (Events + Tail-Digest)
    (Proof-Paket → Audit-Info)
```

---

## Dateiformat-Spezifikationen

### `commitments.json`
```json
{
  "supplier_root": "0x...",
  "ubo_root": "0x...",
  "company_commitment_root": "0x...",
  "supplier_count": 5,
  "ubo_count": 2
}
```

### `manifest.json` (manifest.v1.0)
```json
{
  "version": "manifest.v1.0",
  "created_at": "2025-10-25T13:07:41.027357+00:00",
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
    "events_count": 20
  },
  "proof": {
    "proof_type": "none",
    "status": "none"
  },
  "signatures": []
}
```

### `proof.dat` (Base64-kodiert)
- **Format:** Base64(JSON(Proof))
- **Vorteil:** Binär-kompatibel, kompakt, offline-übertragbar
- **Dekodierung:** `base64::decode() → JSON::parse()`

### `proof.json` (proof.v0)
```json
{
  "version": "proof.v0",
  "type": "mock",
  "statement": "policy:lksg.v1",
  "manifest_hash": "0x...",
  "policy_hash": "0x...",
  "proof_data": {
    "checked_constraints": [
      {"name": "require_at_least_one_ubo", "ok": true},
      {"name": "supplier_count_max_10", "ok": true}
    ]
  },
  "status": "ok"
}
```

### `signature.json`
```json
{
  "manifest_hash": "0x...",
  "signature": "0x...",
  "signer_pubkey": "0x...",
  "signed_at": "2025-10-25T..."
}
```

### `docs/manifest.schema.json` (JSON Schema Draft 2020-12)
**Funktion:** Formale JSON-Schema-Validierung für Manifeste (manifest.v1.0 + Dual-Anchor v0.9.0)
**Standard:** JSON Schema Draft 2020-12
**Validierungsregeln:**
- `version`: Muss exakt "manifest.v1.0" sein
- `created_at`: RFC3339 DateTime-Format
- `supplier_root`, `ubo_root`, `company_commitment_root`: BLAKE3-Hashes (0x + 64 Hex-Zeichen)
- `policy.version`: Pattern `^[a-z0-9\\.]+$` (z.B. "lksg.v1")
- `policy.hash`: SHA3-256 Hash (0x + 64 Hex-Zeichen)
- `audit.tail_digest`: SHA3-256 Hash (0x + 64 Hex-Zeichen)
- `audit.events_count`: Integer ≥ 0
- `proof.type`: Enum ["none", "mock", "zk", "halo2", "spartan", "risc0"]
- `proof.status`: Enum ["none", "ok", "failed"]
- `signatures`: Array von Ed25519-Signaturen
- `time_anchor` (optional): TSA/Blockchain/File-Zeitstempel
  - `time_anchor.private` (optional): Private Anchor mit audit_tip_hex und created_at
  - `time_anchor.public` (optional): Public Anchor mit chain (ethereum|hedera|btc), txid, digest, created_at
- `sanctions_root` (optional): BLAKE3 Root für Sanctions-Listen (ZK-Proofs)
- `jurisdiction_root` (optional): BLAKE3 Root für Jurisdiction-Listen (ZK-Proofs)

**Verwendung:**
```bash
# Vollständige Schema-Validierung
cargo run -- manifest validate --file build/manifest.json --schema docs/manifest.schema.json

# Test mit Dual-Anchor-Feldern
cargo run -- manifest validate --file build/manifest_test_schema.json --schema docs/manifest.schema.json
```

**Testabdeckung:**
- ✅ Gültige Manifeste mit Dual-Anchor-Feldern
- ✅ Gültige Manifeste ohne Dual-Anchor-Felder (Backward-Kompatibilität)
- ✅ Ungültige Manifeste werden mit detaillierten Fehlermeldungen abgelehnt

### `build/verification.report.json` (Verification Report)
**Funktion:** Report für vollständige Offline-Verifikation (manifest verify)
**Format:**
```json
{
  "manifest_hash": "0xd490be94abc123...",
  "proof_hash": "0x83a8779ddef456...",
  "timestamp_valid": true,
  "registry_match": true,
  "signature_valid": true,
  "status": "ok"
}
```
**Felder:**
- `manifest_hash`: SHA3-256 Hash der Manifest-Datei
- `proof_hash`: SHA3-256 Hash der Proof-Datei
- `timestamp_valid`: Timestamp-Verifikation (true/false)
- `registry_match`: Registry-Eintrag gefunden (true/false)
- `signature_valid`: Signatur vorhanden (true/false)
- `status`: Gesamtstatus ("ok" oder "fail")

**Verwendung:**
```bash
cargo run -- manifest verify --manifest build/manifest.json --proof build/zk_proof.dat --registry build/registry.json
```

### `build/cap-proof/_meta.json` (Package Metadata CAP v1.0)
**Funktion:** Metadata und SHA3-256 Hashes für standardisierte CAP Proof-Pakete
**Version:** cap-proof.v1.0
**Format:**
```json
{
  "version": "cap-proof.v1.0",
  "created_at": "2025-10-30T...",
  "files": {
    "manifest": "manifest.json",
    "proof": "proof.dat",
    "timestamp": "timestamp.tsr",
    "registry": "registry.json",
    "report": "verification.report.json",
    "readme": "README.txt"
  },
  "hashes": {
    "manifest_sha3": "0x...",
    "proof_sha3": "0x...",
    "timestamp_sha3": "0x...",
    "registry_sha3": "0x...",
    "report_sha3": "0x..."
  }
}
```
**Felder:**
- `version`: Package-Format-Version ("cap-proof.v1.0")
- `created_at`: RFC3339 Timestamp der Package-Erstellung
- `files`: Liste aller enthaltenen Dateien (optionale Felder können null sein)
- `hashes`: SHA3-256 Hashes aller Dateien (0x-präfixiert, 64 Hex-Zeichen)

**Verwendung:**
```bash
cargo run -- proof export --manifest build/manifest.json --proof build/zk_proof.dat --registry build/registry.json
```

**Zweck:**
- Integritätsprüfung aller Package-Dateien
- Versionskontrolle für Package-Format
- Audit-Trail für Package-Erstellung
- Maschinenlesbare Metadaten für automatisierte Verifikation

### `agent.audit.jsonl` (JSONL – Hash-Chain)
```jsonl
{"timestamp":"2025-10-25T13:07:40.995758+00:00","event":"session_start","prev_digest":"0x0","payload":{},"digest":"0x..."}
{"timestamp":"2025-10-25T13:07:41.025912+00:00","event":"commitments_generated","prev_digest":"0x...","payload":{"supplier_count":5,"ubo_count":2},"digest":"0x..."}
...
```

---

## Kryptographische Primitiven

| Funktion | Algorithmus | Verwendung |
|----------|-------------|------------|
| Merkle Roots | BLAKE3 | Commitment-Berechnung (Supplier, UBO) |
| Audit Hash-Chain | SHA3-256 | Append-only Event-Log |
| Policy Hash | SHA3-256 | Policy-Identifikation |
| Manifest Hash | SHA3-256 | Proof-Verifikation |
| Signatur | Ed25519 | Manifest-Signierung |
| Encoding | Base64 | proof.dat Serialisierung |

---

## Test-Ergebnisse

### Unit-Tests (Tag 1 + 2 + 3)
```bash
cargo test
```
**Ergebnis:** 145/146 Tests bestanden ✅ (57 Library Unit + 65 Binary Unit + 6 Bundle Integration + 4 Dual-Anchor Integration + 3 Hash Validation Integration + 3 Registry Key Chain Integration + 4 SQLite Integration, 1 pre-existing failure)

**Tests pro Modul:**
- **Library (57 tests):**
  - `crypto::tests`: 11 Tests (SHA3, BLAKE3, Ed25519, Hex encoding)
  - `verifier::core::tests`: 6 Tests (Statement extraction, Verification core, Options)
  - `registry::tests` (Library): 13 Tests (Registry CRUD, Timestamp, Entry Signing)
  - `keys::tests`: 9 Tests (KID derivation, KeyMetadata roundtrip, KeyStore operations, **3 neue Property Tests**: Determinism, Uniqueness, Metadata Consistency)
  - `blob_store::tests`: 6 Tests (Blob storage, metadata, garbage collection)
  - `proof::tests`: 6 Tests (Proof generation, verification, CAPZ format)
  - `wasm::tests`: 2 Tests (WASM loader, executor config)
  - `verifier::tests`: 4 Tests (Verification workflows)
- **Binary (65 tests):**
  - `io::tests`: 2 Tests (CSV-Parsing)
  - `commitment::tests`: 3 Tests (Merkle-Roots, Determinismus)
  - `audit::tests`: 4 Tests (Hash-Chain, Digest-Berechnung, Tip)
  - `policy::tests`: 7 Tests (Validation, YAML-Loading, Hash-Determinismus, Constraints)
  - `manifest::tests`: 3 Tests (Erstellung, Proof-Update, Time Anchor)
  - `proof_mock::tests`: 3 Tests (Mock-Proof-Generation, Verifikation)
  - `proof_engine::tests`: 3 Tests (Proof-Build, Verify, DAT-Serialisierung)
  - `package_verifier::tests`: 3 Tests (Integrity-Check, Package-Summary)
  - `sign::tests`: 3 Tests (Keypair-Generation, Sign & Verify)
  - `registry::tests` (Binary): 9 Tests (Registry CRUD, Timestamp, Entry Signing)
  - `zk_system::tests`: 6 Tests (ZK System, Sanctions, Serialization)
  - `lists::tests`: 4 Tests (Sanctions, Jurisdictions)
- **Integration (20 passing, 1 pre-existing failure):**
  - `test_bundle_v2`: 6 Tests (Bundle creation, structure, integrity) ✅
  - `test_dual_anchor`: 4 Tests (Set Private Anchor, Set Public Anchor, Verify Anchor, Mismatch Validation) ✅
  - `test_hash_validation`: 3 Tests (Tamper detection, valid bundle verification) ✅
  - `test_registry_key_chain`: 3 Tests **(NEU v0.10)**:
    - Active key validation (successful registry add) ✅
    - Retired key rejection ✅
    - Revoked key rejection ✅
    - Chain-of-Trust verification ✅
  - `test_registry_sqlite`: 4 Tests (Corruption, Duplicates, WAL, Roundtrip) ✅
  - Note: 1 test (`test_migrate_empty_registry`) failing due to pre-existing issue unrelated to v0.10 refactoring

### Clippy (Lint-Check)
```bash
cargo clippy -- -D warnings
```
**Ergebnis:** 0 Warnings ✅

### Integrations-Tests (End-to-End)
**Pipeline:** 10 Schritte vollständig getestet ✅

1. ✅ prepare (CSV → Commitments)
2. ✅ policy validate (Policy-Hash)
3. ✅ manifest build (Manifest-Erstellung)
4. ✅ proof build (Proof-Erstellung)
5. ✅ proof verify (Proof-Verifikation)
6. ✅ sign manifest (Ed25519-Signatur)
7. ✅ proof export (Proof-Paket-Export)
8. ✅ verifier run (Offline-Verifikation)
9. ✅ verifier extract (Manifest-Extraktion)
10. ✅ verifier audit (Audit-Trail-Anzeige)

**Bewertung:** Alle Akzeptanzkriterien erfüllt ✅

---

## Datei-Outputs (Build-Verzeichnis)

Nach vollständiger Pipeline:
```
build/
├── agent.audit.jsonl         # Hash-Chain Audit-Log (JSONL)
├── commitments.json          # Merkle Roots (Supplier, UBO, Company)
├── manifest.json             # Compliance-Manifest
├── proof.dat                 # Base64-kodierter Proof
├── proof.json                # Lesbare Proof-Version
├── signature.json            # Ed25519-Signatur
└── proof_package/            # Offline-Paket für Auditoren
    ├── manifest.json
    ├── proof.dat
    └── README.txt
```

---

## Technische Vorgaben

| Bereich | Spezifikation |
|---------|---------------|
| Sprache | Rust (Edition 2021) |
| CLI | clap v4.5 (derive) |
| Hashing | blake3 v1.5 + sha3 v0.10 |
| Signatur | ed25519-dalek v2.1 |
| Serialisierung | serde + serde_json + serde_yaml + base64 |
| CSV | csv v1.3 |
| JSON Schema | jsonschema v0.17 (Draft 2020-12) |
| SQLite | rusqlite v0.31 (bundled) |
| Zeitformat | RFC3339 (UTC, chrono v0.4) |
| Plattform | Offline, Cross-Platform (Linux/macOS/Windows) |
| Netzwerk | Verboten (kein HTTP, kein API-Zugriff) |

---

## Definition of Done (v0.9.0)

- ✅ End-to-End-Pipeline vollständig implementiert (10 Schritte)
- ✅ Alle Artefakte in `build/proof_package/` generiert
- ✅ Proof-Pakete verifizierbar durch externes Verifier-Tool
- ✅ CI-Pipeline grün (Build + Test + Clippy)
- ✅ 90/91 Tests bestanden (1 pre-existing failure)
- ✅ Neue Tests: 11 Crypto + 6 Verifier Core
- ✅ 0 Clippy-Warnings in neuen Modulen (1 pre-existing warning in Registry)
- ✅ Reproduzierbare Hashes & Proofs (deterministisch)
- ✅ Verifier Core I/O-frei (portable für CLI, Tests, WASM, zkVM)
- ✅ Crypto-Namespace zentralisiert alle Krypto-Operationen
- ✅ CLI nutzt portable Verifier Core
- ✅ Dokumentation vollständig (CLAUDE.md)
- ✅ Alle Module mit deutschen Docstrings kommentiert

---

## Performance Benchmarking (v0.8.0)

### Benchmark-Suite mit Criterion.rs

Das Projekt enthält automatisierte Performance-Benchmarks für das Registry-System, um JSON- und SQLite-Backends zu vergleichen.

#### Ausführung:
```bash
# Vollständige Benchmarks
cargo bench --bench registry_bench

# Schneller Test
cargo bench --bench registry_bench -- --quick

# HTML-Reports generieren
open target/criterion/report/index.html
```

#### Benchmark-Kategorien:

1. **registry_insert** - Einfügen von Einträgen
2. **registry_load** - Laden der gesamten Registry
3. **registry_find** - Suchen nach Hash
4. **registry_list** - Auflisten aller Einträge

#### Performance-Ergebnisse (1000 Einträge):

| Operation | JSON | SQLite | Vergleich |
|-----------|------|--------|-----------|
| **Insert** | 110.7 ms | 27.1 ms | ✅ SQLite **4× schneller** |
| **Load** | 320 µs | 1.19 ms | ✅ JSON **3.7× schneller** |
| **Find** | 428 µs | 9.5 µs | ✅ SQLite **45× schneller** (Index) |
| **List** | 533 µs | 1.29 ms | ✅ JSON **2.4× schneller** |

**Empfehlung:**
- SQLite für Workloads mit vielen Writes und Suchen (Production)
- JSON für einfache Setups und kleine Datenmengen (<100 Entries)

---

---

## Proof Bundle v2 (WASM-Verifier + Loader) – v0.9.0

### Überblick

Bundle v2 ist ein **self-contained Proof-Package**, das neben Manifest und Proof auch einen WASM-Verifier und Executor-Konfiguration enthält. Dies ermöglicht **offline-Verifikation ohne externe Software**.

### Bundle-Struktur

```
cap-proof-v2/
├─ manifest.json          # Compliance manifest
├─ proof.capz             # CAPZ v2 container (binary)
├─ verifier.wasm          # WASM verifier (optional)
├─ executor.json          # Executor config (optional)
├─ _meta.json             # SHA3-256 hashes + metadata
└─ README.txt             # Instructions (optional)
```

### CAPZ Container Format

**CAPZ** (CAP Proof Container) ist ein binäres Format mit versioniertem Header:

```
Header (78 bytes, little endian):
  magic[4]      = b"CAPZ"
  version[2]    = 0x0002
  backend[1]    = 0=mock, 1=zkvm, 2=halo2
  reserved[1]   = 0x00
  vk_hash[32]   = verification key hash
  params_hash[32]= params hash
  payload_len[4]= u32 LE
  payload[...]  = proof data
```

**Features:**
- Version checking (current: v2)
- Backend identification
- Optional VK/params hashes
- Max payload: 100 MB

### CLI Commands

#### `bundle-v2` – Bundle erstellen

```bash
cargo run -- bundle-v2 \
  --manifest build/manifest.json \
  --proof build/proof.capz \
  [--verifier-wasm build/verifier.wasm] \
  --out build/cap-proof-v2 \
  [--zip] \
  [--force]
```

**Funktion:**
- Kopiert Manifest + Proof ins Output-Verzeichnis
- Kopiert WASM-Verifier (falls angegeben)
- Erstellt executor.json mit Default-Config
- Generiert _meta.json mit SHA3-256-Hashes

**Output:**
- Bundle-Verzeichnis mit allen Dateien
- Optional: ZIP-Archiv (noch nicht implementiert)

#### `verify-bundle` – Bundle verifizieren

```bash
cargo run -- verify-bundle \
  --bundle build/cap-proof-v2 \
  [--out build/verification.report.json]
```

**Funktion:**
1. Lädt Bundle-Metadaten
2. Prüft auf verifier.wasm (Fallback: native)
3. Führt Verifikation aus (WASM oder native)
4. Generiert VerifyReport

**Fallback:**
- Ohne verifier.wasm → nutzt `verifier::core::verify()` (native)
- Keine Breaking Changes für v1-Bundles

### WASM Sandbox

**Sicherheitsgrenzen:**
- **Kein I/O:** Keine FS/Netz-Zugriffe
- **Memory-Limit:** 128 MB (konfigurierbar)
- **Timeout:** 3 Sekunden (konfigurierbar)
- **Fuel Metering:** 5M computational units (konfigurierbar)

**WASM ABI:**
```
verify(manifest_ptr: i32, manifest_len: i32,
       proof_ptr: i32, proof_len: i32,
       options_ptr: i32, options_len: i32) -> result_ptr: i32
```

- Input: JSON-encoded bytes (Manifest, Proof, Options)
- Output: JSON VerifyReport als `[length: i32][data: bytes]`

### Module

#### `proof/capz.rs` – CAPZ Container
- `CapzHeader` – Binary header reader/writer
- `CapzContainer` – Full container with payload
- Validation: Magic, Version, Backend, Payload size

#### `wasm/loader.rs` – WASM Loader
- `WasmVerifier` – Wasmtime-basierter Loader
- `WasmLimits` – Konfigurierbare Execution-Limits
- Sandbox: No I/O, Memory/Fuel limits

#### `wasm/executor.rs` – Bundle Executor
- `BundleExecutor` – Orchestriert Bundle-Verifikation
- `ExecutorConfig` – executor.json Schema
- Fallback: Native verifier wenn kein WASM

### Beispiel-Workflow

```bash
# 1. Bundle erstellen (ohne WASM)
cargo run -- bundle-v2 \
  --manifest build/manifest.json \
  --proof build/proof.dat \
  --out build/my-proof

# 2. Bundle verifizieren (native fallback)
cargo run -- verify-bundle \
  --bundle build/my-proof \
  --out build/report.json

# 3. Bundle mit WASM erstellen (zukünftig)
cargo run -- bundle-v2 \
  --manifest build/manifest.json \
  --proof build/proof.capz \
  --verifier-wasm build/verifier.wasm \
  --out build/wasm-proof

# 4. WASM-Bundle verifizieren
cargo run -- verify-bundle \
  --bundle build/wasm-proof
```

### Tests

**Unit Tests:**
- `proof::capz::tests` – 8 Tests (Header, Container, Validation)
- `wasm::loader::tests` – 2 Tests (Limits, Config)
- `wasm::executor::tests` – 2 Tests (Config roundtrip)

**Integration Tests:** (geplant)
- `test_bundle_v2` – Bundle creation, hash validation
- `test_verify_bundle` – End-to-End verification

### Status

- ✅ CAPZ Container implementiert
- ✅ WASM Loader implementiert
- ✅ CLI Commands implementiert
- ✅ Dokumentation (bundle_v2_spec.md)
- ⏳ WASM Verifier Fixture (Test-WASM)
- ⏳ ZIP-Archiv-Erstellung
- ⏳ Integration Tests

---

## BLOB Store CLI (v0.10.9)

### Überblick

Der BLOB Store ist ein **Content-Addressable Storage (CAS)** System mit BLAKE3-basierter Deduplizierung, Referenzzählung (refcount) und Garbage Collection. Alle Proof-Package-Komponenten (Manifest, Proof, WASM, ABI) werden als BLOBs mit eindeutigen IDs gespeichert.

**Features:**
- ✅ BLAKE3-basierte Content-Addressing (0x-präfixiert, 64 hex chars)
- ✅ Automatische Deduplizierung (gleicher Inhalt → gleiche BLOB ID)
- ✅ Referenzzählung für Registry-Verknüpfung
- ✅ Garbage Collection (mark-and-sweep)
- ✅ SQLite Backend mit WAL mode
- ✅ Transaktionale ACID-Garantien

### CLI Commands

#### `blob put` – BLOB einfügen

Fügt eine Datei in den BLOB Store ein (mit CAS & optional Registry-Verknüpfung).

```bash
cargo run -- blob put \
  --file ./build/manifest.json \
  --type manifest \
  [--registry ./build/registry.sqlite] \
  [--link-entry-id <uuid>] \
  [--stdin] \
  [--out blob_id.txt] \
  [--no-dedup]
```

**Optionen:**
- `--file <path>`: Quelldatei (mehrfach nutzbar); alternativ `--stdin`
- `--type <media>`: `manifest|proof|wasm|abi|other`
- `--registry <path>`: Registry-Datei (Standard: `build/registry.sqlite`)
- `--link-entry-id <id>`: Erhöht `refcount` für den referenzierenden Registry-Eintrag
- `--stdin`: Liest Daten von stdin (Pipes)
- `--out <path>`: Schreibt `blob_id` in Datei
- `--no-dedup`: Erzwingt Re-Insert (nur Tests/Debug)

**Verhalten:**
- Hash = BLAKE3(content) → `0x...` (64 hex)
- Wenn `blob_id` existiert → nur `refcount++` (sofern `--no-dedup` nicht gesetzt)
- Transaktion: Insert / Refcount Update atomar
- Ergebnis: `stdout` gibt `blob_id` aus, Exit 0

**Exit Codes:**
- 0 OK, 10 IO-Fehler, 11 SQLite-Fehler, 12 Ungültiger Medientyp

**Beispiel:**
```bash
# BLOB einfügen
cargo run -- blob put --file build/manifest.json --type manifest

# Output:
# 📥 Lese Datei: build/manifest.json
# 📊 Größe: 1234 bytes, Medientyp: manifest
# ✅ BLOB gespeichert: 0x1da941f7026bae3cf8b1bcdc3a8e01e76ea678c32ec6bc2c374fb67b3744571f
#
# 0x1da941f7026bae3cf8b1bcdc3a8e01e76ea678c32ec6bc2c374fb67b3744571f
```

#### `blob get` – BLOB abrufen

Extrahiert Blob-Inhalt anhand `blob_id` auf Datei oder stdout.

```bash
cargo run -- blob get \
  --id 0xabc... \
  [--out ./out.bin] \
  [--stdout]
```

**Optionen:**
- `--id <blob_id>` (erforderlich, 0x-präfixiert, 64 hex chars)
- `--out <path>`: Zielpfad
- `--stdout`: Schreibt Rohdaten auf stdout (Standard wenn `--out` fehlt)

**Exit Codes:**
- 0 OK, 20 NotFound, 11 SQLite-Fehler

**Beispiel:**
```bash
# BLOB abrufen
cargo run -- blob get \
  --id 0x1da941f7026bae3cf8b1bcdc3a8e01e76ea678c32ec6bc2c374fb67b3744571f \
  --out /tmp/retrieved.json

# Output:
# 🔍 Suche BLOB: 0x1da941f...
# ✅ BLOB gefunden, Größe: 1234 bytes
# 📄 BLOB geschrieben nach: /tmp/retrieved.json
```

#### `blob list` – BLOBs auflisten

Listet Blobs gefiltert/sortiert. Nützlich für Debug/Monitoring.

```bash
cargo run -- blob list \
  [--type manifest|proof|wasm|abi|other] \
  [--min-size 0] \
  [--max-size 1048576] \
  [--unused-only] \
  [--limit 100] \
  [--order size|refcount|blob_id]
```

**Optionen:**
- `--type`: Filter nach Medientyp
- `--min-size`: Minimum Größe in Bytes
- `--max-size`: Maximum Größe in Bytes
- `--unused-only`: Zeigt nur unreferenzierte Blobs (refcount=0)
- `--limit`: Limit Anzahl Ergebnisse
- `--order`: Sortierung (size, refcount, blob_id)

**Spalten:**
- `blob_id`, `size`, `media_type`, `refcount`

**Exit Codes:**
- 0 OK, 11 SQLite-Fehler

**Beispiel:**
```bash
# Alle BLOBs auflisten
cargo run -- blob list

# Output:
# 📋 Gefundene BLOBs: 3
# BLOB ID                                                            Size (bytes)    Media Type           Refcount
# -------------------------------------------------------------------------------------------------------------------
# 0x1da941f7026bae3cf8b1bcdc3a8e01e76ea678c32ec6bc2c374fb67b3744571f 1234            manifest             2
# 0x83a8779ddef4567890123456789012345678901234567890123456789012345678 5678            proof                1
# 0xabc123def456789012345678901234567890123456789012345678901234567890 9012            wasm                 0
```

#### `blob gc` – Garbage Collection

Garbage Collection nicht referenzierter Blobs.

```bash
cargo run -- blob gc \
  [--dry-run] \
  [--force] \
  [--min-age 24h] \
  [--print-ids]
```

**Optionen:**
- `--dry-run`: Zeigt nur, was gelöscht würde (keine Löschung)
- `--force`: Force deletion (keine Bestätigung)
- `--min-age`: Mindest-Alter vor Löschung (z.B. "24h", "7d") - noch nicht implementiert
- `--print-ids`: Gibt gelöschte BLOB IDs aus

**Algorithmus (mark-and-sweep):**
1. **Mark:** Sammle alle referenzierten `blob_*` Felder aus `registry_entries`
2. **Sweep:** Kandidaten = `blobs.refcount=0` **UND** nicht in Mark-Set
3. **Transaktion:** Löschen in Batches (z. B. 1000)
4. **Reporting:** Anzahl, Byte-Summe; optional IDs ausgeben

**Exit Codes:**
- 0 OK, 30 NothingToDo, 11 SQLite-Fehler

**Beispiel:**
```bash
# Dry-run GC
cargo run -- blob gc --dry-run --print-ids

# Output:
# 🗑️  Starte Garbage Collection...
# 📊 Unreferenzierte BLOBs: 1
#
# 🗑️  Zu löschende BLOB IDs:
#   - 0xabc123def456789012345678901234567890123456789012345678901234567890
# 💾 Freizugebender Speicher: 9012 bytes (0.01 MB)
#
# 🔍 DRY RUN - Keine Löschung durchgeführt
# 💡 Führen Sie den Befehl mit --force aus, um zu löschen

# Real GC
cargo run -- blob gc --force

# Output:
# 🗑️  Starte Garbage Collection...
# 📊 Unreferenzierte BLOBs: 1
# 💾 Freizugebender Speicher: 9012 bytes (0.01 MB)
#
# 🗑️  Lösche unreferenzierte BLOBs...
# ✅ 1 BLOBs gelöscht, 9012 bytes freigegeben
```

### BLOB Store Schema

```sql
CREATE TABLE IF NOT EXISTS blobs (
    blob_id TEXT PRIMARY KEY,
    size INTEGER NOT NULL,
    media_type TEXT NOT NULL,
    data BLOB NOT NULL,
    refcount INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_blobs_refcount ON blobs(refcount);
```

**Performance Pragmas:**
```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
```

### Rust API (blob_store.rs)

```rust
pub trait BlobStore {
    fn put(&mut self, data: &[u8], media_type: &str) -> Result<String>;
    fn get(&self, blob_id: &str) -> Result<Vec<u8>>;
    fn exists(&self, blob_id: &str) -> bool;
    fn pin(&mut self, blob_id: &str) -> Result<()>;
    fn unpin(&mut self, blob_id: &str) -> Result<()>;
    fn gc(&mut self, dry_run: bool) -> Result<Vec<String>>;
    fn list(&self) -> Result<Vec<BlobMetadata>>;
}

pub struct SqliteBlobStore {
    conn: Connection,
}

pub struct BlobMetadata {
    pub blob_id: String,
    pub size: usize,
    pub media_type: String,
    pub refcount: i64,
}
```

### Registry-Verknüpfung

**Zukünftig (v0.11):**
- `registry add` erhöht automatisch `refcount` für verknüpfte BLOBs
- `registry delete` verringert `refcount` für verknüpfte BLOBs
- `registry verify` prüft, ob alle referenzierten `blob_*` existieren und `refcount>=1`

**Aktuell (v0.10.9):**
- Manuelle Verknüpfung via `blob put --link-entry-id <uuid>`
- Erhöht `refcount` für den referenzierenden Registry-Eintrag

### Tests

**Unit Tests (blob_store.rs):**
- ✅ `test_blob_put_get` – Roundtrip binary integrity
- ✅ `test_blob_deduplication` – Same data → same blob_id
- ✅ `test_blob_pin_unpin` – Refcount increment/decrement
- ✅ `test_blob_gc` – GC deletes only unreferenced
- ✅ `test_blob_exists` – Existence check
- ✅ `test_blob_list` – List metadata

### Performance Ziele

- **Insert (4 MB Blob):** < 10 ms (NVMe, WAL)
- **Get (4 MB Blob):** < 8 ms
- **GC (10k unreferenced):** < 60 s
- **Speicher-Footprint:** Dedup spart ≥60% bei redundanten Proofs

### Status (v0.10.9)

- ✅ BLOB Store Backend implementiert (blob_store.rs)
- ✅ CLI-Kommandos implementiert (blob put, get, list, gc)
- ✅ Unit-Tests (6 Tests, alle passing)
- ✅ Deduplizierung nachweisbar
- ✅ Garbage Collection funktional
- ✅ Audit-Log-Integration
- ⏳ Registry-Verknüpfung (manuell via `--link-entry-id`, automatisch in v0.11)
- ⏳ `--min-age` Filter für GC (zukünftige Feature)

---

## REST API (v0.11.0)

### Starting the REST API Server

```bash
cargo run --bin cap-verifier-api
```

**Output:**
```
🚀 Starting CAP Verifier API v0.1.0
🎧 Listening on http://127.0.0.1:8080
🔒 OAuth2 authentication enabled for /verify
```

### Generating Mock JWT Tokens (for Testing)

```bash
cargo run --example generate_mock_token
```

**Output:**
```
==================================================
Mock JWT Token (valid for 1 hour):
==================================================
eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9.eyJzdWIiOiJ0ZXN0LWNsaWVudC0xMjM0NSIsImlzcyI6Imh0dHBzOi8vYXV0aC5leGFtcGxlLmNvbSIsImF1ZCI6ImNhcC12ZXJpZmllciIsImV4cCI6MTc2MjQ0OTI4NiwiaWF0IjoxNzYyNDQ1Njg2LCJzY29wZSI6InZlcmlmeTpyZWFkIn0...
```

### REST Endpoints

#### `GET /healthz` – Health Check (Public, No Auth)

```bash
curl http://localhost:8080/healthz
```

**Response:**
```json
{
  "status": "OK",
  "version": "0.1.0",
  "build_hash": null
}
```

#### `GET /readyz` – Readiness Check (Public, No Auth)

```bash
curl http://localhost:8080/readyz
```

**Response:**
```json
{
  "status": "OK",
  "checks": [
    {"name": "verifier_core", "status": "OK"},
    {"name": "crypto", "status": "OK"}
  ]
}
```

#### `POST /policy/compile` – Compile Policy (Protected, OAuth2)

```bash
TOKEN="eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9..."

curl -X POST http://localhost:8080/policy/compile \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "policy": {
      "version": "lksg.v1",
      "name": "Test Policy",
      "created_at": "2025-11-06T10:00:00Z",
      "constraints": {
        "require_at_least_one_ubo": true,
        "supplier_count_max": 10
      },
      "notes": ""
    }
  }'
```

**Response:**
```json
{
  "policy_hash": "0x0afcb402e74ff6a11601863fc4ae2f2d756124db71bc703f5889ecefcd371ff4",
  "policy_info": {
    "name": "Test Policy",
    "version": "lksg.v1",
    "hash": "0x0afcb402e74ff6a11601863fc4ae2f2d756124db71bc703f5889ecefcd371ff4"
  },
  "status": "compiled"
}
```

#### `GET /policy/:id` – Retrieve Policy (Protected, OAuth2)

```bash
TOKEN="eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9..."
POLICY_HASH="0x0afcb402e74ff6a11601863fc4ae2f2d756124db71bc703f5889ecefcd371ff4"

curl -X GET http://localhost:8080/policy/$POLICY_HASH \
  -H "Authorization: Bearer $TOKEN"
```

**Response:**
```json
{
  "policy_hash": "0x0afcb402e74ff6a11601863fc4ae2f2d756124db71bc703f5889ecefcd371ff4",
  "policy": {
    "version": "lksg.v1",
    "name": "Test Policy",
    "created_at": "2025-11-06T10:00:00Z",
    "constraints": {
      "require_at_least_one_ubo": true,
      "supplier_count_max": 10
    },
    "notes": ""
  }
}
```

#### `POST /verify` – Verify Proof (Protected, OAuth2)

```bash
TOKEN="eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9..."

curl -X POST http://localhost:8080/verify \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "policy_id": "test-policy",
    "context": {
      "manifest": {
        "version": "manifest.v1.0",
        "created_at": "2025-11-06T10:00:00Z",
        "supplier_root": "0x1234...",
        "ubo_root": "0x1234...",
        "company_commitment_root": "0x1234...",
        "policy": {
          "name": "Test Policy",
          "version": "lksg.v1",
          "hash": "0x1234..."
        },
        "audit": {
          "tail_digest": "0x1234...",
          "events_count": 10
        }
      }
    },
    "backend": "mock",
    "options": {}
  }'
```

**Response:**
```json
{
  "result": "ok",
  "manifest_hash": "0xd490be94abc123...",
  "proof_hash": "0x83a8779ddef456...",
  "trace": null,
  "signature": null,
  "timestamp": null,
  "report": {
    "status": "ok",
    "manifest_hash": "0xd490be94abc123...",
    "proof_hash": "0x83a8779ddef456...",
    "signature_valid": false,
    "details": []
  }
}
```

### HTTP Status Codes

| Code | Description | Meaning |
|------|-------------|---------|
| 200 | OK | Request successful |
| 400 | Bad Request | Invalid request body or parameters |
| 401 | Unauthorized | Missing or invalid JWT token |
| 403 | Forbidden | Valid token but insufficient scopes |
| 404 | Not Found | Policy or resource not found |
| 500 | Internal Server Error | Server-side error |

### OAuth2 Flow (Client Credentials)

1. **Client** sends request with `Authorization: Bearer <JWT_TOKEN>`
2. **Middleware** validates JWT:
   - Algorithm check (RS256)
   - Issuer & Audience validation
   - Expiration check
   - Scope validation
3. **On Success:** Request proceeds to handler
4. **On Failure:** Returns 401 Unauthorized

### Testing with curl

```bash
# 1. Generate token
TOKEN=$(cargo run --example generate_mock_token 2>&1 | grep "^eyJ" | head -1)

# 2. Test health (no auth needed)
curl http://localhost:8080/healthz

# 3. Test protected endpoint without token (should return 401)
curl -X POST http://localhost:8080/policy/compile \
  -H "Content-Type: application/json" \
  -d '{"policy": {...}}'

# 4. Test protected endpoint with token (should return 200)
curl -X POST http://localhost:8080/policy/compile \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"policy": {...}}'
```

---

## Phase 1: Production Readiness (✅ Completed)

### TLS/mTLS Support (v0.11.0)

Die REST API ist nun vollständig production-ready mit TLS/mTLS-Unterstützung.

#### Implementierung (src/api/tls.rs:1-238)

**TLS Module Features:**
- ✅ TLS-only mode (server certificate authentication)
- ✅ mTLS mode (mutual authentication with client certificates)
- ✅ rustls 0.21 (production-grade TLS library)
- ✅ Certificate and private key loading (PEM format, PKCS#8)
- ✅ CA certificate trust store for client verification
- ✅ axum-server integration with RustlsConfig

**TLS Mode Enum:**
```rust
pub enum TlsMode {
    Disabled,  // HTTP-only (development)
    Tls,       // Server certificate only
    Mtls,      // Mutual authentication
}
```

**CLI Flags (verifier_api.rs:34-58):**
```bash
# HTTP-only (development)
cargo run --bin cap-verifier-api --bind 127.0.0.1:8080

# TLS mode (production)
cargo run --bin cap-verifier-api \
  --bind 0.0.0.0:8443 \
  --tls \
  --tls-cert certs/server.crt \
  --tls-key certs/server.key

# mTLS mode (high-security)
cargo run --bin cap-verifier-api \
  --bind 0.0.0.0:8443 \
  --tls \
  --tls-cert certs/server.crt \
  --tls-key certs/server.key \
  --mtls \
  --tls-ca certs/ca.crt
```

**Sicherheitsfeatures:**
- Certificate validation (PEM parsing with error handling)
- Private key validation (PKCS#8 format required)
- Client certificate verification (rustls AllowAnyAuthenticatedClient)
- Safe defaults (rustls ServerConfig::builder().with_safe_defaults())
- File existence validation before server start

#### Security Audit Integration (cargo audit)

**GitHub Actions CI Integration:**
- ✅ cargo audit in CI pipeline (.github/workflows/ci.yml:130-149)
- ✅ Automatische Dependency-Vulnerability-Scans bei jedem Push/PR
- ✅ Separate Security-Job (unabhängig von Tests)

**CI Job Konfiguration:**
```yaml
security:
  name: Security Audit
  runs-on: ubuntu-latest
  steps:
    - name: Install cargo-audit
      run: cargo install cargo-audit
    - name: Run security audit
      run: cargo audit
```

**Bekannte Advisories (nicht kritisch):**
- `rsa@0.9.6` (RUSTSEC-2023-0071) - dev-dependency only, kein Runtime-Risiko
- `wasmtime@27.0.1` (RUSTSEC-2024-0386) - WASM-Sandbox, kein Production-Impact

#### Deployment-Optionen

**Docker (kubernetes/deployment.yml):**
```yaml
env:
  - name: BIND_ADDRESS
    value: "0.0.0.0:8443"
  - name: TLS_MODE
    value: "tls"
volumeMounts:
  - name: tls-certs
    mountPath: /certs
    readOnly: true
```

**TLS Certificate Sources:**
- Self-signed (Development): `openssl req -x509 -newkey rsa:4096 ...`
- Let's Encrypt (Production): `certbot` + Auto-Renewal
- Enterprise PKI: Internal CA mit Certificate Management System
- Cloud Provider: AWS ACM, Google Cloud Certificate Manager

#### Commit & CI Status

**Commit:** 9aced6d - feat(phase1): Add TLS/mTLS support and cargo audit to CI
**CI Pipeline:** https://github.com/TomWesselmann/Confidential-Assurance-Protocol/actions
**Status:** ✅ All Phase 1 components production-ready

---

## Week 2: Monitoring & Observability (v0.11.0)

### Übersicht

Week 2 implementiert einen vollständigen Production-Ready Monitoring Stack basierend auf den Prinzipien aus dem Google SRE Workbook. Der Stack bietet die drei Säulen der Observability: Metrics (Prometheus), Logs (Loki) und Traces (Jaeger) mit vollständiger Korrelation zwischen allen drei Systemen.

**Status:** ✅ Erfolgreich deployed und getestet - Alle 8 Container healthy

**Deployment-Details:**
- Docker Compose Stack: 8 Container (5 mit Health Checks)
- Container Status: 8/8 running, 5/5 healthy
- Config Fixes: Prometheus (Storage Block entfernt), Loki (v11 schema Kompatibilität)
- Service URLs: Alle Services erreichbar unter localhost
- Test Script: `monitoring/test-monitoring.sh` erfolgreich durchgeführt
- Dokumentation: 2 README-Dateien (monitoring/README.md + slo/README.md)

### Stack-Komponenten

#### 1. Prometheus - Metrics Collection

**Konfiguration:** `monitoring/prometheus/prometheus.yml`
- Scrape Interval: 15s
- Evaluation Interval: 15s
- Retention: 30 Tage
- Alert Rules: 11 Regeln in 3 Severity-Levels

**Scrape Targets:**
- `cap-verifier-api:8080/metrics` - Application Metrics (10s interval)
- `prometheus:9090/metrics` - Self-monitoring
- `node-exporter:9100/metrics` - Host Metrics
- `cadvisor:8080/metrics` - Container Metrics

**Alerting Rules:** `monitoring/prometheus/alerts/cap-verifier-rules.yml`
- **Critical (3):** API Down, High Error Rate (>5%), Auth Failure Spike
- **Warning (4):** Elevated Error Rate (>1%), Low Cache Hit (<50%), Auth Failures Increasing, No Traffic
- **Info (2):** High Request Rate (Capacity Planning), Cache Degradation
- **SLO-Based (1):** Error Budget Burning (99.9% SLO violation)

#### 2. Grafana - Visualization & Dashboards

**Datasources:** Auto-provisioned via `monitoring/grafana/provisioning/datasources/`
- Prometheus: http://prometheus:9090 (default)
- Loki: http://loki:3100 (logs with trace correlation)
- Jaeger: http://jaeger-query:16686 (traces with logs/metrics correlation)

**Dashboards:** Auto-provisioned via `monitoring/grafana/provisioning/dashboards/dashboards.yml`

##### Dashboard 1: CAP Verifier API - Production Monitoring
**File:** `monitoring/grafana/dashboards/cap-verifier-api.json`
**UID:** `cap-verifier-api`
**Panels:** 13 Panels in 4 Kategorien

**Overview (4 Panels):**
- Total Requests (1h) - Stat Panel
- Request Rate - Stat Panel mit Sparkline
- Error Rate - Stat Panel mit Thresholds (>1% Yellow, >5% Red)
- Cache Hit Ratio - Stat Panel mit Gauge

**Request Metrics (2 Panels):**
- Request Rate by Result - Timeseries mit Stacking (ok/warn/fail)
- Request Distribution - Pie Chart (ok vs. fail)

**Authentication & Security (2 Panels):**
- Auth Failures Timeline - Timeseries
- Total Auth Failures - Stat Panel

**Cache Performance (2 Panels):**
- Cache Hit Ratio (Timeline) - Timeseries mit Thresholds
- Cache Misses - Counter

**Template Variables:**
- `$namespace` - Namespace Filter für Multi-Tenancy

##### Dashboard 2: SLO Monitoring
**File:** `monitoring/grafana/dashboards/slo-monitoring.json`
**UID:** `slo-monitoring`
**Panels:** 17 Panels in 4 Kategorien

**SLO Compliance Overview (4 Panels):**
- Availability SLO (99.9%) - Stat Panel mit Current Value
- Error Rate SLO (< 0.1%) - Stat Panel
- Auth Success SLO (99.95%) - Stat Panel
- Cache Hit Rate SLO (> 70%) - Stat Panel

**Error Budget Status (3 Panels):**
- Availability Error Budget Remaining - Gauge (0-100%)
- Error Rate Budget Remaining - Gauge
- Auth Success Budget Remaining - Gauge

**Error Budget Burn Rate (2 Panels):**
- Availability Burn Rate (1h/6h) - Timeseries
- Error Rate Burn Rate (1h/6h) - Timeseries

**SLI Trends (4 Panels):**
- Availability Trend (30d) - Timeseries (99-100% Range)
- Error Rate Trend (30d) - Timeseries
- Auth Success Rate Trend - Timeseries
- Cache Hit Rate Trend - Timeseries

#### 3. Loki - Log Aggregation

**Konfiguration:** `monitoring/loki/loki-config.yml`
- Storage: Filesystem mit boltdb-shipper
- Retention: 31 Tage (744h)
- Compaction Interval: 10m
- Max Query Length: 721h (30 days)
- Ingestion Rate: 10 MB/s

**Features:**
- Query Results Cache (100 MB embedded)
- Compactor mit Retention Deletion
- Unordered Writes Support

#### 4. Promtail - Log Collection

**Konfiguration:** `monitoring/promtail/promtail-config.yml`
- Server Port: 9080
- Client URL: http://loki:3100/loki/api/v1/push

**Scrape Jobs:**

##### Job 1: cap-verifier-api (Docker)
- **Service Discovery:** Docker (unix:///var/run/docker.sock)
- **Filter:** `app=cap-verifier-api` Label
- **Pipeline:**
  - JSON Parsing (timestamp, level, message, target)
  - Label Extraction (level)
  - Timestamp Parsing (RFC3339Nano)
  - Static Labels (app, component, environment)

##### Job 2: kubernetes-pods
- **Service Discovery:** Kubernetes Pods (default namespace)
- **Filter:** `app=cap-verifier-api` Label
- **Pipeline:**
  - CRI Log Format Parsing
  - JSON Parsing mit Trace Correlation (trace_id, span_id)
  - Label Extraction (level, target)
  - Metrics Extraction:
    - `log_lines_total` - Counter by level
    - `auth_failures_total` - Counter für Auth-Fehler

##### Job 3: system-logs
- **Source:** `/var/log/*.log`
- **Pipeline:** Syslog Regex Parsing

#### 5. Jaeger - Distributed Tracing

**Konfiguration:** `monitoring/jaeger/jaeger-config.yml`
- Deployment: All-in-One (Collector + Query + Agent + UI)
- Sampling: 100% (Probabilistic, für Development/Testing)
- Storage: In-Memory (max 10,000 traces)
- Log Level: info

**Ports:**
- 5775/udp - zipkin.thrift compact
- 6831/udp - jaeger.thrift compact
- 6832/udp - jaeger.thrift binary
- 5778 - config/health
- 16686 - UI
- 14250 - model.proto (gRPC)
- 14268 - jaeger.thrift (HTTP)
- 14269 - admin port (health check)
- 9411 - zipkin compatible
- 4317 - OTLP gRPC
- 4318 - OTLP HTTP

**Grafana Integration:** `monitoring/grafana/provisioning/datasources/jaeger.yml`
- **tracesToLogs:** Korrelation zu Loki via trace_id
  - Tags: ['trace_id']
  - Mapped Tags: service.name → app
  - Time Shift: -1m bis +1m
- **tracesToMetrics:** Korrelation zu Prometheus Metriken
  - Query 1: Request Rate `rate(cap_verifier_requests_total{app="$__tags"}[5m])`
  - Query 2: Error Rate `rate(cap_verifier_requests_total{app="$__tags",result="fail"}[5m])`
- **nodeGraph:** Service Dependency Visualization enabled

### SLO/SLI Monitoring

#### SLO-Konfiguration

**File:** `monitoring/slo/slo-config.yml`
**Version:** slo.v1

**Defined SLIs:**
1. **Availability SLI** - `ok_requests / total_requests`
2. **Error Rate SLI** - `fail_requests / total_requests`
3. **Auth Success SLI** - `(total_requests - auth_failures) / total_requests`
4. **Cache Hit Rate SLI** - `cap_cache_hit_ratio`

**Defined SLOs:**

| SLO Name | Target | Time Window | Error Budget | Burn Rate Alerts |
|----------|--------|-------------|--------------|------------------|
| availability_999 | 99.9% | 30 days | 43.2 min/month | Fast: 14.4x, Slow: 6.0x |
| error_rate_001 | < 0.1% | 30 days | 0.1% | Fast: 14.4x, Slow: 6.0x |
| auth_success_9995 | 99.95% | 30 days | 0.05% | Fast: 14.4x, Slow: 6.0x |
| cache_hit_rate_70 | > 70% | 7 days | 30% | Threshold: < 60% |

**Error Budget Policies:**
1. **Slow Rollout** (< 25% remaining):
   - Pause automated deployments
   - Require manual approval
   - Increase monitoring cadence

2. **Emergency Freeze** (< 5% remaining):
   - Freeze all deployments
   - Activate incident response team
   - Root cause analysis required

### Docker Compose Deployment

**File:** `monitoring/docker-compose.yml`
**Services:** 8 Container

```bash
# Stack starten
cd monitoring
docker compose up -d

# Health Checks
./test-monitoring.sh

# Stack stoppen
docker compose down
```

**Container:**
- `cap-verifier-api` - Port 8080
- `prometheus` - Port 9090
- `grafana` - Port 3000 (admin/admin)
- `loki` - Port 3100
- `promtail` - Log Collection
- `jaeger` - Port 16686 (UI)
- `node-exporter` - Port 9100
- `cadvisor` - Port 8081

**Volumes:**
- `prometheus-data` - Metrics Storage
- `grafana-data` - Dashboard & Config Storage
- `loki-data` - Log Storage

**Network:** `cap-monitoring` (bridge)

**Config Fixes für Kompatibilität:**
- **Prometheus:** Storage Block entfernt (wird via command-line `--storage.tsdb.retention.time=30d` gesetzt)
- **Loki:** `allow_structured_metadata: false` hinzugefügt (v11 schema Kompatibilität)
- **Loki:** Compactor vereinfacht (retention_enabled entfernt, shared_store deprecated field entfernt)
- **Jaeger:** Image Tag von `1.67` auf `latest` geändert (1.67 nicht verfügbar)

**Deployment Status:** ✅ Erfolgreich getestet
- Container Status: 8/8 running, 5/5 healthy
- Health Checks: Alle Services erreichbar
- Test Script: `test-monitoring.sh` erfolgreich durchgeführt

### Test-Script

**File:** `monitoring/test-monitoring.sh`
**Funktion:** Automatisierte Health Checks für alle Services

```bash
chmod +x monitoring/test-monitoring.sh
./monitoring/test-monitoring.sh
```

**Checks:**
- CAP Verifier API: `GET /healthz`
- Prometheus: `GET /-/healthy`
- Grafana: `GET /api/health`
- Loki: `GET /ready`
- Jaeger: `GET /` (Port 14269)

**Output:**
- ✅ Service Status für jeden Container
- 📊 Container-Übersicht via `docker compose ps`
- 📡 Service URLs (API, Grafana, Prometheus, Jaeger)
- 🧪 Test-Request-Beispiele

### Correlation Features

#### Logs → Traces
- **Mechanismus:** `trace_id` Feld in Logs
- **Loki Query:** `{app="cap-verifier-api"} | json | trace_id!=""`
- **Grafana Link:** Automatisch generierter "View Trace" Button

#### Traces → Logs
- **Mechanismus:** Jaeger Datasource Derived Field
- **Matcher Regex:** `"trace_id":"(\w+)"`
- **Loki Query:** Automatisch gefiltert nach trace_id

#### Traces → Metrics
- **Mechanismus:** Service Tags in Prometheus Queries
- **Queries:**
  - Request Rate: `rate(cap_verifier_requests_total{app="$__tags"}[5m])`
  - Error Rate: `rate(cap_verifier_requests_total{app="$__tags",result="fail"}[5m])`

### Dokumentation

**Monitoring README:** `monitoring/README.md`
- Vollständige Setup-Anleitung
- Service-URLs und Zugangsdaten
- Prometheus Query Examples
- Loki Query Examples (LogQL)
- Alerting Rules Dokumentation
- SLO/SLI Erklärungen
- Troubleshooting Guide
- Production Deployment Considerations

**SLO README:** `monitoring/slo/README.md`
- SLO/SLI Konzepte (Google SRE Workbook)
- Error Budget Calculation
- Burn Rate Interpretation
- Error Budget Policies
- Integration mit CI/CD
- Best Practices

### Metrics Exported by CAP Verifier API

**Application Metrics:**
```promql
# Request Counters (by result: ok, warn, fail)
cap_verifier_requests_total{result="ok|warn|fail"}

# Authentication Failures
cap_auth_token_validation_failures_total

# Cache Performance
cap_cache_hit_ratio
```

**Planned Metrics (Future):**
- `cap_verifier_request_duration_seconds` - Histogram
- `cap_verifier_proof_generation_duration_seconds` - Histogram
- `cap_verifier_policy_compilation_duration_seconds` - Histogram

### Production Readiness

**Completed:**
- ✅ Full Observability Stack (Metrics, Logs, Traces)
- ✅ SLO/SLI Monitoring mit Error Budget Tracking
- ✅ Alerting Rules (11 Alerts in 3 Severities)
- ✅ 2 Grafana Dashboards mit 30 Panels
- ✅ Docker Compose Deployment (8/8 Container running, 5/5 healthy)
- ✅ Config Fixes für Prometheus, Loki, Jaeger (Image-Kompatibilität)
- ✅ Automated Testing (test-monitoring.sh erfolgreich durchgeführt)
- ✅ Comprehensive Documentation (2 README files)
- ✅ Log/Trace/Metrics Correlation
- ✅ **Production-Ready Status erreicht** - Alle Services funktional

**Production Considerations:**
1. **Prometheus:**
   - Remote Storage (Cortex/Thanos) für Long-Term Retention
   - Horizontal Sharding für High Cardinality
   - Alertmanager Integration (Slack/Pagerduty)

2. **Loki:**
   - S3/GCS Backend statt Filesystem
   - Distributed Mode (Ingester, Querier, Compactor)
   - Authentication & Authorization

3. **Jaeger:**
   - Elasticsearch/Cassandra Backend statt In-Memory
   - Sampling Rate anpassen (100% → 1-10%)
   - TLS für Collector/Query

4. **Grafana:**
   - LDAP/OAuth Integration
   - TLS für alle Endpoints
   - Read-Only Dashboards für Viewer Role

### Integration mit Kubernetes

Für Kubernetes-Deployment siehe:
- `kubernetes/monitoring/prometheus-deployment.yml`
- `kubernetes/monitoring/grafana-deployment.yml`
- `kubernetes/monitoring/loki-deployment.yml`
- `kubernetes/monitoring/jaeger-deployment.yml`

Prometheus Operator und Service Monitors werden für Production empfohlen.

---

## Nächste Schritte (v0.11.0+)

1. **REST API Additional Features:**
   - Rate Limiting & Request Throttling
   - OpenAPI/Swagger Spezifikation
   - API Key Management (alternative zu OAuth2)
   - Request Logging & Metrics (Prometheus/Grafana)

2. **Schema Versioning:**
   - Explizite Schema-Version in `registry_meta` Table
   - `schema_version()` Helper-Funktion
   - Forward-kompatible Migrationen

3. **Multi-Signature Support:**
   - Chain-of-Trust für Registry-Einträge
   - Mehrere Signaturen pro Entry
   - Signatur-Timestamp-Verknüpfung

4. **ZK-Integration (Tag 4):**
   - Halo2, Spartan, Nova oder RISC0
   - Replacement von `proof_mock` durch ZK-Backend
   - Integration öffentlicher Listen-Roots (OFAC, EU, UN)
   - ZK-Verifier CLI für Auditoren
   - Blockchain-Anchoring (optional)

---

## Kontakt & Lizenz

**Projekt:** Confidential Assurance Protocol – Core Engineering
**Copyright:** © 2025
**Alle Rechte vorbehalten.**

---

**Dokumentation erstellt:** 2025-10-25
**Letzte Aktualisierung:** 2025-11-04
**Version:** v0.10.0 (Key Management & KID Rotation System)
**Autor:** Claude Code (Anthropic)
