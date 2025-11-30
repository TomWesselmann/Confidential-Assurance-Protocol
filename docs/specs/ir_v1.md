# IR v1 Specification - Intermediate Representation

**Version:** 1.0
**Status:** Production
**Date:** 2025-11-09
**Schema:** JSON Schema Draft 2020-12

---

## Overview

**IR v1** (Intermediate Representation v1) is the compiled output format of the Policy Compiler. It transforms PolicyV2 YAML into a canonical, deterministic JSON representation optimized for ZK proof generation and verification.

**Key Properties:**
- **Deterministic:** Same policy → same IR hash (100% reproducible)
- **Canonical:** Stable JSON key ordering (BTreeMap)
- **Sorted:** Rules sorted by ID for consistency
- **Hashable:** SHA3-256 for integrity verification
- **Self-contained:** All necessary data for proof generation

---

## IR v1 Structure

### Top-Level Schema

```json
{
  "ir_version": "1.0",
  "policy_id": "lksg.v1",
  "policy_hash": "sha3-256:b98c3db55f874476dc749ea32b70bdf5369a0d7bc5364f236e034f1ddcd94638",
  "rules": [ ... ],
  "adaptivity": { ... },
  "ir_hash": "sha3-256:df3a3eeb7c72f6204131397e4b0a4b16235f1e20cc66102153ad6d4ee78f892c"
}
```

### Field Descriptions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `ir_version` | String | Yes | Must be exactly `"1.0"` |
| `policy_id` | String | Yes | Original policy identifier from PolicyV2.id |
| `policy_hash` | String | Yes | SHA3-256 hash of source PolicyV2 (format: `sha3-256:<64 hex chars>`) |
| `rules` | Array | Yes | Compiled rules (sorted by ID) |
| `adaptivity` | Object | No | Compiled adaptive behavior (predicates + activations) |
| `ir_hash` | String | Yes | SHA3-256 hash of canonical IR JSON |

---

## Rules

### Rule Structure

```json
{
  "id": "no_sanctions",
  "op": "non_membership",
  "lhs": { "var": "supplier_hashes" },
  "rhs": { "var": "sanctions_root" }
}
```

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | String | Yes | Unique rule identifier (lowercase, alphanumeric, underscores) |
| `op` | String | Yes | Operator type (see Operators section) |
| `lhs` | Expression | Yes | Left-hand side expression |
| `rhs` | Expression | Yes | Right-hand side expression |

### Rule Ordering

**Critical:** Rules MUST be sorted by `id` in lexicographic order for deterministic hashing.

**Example:**
```json
{
  "rules": [
    { "id": "check_age", ... },
    { "id": "check_sanctions", ... },
    { "id": "verify_country", ... }
  ]
}
```

---

## Operators

### Week 1 Operators (Implemented)

#### 1. `non_membership` - Set Non-Membership Check

**Description:** Verifies that elements are NOT in a set (e.g., not on sanctions list)

**Signature:**
```
non_membership(element: hex, set_root: hex) -> bool
```

**Example:**
```json
{
  "id": "no_sanctions",
  "op": "non_membership",
  "lhs": { "var": "supplier_hashes" },
  "rhs": { "var": "sanctions_root" }
}
```

**Semantics:** Returns `true` if `supplier_hashes` is NOT in the Merkle tree rooted at `sanctions_root`.

---

#### 2. `eq` - Equality Check

**Description:** Checks equality between two values

**Signature:**
```
eq(left: any, right: any) -> bool
```

**Example:**
```json
{
  "id": "check_status",
  "op": "eq",
  "lhs": { "var": "status" },
  "rhs": "active"
}
```

**Semantics:** Returns `true` if `status == "active"`.

---

#### 3. `range_min` - Minimum Range Check

**Description:** Checks if a value meets a minimum threshold

**Signature:**
```
range_min(value: number, min: number) -> bool
```

**Example:**
```json
{
  "id": "check_age",
  "op": "range_min",
  "lhs": { "var": "age" },
  "rhs": 18
}
```

**Semantics:** Returns `true` if `age >= 18`.

---

### Week 2 Operators (Planned)

- `non_intersection` - Set non-intersection check
- `threshold` - Threshold check with percentage
- `range_max` - Maximum range check
- `in` - Set membership check

---

## Expressions

### Expression Types

IR v1 supports three expression types:

#### 1. Variable Reference
```json
{ "var": "variable_name" }
```

**Example:**
```json
{ "var": "supplier_hashes" }
```

---

#### 2. Literal Value
Literals are represented as raw JSON values (string, number, boolean, null, array, object).

**Examples:**
```json
"active"              // String literal
18                    // Number literal
true                  // Boolean literal
["US", "EU", "UK"]    // Array literal
```

---

#### 3. Function Call (Week 2)
```json
{
  "func": "function_name",
  "args": [...]
}
```

**Example:**
```json
{
  "func": "sub",
  "args": [
    { "func": "now" },
    { "func": "max", "args": [{ "var": "audit_dates" }] }
  ]
}
```

---

## Adaptivity (Optional)

### Structure

```json
{
  "adaptivity": {
    "predicates": [
      {
        "id": "recent_audit",
        "expr": {
          "func": "lt",
          "args": [
            { "func": "sub", "args": [{ "func": "now" }, { "func": "max", "args": ["audit_dates"] }] },
            "P365D"
          ]
        }
      }
    ],
    "activations": [
      {
        "when": "recent_audit",
        "rules": ["check_sanctions", "verify_country"]
      }
    ]
  }
}
```

### Predicates

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | String | Yes | Unique predicate identifier |
| `expr` | Expression | Yes | Boolean expression (evaluates to true/false) |

### Activations

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `when` | String | Yes | Predicate ID that triggers activation |
| `rules` | Array<String> | Yes | List of rule IDs to activate when predicate is true |

**Semantics:** When `when` predicate evaluates to `true`, all rules in `rules` array are activated.

---

## Canonical Ordering

### JSON Key Ordering

IR v1 uses **BTreeMap** for all JSON objects, ensuring lexicographic key ordering:

```json
{
  "adaptivity": { ... },
  "ir_hash": "sha3-256:...",
  "ir_version": "1.0",
  "policy_hash": "sha3-256:...",
  "policy_id": "lksg.v1",
  "rules": [ ... ]
}
```

Keys are sorted alphabetically.

### Rule Sorting

Rules MUST be sorted by `id` field in lexicographic order:

```json
{
  "rules": [
    { "id": "a_first_rule", ... },
    { "id": "b_second_rule", ... },
    { "id": "c_third_rule", ... }
  ]
}
```

### Predicate Sorting

Predicates MUST be sorted by `id` field:

```json
{
  "adaptivity": {
    "predicates": [
      { "id": "pred_a", ... },
      { "id": "pred_b", ... }
    ]
  }
}
```

### Activation Sorting

Activations MUST be sorted by `when` field:

```json
{
  "adaptivity": {
    "activations": [
      { "when": "pred_a", ... },
      { "when": "pred_b", ... }
    ]
  }
}
```

---

## Hashing

### Policy Hash

**Algorithm:** SHA3-256
**Input:** Canonical JSON of PolicyV2
**Format:** `sha3-256:<64 hex characters>`

**Example:**
```
sha3-256:b98c3db55f874476dc749ea32b70bdf5369a0d7bc5364f236e034f1ddcd94638
```

### IR Hash

**Algorithm:** SHA3-256
**Input:** Canonical JSON of IR v1 (without `ir_hash` field)
**Format:** `sha3-256:<64 hex characters>`

**Computation:**
1. Set `ir.ir_hash = ""`
2. Serialize IR to canonical JSON (compact, BTreeMap-ordered)
3. Compute SHA3-256 of JSON bytes
4. Set `ir.ir_hash = "sha3-256:" + hex(hash)`

**Example:**
```
sha3-256:df3a3eeb7c72f6204131397e4b0a4b16235f1e20cc66102153ad6d4ee78f892c
```

---

## Determinism Guarantees

### What is Guaranteed

✅ **Same PolicyV2 → Same policy_hash** (100% reproducible)
✅ **Same IR → Same ir_hash** (100% reproducible)
✅ **Canonical JSON ordering** (BTreeMap)
✅ **Rule sorting by ID** (lexicographic)
✅ **Predicate/Activation sorting** (by id/when)

### Verified Through

- 100-run determinism test suite (100% pass rate)
- CI non-determinism sentinel (fails build on any variance)
- Golden IR validation (byte-exact comparison)

---

## Example IR v1 (Complete)

### Input PolicyV2

```yaml
id: lksg.v1
version: "1.0"
legal_basis:
  - directive: "LkSG"
description: "Minimal LkSG supplier check"
inputs:
  supplier_hashes: {type: array, items: hex}
  sanctions_root: {type: hex}
rules:
  - id: no_sanctions
    op: non_membership
    lhs: supplier_hashes
    rhs: sanctions_root
```

### Output IR v1

```json
{
  "ir_version": "1.0",
  "policy_id": "lksg.v1",
  "policy_hash": "sha3-256:b98c3db55f874476dc749ea32b70bdf5369a0d7bc5364f236e034f1ddcd94638",
  "rules": [
    {
      "id": "no_sanctions",
      "op": "non_membership",
      "lhs": { "var": "supplier_hashes" },
      "rhs": { "var": "sanctions_root" }
    }
  ],
  "ir_hash": "sha3-256:df3a3eeb7c72f6204131397e4b0a4b16235f1e20cc66102153ad6d4ee78f892c"
}
```

---

## JSON Schema

See [`schemas/ir_v1.schema.json`](../schemas/ir_v1.schema.json) for the complete JSON Schema Draft 2020-12 definition.

**Key Constraints:**
- `ir_version` must be exactly `"1.0"`
- `policy_hash` must match pattern `^sha3-256:[0-9a-f]{64}$`
- `ir_hash` must match pattern `^sha3-256:[0-9a-f]{64}$`
- `rules[].op` must be one of: `["non_membership", "eq", "range_min"]`

---

## Validation

### CLI Validation

```bash
# Compile policy to IR
cargo run --bin cap-agent -- policy compile examples/lksg_v1.policy.yml -o output.ir.json

# Show IR
cargo run --bin cap-agent -- policy show output.ir.json
```

### Programmatic Validation

```rust
use cap_agent::policy_v2::{parse_yaml, generate_ir, canonicalize, sha3_256_hex};

let policy = parse_yaml("policy.yml")?;
let policy_json = serde_json::to_string(&policy)?;
let policy_hash = sha3_256_hex(&policy_json);

let mut ir = generate_ir(&policy, policy_hash)?;
let ir_canonical = canonicalize(&ir)?;
let ir_hash = sha3_256_hex(&ir_canonical);

ir.ir_hash = ir_hash;
```

---

## Week 2 Extensions (Planned)

### Builtin Functions

- `now()` - Current timestamp
- `len(array)` - Array length
- `max(array)` - Maximum value in array
- `sub(a, b)` - Subtraction
- `lt(a, b)` - Less-than comparison
- `add(a, b)` - Addition
- `mul(a, b)` - Multiplication

### Duration Parsing

ISO 8601 duration format:
- `P365D` - 365 days
- `P1Y` - 1 year
- `P6M` - 6 months

### Complex Expressions

```json
{
  "func": "lt",
  "args": [
    {
      "func": "sub",
      "args": [
        { "func": "now" },
        { "func": "max", "args": [{ "var": "audit_dates" }] }
      ]
    },
    "P365D"
  ]
}
```

**Semantics:** `now() - max(audit_dates) < 365 days`

---

## Performance

### Compilation Time (Week 1 Baseline)

- Parse YAML: ~0.5ms
- Lint: ~0.2ms
- Generate IR: ~0.3ms
- Compute hashes: ~0.4ms
- **Total: ~1.4ms** (for minimal policy)

### Memory Footprint

- Peak: <5 MB (for typical policies)
- IR size: ~300 bytes (minimal policy)

---

## See Also

- [Policy Lints Catalog](./policy_lints.md) - Error codes and warnings
- [PolicyV2 Schema](../schemas/policy_v2.schema.json) - Input schema
- [IR v1 Schema](../schemas/ir_v1.schema.json) - Output schema
- [Migration Notes](../MIGRATION_NOTES.md) - Upgrade guide

---

**Document Version:** 1.0
**Last Updated:** 2025-11-09
**Maintainer:** Claude Code (Anthropic)
**Status:** Production-Ready
