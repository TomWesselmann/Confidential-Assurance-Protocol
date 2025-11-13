# ✅ Testplan – CAP (Confidential Assurance Protocol) v1.0
**Datum:** 2025‑11‑09  
**Ziel:** Umfassende Teststrategie & ‑fälle, um CAP (Verifier‑API, SAP‑Adapter, Policy‑Compiler, Registry/Signaturen, Container/K8s) **funktions‑, sicherheits‑ und auditfähig** zu verifizieren.  
**Geltungsbereich:** Dev/CI, On‑Prem, SAP BTP (Kyma).

---

## 1) Übersicht & System‑Under‑Test (SUT)
**Komponenten:**
- **Verifier‑API** (`/verify`, `/policy/compile`, `/healthz`, `/readyz`) – Rust/WASM
- **SAP‑Adapter** (OData/CDS → `context.json`, Writeback Z‑Tabelle/BP‑Extension)
- **Policy‑Compiler** (YAML → IR v1, Linter, Rule‑Trace)
- **Registry** (Signaturen Ed25519, RFC3161 Timestamp, Schema‑Versionen)
- **Container/K8s** (Docker, Distroless, Helm, Probes, NetworkPolicy)
- **Security‑Artefakte** (SBOM, Image‑Signatur, Audit‑Log Hash‑Chain, Key‑Rotation)

**Nicht im Scope (dieser Runde):**
- Echte ZK‑Backends (Halo2/zkVM) – Mock‑Backend wird getestet
- Prod‑eIDAS Timestamper – RFC3161 ggf. Stub/Dev‑TSA

---

## 2) Umgebungen & Testmatrix
| Ebene | Umgebung | Zweck |
|---|---|---|
| **Local** | Docker Compose (Verifier + Adapter) | schnelles Iterieren, Unit/IT |
| **CI** | GitHub/GitLab Runner | Build, Unit/IT, Security Scans, SBOM, Contract |
| **On‑Prem Dev** | K8s/Kind/Minikube | Helm‑Chart, Probes, NetworkPolicy |
| **BTP Kyma (optional)** | Cluster‑Namespace | Ingress/APIRule, mTLS via Mesh, ServiceMonitor |

**Matrix (Auszug):**
- API‑Contract: Local & CI
- Security (mTLS/TLS): On‑Prem & BTP
- Performance: Local (k6) + On‑Prem (1‑N Replikas)
- SAP‑Flow: Local (Mock) + On‑Prem (Dev OData)

---

## 3) Testkategorien
1. **Unit‑Tests** – reine Logik, Parser, Hashing, Signaturen, Helpers  
2. **Contract‑Tests** – OpenAPI‑Konformität, JSON‑Schemas, Fehlercodes  
3. **Integration/E2E** – SAP Mock → Adapter → Verifier → Writeback  
4. **Sicherheit** – TLS/mTLS, OAuth2/Scopes, Key‑Rotation, Log‑Redaction, Container‑Hardening  
5. **Datenkorrektheit** – IR‑Hash, Manifest‑Hash, Determinismus, Golden Files  
6. **Performance/Last** – Latenz, Durchsatz, Batch‑Verarbeitung, Ressourcen  
7. **Resilienz/Chaos** – Timeouts, 5xx, Netzwerk, Idempotency/Retry, Partial Failures  
8. **Observability** – /metrics, Latenz‑Histogramme, Dashboards, Alerts  
9. **Compliance** – Linter (legal_basis), Missing Signature Warn, DPIA/TOMs Artefakte vorhanden

---

## 4) Testspezifikation (nach Komponente)

### 4.1 Verifier‑API
**Unit**
- U‑V01: `policy_hash` deterministisch (gleiche YAML → gleicher Hash)
- U‑V02: Rule‑Trace wird korrekt aus IR/Predicates generiert
- U‑V03: Signatur Round‑Trip (sign → verify) Ed25519 OK
- U‑V04: RFC3161 Stub: Timestamp‑Feld vorhanden & wohldefiniert
- U‑V05: Hashing (BLAKE3/SHA3) – Byte‑Order & Hex kodiert

**Contract**
- C‑V01: `POST /verify` validiert gegen OpenAPI (200/422/401/403/409/500)
- C‑V02: Fehler‑Payload enthält keine PII, strukturiertes Feld `violations[]`
- C‑V03: `POST /policy/compile` – Linter‑Warnungen/strict‑Fehler korrekt
- C‑V04: `/healthz` & `/readyz` liefern nur Minimalinfos (kein Secret/Path)

**Integration**
- I‑V01: Verify OK – zwei Suppliers, kein Treffer → `result=OK`, Rule‑Trace ≠ ∅
- I‑V02: Verify FAIL – erzwungener Sanktions‑Hit → `result=FAIL`, Detail in `violations`
- I‑V03: Policy/IR‑Mismatch → `409 Conflict`
- I‑V04: Adaptive Mode an/aus → unterschiedliche aktive Regeln

**Security**
- S‑V01: TLS≥1.2; Abbruch bei 1.0/1.1
- S‑V02: mTLS ON → fehlendes Client‑Zertifikat → `403`
- S‑V03: OAuth2 Scope‑Mismatch → `403`
- S‑V04: Rate‑Limit (global/Client) → `429` bei Überschreitung
- S‑V05: Logs ohne PII (Stichproben), Request/Response Redaction aktiv

**Performance**
- P‑V01: p50 < 200 ms, p95 < 500 ms (Mock, 1 Replica) bei 50 RPS
- P‑V02: Durchsatz 100 RPS stabil, <1 % Error‑Rate 5 min

**Resilienz**
- R‑V01: 5xx‑Fehler injizieren → Service erholt sich; keine Memory‑Leaks
- R‑V02: Teil‑Timeouts → korrekte 504/502, keine hängenden Worker

**Observability**
- O‑V01: `requests_total{result}` zählt OK/WARN/FAIL korrekt
- O‑V02: Latenz‑Histogramm füllt sich; Dashboard zeigt p95/p99

---

### 4.2 SAP‑Adapter
**Unit**
- U‑A01: Mapping LIFNR/LAND1/AUDIT_DATE → `context.json` korrekt
- U‑A02: BLAKE3‑Hashing konstant, Salting/Prefix klar dokumentiert
- U‑A03: Idempotency‑Key aus `RUN_ID|SUPPLIER_ID` deterministisch

**Integration**
- I‑A01: OData‑Pull (Mock) → `context.json` mit Hashes (keine Klartexte)
- I‑A02: POST `/verify` → Response parsen → Writeback `Z_CAP_SUPPLIER_STATUS`
- I‑A03: UPSERT (gleicher RUN_ID) überschreibt statt dupliziert
- I‑A04: CSV‑Fallback funktioniert (ohne OData)

**Security**
- S‑A01: HTTPS‑Handshake zum Verifier mit self‑signed (Dev)
- S‑A02: mTLS optional (Flag), ON im Pilot
- S‑A03: Logs ohne PII; Hash‑Werte whitelisten

**Performance**
- P‑A01: 100 Supplier < 60 s (Mock Backend)
- P‑A02: Speicher < 256 MiB, CPU < 500m unter Last

**Resilienz**
- R‑A01: Verifier 5xx → Retry mit Backoff; Idempotenz gewährleistet
- R‑A02: Writeback‑Fehler → Retry‑Queue; keine Duplikate

**Observability**
- O‑A01: `sap_write_total{status}` stimmt mit DB‑Zeilen überein
- O‑A02: Verifier‑Latenz‑Histogramm im Adapter vorhanden

---

### 4.3 Policy‑Compiler
**Unit**
- U‑P01: YAML‑Parser erkennt fehlende `legal_basis` → Fehler im `strict` Mode
- U‑P02: IR‑Hash deterministisch; Golden File Vergleich
- U‑P03: Predicates evaluieren korrekt (LOW/HIGH Tier Beispiel)

**Contract**
- C‑P01: IR Schema valid (JSON‑Schema), Version `ir_version=1.0` erzwingt Pflichtfelder

**Integration**
- I‑P01: `policy compile` → IR → `/verify` → Rule‑Trace Konsistenz

---

### 4.4 Registry/Signaturen & Audit‑Log
**Unit**
- U‑R01: Ed25519 sign/verify OK; falscher Key → Fail
- U‑R02: `missing signature` → Warnung, kein Abbruch (Backwards‑Compat)
- U‑R03: RFC3161 Feld plausibel (Zeit > Build‑Zeit – 1d)

**Integration**
- I‑R01: Manipuliertes Manifest → Verify Fail
- I‑R02: Key‑Rotation: alter & neuer KID nachvollziehbar; beide verify OK

**Audit‑Log**
- A‑L01: Hash‑Kette lückenlos (`prev_hash` → `entry_hash`)
- A‑L02: Erzwungene Kettenunterbrechung → Alarm & Metrik `audit_chain_break_total`

---

### 4.5 Container/K8s
**Security/Policy**
- S‑K01: Container läuft **non‑root**, `readOnlyRootFilesystem=true`, `capabilities: drop ALL`
- S‑K02: Seccomp/AA Profile aktiv (RuntimeDefault/AppArmor)
- S‑K03: NetworkPolicy: **keine egress** (Test: externer Connect schlägt fehl)
- S‑K04: Liveness/Readiness Probes 200 OK; kein Geheimnis in `/healthz`

**Supply Chain**
- SC‑K01: SBOM artefakt vorhanden; Trivy/Grype ohne High/Critical
- SC‑K02: Image signiert (cosign), Verify Policy im Cluster besteht

**Helm/Deploy**
- H‑K01: Helm‑Install in leeren Namespace klappt ohne manuelle Schritte
- H‑K02: Values Override (mTLS on/off, limits) wirksam
- H‑K03: Rollback funktioniert; Zero‑Downtime bei 2 Replikas

---

## 5) Testfälle (Beispiele, ausformuliert)

### TC‑V‑FAIL‑001 – Sanktions‑Treffer führt zu `FAIL`
**Vorbedingungen:** Verifier & Adapter laufen; `sanctions_root` enthält Hash `H123`.  
**Schritte:**
1. Adapter erzeugt `context.json` mit `supplier_hashes=["H123"]`.
2. `POST /verify` mit Policy `no_sanctions`.
3. Antwort prüfen.  
**Erwartet:** `result="FAIL"`, `violations[0].rule_id="no_sanctions"`, Signatur vorhanden, Logs ohne PII.

---

### TC‑A‑UPSERT‑002 – Idempotenter Writeback
**Vorbedingungen:** Z‑Tabelle leer; RUN_ID = `2025‑11‑09_01`.  
**Schritte:** Zwei identische Runs mit gleichem Supplier.  
**Erwartet:** exakt **eine** Zeile in `Z_CAP_SUPPLIER_STATUS`, Felder identisch, `UPDATED_AT` später.

---

### TC‑S‑MTLS‑003 – mTLS Pflicht
**Vorbedingungen:** Verifier `require_mtls=true`.  
**Schritte:** Call `/verify` ohne Client‑Zertifikat.  
**Erwartet:** `403`, Audit‑Log Event `auth_fail`, kein PII in Log‑Body.

---

### TC‑P‑IR‑HASH‑004 – IR Hash stabil
**Vorbedingungen:** `policy.yml` unverändert.  
**Schritte:** Zwei Kompiliervorgänge in frischen Umgebungen.  
**Erwartet:** identischer `ir_hash`, Golden File passt, Git‑Diff leer.

---

## 6) Testdaten & Fixtures
- **`examples/suppliers_demo.json`** – 50 Lieferanten; 1 mit Hash∈Sanktionsroot, 2 in Hochrisiko.  
- **`examples/context_ok.json`** / **`context_fail.json`** – direkt für `/verify`.  
- **Golden Files** – `golden/ir_v1.json`, `golden/verify_ok.json`, `golden/verify_fail.json`.  
- **Secrets (Dev)** – self‑signed TLS, Dummy‑Ed25519‑Key; **niemals** in Prod.

---

## 7) Tools & Befehle (Vorschlag)

**Rust Unit/IT**
```bash
cargo test --workspace --all-features
cargo bench  # Criterion (optional)
```

**API‑Contract (Dredd oder schemathesis)**
```bash
schemathesis run openapi/verifier.v1.yaml --base-url=https://localhost:8443 -c
```

**k6 Load**
```js
import http from 'k6/http';
import { sleep } from 'k6';
export const options = { vus: 25, duration: '3m' };
export default function () {
  const res = http.post('https://localhost:8443/verify', JSON.stringify(__ENV.PAYLOAD), { timeout: '5s' });
  sleep(0.1);
}
```
```bash
PAYLOAD="$(cat examples/context_ok.json)" k6 run k6/verify.js
```

**Security Scans**
```bash
syft . -o spdx-json=sbom.json
trivy image --severity HIGH,CRITICAL --exit-code 1 cap-verifier:dev
grype cap-verifier:dev --fail-on High
```

**cosign**
```bash
cosign sign registry.example.com/cap/verifier:v1
cosign verify registry.example.com/cap/verifier:v1
cosign attest --predicate provenance.json registry.example.com/cap/verifier:v1
```

---

## 8) CI/CD‑Gates & Metriken
- **Build**: grün + SBOM Artefakt  
- **Tests**: Unit ≥ 90 % grün, Integration 100 % grün  
- **Security**: Trivy/Grype **ohne High/Critical** (Blocker)  
- **Performance**: p95 < 500 ms bei 50 RPS (Mock)  
- **Quality**: Kein PII in Logs (greppbar `PII=` sollte **nicht** vorkommen)  
- **Release Gate**: OpenAPI validiert, Helm‑Install Smoke Pass

---

## 9) Abnahme & DoD (Definition of Done)
1. Alle Testkategorien mindestens einmal auf CI ausgeführt, grün.  
2. Golden Files aktualisiert & versioniert.  
3. Security‑Nachweise beigefügt: SBOM, Scanberichte, cosign verify.  
4. Audit‑Log Konsistenzcheck bestanden (keine Chain‑Breaks).  
5. README/Test‑Guide aktualisiert (How‑to reproduce).  
6. Pilot‑Run‑Bericht: KPIs (OK/WARN/FAIL‑Raten, p95), Lessons Learned.

---

## 10) Fehlerklassifikation & Triage
| Level | Bedeutung | Beispiel | Reaktion |
|---|---|---|---|
| **Blocker** | Pilot nicht möglich | Auth‑Bypass, PII‑Leak, Verify falsches Ergebnis | Hotfix/Stop Release |
| **Hoch** | Risiko/Compliance | mTLS Bug, falsche Latenz, fehlende Logs | Fix vor Release |
| **Mittel** | Komfort/Stabilität | sporadische 5xx unter Last | Fix/Workaround |
| **Niedrig** | Kosmetik/Doku | Schreibfehler, kleine UI‑Dinge | Nächster Sprint |

---

## 11) Anhänge
- **JSON‑Schemas:** `schemas/context.schema.json`, `schemas/verify_response.schema.json`  
- **OpenAPI:** `openapi/verifier.v1.yaml`  
- **Policy‑Lint‑Regeln:** `docs/policy_lints.md`  
- **Runbooks:** `ops/runbooks/*.md` (mTLS, Key‑Rotation, Policy‑Mismatch)

---

**Kurzfazit:** Dieser Testplan prüft CAP **End‑to‑End**, deckt **Sicherheit, Funktionsfähigkeit, Compliance, Performance** und **Betriebsreife** ab und liefert die **Artefakte**, die ein Enterprise‑Pilot (z. B. BASF/EuroDat) erwartet.
