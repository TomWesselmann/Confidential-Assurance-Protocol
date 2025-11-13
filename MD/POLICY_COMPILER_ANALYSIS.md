# 🧩 Policy Compiler v1 - Analysis & Implementation Plan

**Datum:** 2025-11-09
**Projekt:** CAP Policy Compiler (YAML → IR v1)
**Status:** 📊 **Gap Analysis Complete**

---

## Executive Summary

The agent project has a **basic policy implementation** (constraint-based), but the PRD requires a **full compiler** with:
- IR v1 generation (JSON, canonicalized)
- Operator system (non_membership, eq, range_min)
- Adaptivity (predicates → active rules)
- Deterministic hashing (policy_hash, ir_hash)
- Linting (strict/relaxed modes)

**Recommendation:** Leverage existing policy infrastructure and extend it to meet PRD requirements.

---

## Current Implementation (Agent Project)

### Existing Files

**`src/policy.rs`:**
- ✅ YAML parser (serde_yaml)
- ✅ Policy validation (basic)
- ✅ Policy hash computation (SHA3-256)
- ✅ PolicyInfo struct (name, version, hash)
- ❌ No IR generation
- ❌ No operator system
- ❌ No adaptivity
- ❌ No linting

**`examples/policy.lksg.v1.yml`:**
```yaml
version: "lksg.v1"
name: "LkSG Demo Policy"
created_at: "2025-10-25T09:00:00Z"
constraints:
  require_at_least_one_ubo: true
  supplier_count_max: 10
notes: "Demo policy for testing Tag 3 proof system"
```

**Current Policy Structure:**
```rust
pub struct Policy {
    pub version: String,
    pub name: String,
    pub created_at: String,
    pub constraints: PolicyConstraints,
    pub notes: String,
}

pub struct PolicyConstraints {
    pub require_at_least_one_ubo: bool,
    pub supplier_count_max: u32,
    pub ubo_count_min: Option<u32>,
    pub require_statement_roots: Option<Vec<String>>,
}
```

**Limitations:**
- Hard-coded constraints (not extensible)
- No rule-based system
- No operator abstraction
- No adaptivity/predicates
- No IR generation

---

## PRD Requirements (Policy Compiler v1)

### YAML Policy Structure (PRD)

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

### IR v1 Structure (PRD)

```json
{
  "ir_version": "1.0",
  "policy_id": "lksg.v1",
  "policy_hash": "sha3-256:…",
  "rules": [
    {
      "id": "audit_fresh",
      "op": "range_min",
      "lhs": {
        "func": "sub",
        "args": [
          {"func": "now"},
          {"func": "max", "args": [{"var": "audit_dates"}]}
        ]
      },
      "rhs": {"duration": "P365D"}
    },
    {
      "id": "no_conflict_regions",
      "op": "non_membership",
      "lhs": {"var": "supplier_regions"},
      "rhs": {"var": "high_risk_root"}
    },
    {
      "id": "no_sanctions",
      "op": "non_membership",
      "lhs": {"var": "supplier_hashes"},
      "rhs": {"var": "sanctions_root"}
    }
  ],
  "adaptivity": {
    "predicates": [
      {
        "id": "high_exposure",
        "expr": {
          "op": ">=",
          "lhs": {
            "func": "len",
            "args": [{"var": "supplier_hashes"}]
          },
          "rhs": 50
        }
      }
    ],
    "activations": [
      {
        "when": "high_exposure",
        "rules": ["no_sanctions", "no_conflict_regions", "audit_fresh"]
      }
    ]
  },
  "ir_hash": "sha3-256:…"
}
```

---

## Gap Analysis

### Missing Components

| Component | Current Status | PRD Requirement | Gap |
|-----------|---------------|-----------------|-----|
| **Policy Schema** | Simple constraints | Inputs, Rules, Adaptivity, Outputs | 🔴 Major |
| **Operators** | None | non_membership, eq, range_min | 🔴 Major |
| **IR Generation** | None | Canonical JSON with expressions | 🔴 Major |
| **Adaptivity** | None | Predicates + Activations | 🔴 Major |
| **Linting** | Basic validation | Strict/Relaxed modes, legal_basis check | 🟡 Medium |
| **Hashing** | ✅ Implemented | Deterministic, canonical ordering | 🟢 Minor (extend) |
| **CLI** | None | lint, compile, show | 🔴 Major |
| **Builtins** | None | now(), len(), max(), sub() | 🔴 Major |
| **Canonicalization** | None | Stable ordering, whitespace normalization | 🟡 Medium |

---

## Implementation Strategy

### Option 1: Extend Existing Policy Module (Recommended)

**Pros:**
- Leverage existing YAML parser, hashing
- Maintain compatibility with current agent code
- Incremental migration path

**Cons:**
- Need to refactor constraint-based approach
- More complex codebase

**Effort:** 2-3 weeks (as per PRD)

### Option 2: New Standalone Policy Compiler

**Pros:**
- Clean slate, PRD-compliant from start
- Clear separation of concerns
- Easier to test in isolation

**Cons:**
- Duplicate YAML parsing, hashing
- Integration with agent requires API
- More files to maintain

**Effort:** 3-4 weeks

**Recommendation:** **Option 1** (extend existing) for better integration and faster delivery.

---

## Implementation Plan (3 Weeks)

### Week 1: IR Specification & Core Parser

**Tasks:**
1. Define IR v1 Rust structures
   ```rust
   pub struct IrV1 {
       pub ir_version: String,
       pub policy_id: String,
       pub policy_hash: String,
       pub rules: Vec<Rule>,
       pub adaptivity: Option<Adaptivity>,
       pub ir_hash: String,
   }

   pub struct Rule {
       pub id: String,
       pub op: Operator,
       pub lhs: Expression,
       pub rhs: Expression,
   }

   pub enum Operator {
       NonMembership,
       Eq,
       RangeMin,
   }

   pub enum Expression {
       Var(String),
       Func { name: String, args: Vec<Expression> },
       Literal(Value),
   }
   ```

2. Implement PolicyV2 YAML parser
   - Parse `legal_basis` (mandatory in strict mode)
   - Parse `inputs` with type definitions
   - Parse `rules` with operators
   - Parse `adaptivity` (predicates + activations)

3. Implement deterministic canonicalization
   - Sort rules by `id` (alphabetical)
   - Normalize whitespace, numbers, booleans
   - Stable JSON serialization (serde_json + BTreeMap)

4. Extend hashing for `ir_hash`
   - SHA3-256 of canonical IR JSON
   - Exclude `ir_hash` field from hash computation

5. Unit tests
   - Deterministic hash (same YAML → same hash)
   - Parsing edge cases
   - Canonicalization validation

**Deliverables:**
- `src/policy_v2.rs` (new module)
- `schemas/policy_v2.schema.json`
- `schemas/ir_v1.schema.json`
- Unit tests (90%+ coverage)

---

### Week 2: Builtins, Adaptivity & Linting

**Tasks:**
1. Implement builtin functions
   ```rust
   pub enum Builtin {
       Now,
       Len,
       Max,
       Sub,
   }
   ```

2. Implement expression parser
   - Parse `now() - max(audit_dates)` → AST
   - Parse `len(supplier_hashes) >= 50` → AST
   - Duration parsing (`P365D` ISO 8601)

3. Implement linter
   ```rust
   pub enum LintMode {
       Strict,  // Fail on missing legal_basis
       Relaxed, // Warn only
   }

   pub fn lint(policy: &PolicyV2, mode: LintMode) -> Vec<LintDiagnostic> {
       // Check legal_basis (strict: error, relaxed: warn)
       // Check rule IDs uniqueness
       // Check operators valid
       // Check lhs/rhs references exist
   }
   ```

4. Implement adaptivity compiler
   - Evaluate predicates (static analysis)
   - Select active rules based on activations
   - Generate `trace.active_rules` for response

5. Golden file tests
   - `examples/lksg_v1.policy.yml` → `examples/lksg_v1.ir.json`
   - Bit-exact comparison (determinism)

**Deliverables:**
- Builtin function evaluator
- Expression AST parser
- Linter with strict/relaxed modes
- Golden file test suite
- `docs/policy_lints.md`

---

### Week 3: CLI, OpenAPI Integration & Hardening

**Tasks:**
1. Implement CLI commands
   ```bash
   # In agent project
   cargo run -- policy lint examples/lksg_v1.policy.yml --strict
   cargo run -- policy compile examples/lksg_v1.policy.yml -o ir.json
   cargo run -- policy show ir.json
   ```

2. Extend REST API
   ```rust
   // POST /policy/compile
   pub struct CompileRequest {
       policy_yaml: String, // Base64-encoded YAML
       mode: LintMode,
   }

   pub struct CompileResponse {
       ir: IrV1,
       policy_hash: String,
       ir_hash: String,
       lint_diagnostics: Vec<LintDiagnostic>,
   }
   ```

3. Integrate with `/verify` endpoint
   - Accept `policy_id` (server loads IR from store)
   - OR accept inline `ir` (one-time verification)
   - Generate `trace.active_rules` based on adaptivity

4. Edge case handling
   - Circular references in expressions
   - Invalid operator combinations
   - Malformed durations
   - Missing inputs referenced in rules

5. Documentation
   - `docs/ir_v1.md` (IR specification)
   - `docs/policy_lints.md` (lint rules)
   - Update agent CLAUDE.md

**Deliverables:**
- CLI subcommands (lint, compile, show)
- REST API `/policy/compile` endpoint
- Updated `/verify` with IR support
- Edge case tests
- Complete documentation

---

## Technical Design

### Module Structure

```
agent/src/
├── policy.rs                   # Legacy (keep for backwards compat)
├── policy_v2/
│   ├── mod.rs
│   ├── yaml_parser.rs          # PolicyV2 YAML → Rust struct
│   ├── linter.rs               # Strict/Relaxed linting
│   ├── ir.rs                   # IR v1 structures + canonicalization
│   ├── compiler.rs             # PolicyV2 → IrV1
│   ├── builtins.rs             # now(), len(), max(), sub()
│   ├── expression.rs           # AST for lhs/rhs
│   ├── hasher.rs               # SHA3-256 (policy_hash, ir_hash)
│   └── cli.rs                  # lint/compile/show commands
├── api/
│   └── policy.rs               # /policy/compile endpoint
└── main.rs                     # Add policy subcommands
```

### Data Flow

```
YAML Policy (examples/lksg_v1.policy.yml)
         ↓
   yaml_parser.rs (PolicyV2 struct)
         ↓
   linter.rs (validate, check legal_basis)
         ↓
   compiler.rs (PolicyV2 → IrV1)
         ↓
   canonicalization (sort rules by id, stable JSON)
         ↓
   hasher.rs (compute ir_hash)
         ↓
   IR v1 JSON (examples/lksg_v1.ir.json)
         ↓
   /verify endpoint (evaluate rules, adaptivity)
```

---

## Testing Strategy

### Unit Tests (Target: 90%+)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_policy_hash() {
        let yaml1 = load_policy("examples/lksg_v1.policy.yml").unwrap();
        let yaml2 = load_policy("examples/lksg_v1.policy.yml").unwrap();
        assert_eq!(compute_hash(&yaml1), compute_hash(&yaml2));
    }

    #[test]
    fn test_canonical_ir_ordering() {
        let policy = PolicyV2 {
            rules: vec![
                Rule { id: "z_rule".into(), /* ... */ },
                Rule { id: "a_rule".into(), /* ... */ },
            ],
            /* ... */
        };
        let ir = compile(&policy).unwrap();
        assert_eq!(ir.rules[0].id, "a_rule"); // Alphabetical
        assert_eq!(ir.rules[1].id, "z_rule");
    }

    #[test]
    fn test_lint_strict_missing_legal_basis() {
        let policy = PolicyV2 {
            legal_basis: None,
            /* ... */
        };
        let diagnostics = lint(&policy, LintMode::Strict);
        assert!(diagnostics.iter().any(|d| d.level == Level::Error));
    }

    #[test]
    fn test_builtin_now() {
        let expr = parse_expr("now()").unwrap();
        let result = eval(&expr, &context).unwrap();
        assert!(result.is_timestamp());
    }
}
```

### Golden File Tests

```rust
#[test]
fn test_golden_ir_lksg_v1() {
    let policy = PolicyV2::load("examples/lksg_v1.policy.yml").unwrap();
    let ir = compile(&policy).unwrap();
    let ir_json = serde_json::to_string_pretty(&ir).unwrap();

    let golden = std::fs::read_to_string("tests/golden/lksg_v1.ir.json").unwrap();
    assert_eq!(ir_json, golden, "IR does not match golden file");
}
```

### Integration Tests (with REST API)

```rust
#[tokio::test]
async fn test_policy_compile_endpoint() {
    let yaml = std::fs::read_to_string("examples/lksg_v1.policy.yml").unwrap();
    let encoded = base64::encode(&yaml);

    let req = CompileRequest {
        policy_yaml: encoded,
        mode: LintMode::Strict,
    };

    let resp = client.post("/policy/compile").json(&req).send().await.unwrap();
    assert_eq!(resp.status(), 200);

    let compile_resp: CompileResponse = resp.json().await.unwrap();
    assert_eq!(compile_resp.ir.ir_version, "1.0");
    assert!(compile_resp.lint_diagnostics.is_empty());
}
```

---

## Risk Mitigation

### Risk 1: Drift between YAML & Code

**Mitigation:**
- Golden file tests (bit-exact comparison)
- Deterministic hash in DoD
- CI gate: `ir_hash` must match golden

### Risk 2: Complex Expressions

**Mitigation:**
- Limit v1 to basic builtins (now, len, max, sub)
- Expression depth limit (max 5 levels)
- Validate expression AST in linter

### Risk 3: Non-Deterministic Sorting

**Mitigation:**
- Use BTreeMap instead of HashMap (stable ordering)
- Sort rules by ID (alphabetical)
- Custom JSON serializer with canonical formatting

---

## Acceptance Criteria (DoD)

From PRD:

1. ✅ **Deterministic** `policy_hash`/`ir_hash` (Golden tests)
2. ✅ `policy lint --strict` prevents policies without `legal_basis`/invalid `op`
3. ✅ `policy compile` generates **canonical IR v1** (schema valid)
4. ✅ `/verify` with IR delivers **identical** results as hard-coded rules
5. ✅ Logs without PII/YAML dump; only hashes/IDs
6. ✅ Docs + examples; 90%+ Unit/Golden tests green

---

## Integration with SAP Adapter

**Question:** Should the SAP Adapter use policies?

**Current State:**
- Adapter hashes supplier data (BLAKE3)
- Sends context.json to Verifier
- Verifier applies policy (server-side)

**Potential Use Case:**
- Adapter could validate supplier data against policy rules **before** sending to Verifier
- Early rejection of known-bad suppliers (performance optimization)

**Recommendation:** **Server-side only** for v1 (simplicity). Client-side policy evaluation can be added in v2 if needed.

---

## Timeline

| Week | Focus | Deliverables | Effort |
|------|-------|--------------|--------|
| **Week 1** | IR Spec, Parser, Canonicalization | IR structures, deterministic hashing, unit tests | 30-40 hours |
| **Week 2** | Builtins, Adaptivity, Linting | Expression parser, linter, golden tests | 30-40 hours |
| **Week 3** | CLI, OpenAPI, Hardening | CLI commands, REST API, edge cases, docs | 30-40 hours |

**Total:** 90-120 hours (3 weeks full-time)

---

## Dependencies

**Required Crates:**
```toml
[dependencies]
# Existing
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
sha3 = "0.10"

# New for Policy Compiler
iso8601-duration = "0.2"  # Parse P365D durations
jsonschema = "0.17"       # Validate IR against schema
```

---

## Conclusion

The Policy Compiler PRD requires significant extensions to the existing agent policy module, but the foundation (YAML parsing, hashing) is already in place. The 3-week implementation plan is realistic with the right focus and prioritization.

**Recommendation:** Start with Week 1 (IR specification) in parallel with completing SAP Adapter Week 3 tasks, as they are independent work streams.

---

**Report Generated:** 2025-11-09
**Author:** Claude Code
**Project:** CAP Policy Compiler v1
**Status:** 📊 Gap Analysis Complete, Ready for Implementation
