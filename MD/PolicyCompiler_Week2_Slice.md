# 🚀 Week‑2 Slice – Policy‑Compiler (Builtins & Adaptivity)

**Ziel (5–7 Tage):**  
Den v1‑Compiler um **Builtins/Expressions** und **Adaptivity** erweitern, inkl. Linter‑Regeln, deterministischer IR‑Darstellung und Golden‑Tests. Ergebnis: Policies mit einfachen Ausdrücken und aktivierten Regeln je nach Kontext lassen sich **deterministisch** kompilieren und im Verifier verwenden.

**Abhängigkeiten:** Week‑1 Slice fertig (Parser, Linter strict, IR‑Kern, Hashing).

---

## ✅ Deliverables (Ende Woche 2)
- Ausdrucks‑Grammar + Parser → **AST** (`Expr`): `var`, `const`, `func`, `op`
- Builtins (v1): `len(x)`, `max(x)`, `now()`, `sub(a,b)`, `duration("P365D")`
- Operatoren (v1): `==`, `!=`, `>=`, `<=`, `>`, `<`
- **Adaptivity**: `predicates[]` + `activations[]` → IR‑Strukturen
- **Canonicalization** für `predicates/activations` + `Expr` AST
- Linter (neu): Referenzen gültig, keine Zyklen, existierende Rule‑IDs
- CLI‑Erweiterungen: `policy lint --strict`, `policy compile --emit-hash --pretty`, `policy show --expand`
- Golden‑Tests für adaptives Beispiel (`lksg_v1.policy.yml` → `lksg_v1.ir.json`)
- OpenAPI‑Ergänzung: `/policy/compile` akzeptiert `lint_mode` & liefert `lints[]`

---

## 🧱 Dateien (Erweiterungen)
```
policy-compiler/
├─ src/
│  ├─ expr.rs          # AST, Parser, Serializer (kanonisch)
│  ├─ adaptivity.rs    # Predicates/Activations IR + Linter
│  └─ ...
├─ schemas/
│  └─ ir_v1.schema.json   # ergänzt um adaptivity + expr
├─ examples/
│  ├─ lksg_v1.policy.yml  # mit adaptivity
│  └─ lksg_v1.ir.json     # Golden IR (kanonisch)
└─ tests/
   ├─ expr_parse.rs
   ├─ adaptivity_lint.rs
   └─ golden_ir_adaptive.rs
```

---

## 🧠 Ausdrucks‑Grammar (v1, minimiert)

```
Expr := Var | Const | Func | Cmp
Var  := IDENT (z. B. supplier_hashes)
Const:= NUMBER | STRING | DURATION | DATE | BOOL
Func := IDENT "(" [Args] ")"
Args := Expr { "," Expr }
Cmp  := Expr OP Expr
OP   := "=="|"!="|">="|"<="|">"|"<"
DURATION := ISO8601 Dauer, z. B. "P365D"
```

**AST‑Form (JSON‑fähig):**
```json
{"var":"supplier_hashes"}
{"const":{"duration":"P365D"}}
{"func":"len","args":[{"var":"supplier_hashes"}]}
{"op":">=","lhs":{"func":"len","args":[{"var":"supplier_hashes"}]},"rhs":{"const":50}}
{"func":"sub","args":[{"func":"now"}, {"func":"max","args":[{"var":"audit_dates"}]}]}
```

---

## 🔧 IR‑Erweiterung (Ausschnitt)

```json
{
  "adaptivity": {
    "predicates":[
      {"id":"high_exposure","expr":{"op":">=","lhs":{"func":"len","args":[{"var":"supplier_hashes"}]},"rhs":{"const":50}}}
    ],
    "activations":[
      {"when":"high_exposure","rules":["no_sanctions","no_conflict_regions","audit_fresh"]}
    ]
  }
}
```

**Regel `audit_fresh` (range_min):**
```json
{
  "id":"audit_fresh",
  "op":"range_min",
  "lhs":{"func":"sub","args":[{"func":"now"},{"func":"max","args":[{"var":"audit_dates"}]}]},
  "rhs":{"const":{"duration":"P365D"}}
}
```

---

## 🧹 Canonicalization (Ergänzungen)
1. `predicates` & `activations` nach `id` bzw. `when` sortieren.  
2. In `Expr` Knoten: **Schlüsselreihenfolge** = `op|func|var|const` → `lhs`/`rhs` → `args` (alphabetisch).  
3. Konstanten normalisieren: Zahlen (dezimal), Datum (`YYYY-MM-DD`), `duration` als ISO8601 String.  
4. **Keine Pretty‑Spaces** im Hash‑Material (kompakte JSON‑Serialisierung).

---

## 🔍 Linter‑Regeln (neu)
- Jedes `predicates[].id` ist eindeutig.  
- `activations[].rules` referenzieren **existierende** `rules[].id`.  
- `expr` referenziert nur definierte `inputs` oder erlaubte Builtins.  
- **Keine Zyklen** in `activations` (indirekte Selbstreferenzen verhindern).  
- `range_min` → `lhs` muss `sub(now(), X)` Form haben, `rhs.duration` gültig.

---

## 🛠️ CLI‑Erweiterungen
```bash
# Strikter Lint inkl. Adaptivity Checks
cap policy lint examples/lksg_v1.policy.yml --strict

# Compile mit Hash‑Ausgabe & hübscher Anzeige
cap policy compile examples/lksg_v1.policy.yml -o examples/lksg_v1.ir.json --emit-hash --pretty

# Show: IR mit expandierten Exprs (menschenlesbar)
cap policy show examples/lksg_v1.ir.json --expand
```

**Fehlertexte (Stil):**
- `LINT[E1001] unknown rule id 'no_such_rule' in activation 'risk_high'`  
- `LINT[E2003] expr references unknown input 'foo'`  
- `LINT[E3002] range_min.lhs must be sub(now(), max(audit_dates))`

---

## 🧪 Tests (konkret)

**tests/expr_parse.rs**
- `len(supplier_hashes) >= 50` → korrektes AST  
- `now() - max(audit_dates)` → `sub(now(), max(...))`

**tests/adaptivity_lint.rs**
- Unbekannte Rule‑ID in `activations` → Fehler (strict)  
- Duplikat `predicate.id` → Fehler

**tests/golden_ir_adaptive.rs**
- `lksg_v1.policy.yml` → exakt gleiches `lksg_v1.ir.json` (Byte‑gleich)  
- Hashes (`policy_hash`, `ir_hash`) deterministisch über Läufe

---

## 🧾 OpenAPI‑Erweiterung `/policy/compile` (Ausschnitt)
```yaml
requestBody:
  content:
    application/json:
      schema:
        type: object
        properties:
          policy_yaml: { type: string, description: base64-encoded }
          lint_mode: { type: string, enum: [strict, relaxed], default: strict }
responses:
  '200':
    content:
      application/json:
        schema:
          type: object
          properties:
            policy_id: { type: string }
            ir: { $ref: '#/components/schemas/IRv1' }
            lints: { type: array, items: { type: string } }
            policy_hash: { type: string }
            ir_hash: { type: string }
```

---

## ⏱️ Umsetzungsschritte (Tagesplan)
- **Tag 1:** `expr.rs` (Parser + AST + Serializer), Grundtests  
- **Tag 2:** `adaptivity.rs` (Strukturen + Linter), Canonicalization‑Regeln  
- **Tag 3:** CLI‑Erweiterungen + Fehlertexte, Golden‑IR anlegen  
- **Tag 4:** Tests/Polish, Hash‑Stabilität, Doku (`docs/ir_v1.md` Update)  
- **Tag 5 (Puffer):** OpenAPI‑Hook, Edge‑Cases, Review

---

## ✅ DoD (Week‑2 Slice)
1. Parser für Exprs + AST serialisiert **kanonisch**.  
2. Linter deckt Referenzen, Zyklen und Formfehler ab (strict).  
3. IR enthält **adaptivity** (predicates/activations) deterministisch sortiert.  
4. Golden‑IR test grün; Hashes stabil über Läufe.  
5. CLI‑Kommandos funktionieren; klare Fehlermeldungen.  
6. OpenAPI‑Erweiterung dokumentiert; Beispiel‑Payloads aktualisiert.

---

## 📌 Hinweis
Der Evaluations‑Pfad (Auswertung von Prädikaten/Regeln) bleibt **im Verifier**. Der Compiler liefert nur **IR + Lints**. So bleibt die Zuständigkeit klar getrennt und du vermeidest doppelte Logik.
