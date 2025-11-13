
# 🧱 Week 4 – Tests & Hardening Execution Guide

**Scope:** Integration Tests (IT‑01…IT‑09), A/B‑Äquivalenz (policy_id vs. embedded IR), Compiler‑Benchmarks, Load‑Tests, Cache/ETag‑Verifikation, CI‑Gates & Artefakte.  
**Ziel:** Pilotreife nachweisen (stabil, messbar, reproduzierbar).

---

## 1) Voraussetzungen
- Verifier läuft mit `/verify`, `/policy/compile`, `/policy/:id`, `/healthz`, `/readyz` (HTTPS, OAuth2 Dev‑Token, mTLS optional).
- Policy‑Compiler v1 inkl. deterministischer IR‑Serialisierung & ETag‑Support.
- Demo‑Dataset: **50 Suppliers** (47 OK, 2 WARN, 1 FAIL).

Env‑Variablen (lokal/CI):
```
BASE=https://localhost:8443
TOKEN=<dev-oauth-token>
POLICY_YAML_B64=$(base64 -w0 examples/lksg_v1.policy.yml)   # mac: -b 0
PAYLOAD_OK=$(cat examples/context_ok.json)
PAYLOAD_FAIL=$(cat examples/context_fail.json)
```

---

## 2) Integration Tests – HTTP Flows (IT‑01 … IT‑09)

| ID   | Request                                           | Erwartung (Kernaussage) |
|------|---------------------------------------------------|-------------------------|
| IT‑01| `POST /policy/compile` (strict, gültig)           | 200, `ir`, `policy_hash`, `ir_hash`, `ETag` |
| IT‑02| `POST /policy/compile` (fehlende `legal_basis`)   | 422 + `lints[]` enthält `E1002` |
| IT‑03| `POST /verify` (Policy‑Modus, OK)                  | 200, `result=OK`, `trace.active_rules` ≠ ∅ |
| IT‑04| `POST /verify` (Embedded‑IR, OK)                   | 200, **gleiches** Ergebnis wie IT‑03 |
| IT‑05| `POST /verify` (FAIL‑Fall)                         | 200, `result=FAIL`, `violations[]` gefüllt |
| IT‑06| `GET /policy/:id` + `If‑None‑Match`                | 304 Not Modified |
| IT‑07| Ohne OAuth2                                        | 401 Unauthorized |
| IT‑08| Falscher Scope                                     | 403 Forbidden |
| IT‑09| `persist=true` mit Hash‑Konflikt                   | 409 Conflict |

### Beispiel‑Snippets (curl)
```bash
# IT-01
curl -s -k -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json"   -d "{"policy_yaml":"base64:$POLICY_YAML_B64","lint_mode":"strict","persist":true}"   $BASE/policy/compile | tee /tmp/compile.json | jq .

# IT-03 (Policy-Modus)
curl -s -k -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json"   -d "{"policy_id":"lksg.v1","context":$PAYLOAD_OK,"backend":"mock","options":{"adaptive":true}}"   $BASE/verify | tee /tmp/res_policy.json | jq .result,.trace.active_rules

# IT-04 (Embedded-IR) – nehme IR aus IT-01
IR=$(jq -c .ir /tmp/compile.json)
curl -s -k -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json"   -d "{"ir":$IR,"context":$PAYLOAD_OK,"backend":"mock","options":{"adaptive":true}}"   $BASE/verify | tee /tmp/res_ir.json | jq .result,.trace.active_rules

# IT-06 (304)
ETAG=$(jq -r .etag /tmp/compile.json)
curl -s -k -H "Authorization: Bearer $TOKEN" -H "If-None-Match: $ETAG" $BASE/policy/lksg.v1 -o /dev/null -w "%{http_code}
"
```

### Äquivalenz‑Check (A/B)
```bash
diff <(jq -S '{result,trace:.trace.active_rules,manifest_hash}' /tmp/res_policy.json)      <(jq -S '{result,trace:.trace.active_rules,manifest_hash}' /tmp/res_ir.json)
# Exit 0 erwartet
```

> **DoD Integration:** Alle IT‑Cases grün; A/B‑Diff ist leer.

---

## 3) Compiler‑Benchmarks (Criterion)

**Ziele:** p95 **≤ 50 ms** (warm), **≤ 200 ms** (cold); Memory < 64 MiB.

**Bench‑Skeleton (`benches/compile_bench.rs`):**
```rust
use criterion::{criterion_group, criterion_main, Criterion, black_box};
fn compile_cold(pol: &str) { /* parse+lint+ir+hash (no cache) */ }
fn compile_warm(ctx: &mut WarmCtx, pol: &str) { /* with cache */ }
fn bench_compile(c: &mut Criterion) {
    let pol = include_str!("../examples/lksg_v1.policy.yml");
    c.bench_function("compile_cold", |b| b.iter(|| compile_cold(black_box(pol))));
    let mut ctx = WarmCtx::new();
    c.bench_function("compile_warm", |b| b.iter(|| compile_warm(&mut ctx, black_box(pol))));
}
criterion_group!(benches, bench_compile);
criterion_main!(benches);
```
**Run & Artefakte:**
```bash
cargo bench --bench compile_bench
# Exportiere CSV/HTML Reports nach bench/reports/*
```

> **DoD Bench:** Reports im Repo; p95‑Ziele erfüllt.

---

## 4) Load‑Tests (k6) – `/verify`

**Ziele:** **50 RPS**, **p95 < 500 ms**, Error‑Rate < 1 % (Mock‑Backend).

**k6‑Script `k6/verify.js`:**
```js
import http from 'k6/http'; import { sleep } from 'k6';
export const options = { vus: 25, duration: '3m' };
export default function () {
  const payload = JSON.stringify(JSON.parse(open('../examples/context_ok.json')));
  const params = { headers: { Authorization: `Bearer ${__ENV.TOKEN}` }, timeout: '5s' };
  http.post(`${__ENV.BASE}/verify`, payload, params);
  sleep(0.1);
}
```
**Run:**
```bash
BASE=$BASE TOKEN=$TOKEN k6 run k6/verify.js | tee reports/load_week4.txt
```

> **DoD Load:** p95 < 500 ms, <1 % Errors; Report & ggf. Grafana‑Screenshots committen.

---

## 5) Cache/ETag‑Verifikation

- **LRU Cache ≥ 1000**: künstlich 1000 Policies kompilieren → älteste wird verdrängt.  
- **ETag/304**: Wiederholte `GET /policy/:id` mit `If‑None‑Match` → 304‑Rate protokollieren.  
- Ziel: **Cache‑Hit‑Rate > 90 %** in Repeat‑Suite.

**Quick‑Loop:**
```bash
for i in $(seq 1 100); do curl -s -k -H "Authorization: Bearer $TOKEN" $BASE/policy/lksg.v1 -I; done   | grep ETag | sort | uniq -c
```

---

## 6) CI‑Gates (GitHub Actions – Auszug)

```yaml
jobs:
  test-hardening:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --workspace --all-features

      # Contract
      - run: schemathesis run openapi/verifier.v1.yaml --base-url=$BASE -c --checks all

      # Determinismus‑Guard
      - run: ./ci/non_determinism_check.sh

      # Benchmarks
      - run: cargo bench --bench compile_bench
      - uses: actions/upload-artifact@v4
        with: { name: bench, path: bench/** }

      # Load (optional in nightly)
      - run: BASE=$BASE TOKEN=$TOKEN k6 run k6/verify.js
      - uses: actions/upload-artifact@v4
        with: { name: reports, path: reports/** }
```

---

## 7) Artefakte & Doku
- `reports/load_week4.txt` (k6 Output)  
- `bench/**` (Criterion Reports)  
- `IT_RESULTS.md` (IT‑01 … IT‑09 Log & Status)  
- **Hardening‑Report** (2–3 Seiten, Template optional)

---

## 8) Definition of Done (Week 4)
1. **Alle IT‑Cases grün** + A/B‑Äquivalenz nachgewiesen.  
2. Contract‑Suite (Schemathesis) ohne Fehler.  
3. Benchmarks: **p95 warm ≤ 50 ms**, **cold ≤ 200 ms**.  
4. Load‑Test `/verify`: **50 RPS, p95 < 500 ms, < 1 % Errors**.  
5. Cache/ETag: 304‑Treffer messbar, **Hit‑Rate > 90 %** (Repeat).  
6. Artefakte committed, **Hardening‑Report** im Repo.  
7. Keine PII in Logs; TLS/mTLS/OAuth2 konfigurierbar.

---

## 9) Optional (0.5 Tag): `proof adapt` Mini‑Stub
```rust
pub trait RuleSelector { fn select(&self, ir: &IR, ctx: &Context) -> SelectedRules; }
pub struct AdaptiveOrchestrator<S: RuleSelector> { selector: S }
impl<S: RuleSelector> AdaptiveOrchestrator<S> {
  pub fn plan(&self, ir:&IR, ctx:&Context) -> Plan { /* deterministische Reihenfolge */ }
}
```
CLI‑Dry‑Run: `proof adapt --policy lksg.v1 --dry-run` → listet aktivierte Regeln (aus IR).

---

**Ergebnis:** Nach Week 4 ist der Stack **messbar stabil** (IT/Contract/Bench/Load) und **pilotfähig**. Nächster logischer Schritt: `proof adapt` Umsetzung und v1.1‑Erweiterungen.
