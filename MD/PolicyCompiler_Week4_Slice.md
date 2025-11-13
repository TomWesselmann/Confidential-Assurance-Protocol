# 🧱 Week‑4 Slice – Policy‑Compiler & Verifier Hardening (Production)

**Ziel (5–7 Tage):**  
Produktionstaugliche Härtung und Nachweise: **Integration‑Tests der HTTP‑Flows**, `/verify` mit **eingebettetem IR**, **OpenAPI Contract‑Tests (Schemathesis)**, **LRU‑Cache + ETag**, **Benchmarks** & **Load‑Tests**. Abschluss mit **Hardening‑Report**.

**Voraussetzungen:** Week‑1 (Spec/Parser/Linter/Hashing) ✅, Week‑2 (Builtins/Adaptivity) ✅, Week‑3 (OpenAPI/Determinismus/Caching‑Design) ✅.

---

## ✅ Deliverables (Ende Woche 4)
- **Integration‑Tests** (Happy‑Path + Fehlercodes) über HTTP gegen Test‑Server
- **`/verify`** akzeptiert **IR eingebettet** *und* `policy_id` → **identische Ergebnisse**
- **Schemathesis Contract‑Suite** grün (OpenAPI v1) – inkl. Fehlerfälle
- **LRU Cache (≥1000)** basierend auf `policy_hash` – mit **ETag** & `If‑None‑Match` (304)
- **Benchmarks**: Compile **p95 ≤ 50 ms (warm)**, **≤ 200 ms (cold)**, Memory < 64 MiB
- **Load‑Tests** `/verify` (Mock‑Backend): **50 RPS**, **p95 < 500 ms**, Error‑Rate < 1 %
- **Hardening‑Report (2–3 Seiten)**: Ergebnisse, KPIs, offene Punkte, Empfehlungen

---

## 🧱 Architektur‑Skizze (Week‑4 Fokus)
```
[Policy Compiler] <—LRU—> [Registry/Cache (ETag)]
       │                          │
       ├── POST /policy/compile   ├── GET /policy/:id  (If-None-Match → 304)
       │                          │
[Adapter] ── POST /verify ──> [Verifier]  (A) policy_id  (B) embedded IR → equivalence
           (OAuth2/mTLS)          │
                                  └── Metrics/Logs (no PII)
```

---

## 🔌 API‑Verträge (Erinnerung)

### `/verify` – zwei Modi (Äquivalenz gefordert)
- **A: Policy‑Modus**
```json
{
  "policy_id": "lksg.v1",
  "context": { "..."},
  "backend": "mock",
  "options": {"adaptive": true}
}
```
- **B: Embedded‑IR‑Modus**
```json
{
  "ir": { "... IRv1 ..." },
  "context": { "..."},
  "backend": "mock",
  "options": {"adaptive": true}
}
```
**DoD:** Ergebnisse (`result`, `trace.active_rules`, `manifest_hash`) sind **bit‑gleich** bis auf erwartete Felder (z. B. `policy_id` vs. none).

---

## 🧪 Integration‑Tests (HTTP Flows)

**Matrix (Mindestumfang):**
| Fall | Request | Erwartung |
|---|---|---|
| IT‑01 | `/policy/compile` gültig (strict) | 200, `ir`, `policy_hash`, `ir_hash`, ETag |
| IT‑02 | `/policy/compile` fehlende `legal_basis` | 422 + `lints[]` mit `E1002` |
| IT‑03 | `/verify` Policy‑Modus OK | 200, `result=OK`, Trace vorhanden |
| IT‑04 | `/verify` Embedded‑IR OK | 200, **ergibt dasselbe** wie IT‑03 |
| IT‑05 | `/verify` FAIL (Sanktions‑Treffer) | 200, `result=FAIL`, `violations[]` gefüllt |
| IT‑06 | `/policy/:id` + `If‑None‑Match` | 304 Not Modified |
| IT‑07 | OAuth2 fehlend | 401 |
| IT‑08 | Scope‑Mismatch | 403 |
| IT‑09 | Policy‑Konflikt (persist=true, hash differiert) | 409 |

**Werkzeugvorschlag:** `pytest` + `httpx`/`reqwest` + test‑containers (Docker).

---

## 📏 Contract‑Tests (Schemathesis)

```bash
schemathesis run openapi/verifier.v1.yaml   --base-url=https://localhost:8443 -c   --validate-schema=true   --checks all
```
**DoD:** Keine schema‑fremden Responses; Fehlercodes korrekt (400/401/403/409/422/500).

---

## 🗄️ LRU‑Cache & ETag

**Ziel:** Entlastung der Compiler‑Pfad & Netzwerk‑Saves bei `GET /policy/:id`.

**Key:** `policy_hash` → **IR (canonical JSON)**  
**Size:** ≥ **1000** Einträge, LRU‑Eviction  
**ETag‑Format:** `W/"ir:<ir_hash>"`

**Rust‑Skizze:**
```rust
use lru::LruCache;
use std::num::NonZeroUsize;
struct IrCache { inner: LruCache<String, Arc<Ir>> }
impl IrCache {
  fn new() -> Self { Self { inner: LruCache::new(NonZeroUsize::new(1000).unwrap()) } }
  fn get_or_insert(&mut self, policy_hash: &str, loader: impl FnOnce()->Ir) -> Arc<Ir> {
    if let Some(ir) = self.inner.get(policy_hash) { return Arc::clone(ir); }
    let ir = Arc::new(loader());
    self.inner.put(policy_hash.to_owned(), Arc::clone(&ir));
    ir
  }
}
```
**ETag‑Flow:**  
1) Client sendet `If‑None‑Match` → compare mit `ir_hash` → 304 falls identisch.  
2) Sonst 200 + `ETag: W/"ir:<ir_hash>"`.

---

## ⏱️ Benchmarks (Compiler)

**Kriterien:** p95 **≤ 50 ms** warm, **≤ 200 ms** cold; Memory **< 64 MiB**.

**Criterion‑Snippet:**
```rust
fn bench_compile(c: &mut Criterion) {
  let pol = include_str!("../examples/lksg_v1.policy.yml");
  c.bench_function("compile_cold", |b| b.iter(|| compile_cold(pol)));
  let ctx = warm_cache_with(pol);
  c.bench_function("compile_warm", |b| b.iter(|| compile_warm(ctx)));
}
```
**DoD:** Ergebnisse in `bench/` mit CSV/HTML; Report verlinken im Hardening‑Report.

---

## 🔥 Load‑Tests (Verifier)

**Ziel:** 50 RPS, **p95 < 500 ms**, Errors < 1 %.

**k6‑Script (Ausschnitt):**
```js
import http from 'k6/http'; import { sleep } from 'k6';
export const options = { vus: 25, duration: '3m' };
export default function () {
  const payload = JSON.stringify(__ENV.PAYLOAD); // context_ok.json
  const params = { headers: { Authorization: `Bearer ${__ENV.TOKEN}` }, timeout: '5s' };
  http.post(`${__ENV.BASE}/verify`, payload, params);
  sleep(0.1);
}
```
**DoD:** Latenz & Fehler in Report (`reports/load_week4.json`), Grafana‑Screenshots beilegen.

---

## 🔐 Sicherheit & Logs (Prüfpunkte)
- **Keine PII** in Request/Response/Logs.  
- **mTLS** prod‑ready (Dev‑Flag abschaltbar).  
- **Rate‑Limits** aktiv (global & per Client).  
- Fehlertexte **ohne** Dumps von YAML/IR/Context; nur Hashes/IDs/E‑Codes.

---

## 🧪 Äquivalenz‑Test (Policy vs. Embedded‑IR)

**Schritte:**
1) `policy compile` → IR + Hashes.  
2) Zwei `/verify`‑Requests (A & B, identischer `context`).  
3) Vergleiche: `result`, `trace.active_rules`, `manifest_hash`, `signature` (optional).

**Akzeptanz:** Byte‑Gleichheit außer Feldern, die im Embedded‑Modus nicht existieren (`policy_id`).

---

## 🧩 CI‑Gates (Week‑4)

1. **Integration‑Tests** grün (inkl. Äquivalenz‑Case).  
2. **Schemathesis** ohne Contract‑Fehler.  
3. **Determinismus‑Guard** (100× compile) → 1 Hash.  
4. **Benchmarks**: p95 Ziele erfüllt (Compiler).  
5. **Load**: `/verify` 50 RPS, p95 < 500 ms, Error < 1 %.  
6. **Artefakte:** `bench/`, `reports/load_week4.json`, **Hardening‑Report**.

GitHub Actions (Ausschnitt):
```yaml
- run: cargo test --workspace --all-features
- run: schemathesis run openapi/verifier.v1.yaml --base-url=$BASE_URL -c
- run: ./ci/non_determinism_check.sh
- run: cargo bench --bench compile_bench
- run: k6 run k6/verify.js
- uses: actions/upload-artifact@v4
  with: { name: reports, path: reports/* }
```

---

## 🧾 Hardening‑Report (Gliederung, 2–3 Seiten)
1. **Ziel & Kontext** (Week‑4 Scope)  
2. **Methodik** (Tests/Tools)  
3. **Ergebnisse** (KPIs, Tabellen, Screenshots)  
4. **Abweichungen & Risiken** (mit Plan)  
5. **Empfehlungen** (Week‑5 Prioritäten)

---

## ✅ DoD (Definition of Done)
- Äquivalenz Policy/IR belegt (Tests).  
- Contract & Integration‑Tests **grün**.  
- LRU + ETag produzieren 304‑Treffer; Cache‑Hit‑Rate > 90 % (Repeat‑Suite).  
- Compiler‑Benchmarks p95 erfüllt; Verifier‑Load p95 erfüllt.  
- Hardening‑Report im Repo; CI‑Gates aktiv.  
- **Keine PII** in Logs; **mTLS/OAuth2** aktivierbar.

---

## 🧭 Nächste Schritte (Preview Week‑5)
- `proof adapt` **Interface & Evaluator** an den Verifier anbinden (RuleSelector).  
- Sub‑Proof‑Caching (Key = rule_id + params).  
- Erweiterte Operatoren (`non_intersection`, `threshold`) für IR v1.1.
