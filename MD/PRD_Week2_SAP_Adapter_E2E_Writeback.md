# 🧩 PRD – Woche 2: End-to-End Integration & Rückschreiben (SAP-Adapter)

**Ziel (Woche 2):**  
Kompletter E2E-Flow mit realistischem SAP-Mock: **SAP → Adapter (Hashing) → Verifier → Rückschreiben in SAP** (Status/Valid‑Until/Manifest‑Hash) + grundlegende Observability.  
**Dauer:** 1 Woche  
**Abhängigkeiten:** Woche 1 (Adapter-Skeleton, HTTPS/mTLS-Option, CI-Scan) abgeschlossen.

---

## 🎯 Scope
- OData/CDS **Pull** (Mock/Dev) → `context.json` (BLAKE3 im Adapter).  
- `POST /verify` (HTTPS 8443) – **OK/WARN/FAIL** + Rule‑Trace verarbeiten.  
- **Writeback nach SAP**: Ergebnis in **Z‑Tabelle** bzw. **BP‑Extension** ablegen.  
- **/metrics** aktivieren (Prometheus), simple **Grafana Panels** bereitstellen.  
- Fehlerpfade & Idempotenz definieren.

---

## 🧱 Architektur (E2E)
```
SAP S/4 (Dev/Mock OData)
   └─ CDS View (I_Supplier / I_BusinessPartner)
        │ pull (OData v2/v4)
        ▼
[Adapter Service]
   ├─ Map → context.json (BLAKE3 Hashing)
   ├─ POST https://verifier.local:8443/verify
   ├─ Parse Response (result, valid_until, manifest_hash, trace)
   └─ Writeback:
       • OData POST → Z_CAP_SUPPLIER_STATUS (empfohlen)
       • (oder) OData PATCH → BP-Extension-Felder
        │
        ▼
SAP Persistenz (Z‑Tabelle / BP‑Extension) + Fiori Anzeige
```

---

## 🔁 Daten-Mapping

### Input (SAP → context.json)
| Quelle (CDS) | Feld | Transformation | Ziel (`context.json`) |
|---|---|---|---|
| `I_Supplier` | `Supplier` | **BLAKE3** | `supplier_hashes[]` |
| `I_Supplier` | `Country`  | **BLAKE3** (oder Mapping→Code→Hash) | `supplier_regions[]` |
| `I_SupplierPurchasingOrg` | `LastSupplierEvalDate` | ISO 8601 | `audit_dates[]` |
| Referenzlisten (extern geladen) | `SanctionsRoot` | Hex | `sanctions_root` |
| Referenzlisten (intern) | `HighRiskRoot` | Hex | `high_risk_root` |

**Beispiel `context.json`:**
```json
{
  "policy_id": "lksg.v1",
  "context": {
    "supplier_hashes": ["0x2a..","0x9f.."],
    "supplier_regions": ["0x7b..","0xaa.."],
    "sanctions_root": "0xdeadbeef",
    "high_risk_root": "0xfeedcafe",
    "audit_dates": ["2025-03-10","2025-02-01"]
  },
  "backend": "mock",
  "options": {"adaptive": true}
}
```

### Output (Response → SAP)
| Response-Feld | Ziel | Typ |
|---|---|---|
| `result` (OK/WARN/FAIL) | `Z_CAP_SUPPLIER_STATUS.status` | CHAR(8) |
| `valid_until` | `Z_CAP_SUPPLIER_STATUS.valid_until` | DATS |
| `manifest_hash` | `Z_CAP_SUPPLIER_STATUS.manifest_hash` | CHAR(66) |
| `trace.active_rules` | `Z_CAP_SUPPLIER_STATUS.rules_json` | STRING (JSON) |

**Z‑Tabelle (empfohlen):** `Z_CAP_SUPPLIER_STATUS`  
```text
MANDT, SUPPLIER_ID (key, CHAR10), RUN_ID (key, CHAR32), 
STATUS (CHAR8), VALID_UNTIL (DATS), MANIFEST_HASH (CHAR66),
RULES_JSON (STRING), CREATED_AT (TIMS/TS), CREATED_BY (USER)
```

---

## 🔌 Endpunkte & Verträge

### 1) Verifier `/verify` (Reminder)
**Req:** `context.json` (siehe oben)  
**Resp:**
```json
{
  "result":"OK",
  "valid_until":"2026-03-31T00:00:00Z",
  "manifest_hash":"0xa43b8c...",
  "trace":{"risk_tier":"HIGH","active_rules":["no_sanctions"]},
  "signature":"base64(ed25519)",
  "timestamp":"RFC3161"
}
```

### 2) SAP Writeback (Optionen)
**A. OData Z‑Service (empfohlen)**  
Entity: `Z_CAP_SUPPLIER_STATUS` → `POST`/`UPSERT` je Supplier & Run  

**B. BP‑Extension (Advanced)**  
OData PATCH auf `A_BusinessPartner` mit Custom Fields, z. B.  
`CUST_CAP_PROOF_STATUS`, `CUST_CAP_VALID_TO`, `CUST_CAP_MANIFEST_HASH`

---

## 🧪 Tests (Woche 2)

### Funktional
- [ ] 50–100 Lieferanten durch den Flow → **≥ 95 %** `200 OK` vom Verifier.  
- [ ] Mind. 1 **FAIL** (absichtlicher Sanktionshit) → Writeback zeigt `FAIL`, Workflow simuliert.  
- [ ] **Idempotenz:** gleicher `RUN_ID` überschreibt denselben Datensatz (UPSERT).  

### Sicherheit & Qualität
- [ ] Keine PII in Logs (prüfen!).  
- [ ] HTTPS erfolgreich (self‑signed / `-k` nur in Dev).  
- [ ] mTLS Flag off → funktioniert; (Probe mit on nur Smoke).  
- [ ] CI: Trivy/Grype keine **High/Critical** (sonst Build fail).  

### Performance
- [ ] 100 Lieferanten **< 60 s** (Mock Backend).  
- [ ] P95 Verifier‑Latenz **< 500 ms**.  

---

## 📈 Observability (Minimum)

### Prometheus‑Metriken (Adapter)
- `cap_adapter_verify_requests_total{result="ok|warn|fail"}`  
- `cap_adapter_verify_failures_total{reason="http|schema|policy"}`  
- `cap_adapter_sap_write_total{status="ok|error"}`  
- `cap_adapter_verify_latency_seconds_bucket` (Histogram)  

**/metrics** nur **intern** erreichbar (ClusterIP / ServiceMonitor).

### Grafana Panels (JSON Skizze)
- SingleStat: OK/WARN/FAIL (letzte 24 h)  
- Table: Top Fehlerursachen (Label `reason`)  
- Graph: p95 Latenz pro Endpoint

---

## ⚠️ Fehlerfälle & Idempotenz

- **HTTP 4xx Verifier** → Retry **nicht** automatisch (Fehler persistieren + Alert).  
- **HTTP 5xx Verifier** → Retry mit Backoff (Idempotency‑Key = `RUN_ID|SUPPLIER_ID`).  
- **SAP Writeback Fail** → Retry Queue; keine Duplikate (UPSERT via `RUN_ID`).  
- **Mismatch Policy/IR** → Flow abbrechen, Operator‑Alert.

---

## 🧰 Konfiguration (Adapter)

```yaml
verifier:
  base_url: https://verifier.local:8443
  require_mtls: false
  timeout_ms: 5000
sap:
  odata_base: https://sapdev.local/odata
  auth: oauth2_client_credentials
  writeback:
    mode: z_table   # z_table | bp_extension
    z_service: /sap/opu/odata/Z_CAP_SUPPLIER_STATUS_SRV/StatusSet
security:
  hash_algo: blake3
  redact_logs: true
metrics:
  enabled: true
  bind: 0.0.0.0:9464
run:
  batch_size: 100
  idempotency_key: "${DATE}_${BATCH_NO}"
```

---

## 🧩 Beispiel‑Code (Pseudocode)

**Adapter Writeback (OData POST)**
```python
payload = {
  "SupplierId": supplier_id,
  "RunId": run_id,
  "Status": result,
  "ValidUntil": valid_until[:10],
  "ManifestHash": manifest_hash,
  "RulesJson": json.dumps(trace.get("active_rules", []))
}
resp = requests.post(z_status_url, json=payload, headers=auth)
resp.raise_for_status()
```

**ABAP (alternativ, wenn Adapter nur Datei liefert)**
```abap
" Konsumiert CSV und schreibt Z-Tabelle
DATA: ls_row TYPE zcap_supplier_status.
ls_row-supplier_id = lv_sup.
ls_row-run_id      = lv_run.
ls_row-status      = lv_status.
ls_row-valid_until = lv_valid.
ls_row-manifest_hash = lv_hash.
INSERT zcap_supplier_status FROM ls_row.
IF sy-subrc = 4. MODIFY zcap_supplier_status FROM ls_row. ENDIF.
```

---

## ✅ Akzeptanzkriterien (DoD, Woche 2)
1. E2E‑Pipeline lauffähig: Pull → Verify → Writeback.  
2. Kein Klartext‑PII verlässt SAP/Adapter.  
3. Z‑Tabelle (oder BP‑Extension) enthält Status, Valid‑Until, Manifest‑Hash.  
4. `/metrics` liefert Zähler & Latenz‑Histogramme.  
5. FAIL‑Fall belegbar im Writeback.  
6. CI‑Scans ohne High/Critical Findings.

---

## 📦 Deliverables (Ende Woche 2)
- `sap-adapter/` mit E2E‑Flow & Writeback‑Modul.  
- `config/adapter.yaml` (Produktionsnahe Defaults).  
- `k8s/servicemonitor.yaml` (falls Prometheus Operator).  
- `grafana/panels.json` (Minimal‑Dashboard).  
- `README_E2E.md` (How‑to, Fehlerfälle, KPIs).

---

**Ergebnis:**  
Woche 2 liefert einen **vollständigen, prüfbaren Datenfluss** inklusive **SAP‑Rückschreiben** und **Observability**.  
Damit ist die Basis geschaffen für Woche 3 (Security‑Härtung light, Demo/Pilot‑Vorbereitung).
