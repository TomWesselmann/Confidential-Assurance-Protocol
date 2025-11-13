# 🧰 Week‑3 Slice – Policy‑Compiler (OpenAPI‑Integration, Hardening & Edge‑Cases)

**Ziel (5–7 Tage):**  
Den Policy‑Compiler v1 **produktionsreif anbinden**: OpenAPI‑/Verifier‑Integration, deterministische Serialisierung, Fehler‑/Lint‑Katalog, Caching, Performance‑Budget, CI‑Gates, Doku & Release‑Artefakte.

**Voraussetzungen:** Week‑1 (Spec/Parser/Linter/Hashing) ✅, Week‑2 (Builtins/Adaptivity) ✅.

---

## ✅ Deliverables (Ende Woche 3)
- **OpenAPI‑Endpoint** `/policy/compile` (Server‑Seite) + vollständiges Schema `IRv1`  
- **Verifier‑Integration**: `/verify` akzeptiert eingebettetes `ir` **oder** lädt `policy_id` aus Registry  
- **Determinismus‑Suite**: Golden‑IR, Hash‑Stabilität, Canonicalization‑Checks (100 Läufe)  
- **Lint/Fehler‑Katalog** mit IDs (E/W Codes), konsistenten Texten, Mapping → HTTP Status  
- **Caching**: `policy_hash` → IR Cache (LRU) + ETag/If‑None‑Match Support  
- **Performance‑Budget**: Compile p95 ≤ 50 ms (warm), ≤ 200 ms (cold) für typische Policies  
- **CI‑Gates**: Schema‑Validate, Golden‑Tests, Non‑Determinism Sentinel, API‑Contract (schemathesis)  
- **Doku**: `docs/ir_v1.md` (final), `docs/policy_lints.md` (Katalog), `MIGRATION_NOTES.md`  
- **Release**: Tags, changelog, versionierte Schemas (`/schemas/ir/1.0/…`)

---

## 🧱 API‑Design & Verträge

### 1) `POST /policy/compile`
**Request**
```json
{
  "policy_yaml": "base64:...",
  "lint_mode": "strict",
  "persist": true
}
```
**Response 200**
```json
{
  "policy_id":"lksg.v1",
  "policy_hash":"sha3-256:...",
  "ir": { "... IRv1 ..." },
  "ir_hash":"sha3-256:...",
  "lints":[ "W1002: description missing" ],
  "stored": true,
  "etag": ""ir:sha3-256:...""
}
```
**Errors**  
- 400 Schemafehler (YAML/JSON ungültig)  
- 409 Policy‑Konflikt (id/version bereits mit anderem hash vorhanden)  
- 422 Lint‑Fehler im `strict` Mode (E‑Codes)
- 500 Interner Fehler (keine PII, Korrelation‑ID)

### 2) `GET /policy/:id` (mit ETag)
**Request Headers**: `If-None-Match: "ir:sha3-256:..."` → **304** bei unverändert.  
**Response 200**: `policy_id`, `version`, `policy_hash`, `ir`, `ir_hash`, `etag`.

### 3) `/verify` – Annahme von IR
**Request (A)**: `{ "policy_id":"lksg.v1", "context":{...} }` (Server lädt IR)  
**Request (B)**: `{ "ir":{...IRv1...}, "context":{...} }` (Einmal‑Nutzung; `policy_id` optional)

---

## 🔒 Lint & Fehler‑Katalog

| Code | Ebene | Beispieltext | HTTP |
|-----:|:-----:|--------------|:----:|
| E1001 | Strict | unknown rule id 'X' in activation 'Y' | 422 |
| E1002 | Strict | missing `legal_basis` | 422 |
| E2001 | Strict | invalid op 'foo' (allowed: non_membership, eq, range_min) | 422 |
| E2003 | Strict | expr references unknown input 'bar' | 422 |
| E3002 | Strict | range_min.lhs must be sub(now(), max(audit_dates)) | 422 |
| W1002 | Warn  | description missing | 200 |

**Konvention:** `LINT[<LEVEL><4‑Digit>]` – stabil, maschinenlesbar, in `lints[]`/`errors[]`.

---

## 🧮 Determinismus & Canonicalization

- **Stable Sort**: `rules` nach `id`, `predicates` nach `id`, `activations` nach `when`.  
- **Expr‑Knoten‑Ordnung**: `op|func|var|const` → `lhs`/`rhs` → `args`.  
- **Serializer**: kompakte JSON ohne pretty; Schlüssel lexikographisch.  
- **Non‑Determinism Sentinel**: 100× compile → **identischer** `ir_hash`; sonst Fail.

**CI‑Job (Ausschnitt):**
```bash
for i in $(seq 1 100); do cap policy compile examples/lksg_v1.policy.yml -o /tmp/ir$i.json; done
sha256sum /tmp/ir*.json | awk '{print $1}' | sort -u | wc -l  # muss 1 sein
```

---

## ⚙️ Caching & Registry

- **LRU Cache**: Key=`policy_hash`, Value=IR (max 1k Einträge).  
- **ETag**: `W/"ir:<ir_hash>"` – unterstützt 304.  
- **Registry Schreibschutz**: `persist=true` speichert nur, wenn `policy_id` noch **nicht** mit anderem `policy_hash` existiert (sonst 409).  
- **Migration**: `MIGRATION_NOTES.md` beschreibt Feld‑Renames / Semantik.

---

## 📈 Performance‑Budget
- **Warm Compile p95 ≤ 50 ms**, **Cold p95 ≤ 200 ms** bei `examples/lksg_v1.policy.yml`.  
- **Memory Footprint**: Peak < 64 MiB.  
- **/policy/compile QPS**: 50 RPS (warm) ohne 5xx bei 1 Replica.

k6‑Snippet:
```js
export const options = { vus: 25, duration: '2m' };
export default function () {
  http.post(`${__ENV.BASE}/policy/compile`, JSON.stringify({policy_yaml: __ENV.POL}), { timeout: '5s' });
}
```

---

## 🧪 Tests (konkret)

**Contract (schemathesis)**
```bash
schemathesis run openapi/verifier.v1.yaml --base-url=https://localhost:8443 -c --hypothesis-verbosity=quiet
```

**Golden/Determinismus**
- `examples/lksg_v1.policy.yml` → `examples/lksg_v1.ir.json` (Byte‑gleich)  
- 100× Compile Hash‑Gleichheit

**Integration**
- `/policy/compile` strict‑Error (fehlendes `legal_basis`) → 422 + `lints[]`  
- `/verify` mit eingebettetem IR liefert **identisches Ergebnis** wie mit `policy_id`

**Caching**
- `GET /policy/:id` mit `If‑None‑Match` → 304  
- Cache‑Hit‑Rate > 90 % im Repeat‑Test

**Security**
- Keine YAML‑Raw Dumps in Logs; nur Hashes/IDs  
- Keine PII; Lints/Errors ohne Klartext‑Kontext

---

## 🧵 CI‑Gates

1. **Schema‑Validation** (policy/IR)  
2. **Golden‑IR** Bytegleichheit
3. **Non‑Determinism Sentinel** (100×)  
4. **Contract‑Tests** via OpenAPI  
5. **Coverage Ziel**: Compiler‑Core ≥ 85 %

GitHub Actions (Ausschnitt):
```yaml
- run: cargo test --workspace --all-features
- run: schemathesis run openapi/verifier.v1.yaml --base-url=$BASE_URL -c
- run: ./ci/non_determinism_check.sh
```

---

## 📚 Doku & Release

- `docs/ir_v1.md` finalisieren: Operatoren, Builtins, Expr‑Grammar, Beispiele.  
- `docs/policy_lints.md` mit Tabelle (Code/Level/Text/Beispiel).  
- `MIGRATION_NOTES.md` – Breaking Changes & Migrationspfade.  
- `CHANGELOG.md` – semver‑regeln (Compiler v1.x).  
- **Tagging**: `compiler-v1.0.0`, `schemas/ir/1.0/*` eingefroren.

---

## ✅ DoD (Week‑3 Slice)
1. `/policy/compile` produktionsreif (OpenAPI grün, Contract‑Tests grün).  
2. `/verify` akzeptiert `ir` und `policy_id` – Ergebnisse identisch.  
3. Determinismus garantiert (Golden + 100× Hash‑Stabilität).  
4. Caching/ETag funktioniert; 304 korrekt.  
5. Performance‑Budget eingehalten; CI‑Gates grün.  
6. Doku/Changelog/Migration vollständig & versioniert.

---

## 🔭 Nächste Schritte (Post‑v1 Preview)
- IR v1.1: `non_intersection`, `threshold`, benannte Sub‑rules.  
- Policy Registry Signatures (KID/Rotation).  
- ZK‑Backend‑Adapter: `Halo2`, `RISC‑Zero` – Trait‑Bridge gegen IR.
