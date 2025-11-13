
# 🚀 Week 5 Plan — Go‑Live Hardening + Adaptive Proof Orchestrator v0.1

**Context:** Week 4 finished (100% DoD met, KPIs exceeded). This sprint splits work into **Track A (Go‑Live Security & Ops)** and **Track B (Adaptive Orchestrator v0.1)** so you can ship a production‑ready core while seeding the next product differentiator.

---

## 🎯 Goals (End of Week 5)

- **Track A — Go‑Live Security & Ops**
  - Real **OAuth2** (IdP integration) + scope enforcement.
  - **TLS/mTLS** with proper cert management (no self‑signed), secrets rotation.
  - **Monitoring**: Prometheus metrics finalized, **Grafana dashboards** & alerts.
  - **Runbooks**: Auth/mTLS troubleshooting, rollout/rollback, cache operations.
  - **Helm values** for staging/prod, SLOs documented.

- **Track B — Adaptive Proof Orchestrator v0.1**
  - IR‑driven rule selection (deterministic) based on **predicates** and a **simple cost model**.
  - **CLI** (`proof adapt --policy <id> --context <file> --dry-run`) emits **SelectedRules** + **Plan JSON**.
  - Telemetry (selected rules count, selection latency), golden tests, and integration proof with `/verify`.

---

## 🧭 Track A — Go‑Live Security & Ops

### A1) OAuth2 (Real IdP)
- **Tasks**
  - Client‑credentials flow (token endpoint, JWKS cache, issuer/audience validation).
  - Scope mapping: `verify:run`, `policy:compile`, `policy:read` (configurable).
  - JWKS rotation handling, 5xx fail‑closed + retry.
- **Acceptance**
  - `401/403` behave correctly; tokens from IdP pass; expired/issuer‑mismatch fail.
  - Contract tests extended: IT‑07/08 run **against real IdP** (staging).
- **Artifacts**
  - `config/auth.yaml` (issuer, audience, scopes, cache TTL).
  - `docs/runbook_oauth2.md`.

### A2) TLS/mTLS (Prod)
- **Tasks**
  - Certs via cluster secret store; **mTLS optional** per env; min TLS 1.2 (prefer 1.3).
  - CRL/OCSP stapling (if ingress supports), cipher policy documented.
- **Acceptance**
  - mTLS on → client w/o cert = 403; with wrong SAN/CA = 403.
  - Secrets rotation manual test (new cert live without restart where supported).
- **Artifacts**
  - `k8s/ingress.yaml`, `values-prod.yaml` (cert refs), `docs/runbook_mtls.md`.

### A3) Monitoring & SLOs
- **Tasks**
  - Finalize metrics (counters/histograms) for **compiler**, **verifier**, **adapter**.
  - Grafana: SingleStat OK/WARN/FAIL, p95/p99 latency, error rate, cache hit‑rate.
  - Alerts: error‑rate >1% 5m; p95 verify >500ms 5m; 5xx spike; cache hit‑rate <80%.
- **Acceptance**
  - Dashboards render with demo traffic; alert rules fire in sandbox.
- **Artifacts**
  - `grafana/dashboards/*.json`, `prometheus/alerts.yaml`, `docs/slo.md`.

### A4) Helmization & Config Hygiene
- **Tasks**
  - Values per env (dev/stage/prod), feature flags (embedded‑IR allowed?), rate limits.
  - Secrets through sealed‑secrets/external secrets.
- **Acceptance**
  - `helm upgrade --install` in stage works end‑to‑end; rollback verified.
- **Artifacts**
  - `helm/Chart.yaml`, `helm/values-*.yaml`, `docs/deploy.md`.

---

## 🧠 Track B — Adaptive Proof Orchestrator v0.1

### B1) Interfaces
```rust
pub struct SelectedRules { pub active: Vec<String> }
pub trait RuleSelector {
    fn select(&self, ir: &IR, ctx: &Context) -> SelectedRules;
}
pub struct BasicSelector; // predicate‑only
pub struct WeightedSelector { pub weights: HashMap<String, u32> } // simple cost model
```

### B2) Deterministic Planner
```json
{
  "policy_id": "lksg.v1",
  "active_rules": ["no_sanctions","no_conflict_regions","audit_fresh"],
  "order": ["no_sanctions","no_conflict_regions","audit_fresh"],
  "cost": {"no_sanctions":1,"no_conflict_regions":1,"audit_fresh":3}
}
```
- Stable sort by (cost asc, rule_id asc).
- Tie‑breakers deterministic; no randomness.
- Emit trace fields: `reason` (predicate ids), `cost_sum`.

### B3) CLI & API
- **CLI**
```bash
proof adapt --policy lksg.v1 --context examples/context_ok.json --selector weighted --weights rules.yaml --dry-run -o plan.json
```
- **API (optional)**
  - `POST /proof/adapt` → returns `Plan`; **not** required for v0.1.

### B4) Telemetry & Logs
- Counters: `adapt_selected_rules_total`, `adapt_plans_total{selector}`, `adapt_selection_failures_total`.
- Histogram: `adapt_selection_latency_seconds`.
- Logs redact context; include policy/ir hash and active rule ids.

### B5) Tests
- **Unit**: predicates activation; weighted order deterministic.
- **Golden**: fixed `plan.json` for known contexts.
- **Integration**: `/verify` verdict equals non‑adaptive path for equivalent rule sets.

### B6) Benchmarks
- Selection p95 < 5ms for typical policy; negligible vs. verify path.

---

## 📅 Suggested Schedule
- **Day 1–2**: OAuth2 IdP, scopes, IT updates (A1).  
- **Day 2–3**: TLS/mTLS & Helm values (A2, A4).  
- **Day 3–4**: Monitoring dashboards + alerts (A3).  
- **Day 4–5**: Orchestrator interfaces + Basic/Weighted selectors (B1–B3), tests (B5).  
- **Day 5**: Telemetry, micro‑benchmarks, docs & demo (B4, B6).

---

## ✅ Week 5 DoD
- **Security/Ops**: Real OAuth2 + scopes enforced; mTLS working; Helm deploy OK in staging; dashboards & alerts live; SLOs written.
- **Orchestrator v0.1**: `proof adapt --dry-run` produces deterministic plan from IR + context; tests green; selection p95 < 5ms; logs/metrics clean.

---

## 📦 Deliverables
- `src/orchestrator/{selector.rs,planner.rs}`
- `src/bin/proof.rs` (CLI)
- `grafana/dashboards/*.json`, `prometheus/alerts.yaml`
- `helm/values-{dev,stage,prod}.yaml`
- `docs/{slo.md,deploy.md,runbook_oauth2.md,runbook_mtls.md}`
- `examples/plan_ok.json` (golden)

---

## ⚠️ Risks & Mitigation
- **IdP delays** → dev‑token fallback but prod profile requires real OAuth2.
- **Metric cardinality** → limit labels; use traces for per‑rule detail.
- **Scope creep** → v0.1 stays dry‑run/read‑only.

---

**Outcome:** End of Week 5 = secure, observable verifier + first adaptive planning demo. Perfect for stakeholder sign‑off and IR v1.1 discussions.
