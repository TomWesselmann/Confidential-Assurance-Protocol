# 🚀 Week‑1 Slice – Policy‑Compiler (Spec → Parser → Linter → Hashing)

**Warum noch ein Dokument?**  
Die **PRD_Policy_Compiler_v1.md** beschreibt *was* gebaut wird (Spezifikation).  
Dieses Dokument ist die **ausführbare Umsetzungsschablone** für die **erste Woche**: *konkrete Tasks, Ordner, Kommandos, Golden‑Tests, DoD*. Damit kannst du sofort anfangen zu implementieren – ohne die ganze PRD jedes Mal zu interpretieren.

**Ziel (3–5 Tage):**  
Minimal lauffähiger **Compiler‑Kern**: YAML → IR v1 (kanonisch) mit **Linter (strict)** und **Hashes**.  
Ergebnis: `policy lint`, `policy compile`, deterministische `policy_hash` & `ir_hash`, Golden‑Tests grün.

---

## ✅ Deliverables (Ende Woche 1)
- `schemas/policy.schema.json` & `schemas/ir_v1.schema.json` (minimale Schemata)
- `src/yaml_parser.rs`, `src/linter.rs`, `src/ir.rs`, `src/hasher.rs`, `src/cli.rs`
- CLI Kommandos: `policy lint`, `policy compile`, `policy show`
- **Golden‑Pair**: `examples/lksg_v1.policy.yml` → `examples/lksg_v1.ir.json`
- Tests: `tests/lint_strict.rs`, `tests/compile_roundtrip.rs`, `tests/golden_ir.rs`
- CI‑Job (light): Build + Unit‑/Golden‑Tests (ohne Security‑Scans)

---

## 🧱 Ordner & Dateien (Skeleton)

```
policy-compiler/
├─ Cargo.toml
├─ src/
│  ├─ lib.rs
│  ├─ cli.rs
│  ├─ yaml_parser.rs
│  ├─ linter.rs
│  ├─ ir.rs
│  └─ hasher.rs
├─ schemas/
│  ├─ policy.schema.json
│  └─ ir_v1.schema.json
├─ examples/
│  ├─ lksg_v1.policy.yml
│  └─ lksg_v1.ir.json
└─ tests/
   ├─ lint_strict.rs
   ├─ compile_roundtrip.rs
   └─ golden_ir.rs
```

---

## 🧠 Minimal‑Schemata

**schemas/policy.schema.json (Ausschnitt)**
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "required": ["id", "version", "legal_basis", "inputs", "rules"],
  "properties": {
    "id": {"type":"string"},
    "version": {"type":"string"},
    "legal_basis": {"type":"array","minItems":1},
    "inputs": {"type":"object"},
    "rules": {
      "type":"array",
      "items":{"type":"object","required":["id","op","lhs","rhs"]}
    },
    "adaptivity": {"type":"object"}
  }
}
```

**schemas/ir_v1.schema.json (Ausschnitt)**
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "required": ["ir_version","policy_id","policy_hash","rules","ir_hash"],
  "properties": {
    "ir_version":{"const":"1.0"},
    "policy_id":{"type":"string"},
    "policy_hash":{"type":"string"},
    "rules":{"type":"array"},
    "adaptivity":{"type":"object"},
    "ir_hash":{"type":"string"}
  }
}
```

---

## 🔧 Canonicalization‑Regeln (v1)
1. `rules` **alphabetisch nach `id`** sortieren.  
2. Objekt‑Schlüssel **lexikographisch** serialisieren.  
3. Zahlen als **dezimal** ohne unnötige Nullen, Booleans klein, Strings UTF‑8 normalisiert (NFC).  
4. Whitespace **nicht** relevant; Serialisierung übernimmt der Canonical‑Serializer.  
5. Hash‑Material ist die **kanonische JSON** ohne Pretty‑Print.

---

## 🧮 Hashing (SHA3‑256)

**policy_hash** = SHA3‑256(**normalized YAML** → JSON AST, kanonisch)  
**ir_hash**     = SHA3‑256(**kanonische IR‑JSON**)

Pseudocode:
```rust
let policy_ast = parse_yaml(policy_yaml)?;
let policy_canonical_json = canonicalize(&policy_ast)?;
let policy_hash = sha3_256_hex(&policy_canonical_json);

let ir = generate_ir(&policy_ast)?;
let ir_canonical_json = canonicalize(&ir)?;
let ir_hash = sha3_256_hex(&ir_canonical_json);
```

---

## 🧪 Golden‑Paar (kleines Beispiel)

**examples/lksg_v1.policy.yml**
```yaml
id: lksg.v1
version: "1.0"
legal_basis: [{directive: "LkSG"}]
inputs:
  supplier_hashes: {type: array, items: hex}
  sanctions_root: {type: hex}
rules:
  - id: no_sanctions
    op: non_membership
    lhs: supplier_hashes
    rhs: sanctions_root
```

**erwartetes examples/lksg_v1.ir.json**
```json
{
  "ir_version":"1.0",
  "policy_id":"lksg.v1",
  "policy_hash":"sha3-256:TO_BE_FILLED_IN_TEST",
  "rules":[
    {
      "id":"no_sanctions",
      "op":"non_membership",
      "lhs":{"var":"supplier_hashes"},
      "rhs":{"var":"sanctions_root"}
    }
  ],
  "ir_hash":"sha3-256:TO_BE_FILLED_IN_TEST"
}
```

> In Tests wird `TO_BE_FILLED_IN_TEST` nach dem ersten stabilen Lauf ersetzt (Golden‑Update).

---

## 🧰 CLI‑Kommandos (Week‑1‑Scope)

```bash
# Lint (strict)
cap policy lint examples/lksg_v1.policy.yml --strict

# Compile → IR v1
cap policy compile examples/lksg_v1.policy.yml -o examples/lksg_v1.ir.json

# Show (human readable)
cap policy show examples/lksg_v1.ir.json
```

**Exit Codes:** 0 OK, 2 Warn (relaxed), 3 Lint‑Fehler (strict), 4 Schemafehler

---

## 🧷 Linter‑Regeln (strict)
- `legal_basis` muss gesetzt sein (min. 1 Item).  
- `rules[].id` eindeutig, `op ∈ {non_membership, eq, range_min}`.  
- `lhs/rhs` referenzieren **Inputs** oder erlaubte Builtins (`now()`, `len`, `max`).  
- keine **unbekannten Felder** auf Top‑Level; Warnung statt Fehler im `relaxed`‑Mode.

---

## 🧪 Tests (konkret)

**tests/lint_strict.rs**
- Fehlendes `legal_basis` → Exit 3  
- Ungültiger `op` → Exit 3

**tests/compile_roundtrip.rs**
- `compile(policy.yml)` → `ir.json` → **Schema valid** (IR)  
- `policy_hash` & `ir_hash` **stabil** über zwei Läufe

**tests/golden_ir.rs**
- `compile(policy.yml)` → **gleiches** `ir.json` wie Golden (Byte‑gleich)

---

## 🧵 CI (leicht)

GitHub Actions (Auszug):
```yaml
jobs:
  build-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --workspace --locked
      - run: cargo test --workspace --all-features
```

---

## ✅ DoD (Week‑1 Slice)
1. `policy lint --strict` und `policy compile` laufen fehlerfrei auf Beispiel‑Policy.  
2. `policy_hash`/`ir_hash` sind **deterministisch** (Golden grün).  
3. `rules` werden **kanonisch sortiert** und serialisiert.  
4. Unit‑Tests & Golden‑Tests **grün** auf CI.  
5. Keine PII in Logs; nur Hashes und Rule‑IDs.

---

## 🧭 Nächste Schritte (Week‑2 Slice, Preview)
- Builtins: `range_min` mit ISO‑Duration (`P365D`), `now()` eval.  
- Adaptivity: `predicates`/`activations` inkl. Evaluator.  
- Fehlertexte & UX für CLI polishen.  
- `/policy/compile` Endpoint anbinden (OpenAPI).

---

**Hinweis:** Diese Week‑1‑Schablone ergänzt die PRD und macht sie *umsetzbar*: du bekommst Tasks, Beispiele, Tests & DoD in einem kompakten Format.
