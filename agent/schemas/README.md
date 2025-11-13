# CAP Policy Compiler - JSON Schemas

This directory contains JSON Schema definitions for the CAP Policy Compiler v2.

## Schemas

### `policy_v2.schema.json`

JSON Schema for PolicyV2 YAML files. Defines the structure for policy definitions including:
- Policy metadata (id, version, description)
- Legal basis (directive, article)
- Input variable definitions
- Rules with operators (non_membership, eq, range_min)
- Adaptive behavior (predicates and activations)

**Compliance:** JSON Schema Draft 2020-12

**Example:** See `../examples/lksg_v1.policy.yml`

### `ir_v1.schema.json`

JSON Schema for compiled Intermediate Representation (IR) v1. Defines the structure for:
- IR metadata (ir_version, policy_id)
- Deterministic hashes (policy_hash, ir_hash with SHA3-256)
- Compiled rules (sorted by ID for determinism)
- Canonical JSON structure (BTreeMap ordering)

**Compliance:** JSON Schema Draft 2020-12

**Example:** See `../examples/lksg_v1.ir.json`

## Validation

### Using jsonschema CLI

```bash
# Validate policy (requires conversion from YAML to JSON)
jsonschema -i examples/lksg_v1.policy.json schemas/policy_v2.schema.json

# Validate IR
jsonschema -i examples/lksg_v1.ir.json schemas/ir_v1.schema.json
```

### Using Python

```python
import jsonschema
import json

# Load schema
with open('schemas/ir_v1.schema.json') as f:
    schema = json.load(f)

# Load IR
with open('examples/lksg_v1.ir.json') as f:
    ir = json.load(f)

# Validate
jsonschema.validate(instance=ir, schema=schema)
print("✅ Valid")
```

## Schema Details

### PolicyV2 Required Fields

- `id` - Unique policy identifier (pattern: `^[a-z0-9\\._-]+$`)
- `version` - Policy version string
- `legal_basis` - Array with at least one legal basis entry
- `inputs` - Object with input variable definitions
- `rules` - Array of rules with unique IDs

### IR v1 Required Fields

- `ir_version` - Must be exactly "1.0"
- `policy_id` - Original policy identifier
- `policy_hash` - SHA3-256 hash (pattern: `^sha3-256:[0-9a-f]{64}$`)
- `rules` - Array of compiled rules (sorted by ID)
- `ir_hash` - SHA3-256 hash of canonical IR

### Operators

Supported operators in both PolicyV2 and IR v1:
- `non_membership` - Set non-membership check
- `eq` - Equality check
- `range_min` - Minimum range check

### Determinism Guarantees

The IR schema enforces determinism through:
1. **Canonical Ordering**: Rules sorted by ID
2. **Hash Format**: Strict SHA3-256 format validation
3. **BTreeMap**: JSON object keys in sorted order

## Version History

- **v1.0** (2025-11-09) - Initial schema definitions for Policy Compiler Week 1
