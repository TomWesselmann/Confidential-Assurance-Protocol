# Migration Notes - PolicyV1 to PolicyV2

**Version:** 1.0
**Date:** 2025-11-09
**Applies to:** CAP Policy Compiler Week 1 → Week 3

---

## Overview

This document describes the migration path from the legacy **PolicyV1** system to the new **PolicyV2** compiler with IR v1 generation.

**Key Changes:**
- New YAML schema with structured fields
- Compiled IR v1 output format
- REST API endpoint changes
- Deterministic hashing guarantees
- Structured lint/error codes

---

## Breaking Changes

### 1. Policy Schema Changes

#### PolicyV1 (Legacy)
```yaml
version: "lksg.v1"
name: "My Policy"
created_at: "2025-11-06T10:00:00Z"
constraints:
  require_at_least_one_ubo: true
  supplier_count_max: 10
notes: ""
```

#### PolicyV2 (New)
```yaml
id: "lksg.v1"
version: "1.0"
legal_basis:
  - directive: "LkSG"
    article: "§3"
description: "My Policy"
inputs:
  supplier_hashes: {type: array, items: hex}
  sanctions_root: {type: hex}
rules:
  - id: no_sanctions
    op: non_membership
    lhs: supplier_hashes
    rhs: sanctions_root
```

**Migration Actions:**
1. Rename `name` → `description`
2. Add `id` field (new required field)
3. Add `legal_basis` array (required in strict mode)
4. Add `inputs` section for variables
5. Convert `constraints` → `rules` array

---

### 2. Field Renames

| PolicyV1 | PolicyV2 | Notes |
|----------|----------|-------|
| `name` | `description` | Semantic change: now optional |
| `version` (policy version) | `version` | Format change: semver-like |
| `created_at` | — | Removed (now in manifest) |
| `constraints` | `rules` | Structural change |
| `notes` | — | Removed |

---

### 3. New Required Fields

#### `id` (Required)
- **Type:** String
- **Pattern:** `^[a-z0-9\._-]+$`
- **Purpose:** Unique policy identifier

**Example:**
```yaml
id: "lksg.supplier_check.v1"
```

#### `legal_basis` (Required in Strict Mode)
- **Type:** Array
- **Min Items:** 1 (in strict mode)
- **Purpose:** Legal compliance reference

**Example:**
```yaml
legal_basis:
  - directive: "LkSG"
    article: "§3"
  - directive: "GDPR"
    article: "Article 6(1)(f)"
```

#### `inputs` (Required)
- **Type:** Object
- **Purpose:** Define input variables and types

**Example:**
```yaml
inputs:
  supplier_hashes: {type: array, items: hex}
  sanctions_root: {type: hex}
  age: {type: number}
```

#### `rules` (Required)
- **Type:** Array
- **Min Items:** 1
- **Purpose:** Define policy constraints as rules

**Example:**
```yaml
rules:
  - id: check_sanctions
    op: non_membership
    lhs: supplier_hashes
    rhs: sanctions_root
```

---

### 4. Constraints → Rules Migration

#### PolicyV1 Constraints (Boolean Flags)

```yaml
constraints:
  require_at_least_one_ubo: true
  supplier_count_max: 10
  ubo_count_min: 1
```

#### PolicyV2 Rules (Explicit Operators)

```yaml
rules:
  - id: check_ubo_count
    op: range_min
    lhs: {var: "ubo_count"}
    rhs: 1

  - id: check_supplier_count
    op: range_max
    lhs: {var: "supplier_count"}
    rhs: 10
```

**Migration Table:**

| V1 Constraint | V2 Rule |
|---------------|---------|
| `require_at_least_one_ubo: true` | `{op: "range_min", lhs: "ubo_count", rhs: 1}` |
| `supplier_count_max: 10` | `{op: "range_max", lhs: "supplier_count", rhs: 10}` |
| `ubo_count_min: 2` | `{op: "range_min", lhs: "ubo_count", rhs: 2}` |

---

## API Endpoint Changes

### REST API Migration

#### PolicyV1 API (Legacy)

**Endpoint:** `POST /policy/compile`

**Request:**
```json
{
  "policy": {
    "version": "lksg.v1",
    "name": "Test Policy",
    "created_at": "2025-11-06T10:00:00Z",
    "constraints": {
      "require_at_least_one_ubo": true,
      "supplier_count_max": 10
    },
    "notes": ""
  }
}
```

**Response:**
```json
{
  "policy_hash": "0x...",
  "policy_info": {
    "name": "Test Policy",
    "version": "lksg.v1",
    "hash": "0x..."
  },
  "status": "compiled"
}
```

---

#### PolicyV2 API (New)

**Endpoint:** `POST /policy/v2/compile`

**Request:**
```json
{
  "policy_yaml": "base64:aWQ6IGxrc2cudjEKdmVyc2lvbjogIjEuMCIKbGVnYWxfYmFzaXM6CiAgLSBkaXJlY3RpdmU6ICJMS3NHIgo=...",
  "lint_mode": "strict",
  "persist": true
}
```

**Alternative Request (Direct JSON):**
```json
{
  "policy": {
    "id": "lksg.v1",
    "version": "1.0",
    "legal_basis": [{"directive": "LkSG"}],
    "description": "Test Policy",
    "inputs": {...},
    "rules": [...]
  },
  "lint_mode": "strict",
  "persist": true
}
```

**Response (Success):**
```json
{
  "policy_id": "lksg.v1",
  "policy_hash": "sha3-256:...",
  "ir": {
    "ir_version": "1.0",
    "policy_id": "lksg.v1",
    "policy_hash": "sha3-256:...",
    "rules": [...],
    "ir_hash": "sha3-256:..."
  },
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

**Response (Error - HTTP 422):**
```json
{
  "policy_id": "lksg.v1",
  "policy_hash": "",
  "ir": {...},
  "ir_hash": "",
  "lints": [
    {
      "code": "E1002",
      "level": "error",
      "message": "missing `legal_basis`",
      "rule_id": null
    }
  ],
  "stored": false,
  "etag": ""
}
```

---

### API Endpoint Comparison

| Feature | V1 API | V2 API |
|---------|--------|--------|
| **Endpoint** | `/policy/compile` | `/policy/v2/compile` |
| **Input** | JSON Policy | Base64 YAML or JSON |
| **Lint Mode** | No | Yes (strict/relaxed) |
| **Lint Diagnostics** | No | Yes (structured codes) |
| **IR Output** | No | Yes (IR v1) |
| **ETag Support** | No | Yes |
| **HTTP 422 Errors** | No | Yes |
| **Hash Format** | `0x<hex>` | `sha3-256:<hex>` |
| **Determinism** | Not guaranteed | 100% deterministic |

---

## Hash Format Changes

### PolicyV1
```
0xd490be94abc123def456...
```
- Format: `0x` prefix + hex
- Algorithm: SHA3-256 (implicit)

### PolicyV2
```
sha3-256:b98c3db55f874476dc749ea32b70bdf5369a0d7bc5364f236e034f1ddcd94638
```
- Format: `sha3-256:` prefix + 64 hex chars
- Algorithm: Explicit SHA3-256

**Migration:** Strip `0x`, prepend `sha3-256:`

---

## Determinism Improvements

### PolicyV1
- ❌ Non-deterministic JSON serialization
- ❌ HashMap key ordering
- ❌ No canonical rule sorting
- ❌ No hash verification

### PolicyV2
- ✅ Deterministic BTreeMap ordering
- ✅ Rules sorted by ID
- ✅ Canonical JSON serialization
- ✅ Verified with 100-run test suite

**Result:** 100% reproducible hashes across compilations

---

## Lint Error Handling

### PolicyV1
- No structured error codes
- Free-form error messages
- No HTTP status mapping

**Example:**
```
Error: Policy validation failed: Missing legal basis
```

### PolicyV2
- Structured error codes (E/W format)
- Machine-readable diagnostics
- HTTP status mapping (422 for errors, 200 for warnings)

**Example:**
```json
{
  "code": "E1002",
  "level": "error",
  "message": "missing `legal_basis`",
  "rule_id": null
}
```

**See:** [Policy Lints Catalog](docs/policy_lints.md) for all codes

---

## CLI Changes

### PolicyV1
```bash
# No CLI available
```

### PolicyV2
```bash
# Lint policy
cargo run --bin cap-agent -- policy lint examples/lksg_v1.policy.yml --strict

# Compile to IR
cargo run --bin cap-agent -- policy compile examples/lksg_v1.policy.yml -o output.ir.json

# Show IR
cargo run --bin cap-agent -- policy show output.ir.json
```

---

## Migration Checklist

### For Policy Authors

- [ ] Add `id` field to policy
- [ ] Rename `name` → `description`
- [ ] Add `legal_basis` array
- [ ] Add `inputs` section
- [ ] Convert `constraints` → `rules`
- [ ] Remove `created_at` (now in manifest)
- [ ] Remove `notes` field
- [ ] Validate with `policy lint --strict`

### For API Consumers

- [ ] Update endpoint: `/policy/compile` → `/policy/v2/compile`
- [ ] Handle new response structure (includes `ir`)
- [ ] Handle lint diagnostics in response
- [ ] Handle HTTP 422 for errors
- [ ] Support ETag caching (optional)
- [ ] Update hash format parsing: `0x...` → `sha3-256:...`

### For Developers

- [ ] Update PolicyV1 imports → PolicyV2
- [ ] Replace Policy → PolicyV2 in code
- [ ] Use `policy_v2` module instead of `policy`
- [ ] Update tests to use new schema
- [ ] Update mocks to include required fields

---

## Backward Compatibility

### ❌ No Backward Compatibility

PolicyV2 is **NOT backward compatible** with PolicyV1. All policies must be migrated.

**Reasons:**
- Fundamental schema changes (constraints → rules)
- New required fields (`id`, `legal_basis`, `inputs`)
- Different semantics (declarative rules vs boolean constraints)

### Coexistence Strategy

Both APIs are available during transition:

- **Legacy:** `POST /policy/compile` (PolicyV1)
- **New:** `POST /policy/v2/compile` (PolicyV2)

**Recommendation:** Migrate all policies to PolicyV2 within 1 month, then deprecate V1 API.

---

## Migration Example

### Before (PolicyV1)

```yaml
version: "lksg.v1"
name: "LkSG Supplier Check"
created_at: "2025-11-06T10:00:00Z"
constraints:
  require_at_least_one_ubo: true
  supplier_count_max: 10
notes: "Production policy"
```

### After (PolicyV2)

```yaml
id: "lksg.supplier_check.v1"
version: "1.0"
legal_basis:
  - directive: "LkSG"
    article: "§3"
description: "LkSG Supplier Check"
inputs:
  supplier_hashes: {type: array, items: hex}
  sanctions_root: {type: hex}
  ubo_count: {type: number}
  supplier_count: {type: number}
rules:
  - id: check_sanctions
    op: non_membership
    lhs: supplier_hashes
    rhs: sanctions_root

  - id: check_ubo_count
    op: range_min
    lhs: ubo_count
    rhs: 1

  - id: check_supplier_count
    op: range_max
    lhs: supplier_count
    rhs: 10
```

---

## Testing Migration

### Validate PolicyV2

```bash
# Lint with strict mode
cargo run --bin cap-agent -- policy lint policy_v2.yml --strict

# Compile to IR
cargo run --bin cap-agent -- policy compile policy_v2.yml -o output.ir.json

# Verify determinism
./ci/non_determinism_check.sh
```

### Test API

```bash
TOKEN="<your-oauth2-token>"

# Compile policy
curl -X POST http://localhost:8080/policy/v2/compile \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "policy_yaml": "base64:...",
    "lint_mode": "strict",
    "persist": true
  }'
```

---

## Support

For migration assistance:
- Read [IR v1 Specification](docs/ir_v1.md)
- Read [Policy Lints Catalog](docs/policy_lints.md)
- Run `policy lint` for validation errors
- Check [Week 3 Progress](../sap-adapter/POLICY_COMPILER_WEEK3_PROGRESS.md)

---

**Document Version:** 1.0
**Last Updated:** 2025-11-09
**Maintainer:** Claude Code (Anthropic)
**Status:** Production-Ready
