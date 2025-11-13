# 🧩 PRD – Policy-Compiler v1 (YAML → IR v1)

**Ziel:**  
Policies aus YAML deterministisch in eine **IR v1 (JSON)** übersetzen – inkl. Linting, Hashing & Versionierung – als Grundlage für **adaptive Proofs** und rechtssichere Nachvollziehbarkeit.

**Scope (v1):**  
- Parser + Linter (strict/relaxed)  
- IR-Generator (deterministisch, stable ordering)  
- Minimaler Operator-Kern: `non_membership`, `eq`, `range_min`  
- Adaptivity (einfache Prädikate → aktive Regeln)  
- Policy-/IR-Hashing (SHA3-256), `legal_basis` Pflichtfeld  
- CLI: `policy lint`, `policy compile`, `policy show`  
- Artefakte: `policy.yml` → `ir.json` (+ `policy_hash`, `ir_hash`)

**Out-of-Scope (v1):**  
- Echte ZK-Backends (Halo2/zkVM) – weiter Mock  
- Komplexe Datentypen (nur Arrays, Strings, Numbers, Dates ISO8601)

---

## 📁 Struktur

```
policy-compiler/
├─ src/
│  ├─ lib.rs
│  ├─ yaml_parser.rs
│  ├─ linter.rs
│  ├─ ir.rs            # IR-Strukturen + Canonicalization
│  ├─ hasher.rs        # SHA3-256 (policy_hash, ir_hash)
│  └─ cli.rs           # lint/compile/show
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

## 🧠 YAML Policy (v1)

```yaml
id: lksg.v1
version: "1.0"
legal_basis:
  - directive: "LkSG"
  - article: "§ 5 Abs. 1"
description: "Lieferantencheck: Sanktionslisten & Hochrisikoregionen"
inputs:
  supplier_hashes: { type: array, items: hex }
  supplier_regions: { type: array, items: hex }
  sanctions_root: { type: hex }
  high_risk_root: { type: hex }
  audit_dates: { type: array, items: date }
rules:
  - id: no_sanctions
    op: non_membership
    lhs: supplier_hashes
    rhs: sanctions_root
  - id: no_conflict_regions
    op: non_membership
    lhs: supplier_regions
    rhs: high_risk_root
  - id: audit_fresh
    op: range_min
    lhs: "now() - max(audit_dates)"
    rhs: "P365D"
adaptivity:
  predicates:
    - id: high_exposure
      expr: "len(supplier_hashes) >= 50"
  activations:
    - when: high_exposure
      rules: ["no_sanctions","no_conflict_regions","audit_fresh"]
outputs:
  verdicts:
    ok:   "alle aktiven Regeln erfüllt"
    warn: "Regeln teilweise nicht evaluiert/alt"
    fail: "mind. eine Regel verletzt"
```

**Lint-Pflichten (strict):**
- `id`, `version`, `legal_basis` vorhanden  
- `rules[].id` eindeutig, nur erlaubte `op`-Werte  
- alle `lhs/rhs` referenzieren existierende Inputs oder erlaubte Builtins (`now()`, `len()`, `max()`)

---

## 📦 IR v1 (JSON, kanonisch sortiert)

```json
{
  "ir_version": "1.0",
  "policy_id": "lksg.v1",
  "policy_hash": "sha3-256:…", 
  "rules": [
    {"id":"audit_fresh","op":"range_min","lhs":{"func":"sub","args":[{"func":"now"},{"func":"max","args":[{"var":"audit_dates"}]}]},"rhs":{"duration":"P365D"}},
    {"id":"no_conflict_regions","op":"non_membership","lhs":{"var":"supplier_regions"},"rhs":{"var":"high_risk_root"}},
    {"id":"no_sanctions","op":"non_membership","lhs":{"var":"supplier_hashes"},"rhs":{"var":"sanctions_root"}}
  ],
  "adaptivity": {
    "predicates":[{"id":"high_exposure","expr":{"op":">=","lhs":{"func":"len","args":[{"var":"supplier_hashes"}]},"rhs":50}}],
    "activations":[{"when":"high_exposure","rules":["no_sanctions","no_conflict_regions","audit_fresh"]}]
  },
  "ir_hash": "sha3-256:…"
}
```

**Anforderungen:**  
- **Determinismus:** gleiche YAML → identisches `policy_hash`/`ir_hash`  
- **Canonicalization:** sortiere `rules` nach `id`, normalisiere Whitespace/Numbers/Booleans

---

## 🛠️ CLI

```bash
# Lint (strict mode)
cap policy lint examples/lksg_v1.policy.yml --strict

# Compile → IR (JSON) + Hashes
cap policy compile examples/lksg_v1.policy.yml -o examples/lksg_v1.ir.json

# Show (menschlich)
cap policy show examples/lksg_v1.ir.json
```

**Exit Codes:**  
- 0 OK, 2 Lint-Warnungen (relaxed), 3 Lint-Fehler (strict), 4 Schemafehler

---

## 🔗 Integration mit Verifier

- `/policy/compile` akzeptiert Policy-YAML (Base64) und liefert **IR v1** + `policy_hash`.  
- `/verify` akzeptiert **entweder** `policy_id` (Server lädt IR) **oder** eingebettetes `ir` (einmalig).  
- `trace.active_rules` spiegelt **nach Adaptivity** ausgewählte Regeln.

---

## 🔒 Sicherheit & Recht

- **legal_basis** Pflicht; im Manifest übernehmen (Nachvollziehbarkeit).  
- **Hash-Verkettung:** `policy_hash` Teil des Manifests; `ir_hash` im Verify-Response.  
- **Logs:** Keine YAML-Originale in Logs; nur Hashes und Rule-IDs.  
- **Doku:** `docs/policy_lints.md` (alle Lint-Regeln + Beispiele).

---

## 🧪 Tests

**Unit**
- deterministische Hashes (stable ordering)  
- Lint strict vs. relaxed  
- Parser für Builtins (`now()`, `len`, `max`), Durations (`P365D`)

**Golden**
- `examples/lksg_v1.policy.yml` → exakt gleiches `ir.json` (Golden-File-Vergleich)

**Integration**
- `/policy/compile` strict-Fehler bei fehlender `legal_basis`  
- `/verify` mit kompiliertem IR → gleiches Ergebnis wie bei handcodierten Regeln

---

## ⏱️ Umsetzung (3-Wochen-Slice)

- **Woche 1 (parallel zu E2E-Finish):** IR-Spec, Parser, Linter (strict), deterministische Canonicalization, `policy_hash`/`ir_hash`.  
- **Woche 2:** Builtins & Adaptivity, Golden-Tests, CLI UX, Fehlertexte.  
- **Woche 3:** OpenAPI-Integration `/policy/compile`, Doku, Edge-Cases + Hardening.

---

## ✅ Abnahmekriterien (DoD)

1. **Deterministisches** `policy_hash`/`ir_hash` (Golden-Tests)  
2. `policy lint --strict` verhindert Policies ohne `legal_basis`/mit ungültigen `op`  
3. `policy compile` erzeugt **kanonisches IR v1** (Schema valid)  
4. `/verify` mit IR liefert **identische** Ergebnisse wie „hart verdrahtete“ Regeln  
5. Logs ohne PII/YAML-Dump; nur Hashes/IDs  
6. Doku + Beispiele vorhanden; 90 %+ Unit-/Golden-Tests grün

---

## 🚩 Risiken & Gegenmaßnahmen

- **Drift zwischen YAML & Code** → Golden-Files + Hash-DoD  
- **Komplexe Ausdrücke** → v1 nur Basisausdrücke/Builtins  
- **Nicht-deterministische Sortierung** → strikte Canonicalization + Serializer

---

## 📑 Deliverables

- `schemas/policy.schema.json`, `schemas/ir_v1.schema.json`  
- `examples/lksg_v1.policy.yml` + `lksg_v1.ir.json`  
- `cap policy {lint,compile,show}`  
- `docs/policy_lints.md`, `docs/ir_v1.md`
