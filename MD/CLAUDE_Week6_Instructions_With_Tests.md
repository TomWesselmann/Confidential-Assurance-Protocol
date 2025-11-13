
# 🧭 Week 6 — Claude Instruction Pack (Go‑Live Cutover & Adaptive Enforcement)

**Kontext:** Week 5 ist komplett. Ziel von Week 6: **sauberes Go‑Live** (Cutover/ORR) + **Adaptive Orchestrator von Dry‑Run → Enforce** unter kontrollierter Beobachtung. Alles mit klaren Tests, Kommandos, Artefakten und DoD.

> Arbeitsmodus: Produktionsreif, fail‑closed, **keine PII in Logs**, deterministische Outputs, reproduzierbare Tests.


---

## 🎯 Ziele (Ende Week 6)
- **Production Cutover** nach ORR: Helm‑Deploy **prod** grün, Canary‑Rollout + Smoke erfolgreich.
- **Adaptive Enforcement v0.2**: `--enforce` Flag & progressive Aktivierung (0%→25%→100%) mit **Drift‑Metrik**.
- **Pilot‑E2E (SAP Adapter)**: Live‑OData/Writeback Testfall, Audit‑Trail, Idempotenz verifiziert.
- **DR/Rotation**: Backup/Restore Drill + KMS‑Key‑Rotation (signierte Registry/IR).


---

## 🧱 Track A — Production Cutover (ORR, Deploy, Smoke)

### A1) ORR‑Checkliste (Operational Readiness Review)
**Implementieren**: `docs/ORR_Checklist.md`
- **Infra**: HPA, Ingress, NetworkPolicy, PodSecurity, ResourceLimits
- **Security**: OAuth2 (IdP prod), mTLS an/aus per Env, Secrets rotationable
- **Observability**: Dashboards/Alerts laden, SLOs dokumentiert
- **Runbooks**: Auth/mTLS, Rollback, Cache‑Flush, Incident Escalation

**Akzeptanz**: ORR Dokument **vollständig**, von Tech‑Lead gezeichnet.

### A2) Prod‑Deploy (Helm)
**Kommandos**
```bash
helm upgrade --install cap helm/ -f helm/values-prod.yaml --wait --timeout 10m
kubectl -n cap get deploy,po,svc,ingress,hpa
```

**Smoke (curl)**
```bash
curl -s -k $BASE/healthz
curl -s -k -H "Authorization: Bearer $TOKEN" $BASE/readyz
curl -s -k -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json"   -d '{"policy_id":"lksg.v1","context":{"_demo":true},"backend":"mock"}' $BASE/verify | jq .result
```

**Akzeptanz**: Alle Smokes 200; HPA skaliert bei Lasttest; keine 5xx in 15 Minuten Canary.


---

## 🧠 Track B — Adaptive Orchestrator v0.2 (Enforce + Drift‑Metrik)

### B1) Enforce‑Modus
**Implementieren**
- `src/orchestrator/enforcer.rs`
```rust
pub struct EnforceOptions { pub enforce: bool, pub rollout_percent: u8, pub drift_max_ratio: f64 }
pub struct VerdictPair { pub shadow: Verdict, pub enforced: Verdict }

pub fn decide(ir: &IR, ctx: &Context, opts: &EnforceOptions) -> VerdictPair {
    // 1) Shadow: berechne Plan + Result ohne Wirkung
    // 2) Enforce: wenn rollout_percent greift, nutze Plan aktiv
    // 3) Ergebnispaar + Telemetrie (Drift = shadow != enforced)
    #![allow(unused)]
    VerdictPair { shadow: Verdict::Ok, enforced: Verdict::Ok }
}
```
- **CLI**: `proof adapt --policy lksg.v1 --context ctx.json --enforce --rollout 25 --drift-max 0.005`

**Metriken**
- `adapt_enforce_rollout_percent`
- `adapt_drift_events_total{policy_id}`
- `adapt_drift_ratio` (Gauge, 5m rolling)

### B2) Drift‑Analyse
**Implementieren**
- `src/orchestrator/drift.rs`  
  - Drift = Anteil Requests, bei denen `shadow_verdict != enforced_verdict`
  - Ring‑Puffer über 5/15/60 Minuten; Export als Metrics

**Akzeptanz**
- Bei **rollout 0%**: Drift=0; bei **25%**: Drift gemessen; **Gate**: `adapt_drift_ratio <= drift_max_ratio`

### Tests
- `tests/orchestrator_enforce.rs`
  - `rollout_zero_shadow_only()` → enforced==shadow, drift=0
  - `rollout_partial_with_equal_verdicts()` → drift≈0
  - `rollout_partial_with_forced_difference()` → drift>0, Gate triggert

**Run**
```bash
cargo test --test orchestrator_enforce -- --nocapture
```


---

## 🔗 Track C — Pilot E2E (SAP Adapter)

### C1) Live‑OData‑Zugriff + Writeback
**Kommandos**
```bash
# Pull demo suppliers
cap-adapter pull --odata $SAP_URL --client $SAP_CLIENT --out context.json

# Verify (Mode A)
curl -s -k -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json"   -d "{"policy_id":"lksg.v1","context":$(cat context.json),"backend":"mock"}"   $BASE/verify | tee /tmp/verify.json

# Writeback
cap-adapter writeback --in /tmp/verify.json --odata $SAP_URL --table Z_CAP_SUPPLIER_STATUS --idempotency RUN_$(date +%s)
```

**Akzeptanz**
- **50/50** Datensätze geschrieben, **keine Duplikate** bei erneutem Lauf (gleicher RUN_ID).
- Audit‑Trail: `manifest_hash`, `policy_hash`, `ir_hash` gespeichert.

### C2) Rate‑Limit & Retries
- Adapter respektiert 429 (Retry‑After) und Backoff‑Policy (exp jitter).

**Tests**
- `tests/adapter_pilot.rs` (ignored/IT): schreibt Demo‑Datensätze in Staging; prüft Idempotenz.
```bash
cargo test --test adapter_pilot -- --ignored --nocapture
```


---

## 🔁 Track D — Backup/Restore Drill & Key‑Rotation

### D1) Backup/Restore
- Sichern: IR‑Registry (JSON/SQLite), Policy‑Blobs, Configs.
- Restore Drill in **leerer** Namespace; 200 OK auf `/readyz`, ETag gleich.

**Akzeptanz**: `GET /policy/:id` liefert identisches `ir_hash` und `ETag` wie vor dem Backup.

### D2) KMS‑Key‑Rotation
- Rotations‑Prozedur dokumentieren; KID‑Wechsel in Signaturen; Kompat‑Test (alt+neu akzeptiert).

**Tests**
- `tests/rotation.rs` (unit): Verify akzeptiert alte **und** neue Schlüssel bis Decommission‑Zeitpunkt.


---

## 🧪 Gesamt‑Testplan (Week 6)

**Unit**
- `orchestrator_enforce.rs`, `drift.rs`, `rotation.rs`

**Integration**
- ORR Smoke (Script), Prod Helm Dry‑Run, Canary Rollout, Adapter Pilot

**Contract**
- OpenAPI Update: `securitySchemes` + Scopes final; Schemathesis erneut grün

**Load (Spot)**
- 10–20 RPS mit `--enforce` und rollout=25% → p95 < 600ms, Error < 1%

**DoD (Week 6)**
1. ORR unterschrieben, prod Canary & Smoke **grün**.  
2. `--enforce` verfügbar; rollout 0%→25%→100% ohne KPI‑Bruch.  
3. Drift‑Metriken live; `adapt_drift_ratio <= drift_max_ratio`.  
4. Pilot E2E: OData Pull → Verify → Writeback **idempotent**.  
5. Restore‑Drill & Key‑Rotation Tests bestanden.


---

## 📂 Artefakte, die Claude erstellen/ändern soll
- `docs/ORR_Checklist.md`
- `src/orchestrator/{enforcer.rs,drift.rs}` (+ Export in lib)
- `src/bin/proof.rs` (Flags: `--enforce`, `--rollout`, `--drift-max`)
- `tests/{orchestrator_enforce.rs,adapter_pilot.rs,rotation.rs}`
- `helm/values-prod.yaml` (FeatureFlags: enforce on/off, rollout%)
- `grafana/dashboards/verifier.json` (Panels für Drift/Enforce ergänzen)
- `docs/{runbook_rollout.md,runbook_restore.md,runbook_rotation.md}`
- `openapi/openapi.yaml` (SecuritySchemes final; scopes auf Endpunkte)

---

## ▶️ Befehle (Sammelblock)
```bash
# Unit
cargo test --test orchestrator_enforce -- --nocapture
cargo test --test rotation -- --nocapture

# Pilot (ignored)
cargo test --test adapter_pilot -- --ignored --nocapture

# Deploy & Smoke
helm upgrade --install cap helm/ -f helm/values-prod.yaml --wait
./scripts/smoke_prod.sh

# Contract
schemathesis run openapi/openapi.yaml --base-url=$BASE -c --checks all

# Enforce (25%)
proof adapt --policy lksg.v1 --context examples/context_ok.json --enforce --rollout 25 --drift-max 0.005 -o plan.json
```

---

**Erwartetes Ergebnis:** Nach Week 6 bist du **live** mit kontrolliert aktivierter adaptiver Durchsetzung. Risiken werden über **Drift‑Metrik** und **progressive Rollouts** gemanagt. Pilot‑E2E und DR‑/Rotation‑Drills sichern Betrieb & Compliance ab.
