# Policy Compiler - Lint Codes Catalog (Week 3)

**Version:** 1.0
**Status:** Production
**Date:** 2025-11-09

---

## Overview

The Policy Compiler uses structured, machine-readable lint codes to report validation errors and warnings. Each code follows the format `[E|W]NNNN` where:
- **E** = Error (blocks compilation in strict mode)
- **W** = Warning (informational only)
- **NNNN** = 4-digit code

---

## Lint Code Categories

| Category | Range | Description |
|----------|-------|-------------|
| **E1xxx** | 1001-1999 | Policy structure errors |
| **E2xxx** | 2001-2999 | Expression/operator errors |
| **E3xxx** | 3001-3999 | Constraint errors |
| **W1xxx** | 1001-1999 | Policy structure warnings |

---

## Error Codes (E)

### E1xxx - Policy Structure Errors

#### E1001 - Unknown Rule ID in Activation
**Level:** Error (Strict)
**HTTP Status:** 422 Unprocessable Entity
**Message:** `unknown rule id '{id}' in activation '{activation_id}'`

**Description:**
An activation references a rule ID that does not exist in the policy's `rules` array.

**Example:**
```yaml
rules:
  - id: rule_a
    op: eq
    lhs: var1
    rhs: var2

adaptivity:
  predicates:
    - id: pred1
      expr: "now() > '2025-01-01'"
  activations:
    - when: pred1
      rules: ["rule_b"]  # ❌ ERROR: rule_b doesn't exist
```

**Fix:**
Ensure all rule IDs referenced in `activations` exist in the `rules` array.

---

#### E1002 - Missing Legal Basis
**Level:** Error (Strict) / Warning (Relaxed)
**HTTP Status:** 422 Unprocessable Entity (strict) / 200 OK (relaxed)
**Message:** `missing \`legal_basis\``

**Description:**
The policy's `legal_basis` array is empty. In strict mode, this is a compilation error. In relaxed mode, it's a warning.

**Example:**
```yaml
id: my.policy
version: "1.0"
legal_basis: []  # ❌ ERROR in strict mode
rules:
  - id: rule1
    op: eq
    lhs: a
    rhs: b
```

**Fix:**
Add at least one legal basis entry:
```yaml
legal_basis:
  - directive: "GDPR"
    article: "Article 6(1)(f)"
```

---

#### E1003 - Duplicate Rule ID
**Level:** Error
**HTTP Status:** 422 Unprocessable Entity
**Message:** `duplicate rule ID '{id}'`

**Description:**
Two or more rules have the same `id` field. Rule IDs must be unique within a policy.

**Example:**
```yaml
rules:
  - id: check_sanctions  # ❌ ERROR: duplicate ID
    op: non_membership
    lhs: supplier_hashes
    rhs: sanctions_root
  - id: check_sanctions  # ❌ ERROR: duplicate ID
    op: eq
    lhs: status
    rhs: "active"
```

**Fix:**
Ensure each rule has a unique ID:
```yaml
rules:
  - id: check_sanctions
    ...
  - id: check_status
    ...
```

---

### E2xxx - Expression/Operator Errors

#### E2001 - Invalid Operator
**Level:** Error
**HTTP Status:** 422 Unprocessable Entity
**Message:** `invalid op '{operator}' (allowed: non_membership, eq, range_min)`

**Description:**
The rule's `op` field contains an unsupported operator.

**Allowed operators:**
- `non_membership` - Set non-membership check
- `eq` - Equality check
- `range_min` - Minimum range check

**Example:**
```yaml
rules:
  - id: bad_rule
    op: greater_than  # ❌ ERROR: unsupported operator
    lhs: age
    rhs: 18
```

**Fix:**
Use only supported operators:
```yaml
rules:
  - id: valid_rule
    op: range_min
    lhs: age
    rhs: 18
```

---

#### E2003 - Unknown Input Reference
**Level:** Error
**HTTP Status:** 422 Unprocessable Entity
**Message:** `expr references unknown input '{input_name}'`

**Description:**
An expression references an input variable that is not defined in the policy's `inputs` section.

**Status:** Not yet implemented (Week 2 feature)

**Example:**
```yaml
inputs:
  supplier_id: {type: hex}

rules:
  - id: check_country
    op: eq
    lhs: country_code  # ❌ ERROR: not in inputs
    rhs: "US"
```

**Fix:**
Add the referenced input to the `inputs` section:
```yaml
inputs:
  supplier_id: {type: hex}
  country_code: {type: string}
```

---

### E3xxx - Constraint Errors

#### E3002 - Invalid range_min Expression
**Level:** Error
**HTTP Status:** 422 Unprocessable Entity
**Message:** `range_min.lhs must be sub(now(), max(audit_dates))`

**Description:**
The `range_min` operator requires specific expression patterns for temporal constraints.

**Status:** Not yet implemented (Week 2 feature)

**Example:**
```yaml
rules:
  - id: audit_age_check
    op: range_min
    lhs: some_var  # ❌ ERROR: invalid expression
    rhs: P365D
```

**Fix:**
Use the correct expression pattern:
```yaml
rules:
  - id: audit_age_check
    op: range_min
    lhs: {func: "sub", args: [{func: "now"}, {func: "max", args: ["audit_dates"]}]}
    rhs: P365D
```

---

## Warning Codes (W)

### W1xxx - Policy Structure Warnings

#### W1002 - Description Missing
**Level:** Warning
**HTTP Status:** 200 OK
**Message:** `description missing`

**Description:**
The policy's `description` field is empty or missing. This is informational only and does not block compilation.

**Example:**
```yaml
id: my.policy
version: "1.0"
description: ""  # ⚠️ WARNING
legal_basis:
  - directive: "GDPR"
rules:
  - id: rule1
    op: eq
    lhs: a
    rhs: b
```

**Fix (optional):**
Add a descriptive text:
```yaml
description: "GDPR compliance policy for supplier verification"
```

---

## HTTP Status Code Mapping

| Lint Level | HTTP Status | Use Case |
|------------|-------------|----------|
| **Error** (E-codes) | 422 Unprocessable Entity | Compilation failed in strict mode |
| **Warning** (W-codes) | 200 OK | Compilation succeeded with warnings |
| No diagnostics | 200 OK | Compilation succeeded |

---

## Lint Modes

### Strict Mode
- **E-codes** → Compilation fails (HTTP 422)
- **W-codes** → Included in response
- Default for production use

### Relaxed Mode
- **E1002** (missing legal_basis) → Downgraded to Warning
- Other **E-codes** → Still treated as errors
- **W-codes** → Included in response
- Used for development/testing

---

## API Response Format

### Success with Warnings
```json
{
  "policy_id": "my.policy",
  "policy_hash": "sha3-256:...",
  "ir": { ... },
  "ir_hash": "sha3-256:...",
  "lints": [
    {
      "code": "W1002",
      "level": "warning",
      "message": "description missing",
      "rule_id": null
    }
  ],
  "stored": true,
  "etag": "\"ir:sha3-256:...\""
}
```

### Failure with Errors (HTTP 422)
```json
{
  "policy_id": "my.policy",
  "policy_hash": "",
  "ir": { "ir_version": "1.0", "policy_id": "my.policy", ... },
  "ir_hash": "",
  "lints": [
    {
      "code": "E1002",
      "level": "error",
      "message": "missing `legal_basis`",
      "rule_id": null
    },
    {
      "code": "E2001",
      "level": "error",
      "message": "invalid op 'foo' (allowed: non_membership, eq, range_min)",
      "rule_id": "bad_rule"
    }
  ],
  "stored": false,
  "etag": ""
}
```

---

## Testing

### Unit Tests
```bash
cargo test --lib policy_v2::linter
```

### Integration Tests
```bash
cargo test --test test_policy_determinism
```

---

## Future Lint Codes (Week 2+)

| Code | Description | Status |
|------|-------------|--------|
| E2003 | Unknown input reference | Planned for Week 2 |
| E2004 | Type mismatch in expression | Planned for Week 2 |
| E3002 | Invalid range_min expression | Planned for Week 2 |
| E3003 | Invalid duration format | Planned for Week 2 |
| W2001 | Unused input variable | Planned for Week 3 |
| W2002 | Unused predicate | Planned for Week 3 |

---

## See Also

- [IR v1 Specification](./ir_v1.md) - Intermediate Representation format
- [Migration Notes](../MIGRATION_NOTES.md) - Breaking changes and upgrades
- [Week 3 Specification](/Users/tomwesselmann/Desktop/PolicyCompiler_Week3_Slice.md) - Full requirements

---

**Document Version:** 1.0
**Last Updated:** 2025-11-09
**Maintainer:** Claude Code (Anthropic)
