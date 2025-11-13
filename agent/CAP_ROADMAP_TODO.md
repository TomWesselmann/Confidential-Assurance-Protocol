# 🎯 CAP-Roadmap 2026 – TODO-Liste & Gap-Analyse

**Basis:** CAP Agent v0.11.0 (Week 7 Complete)
**Stand:** 2025-11-10
**Ziel:** Integration-Ready & Audit-fähig (BASF / EuroDat-Pilot)

---

## 📊 Executive Summary

### Aktueller Stand (v0.11.0)

| Kategorie | Status | Abgeschlossen | Offen | Fortschritt |
|-----------|--------|---------------|-------|-------------|
| **1️⃣ Technisch** | 🟡 60% | 6/10 | 4/10 | ████████░░░░░░░░ 60% |
| **2️⃣ Sicherheit** | 🟡 40% | 3/7 | 4/7 | ██████░░░░░░░░░░ 40% |
| **3️⃣ Recht/Compliance** | 🔴 10% | 0/6 | 6/6 | ██░░░░░░░░░░░░░░ 10% |
| **4️⃣ Integration** | 🔴 0% | 0/6 | 6/6 | ░░░░░░░░░░░░░░░░ 0% |
| **5️⃣ Markt/Partner** | 🔴 0% | 0/6 | 6/6 | ░░░░░░░░░░░░░░░░ 0% |
| **GESAMT** | 🟡 30% | 9/35 | 26/35 | █████░░░░░░░░░░░ 30% |

### Priorisierung nach Critical Path

```
🔴 BLOCKER (Woche 1-2):
  1. REST API Production (TLS, mTLS, OAuth2) ⏳
  2. SAP-Adapter (OData → context.json) ⏳
  3. Docker/K8s Container ⏳
  4. Health-Checks + Monitoring ⏳

🟡 HIGH PRIORITY (Woche 3-4):
  5. Policy-Compiler (YAML → IR v1) ⏳
  6. Adaptive Proof Orchestrator ⏳
  7. HSM/TPM Key Management ⏳
  8. SBOM + Security Scan ⏳

🟢 MEDIUM PRIORITY (Woche 5-8):
  9. SAP Integration (Z-Felder, BRFplus, Fiori) ⏳
 10. Legal Review + DPIA ⏳
 11. Fraunhofer/TÜV Audit ⏳
 12. OpenAPI Spec ⏳

🔵 LOW PRIORITY (Woche 9-12):
 13. BMWK-Förderantrag ⏳
 14. Investor Outreach ⏳
 15. Pilot-Referenz ⏳
```

---

## ⚙️ 1️⃣ TECHNISCH – Funktions- und Integrationsreife

### ✅ BEREITS IMPLEMENTIERT (6/10)

1. **✅ REST-Verifier-API implementieren** (`/verify`, `/policy`)
   - Status: ✅ **FERTIG** (v0.11.0)
   - Implementiert:
     - OAuth2 JWT Middleware (RS256)
     - POST /verify Endpoint
     - POST /policy/compile Endpoint
     - GET /policy/:id Endpoint
     - GET /healthz, /readyz
   - Tests: 5/5 ✅
   - Dateien: `src/api/*.rs`, `src/bin/verifier_api.rs`

2. **✅ Structured Audit-Log / Hash-Chain**
   - Status: ✅ **FERTIG** (Week 7, Track A)
   - Implementiert:
     - SHA3-256 Hash-Chain
     - JSONL Storage
     - Tamper Detection
     - Event Export (Time/Policy Filters)
     - CLI Commands (append, verify, export)
   - Tests: 21/21 ✅
   - DoD: 7/7 ✅ (Performance 83-86% besser als Ziel)
   - Dateien: `src/audit/hash_chain.rs`

3. **✅ Key-Management-Modul** (Basis)
   - Status: ✅ **FERTIG** (Week 6)
   - Implementiert:
     - KID Derivation (BLAKE3)
     - Key Rotation
     - Chain-of-Trust (Attestations)
     - Key Status (active/retired/revoked)
     - CLI Commands (keygen, rotate, attest, verify-chain)
   - Tests: 12/12 ✅
   - Dateien: `src/keys.rs`
   - **FEHLT:** HSM/TPM Integration ⏳

4. **✅ CLI-Refactoring + Unit-Tests**
   - Status: ✅ **FERTIG**
   - CLI Commands: 45+ implementiert
   - Tests: 146 Tests (145 passing)
   - Binary: `src/main.rs` (4215 Zeilen)

5. **✅ Health-Checks** (Basis)
   - Status: ✅ **FERTIG** (v0.11.0)
   - Implementiert: GET /healthz, GET /readyz
   - **FEHLT:** Prometheus Metrics ⏳

6. **✅ OpenAPI-Spec** (Basis)
   - Status: 🟡 **TEILWEISE**
   - Implementiert: REST Endpoints dokumentiert
   - **FEHLT:** OpenAPI 3.0 JSON/YAML Spec ⏳

---

### ⏳ NOCH ZU IMPLEMENTIEREN (4/10)

#### 🔴 BLOCKER 1: Docker-/K8s-Container erstellen

**Status:** ⏳ **OFFEN**
**Aufwand:** 1 Woche
**Priorität:** 🔴 BLOCKER (Woche 1)

**Anforderungen:**
- [ ] Dockerfile (Multi-Stage Build)
- [ ] Docker Compose (API + Registry + BLOB Store)
- [ ] Kubernetes Manifests (Deployment, Service, ConfigMap, Secret)
- [ ] Helm Chart (optional)
- [ ] Health Probes (liveness, readiness)
- [ ] Resource Limits (CPU, Memory)
- [ ] Environment Variables (Config)
- [ ] Volume Mounts (Keys, Registry, BLOBs)

**Deliverables:**
- `Dockerfile` (cap-agent + cap-verifier-api)
- `docker-compose.yml`
- `k8s/` Verzeichnis (manifests)
- `README_DEPLOYMENT.md`

**Tests:**
- [ ] Docker Build erfolgreich
- [ ] Docker Run + Health Check
- [ ] K8s Deploy + Rollout
- [ ] Smoke Test (API Call)

**Abhängigkeiten:**
- Benötigt: TLS/mTLS (für Production) ⏳

---

#### 🔴 BLOCKER 2: SAP-Adapter (OData/CDS → context.json)

**Status:** ⏳ **OFFEN**
**Aufwand:** 2-3 Wochen
**Priorität:** 🔴 BLOCKER (Woche 1-2)

**Anforderungen:**
- [ ] OData Client (SAP Gateway / CDS Views)
- [ ] Supplier Data Mapper (SAP → CAP Context)
- [ ] UBO Data Mapper
- [ ] Policy Mapper (LkSG Rules → CAP Policy)
- [ ] BLAKE3 Data Hashing (DSGVO-safe)
- [ ] CLI Command (`sap-adapter fetch`)
- [ ] Error Handling (Timeout, Auth, Schema Mismatch)
- [ ] Configuration (SAP URL, Credentials, Entities)

**Deliverables:**
- `src/sap_adapter.rs` (300-400 Zeilen)
- `config/sap_adapter.yml` (Config-Template)
- CLI Integration in `src/main.rs`
- `examples/sap_context.json` (Sample Output)

**Tests:**
- [ ] Unit Tests (Mapper Logic)
- [ ] Integration Test (Mock OData)
- [ ] E2E Test (Real SAP Gateway)

**Abhängigkeiten:**
- Benötigt: SAP System Access (Dev/QA)
- Benötigt: OData Service URL + Credentials

**Dateien:**
```
agent/
├── src/
│   └── sap_adapter.rs          # OData Client + Mapper
├── config/
│   └── sap_adapter.yml         # SAP Connection Config
└── examples/
    └── sap_context.json        # Sample Output
```

---

#### 🟡 HIGH PRIORITY 3: Policy-Compiler (YAML → IR v1)

**Status:** ⏳ **OFFEN**
**Aufwand:** 3 Wochen
**Priorität:** 🟡 HIGH (Woche 3-4)

**Anforderungen:**
- [ ] IR v1 Schema Definition (JSON Schema)
- [ ] YAML → IR Compiler
- [ ] IR Validator
- [ ] IR → Constraint Mapper (für ZK)
- [ ] IR Serialization (JSON)
- [ ] CLI Command (`policy compile --ir`)
- [ ] Backward Compatibility (YAML Policy weiterhin nutzbar)

**IR v1 Schema (Draft):**
```yaml
ir_version: "1.0"
policy_id: "lksg.v1"
rules:
  - rule_id: "r1_ubo_required"
    condition: "count(ubos) >= 1"
    severity: "error"
    message: "At least one UBO required"
  - rule_id: "r2_supplier_limit"
    condition: "count(suppliers) <= 10"
    severity: "warning"
    message: "Max 10 suppliers"
constraints:
  - type: "zk_range"
    field: "ubo_count"
    min: 1
    max: 100
  - type: "zk_membership"
    field: "jurisdiction"
    set: ["DE", "US", "FR", "GB"]
```

**Deliverables:**
- `src/policy/ir.rs` (IR Schema + Compiler)
- `docs/ir_v1.schema.json` (JSON Schema)
- CLI Integration
- `examples/policy.ir.v1.json`

**Tests:**
- [ ] YAML → IR Roundtrip
- [ ] IR Validation
- [ ] Backward Compatibility

**Abhängigkeiten:**
- Benötigt für: Adaptive Proof Orchestrator

---

#### 🟡 HIGH PRIORITY 4: Adaptive Proof Orchestrator (`proof adapt`)

**Status:** ⏳ **OFFEN**
**Aufwand:** 3-4 Wochen
**Priorität:** 🟡 HIGH (Woche 4-5)

**Anforderungen:**
- [ ] Risk Scoring Engine
- [ ] Rule Activation Logic (IR-basiert)
- [ ] Adaptive Constraint Selection
- [ ] Dynamic ZK Circuit Generation
- [ ] CLI Command (`proof adapt --risk-level <level>`)
- [ ] Integration mit ZK Backend

**Risk Scoring (Draft):**
```rust
enum RiskLevel {
    Low,    // Basic checks only
    Medium, // Standard checks
    High,   // Full checks + sanctions
    Critical, // Full checks + sanctions + UBO verification
}

fn compute_risk(context: &Context) -> RiskLevel {
    let score = 0;
    if context.jurisdictions.contains("IR") { score += 30; }
    if context.supplier_count > 5 { score += 10; }
    if context.ubo_count < 1 { score += 20; }
    match score {
        0..=10 => RiskLevel::Low,
        11..=30 => RiskLevel::Medium,
        31..=50 => RiskLevel::High,
        _ => RiskLevel::Critical,
    }
}
```

**Deliverables:**
- `src/orchestrator/adaptive.rs` (400-500 Zeilen)
- CLI Integration
- Risk Scoring Config (`config/risk_scoring.yml`)

**Tests:**
- [ ] Risk Scoring Logic
- [ ] Rule Activation
- [ ] Adaptive Proof Generation

**Abhängigkeiten:**
- Benötigt: Policy-Compiler (IR v1) ⏳
- Benötigt: ZK Backend Integration ⏳

---

### 🔵 TECHNICAL DEBT

#### 7. Health-Checks + Monitoring (Prometheus)

**Status:** 🟡 **TEILWEISE** (Basis vorhanden)
**Aufwand:** 1 Woche
**Priorität:** 🔴 BLOCKER (Woche 1)

**Bereits implementiert:**
- ✅ GET /healthz (Status + Version)
- ✅ GET /readyz (Dependency Checks)

**Noch zu implementieren:**
- [ ] Prometheus Metrics Endpoint (`/metrics`)
- [ ] Custom Metrics (Request Count, Latency, Error Rate)
- [ ] Grafana Dashboard (JSON)
- [ ] Alerting Rules (Prometheus)
- [ ] Log Aggregation (Loki/ELK)

**Deliverables:**
- Prometheus Integration (`prometheus` crate)
- Grafana Dashboard (`grafana/cap_dashboard.json`)
- Alerting Rules (`prometheus/alerts.yml`)

---

#### 8. OpenAPI-Spec + CLI-Docs

**Status:** 🟡 **TEILWEISE**
**Aufwand:** 3 Tage
**Priorität:** 🟢 MEDIUM (Woche 5)

**Bereits dokumentiert:**
- ✅ REST Endpoints in CLAUDE.md
- ✅ CLI Commands in CLAUDE.md

**Noch zu erstellen:**
- [ ] OpenAPI 3.0 Spec (`docs/openapi.yaml`)
- [ ] Swagger UI Integration (`/swagger`)
- [ ] CLI Command Reference (Markdown)
- [ ] Integration Guide (SAP)

**Deliverables:**
- `docs/openapi.yaml`
- `docs/CLI_REFERENCE.md`
- `docs/SAP_INTEGRATION_GUIDE.md`

---

## 🔐 2️⃣ SICHERHEIT – IT- & Datenschutz-Freigabe

### ✅ BEREITS IMPLEMENTIERT (3/7)

1. **✅ Logging ohne PII**
   - Status: ✅ **FERTIG**
   - Implementiert: Keine PII in Audit-Logs (nur Hashes, IDs)
   - Track A: Privacy-compliant (DoD 7/7)

2. **✅ Data-Hashing (BLAKE3)** (Basis)
   - Status: ✅ **FERTIG**
   - Implementiert:
     - BLAKE3 für Commitments
     - SHA3-256 für Audit-Chain
     - Crypto Namespace (`src/crypto/mod.rs`)
   - **FEHLT:** SAP-Adapter Integration ⏳

3. **✅ Key-Rotation CLI** (Basis)
   - Status: ✅ **FERTIG**
   - CLI: `keys rotate --current <old> --new <new>`
   - **FEHLT:** HSM Integration ⏳

---

### ⏳ NOCH ZU IMPLEMENTIEREN (4/7)

#### 🔴 BLOCKER 9: mTLS + OAuth2 in REST-API

**Status:** 🟡 **TEILWEISE** (OAuth2 JWT vorhanden)
**Aufwand:** 1 Woche
**Priorität:** 🔴 BLOCKER (Woche 1)

**Bereits implementiert:**
- ✅ OAuth2 JWT RS256 Validation
- ✅ Bearer Token Middleware
- ✅ Scope-based Authorization

**Noch zu implementieren:**
- [ ] TLS Configuration (Port 8443)
- [ ] mTLS (Client Certificate Validation)
- [ ] Certificate Management (CA, CRL)
- [ ] Auto-Renewal (Let's Encrypt / Cert-Manager)

**Deliverables:**
- TLS Integration (`rustls` crate)
- Config: `config/tls.yml`
- Dokumentation: `docs/TLS_SETUP.md`

**Tests:**
- [ ] TLS Handshake
- [ ] mTLS Client Auth
- [ ] Certificate Expiry Handling

---

#### 🟡 HIGH PRIORITY 10: SBOM (Software Bill of Materials)

**Status:** ⏳ **OFFEN**
**Aufwand:** 1 Tag
**Priorität:** 🟡 HIGH (Woche 2)

**Anforderungen:**
- [ ] SBOM Generation (CycloneDX / SPDX)
- [ ] Dependency Audit (`cargo audit`)
- [ ] Vulnerability Scan (Dependabot / Snyk)
- [ ] License Compliance Check
- [ ] CI Integration (GitHub Actions)

**Tools:**
```bash
# SBOM generieren
cargo install cargo-cyclonedx
cargo cyclonedx --format json > sbom.json

# Vulnerability Scan
cargo audit
```

**Deliverables:**
- `sbom.json` (CycloneDX)
- `LICENSES.md` (Dependency Licenses)
- CI Job (`.github/workflows/security.yml`)

---

#### 🟡 HIGH PRIORITY 11: HSM/TPM Key Management

**Status:** ⏳ **OFFEN**
**Aufwand:** 2 Wochen
**Priorität:** 🟡 HIGH (Woche 3-4)

**Anforderungen:**
- [ ] PKCS#11 Client (HSM Integration)
- [ ] TPM 2.0 Support (via `tpm2-tss`)
- [ ] HSM Key Generation
- [ ] HSM Signing (Ed25519)
- [ ] Key Rotation mit HSM
- [ ] Fallback auf File-based Keys
- [ ] CLI Commands (`keys hsm-init`, `keys hsm-sign`)

**Deliverables:**
- `src/keys/hsm.rs` (PKCS#11 Integration)
- `src/keys/tpm.rs` (TPM Integration)
- Config: `config/hsm.yml`
- Dokumentation: `docs/HSM_SETUP.md`

**Tests:**
- [ ] HSM Mock Tests
- [ ] TPM Simulator Tests
- [ ] Key Rotation Tests

**Abhängigkeiten:**
- Benötigt: HSM Hardware / SoftHSM
- Benötigt: TPM 2.0 Device

---

#### 🟡 HIGH PRIORITY 12: Pen-Test / Security-Scan (SAST/DAST)

**Status:** ⏳ **OFFEN**
**Aufwand:** 2 Wochen
**Priorität:** 🟡 HIGH (Woche 4-5)

**Anforderungen:**
- [ ] SAST (Static Analysis): Clippy, cargo-audit
- [ ] DAST (Dynamic Analysis): OWASP ZAP
- [ ] Fuzzing (cargo-fuzz)
- [ ] Pen-Test Report
- [ ] Remediation Plan

**Tools:**
```bash
# SAST
cargo clippy -- -D warnings
cargo audit

# Fuzzing
cargo install cargo-fuzz
cargo fuzz run fuzz_target

# DAST
docker run -t owasp/zap2docker-stable zap-baseline.py \
  -t http://localhost:8080
```

**Deliverables:**
- Pen-Test Report (`docs/PEN_TEST_REPORT.md`)
- Remediation Tracking (GitHub Issues)

---

#### 🟢 MEDIUM PRIORITY 13: Build-Hash Verification

**Status:** ⏳ **OFFEN**
**Aufwand:** 2 Tage
**Priorität:** 🟢 MEDIUM (Woche 5)

**Anforderungen:**
- [ ] Reproducible Builds (cargo-build-reproducible)
- [ ] Build Hash in Binary (`--build-hash`)
- [ ] Verification CLI (`verify-build --hash <hash>`)
- [ ] CI Integration (Build Hash in Artifact Name)

**Deliverables:**
- Build Hash Integration (`build.rs`)
- CI Job (`.github/workflows/build.yml`)

---

## ⚖️ 3️⃣ RECHT & COMPLIANCE – Audit- und Beweisfähigkeit

**Status:** 🔴 **ALLE OFFEN** (0/6)
**Verantwortlich:** Legal Team + Externe Partner

### ⏳ NOCH ZU IMPLEMENTIEREN (6/6)

#### 14. Legal Review der Policies (LkSG, CSRD, DSGVO)

**Status:** ⏳ **OFFEN**
**Aufwand:** 2 Wochen (mit Kanzlei)
**Priorität:** 🟢 MEDIUM (Woche 6-7)

**Anforderungen:**
- [ ] Rechtskanzlei beauftragen (LkSG-Spezialist)
- [ ] Policy Review (examples/policy.lksg.v1.yml)
- [ ] CSRD Compliance Check
- [ ] DSGVO Compliance Check
- [ ] Formale Gültigkeit bestätigen
- [ ] Anpassungsempfehlungen umsetzen

**Deliverables:**
- Legal Review Report
- Angepasste Policy Templates
- `docs/LEGAL_COMPLIANCE.md`

**Abhängigkeiten:**
- Benötigt: Budget für Rechtskanzlei (~€5k-10k)

---

#### 15. DPIA / TOMs nach DSGVO

**Status:** ⏳ **OFFEN**
**Aufwand:** 1 Woche
**Priorität:** 🟢 MEDIUM (Woche 7)

**Anforderungen:**
- [ ] Datenschutz-Folgenabschätzung (DPIA)
- [ ] Technische und Organisatorische Maßnahmen (TOMs)
- [ ] Verarbeitungsverzeichnis (Art. 30 DSGVO)
- [ ] Auftragsverarbeitungs-Vertrag (AVV) Template
- [ ] Dokumentation für Auditoren

**Deliverables:**
- `docs/DPIA.md`
- `docs/TOMs.md`
- AVV Template (`docs/AVV_TEMPLATE.md`)

**Abhängigkeiten:**
- Benötigt: Datenschutzbeauftragter (intern/extern)

---

#### 16. Fraunhofer / TÜV-Audit (Security Review)

**Status:** ⏳ **OFFEN**
**Aufwand:** 4-6 Wochen
**Priorität:** 🟢 MEDIUM (Woche 8-12)

**Anforderungen:**
- [ ] Audit-Partner auswählen (Fraunhofer AISEC / TÜV)
- [ ] Audit-Scope definieren
- [ ] Code Review
- [ ] Security Architecture Review
- [ ] Pen-Test (extern)
- [ ] Zertifikat / Testat erhalten

**Deliverables:**
- Audit Report
- Zertifikat (PDF)
- Remediation Plan

**Abhängigkeiten:**
- Benötigt: Budget (~€20k-50k)
- Benötigt: Code Freeze (Stable Release)

---

#### 17. Rechtsgutachten „Proof als Beweismittel §371a ZPO"

**Status:** ⏳ **OFFEN**
**Aufwand:** 2 Wochen
**Priorität:** 🟢 MEDIUM (Woche 7-8)

**Anforderungen:**
- [ ] Rechtskanzlei beauftragen (Prozessrecht-Spezialist)
- [ ] Gutachten erstellen lassen
- [ ] Juristische Anerkennung prüfen
- [ ] Anforderungen für Gerichtsfestigkeit

**Deliverables:**
- Rechtsgutachten (PDF)
- `docs/LEGAL_PROOF_VALIDITY.md`

**Abhängigkeiten:**
- Benötigt: Budget (~€5k-10k)

---

#### 18. CAP Markenanmeldung (DPMA/EUIPO)

**Status:** ⏳ **OFFEN**
**Aufwand:** 2 Tage (Antrag) + 6-12 Monate (Eintragung)
**Priorität:** 🔵 LOW (Woche 10)

**Anforderungen:**
- [ ] Markenrecherche (Collision Check)
- [ ] DPMA Anmeldung (Deutschland)
- [ ] EUIPO Anmeldung (EU)
- [ ] Nizza-Klassen definieren (09, 42, 45)

**Deliverables:**
- Markenanmeldungs-Bestätigung
- Eintragungsurkunde (nach 6-12 Monaten)

**Kosten:**
- DPMA: ~€300
- EUIPO: ~€850

---

#### 19. Lizenzmodell (EUPL + Commercial)

**Status:** ⏳ **OFFEN**
**Aufwand:** 3 Tage
**Priorität:** 🟢 MEDIUM (Woche 8)

**Anforderungen:**
- [ ] Dual-License Modell definieren
- [ ] Open Source: EUPL 1.2 (oder Apache 2.0)
- [ ] Commercial: Proprietary License
- [ ] CLA (Contributor License Agreement)
- [ ] LICENSE.md erstellen

**Deliverables:**
- `LICENSE.md` (Dual-License)
- `CLA.md` (Contributor Agreement)
- Commercial License Template

---

## 🧩 4️⃣ INTEGRATION – Pilot & SAP-Konnektivität

**Status:** 🔴 **ALLE OFFEN** (0/6)
**Verantwortlich:** SAP Team + Integration Developers

### ⏳ NOCH ZU IMPLEMENTIEREN (6/6)

#### 20. Z-Felder in SAP (Supplier-Status)

**Status:** ⏳ **OFFEN**
**Aufwand:** 2 Tage
**Priorität:** 🟡 HIGH (Woche 3)

**Anforderungen:**
- [ ] Z-Feld Definition (LFA1/LFB1 Customizing)
- [ ] Feld: `Z_CAP_PROOF_STATUS` (CHAR 10)
- [ ] Werte: "OK", "WARN", "FAIL", "PENDING"
- [ ] Feld: `Z_CAP_PROOF_HASH` (CHAR 64)
- [ ] Feld: `Z_CAP_LAST_CHECK` (TIMESTAMP)
- [ ] ABAP Update Logic (nach Proof-Verifikation)

**Deliverables:**
- SAP Transport (Z-Felder)
- ABAP Update Function Module
- Dokumentation (`docs/SAP_Z_FIELDS.md`)

**Abhängigkeiten:**
- Benötigt: SAP System Access (Customizing)

---

#### 21. BRFplus-Workflow (bei FAIL)

**Status:** ⏳ **OFFEN**
**Aufwand:** 3 Tage
**Priorität:** 🟡 HIGH (Woche 3)

**Anforderungen:**
- [ ] BRFplus Rule: "CAP Proof Failed"
- [ ] Trigger: Workflow bei `Z_CAP_PROOF_STATUS = "FAIL"`
- [ ] Workflow: Task an Einkauf (Nachprüfung)
- [ ] Eskalation: Task an Compliance (nach 3 Tagen)
- [ ] SAP Business Workflow Integration

**Deliverables:**
- BRFplus Rule Export
- Workflow Definition (SAP BW)
- Dokumentation (`docs/SAP_WORKFLOW.md`)

**Abhängigkeiten:**
- Benötigt: Z-Felder implementiert
- Benötigt: SAP BW License

---

#### 22. Fiori-App (Proof Upload + Viewer)

**Status:** ⏳ **OFFEN**
**Aufwand:** 1 Woche
**Priorität:** 🟢 MEDIUM (Woche 6)

**Anforderungen:**
- [ ] Fiori App: "CAP Proof Manager"
- [ ] Features:
  - [ ] Proof Upload (manifest.json + proof.dat)
  - [ ] Proof Viewer (JSON Pretty Print)
  - [ ] Verification Status
  - [ ] Supplier List (mit Status)
  - [ ] Download Proof Package
- [ ] OData Service (Backend)
- [ ] Deployment auf BTP / On-Prem Gateway

**Deliverables:**
- Fiori App (`fiori/cap_proof_manager/`)
- OData Service (ABAP)
- Deployment Guide

**Abhängigkeiten:**
- Benötigt: SAP Fiori License
- Benötigt: BTP / Gateway Access

---

#### 23. CPI-Flow (BTP OData → CAP → OData)

**Status:** ⏳ **OFFEN**
**Aufwand:** 1 Woche
**Priorität:** 🟢 MEDIUM (Woche 7)

**Anforderungen:**
- [ ] CPI Integration Flow: "SAP → CAP → SAP"
- [ ] Input: OData (Supplier Data)
- [ ] Transformation: JSON → context.json
- [ ] HTTP Call: POST /verify (CAP API)
- [ ] Output: OData (Proof Result)
- [ ] Error Handling (Retry, Dead Letter Queue)

**Deliverables:**
- CPI Integration Flow (IFlow Export)
- Deployment Instructions
- Dokumentation (`docs/BTP_CPI_INTEGRATION.md`)

**Abhängigkeiten:**
- Benötigt: SAP BTP License (CPI)
- Benötigt: CAP API deployed

---

#### 24. SAP Integration Guide

**Status:** ⏳ **OFFEN**
**Aufwand:** 3 Tage
**Priorität:** 🟢 MEDIUM (Woche 7)

**Anforderungen:**
- [ ] Schritt-für-Schritt Anleitung
- [ ] Architektur-Diagramme
- [ ] Code-Beispiele (ABAP, OData, CPI)
- [ ] Troubleshooting Guide
- [ ] FAQ

**Deliverables:**
- `docs/SAP_INTEGRATION_GUIDE.md` (50+ Seiten)
- Architektur-Diagramme (draw.io / PlantUML)

---

#### 25. Proof-Attachment via DMS/ArchiveLink

**Status:** ⏳ **OFFEN**
**Aufwand:** 3 Tage
**Priorität:** 🟢 MEDIUM (Woche 8)

**Anforderungen:**
- [ ] ArchiveLink Integration (SAP DMS)
- [ ] Business Object: LFA1 (Supplier)
- [ ] Attachment Type: "CAP_PROOF"
- [ ] ABAP Function Module (Proof Upload)
- [ ] GOS (Generic Object Services) Integration
- [ ] Audit-Trail (Anzeige in SAP)

**Deliverables:**
- ABAP Function Module
- ArchiveLink Customizing
- Dokumentation (`docs/SAP_ARCHIVELINK.md`)

**Abhängigkeiten:**
- Benötigt: SAP DMS / Content Server

---

## 💼 5️⃣ MARKT & PARTNER – Business / Pilot / Förderung

**Status:** 🔴 **ALLE OFFEN** (0/6)
**Verantwortlich:** Business Development Team

### ⏳ NOCH ZU IMPLEMENTIEREN (6/6)

#### 26. 1-Seiten-Pilotangebot (BASF/Bosch)

**Status:** ⏳ **OFFEN**
**Aufwand:** 3 Tage
**Priorität:** 🟡 HIGH (Woche 4)

**Anforderungen:**
- [ ] Pilotangebot erstellen (1 Seite)
- [ ] Value Proposition
- [ ] Pilot Scope (Dauer, Umfang)
- [ ] Pricing (kostenlos / Symbolpreis)
- [ ] Success Criteria
- [ ] Kontaktpersonen identifizieren

**Deliverables:**
- Pilotangebot (PDF)
- Ansprechpartner-Liste (BASF, Bosch)

---

#### 27. BMWK-Förderantrag „Datensouveränität"

**Status:** ⏳ **OFFEN**
**Aufwand:** 2 Wochen
**Priorität:** 🟢 MEDIUM (Woche 9-10)

**Anforderungen:**
- [ ] Förder-Programm identifizieren
- [ ] Antrag erstellen (BMWK)
- [ ] Projektplan (Meilensteine)
- [ ] Budget-Kalkulation
- [ ] Partner-Konsortium (optional)

**Deliverables:**
- Förderantrag (PDF)
- Projektplan (Gantt)
- Budget-Kalkulation (Excel)

**Fördersumme:**
- Ziel: €200k-500k (50% Zuschuss)

---

#### 28. Fraunhofer/TÜV Kooperations-MoU

**Status:** ⏳ **OFFEN**
**Aufwand:** 2 Wochen
**Priorität:** 🟢 MEDIUM (Woche 10)

**Anforderungen:**
- [ ] Kooperationspartner ansprechen
- [ ] MoU (Memorandum of Understanding) erstellen
- [ ] Scope definieren (Audit, Zertifizierung)
- [ ] Unterschriften einholen

**Deliverables:**
- MoU (PDF, unterschrieben)

---

#### 29. Investor-Outreach (HTGF, Planet A)

**Status:** ⏳ **OFFEN**
**Aufwand:** 3 Wochen
**Priorität:** 🔵 LOW (Woche 11-12)

**Anforderungen:**
- [ ] Investor-Liste erstellen
- [ ] Pitch Deck erstellen
- [ ] Teaser-Deck (5 Slides)
- [ ] Erstgespräche vereinbaren
- [ ] Due Diligence vorbereiten

**Deliverables:**
- Pitch Deck (PDF)
- Teaser-Deck (PDF)
- Investor-Liste (Excel)

**Ziel:**
- Pre-Seed: €500k-1M

---

#### 30. CAP Readiness Deck (Pitch)

**Status:** ⏳ **OFFEN**
**Aufwand:** 1 Woche
**Priorität:** 🟡 HIGH (Woche 4)

**Anforderungen:**
- [ ] Pitch Deck (15-20 Slides)
- [ ] Slides:
  - Problem Statement
  - Solution (CAP)
  - Market Size
  - Business Model
  - Technology Stack
  - Traction (Pilot)
  - Team
  - Ask (Funding / Partnership)
- [ ] Demo Video (2-3 Min)

**Deliverables:**
- Pitch Deck (PDF)
- Demo Video (MP4)

---

#### 31. Referenz-Pilot (Mittelstand)

**Status:** ⏳ **OFFEN**
**Aufwand:** 2 Monate
**Priorität:** 🟢 MEDIUM (Woche 8-16)

**Anforderungen:**
- [ ] Mittelstands-Unternehmen identifizieren
- [ ] Pilot-Scope definieren (10-50 Supplier)
- [ ] Pilotvertrag abschließen
- [ ] CAP Integration
- [ ] Go-Live
- [ ] Success Story dokumentieren

**Deliverables:**
- Pilotvertrag
- Success Story (Case Study)
- Referenz-Zitat

**Ziel:**
- Schneller Erfolg vor BASF-Pilot
- Referenz für weitere Akquise

---

## 📅 Roadmap-Umsetzungsplan (3 Monate)

### Monat 1 (Woche 1-4): Technische Basis + Sicherheit

**Woche 1-2: BLOCKER**
- [ ] REST API Production (TLS, mTLS) - 1 Woche
- [ ] Docker/K8s Container - 1 Woche
- [ ] SAP-Adapter (OData → context.json) - 2 Wochen ⏩
- [ ] Health-Checks + Monitoring (Prometheus) - 1 Woche
- [ ] SBOM - 1 Tag

**Woche 3-4: HIGH PRIORITY**
- [ ] Policy-Compiler (YAML → IR v1) - 3 Wochen ⏩
- [ ] HSM/TPM Key Management - 2 Wochen ⏩
- [ ] Z-Felder + BRFplus (SAP) - 1 Woche
- [ ] Pilotangebot (BASF) - 3 Tage
- [ ] CAP Readiness Deck - 1 Woche

**Deliverables Monat 1:**
- ✅ Production-ready REST API (TLS, OAuth2, Monitoring)
- ✅ Docker/K8s Deployment
- ✅ SAP-Adapter (OData Integration)
- ✅ SBOM + Security Baseline
- ✅ Pilotangebot ready

---

### Monat 2 (Woche 5-8): Recht + Auditfähigkeit

**Woche 5-6: COMPLIANCE**
- [ ] Adaptive Proof Orchestrator - 3-4 Wochen ⏩
- [ ] Pen-Test / Security-Scan - 2 Wochen ⏩
- [ ] Legal Review (Policies) - 2 Wochen ⏩
- [ ] DPIA / TOMs - 1 Woche
- [ ] OpenAPI Spec + CLI Docs - 3 Tage
- [ ] Fiori-App (Proof Viewer) - 1 Woche

**Woche 7-8: AUDIT**
- [ ] Rechtsgutachten (§371a ZPO) - 2 Wochen ⏩
- [ ] CPI-Flow (BTP Integration) - 1 Woche
- [ ] SAP Integration Guide - 3 Tage
- [ ] Proof-Attachment (DMS) - 3 Tage
- [ ] Lizenzmodell - 3 Tage
- [ ] Referenz-Pilot Start - Monat 2+3 ⏩

**Deliverables Monat 2:**
- ✅ Adaptive Proof Orchestrator
- ✅ Security Audit + Pen-Test Report
- ✅ Legal Compliance (DSGVO, LkSG)
- ✅ SAP Integration Complete
- ✅ Referenz-Pilot gestartet

---

### Monat 3 (Woche 9-12): Integration + Pilotstart

**Woche 9-10: FÖRDERUNG**
- [ ] Fraunhofer/TÜV Audit - 4-6 Wochen ⏩
- [ ] BMWK-Förderantrag - 2 Wochen ⏩
- [ ] Fraunhofer/TÜV MoU - 2 Wochen ⏩
- [ ] CAP Markenanmeldung - 2 Tage
- [ ] Build-Hash Verification - 2 Tage

**Woche 11-12: PILOT**
- [ ] BASF-Pilot Vorbereitung
- [ ] Investor-Outreach - 3 Wochen ⏩
- [ ] Referenz-Pilot Completion - ⏩
- [ ] Success Story dokumentieren

**Deliverables Monat 3:**
- ✅ Fraunhofer/TÜV Audit Zertifikat
- ✅ BMWK-Förderantrag eingereicht
- ✅ BASF-Pilot Go-Live
- ✅ Referenz-Kunde (Success Story)
- ✅ Investor-Gespräche gestartet

---

## 🎯 Critical Path & Abhängigkeiten

### Kritischer Pfad (BLOCKER → BASF Pilot)

```
Week 1-2:
  REST API Production ─┐
  Docker/K8s          ─┤
  SAP-Adapter         ─┼─> Week 3-4:
  Monitoring          ─┘    Policy-Compiler ─┐
                            HSM/TPM         ─┤
                            Z-Felder/BRF    ─┼─> Week 5-8:
                                            ─┘    Adaptive Orchestrator ─┐
                                                  Pen-Test              ─┤
                                                  Legal Review          ─┼─> Week 9-12:
                                                  SAP Integration       ─┘    BASF Pilot
                                                                              Go-Live
```

### Parallele Tracks

```
TECH Track:
  REST API → Docker → SAP-Adapter → Policy-Compiler → Adaptive Orch.

SECURITY Track:
  SBOM → mTLS → HSM → Pen-Test → Audit

LEGAL Track:
  Legal Review → DPIA → Rechtsgutachten → Lizenz

SAP Track:
  SAP-Adapter → Z-Felder → BRFplus → Fiori → CPI → DMS

BUSINESS Track:
  Pilotangebot → Readiness Deck → Förderantrag → Investor → Pilot
```

---

## 📊 Ressourcen-Planung

### Team-Rollen

| Rolle | FTE | Wochen | Tasks |
|-------|-----|--------|-------|
| **Backend Dev** | 2.0 | 12 | REST API, Docker, SAP-Adapter, Orchestrator |
| **Security Eng** | 1.0 | 12 | mTLS, HSM, Pen-Test, SBOM |
| **SAP Dev** | 1.0 | 8 | Z-Felder, BRFplus, Fiori, CPI |
| **DevOps** | 0.5 | 12 | K8s, Monitoring, CI/CD |
| **Legal** | 0.5 | 8 | Legal Review, DPIA, Gutachten |
| **Business Dev** | 1.0 | 12 | Pilotangebot, Förderantrag, Investor |
| **GESAMT** | **6.0 FTE** | **12 Wochen** | **35 Tasks** |

### Budget-Schätzung

| Kategorie | Kosten |
|-----------|--------|
| **Entwicklung** (6 FTE × 12 Wochen × €1500/Woche) | €108.000 |
| **Externe Audits** (Fraunhofer/TÜV, Pen-Test) | €50.000 |
| **Legal** (Rechtskanzlei, DPIA) | €15.000 |
| **Infrastructure** (Cloud, HSM, SAP Access) | €5.000 |
| **Marketing** (Pitch Deck, Website) | €10.000 |
| **Markenanmeldung** (DPMA, EUIPO) | €2.000 |
| **GESAMT** | **€190.000** |
| **Mit Förderung (50%)** | **€95.000** |

---

## ✅ Definition of Done (Pilot-Ready)

### Technisch ✅
- [ ] REST API deployed (TLS, OAuth2, Monitoring)
- [ ] Docker/K8s Production-ready
- [ ] SAP-Adapter funktional (OData Integration)
- [ ] Policy-Compiler (IR v1)
- [ ] Adaptive Proof Orchestrator
- [ ] HSM Key Management
- [ ] Health-Checks + Prometheus Metrics
- [ ] OpenAPI Spec + CLI Docs

### Sicherheit ✅
- [ ] SBOM generiert
- [ ] mTLS implementiert
- [ ] Pen-Test bestanden
- [ ] Security Audit Zertifikat (Fraunhofer/TÜV)
- [ ] DSGVO-konform (Logging, Hashing)

### Recht ✅
- [ ] Legal Review abgeschlossen
- [ ] DPIA erstellt
- [ ] Rechtsgutachten (§371a ZPO)
- [ ] Lizenzmodell definiert
- [ ] Marke angemeldet

### Integration ✅
- [ ] SAP Z-Felder implementiert
- [ ] BRFplus Workflow konfiguriert
- [ ] Fiori-App deployed
- [ ] CPI-Flow funktional
- [ ] DMS/ArchiveLink Integration
- [ ] SAP Integration Guide

### Business ✅
- [ ] Pilotangebot (BASF) erstellt
- [ ] BMWK-Förderantrag eingereicht
- [ ] Fraunhofer/TÜV MoU unterschrieben
- [ ] Investor-Pitch Deck ready
- [ ] Referenz-Pilot abgeschlossen
- [ ] Success Story veröffentlicht

---

## 🚀 Quick Wins (Woche 1-2)

**Sofort umsetzbar:**

1. **SBOM generieren** (1 Tag)
   ```bash
   cargo install cargo-cyclonedx
   cargo cyclonedx --format json > sbom.json
   ```

2. **Dockerfile erstellen** (2 Tage)
   - Multi-Stage Build
   - Docker Compose

3. **TLS aktivieren** (3 Tage)
   - rustls Integration
   - Self-signed Cert (Dev)

4. **Prometheus Metrics** (3 Tage)
   - `/metrics` Endpoint
   - Basic Metrics (Requests, Latency)

5. **OpenAPI Spec** (2 Tage)
   - Swagger UI Integration

**Total: 11 Tage → 2 Wochen mit Testing**

---

## 📞 Nächste Schritte

### JETZT (Diese Woche):
1. Team-Meeting: Roadmap Review
2. Ressourcen-Allocation (6 FTE)
3. Quick Wins starten (SBOM, Docker)
4. SAP System Access sicherstellen

### Woche 1-2:
1. REST API Production (TLS, mTLS, Monitoring)
2. Docker/K8s Container
3. SAP-Adapter Development starten

### Woche 3-4:
1. Policy-Compiler
2. HSM Integration
3. SAP Z-Felder + BRFplus
4. Pilotangebot finalisieren

---

**Stand:** 2025-11-10
**Nächstes Review:** 2025-11-17 (Weekly)
**Verantwortlich:** Core Team + Business Development
