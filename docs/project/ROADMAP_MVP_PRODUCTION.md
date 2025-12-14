# Roadmap: Minimal Local Agent → Enterprise MVP

## Über diese Roadmap

Diese Roadmap dokumentiert den aktuellen Stand des **Minimal Local Agent** (v0.12.0) und die geplanten Schritte zum Enterprise-MVP.

**Aktueller Stand:** v0.12.0 - Minimal Local Agent (11. Dezember 2025)

---

## Minimal Local Agent v0.12.0 - Status

### Verfügbare Features

| Feature | Status | Beschreibung |
|---------|--------|--------------|
| **Desktop App** | ✅ | Tauri 2.0 mit 6-Schritt Proofer Workflow |
| **CLI Tool** | ✅ | Vollständige Command-Line Funktionalität |
| **Proof Engine** | ✅ | SimplifiedZK (Mock Backend) |
| **Registry** | ✅ | JSON + SQLite Backend |
| **Key Management** | ✅ | Ed25519, KID Rotation, Attestation |
| **BLOB Store** | ✅ | Content-Addressable Storage mit GC |
| **Bundle V2** | ✅ | CAPZ Format mit Integritätsprüfung |
| **Audit Trail** | ✅ | SHA3-256 Hash-Chain (V1.0 Format) |
| **Policy Engine** | ✅ | YAML v2 mit Validation/Lint/Compile |
| **Tests** | ✅ | 538+ Tests passing (100% Success Rate) |

### Entfernte Features (seit v0.12.0)

| Feature | Status | Grund |
|---------|--------|-------|
| REST API Server | ❌ ENTFERNT | Fokus auf lokale Nutzung |
| Web UI (React) | ❌ ENTFERNT | Desktop App ersetzt WebUI |
| TLS/mTLS Support | ❌ ENTFERNT | Keine Server-Kommunikation |
| Policy Store (API) | ❌ ENTFERNT | CLI-basierte Policy-Verwaltung |
| Monitoring Stack | ❌ ENTFERNT | Prometheus, Grafana, Loki, Jaeger |
| WASM Loader | ❌ ENTFERNT | Vereinfachte Architektur |
| ZK Backend Abstraction | ❌ ENTFERNT | Nur SimplifiedZK aktiv |
| Lists Module | ❌ ENTFERNT | Sanctions, Jurisdictions |

**Rationale:** Fokus auf minimalen lokalen Agenten für einfachere Deployment- und Wartungsszenarien.

---

## Desktop App - 6-Schritt Workflow

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

## CLI Commands

```bash
# Daten vorbereiten
cap-agent prepare --suppliers data/suppliers.csv --ubos data/ubos.csv

# Manifest erstellen
cap-agent manifest build --policy policy.lksg.v1.yml

# Proof generieren
cap-agent proof build --manifest build/manifest.json --policy policy.yml

# Proof exportieren
cap-agent proof export --manifest build/manifest.json --proof build/proof.dat --out bundle.zip

# Proof verifizieren
cap-agent verify --bundle bundle.zip

# Bundle inspizieren
cap-agent inspect --bundle bundle.zip

# Policy-Befehle
cap-agent policy validate --policy policy.yml
cap-agent policy lint --policy policy.yml
cap-agent policy compile --policy policy.yml --out policy.bin
```

---

## Geplante Features (Future Versions)

### v1.0 - Enterprise MVP (2026)

| Feature | Priorität | Beschreibung |
|---------|-----------|--------------|
| SAP Adapter | Hoch | OData v4 Client für S/4HANA Integration |
| Halo2 ZK-Proofs | Mittel | Echte Zero-Knowledge Proofs (aktuell Mock) |
| REST API (optional) | Mittel | Wiedereinführung für Server-Deployments |
| WebUI (optional) | Niedrig | Wiedereinführung für Browser-Zugriff |

### v2.0 - Enterprise Scale (2026+)

| Feature | Beschreibung |
|---------|--------------|
| Multi-Tenancy | Mandantenfähigkeit |
| HSM Integration | PKCS#11 Hardware Security Modules |
| Blockchain Anchoring | Zeitstempel auf Blockchain |
| Advanced ZK | Multi-Backend (Halo2, Spartan, etc.) |
| SOC 2 Certification | Enterprise Compliance |

---

## Technische Architektur

### Aktuelle Struktur (v0.12.0)

```
agent/
├── src/
│   ├── main.rs              # CLI Entry Point
│   ├── lib.rs               # Library Root
│   ├── cli/                 # CLI Commands
│   ├── proof_engine/        # SimplifiedZK Backend
│   ├── registry/            # JSON + SQLite Storage
│   ├── keys/                # Ed25519 Key Management
│   ├── blob_store/          # Content-Addressable Storage
│   ├── bundle/              # Bundle V2 Format
│   ├── audit/               # SHA3-256 Hash-Chain
│   └── policy/              # YAML Policy Engine
│
src-tauri/
├── src/
│   ├── main.rs              # Tauri Entry Point
│   ├── commands/            # IPC Commands
│   ├── workflow/            # 6-Step Proofer State
│   └── audit/               # Audit Trail Integration
└── src/                     # React Frontend (TypeScript)
    ├── components/          # UI Components
    ├── stores/              # Zustand State
    └── api/                 # Tauri IPC Client
```

### Kryptographische Primitiven

| Algorithmus | Verwendung |
|-------------|------------|
| BLAKE3 | Merkle Roots für Commitments |
| SHA3-256 | Audit Trail Hash-Chain |
| Ed25519 | Digitale Signaturen |
| SimplifiedZK | Mock Proof Backend |

---

## Qualitätsmetriken

| Metrik | Wert | Status |
|--------|------|--------|
| Tests | 538+ passing | ✅ |
| Test Success Rate | 100% | ✅ |
| Clippy Warnings | 0 Critical/High | ✅ |
| Cargo Audit | 0 Critical | ✅ |

---

## Dokumentation

| Dokument | Beschreibung |
|----------|--------------|
| [README.md](README.md) | Projekt-Übersicht |
| [DESKTOP_APP_ARCHITEKTUR.md](DESKTOP_APP_ARCHITEKTUR.md) | Tauri 2.0 Architektur |
| [REFACTORING_GUIDE.md](REFACTORING_GUIDE.md) | CLI Refactoring (abgeschlossen) |
| [SAP_Adapter_Pilot_E2E.md](SAP_Adapter_Pilot_E2E.md) | SAP Integration (geplant) |

---

## Versionshistorie

| Version | Datum | Änderungen |
|---------|-------|------------|
| v0.12.0 | 11.12.2025 | Minimal Local Agent - Server-Features entfernt |
| v0.11.0 | 24.11.2025 | Desktop App (Tauri 2.0), Full Stack |
| v0.10.x | Nov 2025 | REST API, WebUI, Monitoring Stack |

---

*Erstellt: 17. November 2025*
*Aktualisiert: 11. Dezember 2025 - Minimal Local Agent Refactoring*
*Version: 4.0*
