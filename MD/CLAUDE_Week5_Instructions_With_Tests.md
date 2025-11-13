
# 🧭 Week 5 — Claude Instruction Pack (mit Tests & DoD)
**Kontext:** Week 4 ist abgeschlossen (IT/Contract/Bench/Load/Cache/ETag ✅).  
**Ziel dieser Datei:** Konkrete, umsetzbare Anweisungen für **Claude Code** mit **Tests, Akzeptanzkriterien, Kommandos** und **Artefakten** für **Go‑Live‑Härtung** + **Adaptive Proof Orchestrator v0.1**.

> **Arbeitsmodus:** Schreibe produktionsreifen Rust‑Code mit klarer Modulstruktur, Tests und Dokumentation. Bei Unsicherheit: fail‑closed, keine PII in Logs.

---

## 🔐 Track A — Go‑Live Security & Ops

### A1) OAuth2 (Real IdP, RS256/JWKS)
**Aufgabe:** Implementiere eine Auth‑Middleware für `cap-verifier-api`, die RS256‑JWTs prüft (issuer, audience, exp/nbf), JWKS cached, Scopes erzwingt.

**Implementieren**
- `src/auth/mod.rs`  
  - `fn validate_token(token: &str, cfg: &AuthConfig) -> Result<Claims, AuthError>`  
  - JWKS Cache (TTL konfigurierbar), Key‑Rotation tolerieren.
  - Scopes: `verify:run`, `policy:compile`, `policy:read` (aus `auth.yaml`).  
- `src/http/middleware/auth.rs`
  - Extrahiere `Authorization: Bearer` → `validate_token`  
  - Mappe Scopes auf Endpunkte:  
    - `/verify`: `verify:run`  
    - `/policy/compile`: `policy:compile`  
    - `/policy/:id`: `policy:read`

**Konfiguration**
- `config/auth.yaml`
```yaml
issuer: "https://idp.example.com"
audience: "cap-verifier"
jwks_url: "https://idp.example.com/.well-known/jwks.json"
jwks_cache_ttl_sec: 600
required_scopes:
  verify: ["verify:run"]
  policy_compile: ["policy:compile"]
  policy_read: ["policy:read"]
```

**Tests**
- `tests/auth_jwt.rs`
  - `RS256_ok_valid_token()` → 200 für `/verify`
  - `expired_token_rejected()` → 401
  - `issuer_or_aud_mismatch()` → 401
  - `missing_scope_for_endpoint()` → 403
  - JWKS Rotation: alter JWK → 401, neuer JWK → 200

**Akzeptanzkriterien**
- Korrekte 401/403 Pfade, deterministische Fehlermeldungen ohne PII.  
- Tokenprüfung innerhalb < 5ms (warm; JWKS gecached).

**Run**
```bash
cargo test --test auth_jwt -- --nocapture
```

---

### A2) TLS/mTLS (Prod‑fähig)
**Aufgabe:** TLS via Ingress/Native TLS; mTLS pro Umgebung aktivierbar.

**Implementieren**
- `config/tls.yaml`
```yaml
require_mtls: true
tls_min_version: "1.2"
cipher_profile: "modern"
client_ca_bundle: "/etc/ssl/clients/ca.crt"
```
- Serverkonfig: Wenn `require_mtls=true` → verweigere Requests ohne Client‑Zertifikat.

**Tests (Integration)**
- `tests/tls_mtls.rs`
  - `mtls_required_without_cert()` → 403
  - `mtls_required_with_wrong_san()` → 403
  - `mtls_required_with_valid_cert()` → 200

**Run (Beispiel, curl)**
```bash
curl -s -k --cert client.crt --key client.key https://localhost:8443/verify
```

**Akzeptanzkriterien**
- mTLS erzwingbar; sichere Cipher‑Suites; keine Secrets in Logs.

---

### A3) Monitoring & SLOs
**Aufgabe:** Finalisiere Prometheus‑Metriken & Grafana Dashboards.

**Implementieren**
- Zähler/Histogramme ergänzen:
  - `cap_verifier_requests_total{result}`
  - `cap_verifier_request_duration_seconds_bucket`
  - `cap_auth_token_validation_failures_total`
  - `cap_cache_hit_ratio` (Gauge)
- `grafana/dashboards/verifier.json` (4 Panels): OK/WARN/FAIL, p95, error rate, cache hit‑rate.  
- `prometheus/alerts.yaml`: error‑rate>1%/5m, p95>500ms/5m, 5xx‑Spike, cache‑hit<80%

**Tests**
- `tests/metrics_export.rs`: `/metrics` enthält oben genannte Metriken; Hist‑Buckets > 0 nach Traffic.

**Run**
```bash
cargo test --test metrics_export -- --nocapture
```

**Akzeptanzkriterien**
- Dashboards rendern; Alerts funktional in Sandbox.

---

### A4) Helmization & Deploy
**Aufgabe:** Parameterisiere Deployments; sichere Secrets‑Handhabung.

**Implementieren**
- `helm/Chart.yaml`, `helm/values-{dev,stage,prod}.yaml`  
- Feature Flags: `allow_embedded_ir`, `require_mtls`, Ratenlimits.  
- Secrets via Sealed/External‑Secrets.

**Test**
```bash
helm upgrade --install cap cap-chart/ -f helm/values-stage.yaml --dry-run
```

**Akzeptanzkriterien**
- Stage‑Deploy lauffähig; Rollback verifiziert.

---

## 🧠 Track B — Adaptive Proof Orchestrator v0.1

### B1) Interfaces & Selektoren
**Aufgabe:** IR‑gesteuerte Aktivierung aktiver Regeln + deterministische Reihenfolge.

**Implementieren**
- `src/orchestrator/selector.rs`
```rust
pub struct SelectedRules { pub active: Vec<String> }
pub trait RuleSelector { fn select(&self, ir: &IR, ctx: &Context) -> SelectedRules; }

pub struct BasicSelector;
pub struct WeightedSelector { pub weights: std::collections::HashMap<String, u32> }
```
- `src/orchestrator/planner.rs`
```rust
pub struct Plan { pub policy_id: String, pub active_rules: Vec<String>, pub order: Vec<String>, pub cost: std::collections::HashMap<String,u32> }
pub fn plan_deterministic(active: &[String], weights: &std::collections::HashMap<String,u32>) -> Plan { /* sort by (cost, rule_id) */ }
```

**CLI**
- `src/bin/proof.rs`
```bash
proof adapt --policy lksg.v1 --context examples/context_ok.json --selector weighted --weights examples/rule_weights.yaml --dry-run -o plan.json
```

**Tests**
- `tests/orchestrator_unit.rs`
  - Predicates aktivieren richtige Menge (aus IR).  
  - Weighted‑Order deterministisch.  
- `tests/orchestrator_golden.rs`
  - Golden `examples/plan_ok.json` (Byte‑gleich).

**Akzeptanzkriterien**
- `proof adapt --dry-run` erzeugt reproduzierbaren Plan in < 5ms p95.

---

## 🧪 Gesamt‑Testplan (Week 5)

**Unit/IT**
- `auth_jwt.rs`, `tls_mtls.rs`, `metrics_export.rs`, `orchestrator_unit.rs`, `orchestrator_golden.rs`  
- Erweiterung Integration IT‑07/IT‑08 gegen realen IdP (Staging).

**Contract**
- OpenAPI aktualisieren (Scopes), Schemathesis erneut laufen lassen.

**Load (Spot‑Check)**
- 10–20 RPS Smoke mit aktivem OAuth2/mTLS → p95 < 600ms.

**DoD (Week 5)**
1. Real OAuth2 wirkt (401/403/200 korrekt; Rotation ok).  
2. mTLS produktionsreif (403 für fehlende/falsche Certs).  
3. Dashboards & Alerts live (Sandbox), SLOs dokumentiert.  
4. Helm‑Deploy in Staging grünes Ende‑zu‑Ende.  
5. `proof adapt --dry-run` erzeugt deterministischen Plan (Unit/Golden grün).

---

## 📂 Artefakte, die Claude erzeugen soll
- `src/auth/{mod.rs, errors.rs}`  
- `src/http/middleware/auth.rs`  
- `config/{auth.yaml,tls.yaml}`  
- `grafana/dashboards/verifier.json`, `prometheus/alerts.yaml`  
- `helm/{Chart.yaml,values-dev.yaml,values-stage.yaml,values-prod.yaml}`  
- `src/orchestrator/{selector.rs,planner.rs}`  
- `src/bin/proof.rs`  
- Tests: `tests/{auth_jwt.rs,tls_mtls.rs,metrics_export.rs,orchestrator_unit.rs,orchestrator_golden.rs}`  
- Update: `openapi/openapi.yaml` (Scopes, 401/403)  
- Doku: `docs/{deploy.md,runbook_oauth2.md,runbook_mtls.md,slo.md}`

---

## ▶️ Befehle (Sammelblock)
```bash
# Unit
cargo test --test auth_jwt -- --nocapture
cargo test --test tls_mtls -- --nocapture
cargo test --test metrics_export -- --nocapture
cargo test --test orchestrator_unit -- --nocapture
cargo test --test orchestrator_golden -- --nocapture

# Contract
schemathesis run openapi/openapi.yaml --base-url=$BASE -c --checks all

# Helm (Stage)
helm upgrade --install cap helm/ -f helm/values-stage.yaml --dry-run

# Orchestrator
cargo run --bin proof -- adapt --policy lksg.v1 --context examples/context_ok.json --selector weighted --weights examples/rule_weights.yaml --dry-run -o plan.json
```

---

## ✅ Übergabeblock (an Claude)
> **Bitte implementiere exakt die oben spezifizierten Dateien, Tests und Konfigurationen.**  
> **Fokuspunkte:** Security fail‑closed, deterministische Outputs, saubere Logs (ohne PII), reproduzierbare Tests.  
> **Abschluss:** Führe alle Test‑Befehle aus und erstelle eine kurze `WEEK5_SUMMARY.md` mit Ergebnissen/KPIs/Screenshots‑Hinweisen.
