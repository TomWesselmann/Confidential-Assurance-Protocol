# 🧩 PRD – Woche 1: SAP-Adapter Skeleton + Security-Baseline

**Ziel:**  
Funktionsfähiger SAP-Adapter (Mock/CDS-Demo) mit gesicherter HTTPS-Verbindung zum Verifier und CI-Security-Baseline.  
**Dauer:** 1 Woche  
**Abhängigkeiten:** REST-Verifier-API & Docker-Container vorhanden  

---

## 🔧 1. Ziele und Scope
- SAP-Mockdaten (OData/CDS-Demo) werden in **`context.json`** überführt.  
- Hashing (BLAKE3) findet **im Adapter** statt – keine Rohdaten-Übertragung.  
- Verifier-API wird über **HTTPS (Port 8443)** getestet (self-signed).  
- mTLS optional über Flag `--require-mtls=false`.  
- CI-Job (GitHub Actions / GitLab CI) erstellt Container, führt Trivy/Grype-Scan aus (fail on High/Critical).  

---

## 🧱 2. System-Architektur

```
SAP S/4 (OData/CDS Mock)
   │
   ▼
[Adapter CLI/Service]
   ├─ Pull Supplier Data
   ├─ BLAKE3 Hashing → context.json
   ├─ HTTPS POST → Verifier /verify
   │      └─ self-signed TLS (Port 8443)
   └─ Writeback Mock (Supplier Status)
```

---

## ⚙️ 3. Implementierungsschritte

### Phase 1 – Adapter-Skeleton
- [ ] Mock-SAP-Datenquelle (OData/CDS) erstellen → JSON-Response (10 Supplier).  
- [ ] Mapper bauen: Felder `LIFNR`, `LAND1`, `AUDIT_DATE` → `context.json`.  
- [ ] BLAKE3-Hashing (lib b3sum oder blake3 crate).  
- [ ] POST `context.json` → `https://localhost:8443/verify` (Verifier-API).  
- [ ] Response (`result`, `manifest_hash`, `valid_until`) anzeigen.  

### Phase 2 – HTTPS/mTLS Baseline
- [ ] Self-signed TLS (`openssl req -new -x509 -days 365`) in `/etc/tls/`.  
- [ ] Adapter-Option `--require-mtls=false` (CLI-Flag oder Config).  
- [ ] Testlauf mit `curl -k https://localhost:8443/healthz`.  

### Phase 3 – CI/CD + Security Scan
- [ ] `.github/workflows/build.yml` oder `.gitlab-ci.yml`:  
  - Build Docker-Image (`docker build -t cap-adapter:dev .`)  
  - Run Trivy + Grype (`--exit-code 1 --severity HIGH,CRITICAL`)  
- [ ] CI-Logs → Artifacts (`sbom.json`, `scan_report.html`).  

---

## 🔐 4. Security & Config

| Komponente | Maßnahme | Ziel |
|-------------|-----------|------|
| **Adapter** | Hashing im Adapter (BLAKE3) | DSGVO-Sicherheit |
| **Verifier** | HTTPS 8443 / TLS1.3 | Transportverschlüsselung |
| **CI/CD** | Trivy/Grype Scans | Schwachstellenprüfung |
| **Logs** | kein PII | Datenschutz |
| **mTLS** | optional aktivierbar | On-Prem Pilot kompatibel |

---

## 📁 5. Dateistruktur

```
sap-adapter/
├─ src/
│  ├─ main.rs / adapter.py
│  └─ hash.rs / utils/
├─ config/
│  ├─ adapter.yaml
│  ├─ tls/
│  │   ├─ server.crt
│  │   ├─ server.key
│  │   └─ ca.crt
├─ examples/
│  ├─ suppliers.json
│  └─ context_sample.json
├─ .github/workflows/
│  └─ build.yml
└─ Dockerfile
```

---

## 🧪 6. Testfälle

| Test | Erwartung |
|------|------------|
| Pull Mock-Data → context.json | Hashes erzeugt, keine Rohdaten |
| POST Verify (HTTPS) | 200 OK, `result=OK` |
| TLS Handshake | Erfolgreich self-signed |
| mTLS deaktiviert | Kein Abbruch |
| CI Scan | Keine High/Critical Findings |
| SBOM Artefakt | Enthalten in CI-Output |

---

## 🧭 7. Deliverables (End Woche 1)

✅ `adapter/` Modul mit BLAKE3-Hashing  
✅ HTTPS Testlauf gegen Verifier  
✅ CI/CD-Pipeline mit Trivy/Grype  
✅ `sbom.json` und `scan_report.html`  
✅ README „How to Run Adapter Skeleton“  

---

**Ergebnis:**  
Nach Woche 1 existiert ein funktionsfähiger, sicherer **Proof-Datenfluss** (SAP Mock → Verifier → Response).  
→ Basis für Woche 2 (End-to-End Integration & Rückschreiben).
