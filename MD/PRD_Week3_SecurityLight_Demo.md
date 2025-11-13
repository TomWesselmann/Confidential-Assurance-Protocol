# 🛡️ PRD – Woche 3: Security‑Härtung (light) & Demo/Pilot‑Vorbereitung

**Ziel (Woche 3):**  
CAP‑Adapter + Verifier **betriebssicher** machen (mTLS standardmäßig aktivierbar, signierte Builds, SBOM, Audit‑Logs) und **Demo/Pilotpaket** schnüren.  
**Dauer:** 1 Woche  
**Abhängigkeiten:** Woche 1 (Adapter Skeleton), Woche 2 (E2E + Writeback).

---

## 🎯 Scope
- **Security‑Härtung (light):** mTLS **einschaltbar** → Standard **ON** in Pilot, OFF in Dev. Rate‑Limits, sichere Cipher‑Suites, Logs ohne PII.  
- **Supply‑Chain:** SBOM erstellen, Image **signieren** (cosign), einfache **Provenance** (SLSA‑like) in CI.  
- **Auditfähigkeit:** append‑only **Audit‑Log** (hash‑verkettet), Key‑Rotation‑Befehl, kurzer Security‑Whitepaper.  
- **Observability:** Prometheus‑Metriken vollständig, **Grafana‑Panels** (JSON).  
- **Demo‑Bundle:** Datenset (50 Supplier, 1 FAIL, 2 WARN), Skripte, README_DEMO.

---

## 🧱 Architektur‑Update (Security‑relevant)
```
[Adapter] ──(HTTPS+mTLS)──> [Verifier]
   │                          │
   │                          ├─ Signatur (Ed25519) + RFC3161 (optional stub)
   │                          ├─ Audit‑Log (hash‑chain, append‑only)
   │                          └─ /metrics, /healthz, /readyz
   └─ CI/CD: SBOM + Image‑Sign + Scan (Trivy/Grype) + Provenance (attest)
```

---

## ⚙️ Umsetzung – Aufgabenpakete

### A) Security‑Härtung
- [ ] **mTLS standardisierbar**: Config‑Schalter `require_mtls=true` (Pilot default), CA‑Bundle mounten.  
- [ ] **TLS Policy**: TLS≥1.2 (bevorzugt 1.3), sichere Ciphers; HSTS auf Ingress‑Ebene.  
- [ ] **Rate‑Limiting**: global + per Client (z. B. 100 RPS / 20 RPS).  
- [ ] **PII‑Safe Logging**: strukturierte JSON‑Logs, Felder whitelisten, Redaction aktiv.  
- [ ] **Key‑Rotation**: `key rotate --kid <new>` + Registry‑Update (KID).  
- [ ] **Audit‑Log**: append‑only Datei/Stream mit `prev_hash` → `entry_hash` (SHA3‑256); Event‑Schema dokumentieren.

### B) Supply‑Chain & CI/CD
- [ ] **SBOM** erzeugen (syft) → `sbom.json` als Artefakt.  
- [ ] **Security‑Scan** (Trivy + Grype) → **fail on High/Critical**.  
- [ ] **Image‑Signatur** (cosign) + **Verify‑Policy** im Cluster/Doc.  
- [ ] **Provenance‑Attest** (cosign attest) mit Build‑Hash, Git‑SHA, Zeitstempel.

### C) Observability & Runbooks
- [ ] **Prometheus‑Metriken** erweitern: Errors per Reason, Latenz‑Histogramme, TLS‑Handshake‑Fehler.  
- [ ] **Grafana Panels**: OK/WARN/FAIL, p95 Latenz, Fehlerursachen‑Breakdown.  
- [ ] **Runbooks**: „mTLS Fehler beheben“, „Key‑Rotation durchführen“, „Policy‑Mismatch analysieren“.

### D) Demo/Pilot‑Bundle
- [ ] **Dataset**: `examples/suppliers_demo.json` (50 Einträge; 1 FAIL, 2 WARN).  
- [ ] **Skripte**: `make demo-run` → Pull→Hash→Verify→Writeback→Metrics Snapshot.  
- [ ] **README_DEMO.md**: 10‑Min‑Guide inkl. Screenshots (Grafana), Beispiel‑Responses.  
- [ ] **Security‑Whitepaper (Kurz)**: 3–4 Seiten (TLS/mTLS, Keys, Logs, Datenminimierung).

---

## 🔐 Konfiguration (ergänzt)
```yaml
security:
  require_mtls: true         # Pilot default
  tls_min_version: "1.2"     # 1.3 bevorzugt, aber 1.2 erlaubt
  ciphers_profile: "secure"  # Cluster/Ingress‑Policy
  rate_limit:
    global_rps: 100
    client_rps: 20
  redact_logs: true
crypto:
  sign_key_path: /etc/keys/agent.ed25519
  key_rotation_enabled: true
audit:
  log_path: /var/log/cap/audit.log
  hash_chain: sha3-256
```

---

## 🧪 Tests (Woche 3)

### Security
- [ ] **mTLS ON**: Call ohne Client‑Zertifikat → **403**.  
- [ ] **mTLS ON**: Call mit falscher CA/SAN → **403**.  
- [ ] **Key‑Rotation**: alter KID → verify OK; neuer KID ab Zeitpunkt T → verify OK; Nachweis in Audit‑Log.  
- [ ] **Rate‑Limit** greift (HTTP 429 bei Überschreitung).  
- [ ] **Logs** enthalten keine PII (Stichprobenprüfung).

### Supply‑Chain
- [ ] **SBOM vorhanden**, **Scans** ohne High/Critical (sonst Build fail).  
- [ ] **cosign verify** erfolgreich; Provenance‑Attest vorhanden.

### Observability
- [ ] **/metrics** exportiert Zähler/Histogramme; Grafana‑Dashboard zeigt OK/WARN/FAIL + p95.  
- [ ] **Alerts (optional)**: Error‑Rate >1 % 5min, p95>1s 5min → Warnung.

### Demo
- [ ] **Demo‑Run** erzeugt 1 FAIL, 2 WARN deterministisch; Writeback sichtbar.  
- [ ] README‑Schritte reproduzierbar in <10 Min.

---

## 📊 Beispiel‑Metriken (Erweiterung)
- `cap_verifier_tls_handshake_failures_total`  
- `cap_verifier_requests_total{result="ok|warn|fail"}`  
- `cap_verifier_request_duration_seconds_bucket`  
- `cap_adapter_sap_write_total{status="ok|error"}`  
- `cap_audit_chain_break_total` (soll 0 sein)

---

## 🧰 CI/CD – Pipeline‑Skizze
```yaml
jobs:
  build:
    steps:
      - checkout
      - docker-build-push
      - syft-sbom: out=sbom.json
      - trivy: severity=HIGH,CRITICAL, exit-code=1
      - grype: severity=HIGH,CRITICAL, fail-on-severity
      - cosign-sign: image:${IMAGE}, key:${COSIGN_KEY}
      - cosign-attest: predicate:provenance.json
      - upload-artifacts: [sbom.json, scan_report.html, provenance.json]
```

---

## 🧾 Audit‑Log – Event‑Schema
```json
{
  "ts":"2025-11-09T10:12:33Z",
  "event":"verify",
  "run_id":"2025-11-09_01",
  "supplier_batch":100,
  "policy_id":"lksg.v1",
  "manifest_hash":"0xa43b8c...",
  "result":"OK",
  "prev_hash":"0x...",
  "entry_hash":"0x..."
}
```

**Chain‑Check:** `entry_hash = H(prev_hash || ts || event || manifest_hash || result || ...)`

---

## 📦 Deliverables (Ende Woche 3)
- `security/`:
  - `SECURITY_WHITEPAPER.md` (3–4 Seiten)
  - `AUDIT_LOG_SPEC.md` (Schema + Prüfschritte)
- `ci/`:
  - Pipeline mit SBOM, Scan, cosign sign + attest
- `grafana/panels.json` (Dashboards)  
- `examples/suppliers_demo.json` + `Makefile` (`demo-run`)  
- `README_DEMO.md` (Screenshots, Befehle)  
- Aktivierte mTLS‑Option + Rate‑Limits (konfigurierbar)

---

## ✅ Akzeptanzkriterien (DoD, Woche 3)
1. mTLS **einschaltbar**; Pilot‑Config **default ON**.  
2. SBOM & signiertes Image; Scans ohne **High/Critical**.  
3. Audit‑Log hash‑verkettet; Rotation im Log nachvollziehbar.  
4. Prometheus‑Metriken & Grafana‑Panels einsatzbereit.  
5. Demo‑Run reproduzierbar: 1 FAIL, 2 WARN, Writeback sichtbar.  
6. README/Runbooks vorhanden; keine PII in Logs.

---

**Ergebnis:**  
Nach Woche 3 ist der Stack **sicher genug für Pilotbetrieb** (on‑prem, ohne Internet), mit **prüfbarer Supply‑Chain** und **vollständiger Demo** – ideal für BASF/EuroDat‑Vorstellung.
