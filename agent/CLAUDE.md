# LkSG Proof Agent – System Documentation

## Projektübersicht

Der **LkSG Proof Agent** ist ein Rust-basiertes CLI-Tool für die Erzeugung und Verifikation von kryptographischen Nachweisen im Kontext des deutschen Lieferkettensorgfaltspflichtengesetzes (LkSG).

**Version:** 0.7.1
**Status:** Tag 3 MVP (Proof & Verifier Layer) + Manifest Schema Validation v1.0 + Complete Verifier CLI + Standardized Proof Export v1.0 + Registry SQLite Adapter v1.0 + SQLite Edge-Case Tests – Vollständig implementiert
**Entwicklung:** Tag 1 (Commitment Engine) + Tag 2 (Policy Layer) + Tag 3 (Proof Layer) + Manifest Schema Validation + Complete Verifier CLI + Standardized Proof Export + Registry SQLite Backend

---

## Systemarchitektur

### Architekturdiagramm

```
┌─────────────────────────────────────────────────────────────┐
│                    LkSG Proof Agent v0.3.0                  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Input Layer                                                │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                │
│  │ CSV Data │  │ Policy   │  │ Keys     │                │
│  │ (S + U)  │  │ (YAML)   │  │ (Ed25519)│                │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘                │
│       │             │             │                        │
│  ┌────▼─────────────▼─────────────▼─────┐                │
│  │      Commitment Engine (Tag 1)       │                │
│  │  - BLAKE3 Merkle Roots              │                │
│  │  - SHA3-256 Hash Chain Audit        │                │
│  └────┬─────────────────────────────────┘                │
│       │                                                    │
│  ┌────▼─────────────────────────────────┐                │
│  │      Policy Layer (Tag 2)            │                │
│  │  - Policy Validation                 │                │
│  │  - Manifest Builder                  │                │
│  │  - Ed25519 Signing                   │                │
│  └────┬─────────────────────────────────┘                │
│       │                                                    │
│  ┌────▼─────────────────────────────────┐                │
│  │      Proof Engine (Tag 3)            │                │
│  │  - Proof Builder (Mock → ZK-Ready)  │                │
│  │  - Constraint Verification           │                │
│  │  - Base64 Serialization              │                │
│  └────┬─────────────────────────────────┘                │
│       │                                                    │
│  ┌────▼─────────────────────────────────┐                │
│  │      Verifier (Tag 3)                │                │
│  │  - Package Verification              │                │
│  │  - Manifest Extraction               │                │
│  │  - Audit Trail Display               │                │
│  └────┬─────────────────────────────────┘                │
│       │                                                    │
│  Output Layer                                              │
│  ┌────▼─────────────────────────────────┐                │
│  │  Proof Package (Offline-Ready)       │                │
│  │  - manifest.json                     │                │
│  │  - proof.dat (Base64)                │                │
│  │  - README.txt                        │                │
│  └──────────────────────────────────────┘                │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

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

#### `registry.rs` – Registry Store (Pluggable Backend)
- **Funktion:** Pluggable Persistence-Layer für Proof-Registry mit JSON und SQLite Backends
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
  - `registry_entries` – Proof-Einträge mit Index auf (manifest_hash, proof_hash)
- **Migration:** Vollständige Migration zwischen Backends via `registry migrate` Command
- **Output:** `build/registry.json` oder `build/registry.sqlite`

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
**Funktion:** Formale JSON-Schema-Validierung für Manifeste (manifest.v1.0)
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

**Verwendung:**
```bash
cargo run -- manifest validate --file build/manifest.json --schema docs/manifest.schema.json
```

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
**Ergebnis:** 58/58 Tests bestanden ✅ (53 Unit + 5 Integration)

**Tests pro Modul:**
- `io::tests`: 2 Tests (CSV-Parsing)
- `commitment::tests`: 3 Tests (Merkle-Roots, Determinismus)
- `audit::tests`: 3 Tests (Hash-Chain, Digest-Berechnung)
- `policy::tests`: 4 Tests (Validation, YAML-Loading, Hash-Determinismus)
- `manifest::tests`: 2 Tests (Erstellung, Proof-Update)
- `proof_mock::tests`: 3 Tests (Mock-Proof-Generation, Verifikation)
- `proof_engine::tests`: 3 Tests (Proof-Build, Verify, DAT-Serialisierung)
- `verifier::tests`: 3 Tests (Integrity-Check, Package-Summary)
- `sign::tests`: 3 Tests (Keypair-Generation, Sign & Verify)
- `test_registry_sqlite`: 5 Tests (Corruption, Migration, Duplicates, WAL, Roundtrip)

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

## Definition of Done (Tag 3)

- ✅ End-to-End-Pipeline vollständig implementiert (10 Schritte)
- ✅ Alle Artefakte in `build/proof_package/` generiert
- ✅ Proof-Pakete verifizierbar durch externes Verifier-Tool
- ✅ CI-Pipeline grün (Build + Test + Clippy)
- ✅ 58/58 Tests bestanden
- ✅ 0 Clippy-Warnings
- ✅ Reproduzierbare Hashes & Proofs (deterministisch)
- ✅ Dokumentation vollständig (CLAUDE.md)
- ✅ Alle Module mit deutschen Docstrings kommentiert

---

## Nächste Schritte (v0.8.0 – Registry Enhancements)

1. **Registry Entry Signing:**
   - Optional Ed25519-Signatur für Registry-Einträge
   - Flag: `--sign-key keys/company.ed25519`
   - Verifikation in `registry verify`

2. **Schema Versioning:**
   - Explizite Schema-Version in `registry_meta` Table
   - `schema_version()` Helper-Funktion
   - Forward-kompatible Migrationen

3. **Performance Benchmarks:**
   - JSON vs SQLite Load/Save Benchmarks
   - Criterion.rs Integration (`cargo bench`)
   - Target: ≥ 2× Improvement für ≥ 1000 Entries

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
**Letzte Aktualisierung:** 2025-10-30
**Version:** v0.7.1 (Registry SQLite Adapter + Edge-Case Tests)
**Autor:** Claude Code (Anthropic)
