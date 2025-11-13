# WEEK7_P (optional) — /proof/adapt API

**Ziel:** Orchestrator als HTTP‑API, A/B‑gleich zur CLI `proof adapt`.

## Endpunkte (Skizze)
- `POST /proof/adapt/plan` → Input: `policy_id|embedded_ir`, `context`, `selector`, `rollout`, `drift_max`; Output: Plan + Kosten
- `POST /proof/adapt/run` → Shadow/Enforce ausführen; Output: `verdict_pair`, `drift`, Metriken

## Akzeptanzkriterien (DoD)
1. **A/B‑Äquivalenz:** gleiche Inputs → gleiche Pläne/Ergebnisse (CLI vs. API)
2. **Latenz:** p95 < 200ms
3. **Security:** Scopes (z. B. `adapt:plan`), Fehlercodes ohne PII

## Tests & Befehle
```bash
cargo run --bin cap-verifier-api &
cargo test --test adapt_http_it -- --ignored --nocapture

# (Optional) Contract-Tests falls OpenAPI erweitert wurde
# schemathesis run openapi/openapi.yaml --base-url=$BASE -c --checks all -E '/proof/adapt'
```

## Dateien (neu/ändern)
```
src/api/adapt.rs
tests/adapt_http_it.rs
openapi/openapi.yaml   # optional erweitern
```