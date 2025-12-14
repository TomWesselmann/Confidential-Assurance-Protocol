# CAP-Agent: Implementierter Funktionsumfang

**Version:** v0.12.2 (Production-Ready)
**Stand:** 14. Dezember 2025
**Typ:** Minimal Local Agent

---

## Übersicht

Der CAP-Agent ist ein lokaler Compliance-Agent für das Lieferkettensorgfaltspflichtengesetz (LkSG). Diese Dokumentation beschreibt den **tatsächlich implementierten** Funktionsumfang basierend auf dem aktuellen Code.

---

## Desktop App (Tauri 2.0)

### Verfügbare Tauri Commands

| Command | Modul | Beschreibung |
|---------|-------|--------------|
| `create_project` | project | Neues Projekt erstellen |
| `list_projects` | project | Projekte im Workspace auflisten |
| `get_project_status` | project | Workflow-Status abrufen |
| `read_file_content` | project | Dateiinhalt lesen |
| `list_all_projects` | project | Alle Projekte auflisten |
| `create_new_project` | project | Neues Projekt anlegen |
| `create_temp_project` | project | Temporäres Projekt erstellen |
| `create_project_in_folder` | project | Projekt in Ordner erstellen |
| `get_app_info` | settings | App-Informationen abrufen |
| `set_workspace_path` | settings | Workspace-Pfad setzen |
| `reset_workspace_path` | settings | Workspace-Pfad zurücksetzen |
| `import_csv` | import | CSV-Dateien importieren |
| `create_commitments` | commitments | Merkle Roots generieren |
| `load_policy` | policy | Policy-YAML laden |
| `build_manifest` | manifest | Manifest erstellen |
| `build_proof` | proof | Proof generieren |
| `export_bundle` | export | Bundle exportieren |
| `verify_bundle` | verify | Bundle verifizieren |
| `get_bundle_info` | verify | Bundle-Metadaten abrufen |
| `get_audit_log` | audit | Audit-Log abrufen |
| `verify_audit_chain` | audit | Audit-Chain verifizieren |
| `generate_keys` | signing | Schlüssel generieren |
| `list_keys` | signing | Schlüssel auflisten |
| `sign_project_manifest` | signing | Manifest signieren |
| `verify_manifest_signature` | signing | Signatur verifizieren |

### 6-Schritt Proofer Workflow

```
┌─────────────────────────────────────────────────────────────┐
│  1. IMPORT                                                  │
│     Lieferanten- und UBO-Daten importieren (CSV/JSON)      │
├─────────────────────────────────────────────────────────────┤
│  2. COMMITMENTS                                             │
│     BLAKE3 Merkle Roots generieren                         │
├─────────────────────────────────────────────────────────────┤
│  3. POLICY                                                  │
│     Compliance-Regeln auswählen/konfigurieren              │
├─────────────────────────────────────────────────────────────┤
│  4. MANIFEST                                                │
│     Compliance-Manifest erstellen                          │
├─────────────────────────────────────────────────────────────┤
│  5. PROOF                                                   │
│     Zero-Knowledge Proof generieren (SimplifiedZK)         │
├─────────────────────────────────────────────────────────────┤
│  6. EXPORT                                                  │
│     Proof-Package als ZIP/CAPZ exportieren                 │
└─────────────────────────────────────────────────────────────┘
```

### App-Modi

1. **Proofer Mode** - Proofs erstellen (6-Schritt Workflow)
2. **Verifier Mode** - Offline Bundle-Verifizierung
3. **Audit Mode** - Timeline mit SHA3-256 Hash-Chain

---

## CLI Tool (cap-agent)

### Implementierte Commands

#### Daten-Vorbereitung
```bash
cap-agent prepare --suppliers data/suppliers.csv --ubos data/ubos.csv
cap-agent inspect --path bundle.zip
```

#### Policy-Verwaltung
```bash
cap-agent policy validate --file policy.yml
cap-agent policy lint --file policy.yml [--strict]
cap-agent policy compile --file policy.yml --output policy.bin
cap-agent policy show --file policy.yml
```

#### Manifest-Erstellung
```bash
cap-agent manifest build --policy policy.yml [--out manifest.json]
cap-agent manifest validate --file manifest.json [--schema schema.json]
cap-agent manifest verify --manifest manifest.json --proof proof.dat \
    --registry registry.json [--timestamp ts.json] [--out report.json]
```

#### Proof-Generierung
```bash
cap-agent proof mock --policy policy.yml --manifest manifest.json
cap-agent proof build --policy policy.yml --manifest manifest.json
cap-agent proof verify --proof proof.dat --manifest manifest.json
cap-agent proof export --manifest manifest.json --proof proof.dat \
    [--timestamp ts.json] [--registry reg.json] [--report report.json] \
    --out bundle.zip [--force]
```

#### Signatur-Verwaltung
```bash
cap-agent sign keygen [--dir keys/]
cap-agent sign manifest --key private.pem --manifest-in m.json --out signed.json [--signer name]
cap-agent sign verify-manifest --pub-key public.pem --signed-in signed.json
```

#### Verifier
```bash
cap-agent verifier run --package bundle.zip
cap-agent verifier extract --package bundle.zip
cap-agent verifier audit --package bundle.zip
```

#### Audit Trail
```bash
cap-agent audit tip [--out tip.json]
cap-agent audit anchor --kind private|public --reference ref --manifest-in m.json --manifest-out m2.json
cap-agent audit timestamp --head hash [--out ts.json] [--mock] [--tsa-url url]
cap-agent audit verify-timestamp --head hash --timestamp ts.json
cap-agent audit set-private-anchor --manifest m.json --audit-tip tip [--created-at time]
cap-agent audit set-public-anchor --manifest m.json --chain chain --txid txid --digest digest [--created-at time]
cap-agent audit verify-anchor --manifest m.json [--out report.json]
cap-agent audit append --file audit.jsonl --event event --policy-id id --ir-hash hash \
    --manifest-hash hash --result pass|fail [--run-id id]
cap-agent audit verify --file audit.jsonl [--out report.json]
cap-agent audit export --file audit.jsonl [--from time] [--to time] [--policy-id id] [--out export.jsonl]
```

#### Registry
```bash
cap-agent registry add --manifest m.json --proof p.dat [--timestamp ts.json] \
    --registry reg.json --backend json|sqlite [--signing-key key] [--validate-key] [--keys-dir dir]
cap-agent registry list [--registry reg.json] --backend json|sqlite
cap-agent registry verify --manifest m.json --proof p.dat [--registry reg.json] --backend json|sqlite
cap-agent registry migrate --from json|sqlite --input in --to json|sqlite --output out
cap-agent registry inspect [--registry reg.json]
cap-agent registry backfill-kid [--registry reg.json] [--output out.json]
```

#### Key Management
```bash
cap-agent keys keygen --owner name [--algo ed25519] [--out dir] [--valid-days 365] [--comment text]
cap-agent keys list --dir keys/ [--status active|expired|revoked] [--owner name]
cap-agent keys show --dir keys/ --kid key-id
cap-agent keys rotate --dir keys/ --current kid --new kid
cap-agent keys attest --signer signer-kid --subject subject-kid --out attestation.json
cap-agent keys archive --dir keys/ --kid key-id
cap-agent keys verify-chain --dir keys/ --attestations att.json
```

#### BLOB Store
```bash
cap-agent blob put --file data.bin --type proof|manifest|timestamp --registry reg.json \
    [--link-entry-id id] [--stdin] [--out id.txt] [--no-dedup]
cap-agent blob get --id blob-id [--out data.bin] [--stdout] --registry reg.json
cap-agent blob list [--type proof] [--min-size 0] [--max-size 1000000] [--unused-only] \
    [--limit 100] [--order asc|desc] --registry reg.json
cap-agent blob gc [--dry-run] [--force] [--min-age 30d] [--print-ids] --registry reg.json
```

#### Bundle
```bash
cap-agent bundle-v2 --manifest m.json --proof p.dat [--verifier-wasm v.wasm] \
    --out bundle.capz [--zip] [--force]
cap-agent verify-bundle --bundle bundle.zip [--out report.json]
```

### Deaktivierte Commands (Minimal Local Agent)

| Command | Status | Grund |
|---------|--------|-------|
| `proof zk-build` | ❌ Deaktiviert | ZK-Backend entfernt |
| `proof zk-verify` | ❌ Deaktiviert | ZK-Backend entfernt |
| `proof bench` | ❌ Deaktiviert | ZK-Benchmark entfernt |
| `proof adapt` | ❌ Deaktiviert | Orchestrator entfernt |
| `lists *` | ❌ Deaktiviert | Lists-Modul entfernt |

---

## Module & Architektur

### Agent-Module (agent/src/)

| Modul | Beschreibung | Status |
|-------|--------------|--------|
| `audit/` | SHA3-256 Hash-Chain Audit Trail | ✅ Implementiert |
| `blob_store.rs` | Content-Addressable Storage mit GC | ✅ Implementiert |
| `bundle/` | Bundle V2 Format (CAPZ) | ✅ Implementiert |
| `cli/` | CLI Command Handler | ✅ Implementiert |
| `commitment.rs` | BLAKE3 Merkle Roots | ✅ Implementiert |
| `crypto/` | Kryptographische Primitiven | ✅ Implementiert |
| `io.rs` | I/O Utilities | ✅ Implementiert |
| `keys/` | Ed25519 Key Management | ✅ Implementiert |
| `manifest/` | Manifest-Erstellung & Validierung | ✅ Implementiert |
| `package_verifier/` | Bundle-Verifikation | ✅ Implementiert |
| `policy/` | Policy Engine (YAML v1) | ✅ Implementiert |
| `policy_v2/` | Policy Engine (YAML v2) | ✅ Implementiert |
| `proof/` | Proof-Strukturen | ✅ Implementiert |
| `proof_engine.rs` | SimplifiedZK Backend | ✅ Implementiert |
| `proof_mock.rs` | Mock Proof Backend | ✅ Implementiert |
| `providers/` | Daten-Provider | ✅ Implementiert |
| `registry/` | JSON + SQLite Storage | ✅ Implementiert |
| `sign.rs` | Ed25519 Signaturen | ✅ Implementiert |
| `verifier/` | Verifier-Logik | ✅ Implementiert |

### Tauri-Module (src-tauri/src/)

| Modul | Beschreibung | Status |
|-------|--------------|--------|
| `commands/` | Tauri IPC Commands | ✅ Implementiert |
| `audit_logger.rs` | Audit-Logging | ✅ Implementiert |
| `security.rs` | Security-Validierung | ✅ Implementiert |
| `settings.rs` | App-Einstellungen | ✅ Implementiert |
| `types.rs` | Shared Types | ✅ Implementiert |

---

## Kryptographische Primitiven

| Algorithmus | Verwendung | Status |
|-------------|------------|--------|
| BLAKE3 | Merkle Roots für Commitments | ✅ Implementiert |
| SHA3-256 | Audit Trail Hash-Chain | ✅ Implementiert |
| Ed25519 | Digitale Signaturen | ✅ Implementiert |
| SimplifiedZK | Mock Proof Backend | ✅ Implementiert |

---

## Entfernte Features

Diese Features wurden im Minimal Local Agent entfernt:

| Feature | Grund |
|---------|-------|
| REST API Server | Fokus auf lokale Nutzung |
| Web UI (React Standalone) | Desktop App ersetzt WebUI |
| TLS/mTLS Support | Keine Server-Kommunikation |
| Policy Store (API) | CLI-basierte Policy-Verwaltung |
| Monitoring Stack | Prometheus, Grafana, Loki, Jaeger |
| WASM Loader | Vereinfachte Architektur |
| ZK Backend Abstraction | Nur SimplifiedZK aktiv |
| Lists Module | Sanctions, Jurisdictions |
| Halo2 ZK-Proofs | Nur Mock-Backend implementiert |

---

## Qualitätsmetriken

| Metrik | Wert | Status |
|--------|------|--------|
| Rust Tests | 342 passing | ✅ |
| Frontend Tests | 268 passing (98.95% Coverage) | ✅ |
| Gesamt Tests | 610 passing | ✅ |
| Clippy Warnings | 0 (--deny warnings) | ✅ |
| ESLint | 0 Errors/Warnings | ✅ |
| Cargo Audit | 0 Critical | ✅ |

---

## Dokumentation

| Dokument | Beschreibung |
|----------|--------------|
| [README.md](README.md) | Projekt-Übersicht |
| [DESKTOP_APP_ARCHITEKTUR.md](DESKTOP_APP_ARCHITEKTUR.md) | Tauri 2.0 Architektur |
| [REFACTORING_GUIDE.md](REFACTORING_GUIDE.md) | CLI Refactoring (abgeschlossen) |
| [CONTRIBUTING.md](../../CONTRIBUTING.md) | Entwicklungsrichtlinien |
| [CHANGELOG.md](../../CHANGELOG.md) | Versionshistorie |

---

## Versionshistorie

| Version | Datum | Änderungen |
|---------|-------|------------|
| v0.12.2 | 14.12.2025 | Production-Ready: 268 Frontend Tests, CI/CD komplett |
| v0.12.1 | 13.12.2025 | Frontend Test Coverage erhöht |
| v0.12.0 | 11.12.2025 | Minimal Local Agent - Server-Features entfernt |
| v0.11.0 | 24.11.2025 | Desktop App (Tauri 2.0) |

---

*Erstellt: 17. November 2025*
*Aktualisiert: 14. Dezember 2025*
