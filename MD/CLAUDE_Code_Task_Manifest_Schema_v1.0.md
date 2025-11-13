# Claude Task: Integrate Manifest Schema v1.0 Validation into CAP Agent

## 🎯 Goal
Integrate the **Confidential Assurance Manifest v1.0 JSON Schema** (`docs/manifest.schema.json`)  
into the Rust project, enabling validation via CLI command `cap manifest validate`.

---

## 🧱 Implementation Scope

### 1️⃣ Add Schema Version Constant
**File:** `agent/src/manifest.rs`

```rust
pub const MANIFEST_SCHEMA_VERSION: &str = "manifest.v1.0";

fn apply_defaults(manifest: &mut Manifest) {
    if manifest.version.is_empty() {
        manifest.version = MANIFEST_SCHEMA_VERSION.to_string();
    }
    // ensure required proof fields exist
    manifest.proof.backend.get_or_insert("mock".into());
    manifest.proof.vk_hash.get_or_insert_with(|| String::from("0x0"));
    manifest.proof.params_hash.get_or_insert_with(|| String::from("0x0"));
}
```

---

### 2️⃣ Add JSON Schema Validation Command
**New file:** `agent/src/cli/manifest_validate.rs`

```rust
use anyhow::{Result, bail};
use jsonschema::{Draft, JSONSchema};
use std::fs;

pub fn run(manifest_path: &str, schema_path: &str) -> Result<()> {
    let manifest_json: serde_json::Value = serde_json::from_str(&fs::read_to_string(manifest_path)?)?;
    let schema_json: serde_json::Value = serde_json::from_str(&fs::read_to_string(schema_path)?)?;

    let compiled = JSONSchema::options()
        .with_draft(Draft::Draft202012)
        .compile(&schema_json)?;

    if let Err(errors) = compiled.validate(&manifest_json) {
        for e in errors { eprintln!("validation error: {e}"); }
        bail!("manifest validation failed");
    }

    println!("✅ manifest is valid according to schema {schema_path}");
    Ok(())
}
```

---

### 3️⃣ Register CLI Command
**File:** `agent/src/main.rs`

```rust
.subcommand(
    Command::new("manifest")
        .about("Manifest operations")
        .subcommand(
            Command::new("validate")
                .about("Validate a manifest against the JSON schema")
                .arg(arg!(--file <FILE> "Manifest JSON file"))
                .arg(arg!(--schema <SCHEMA> "Schema JSON file"))
        )
)
```

---

### 4️⃣ Update Dependencies
**File:** `agent/Cargo.toml`

```toml
jsonschema = "0.17"
serde_json = "1"
```

---

### 5️⃣ Add Unit Tests
**File:** `agent/tests/test_manifest_validate.rs`

```rust
#[test]
fn test_manifest_validation_ok() {
    let res = run("tests/data/valid_manifest.json", "docs/manifest.schema.json");
    assert!(res.is_ok());
}

#[test]
fn test_manifest_validation_fail() {
    let res = run("tests/data/invalid_manifest.json", "docs/manifest.schema.json");
    assert!(res.is_err());
}
```

---

### 6️⃣ CLI Usage Example

```bash
cargo run -- manifest validate   --file build/manifest.json   --schema docs/manifest.schema.json
```

**Output**
```bash
✅ manifest is valid according to schema docs/manifest.schema.json
```

or (on error)
```bash
validation error: $.signatures[0].role: must be one of [company, auditor, verifier]
manifest validation failed
```

---

### 7️⃣ Update Documentation

**README.md → new section**
```markdown
### 🔍 Manifest Validation

Validates a generated Manifest against the official JSON schema.

```bash
cap manifest validate   --file build/manifest.json   --schema docs/manifest.schema.json
```

Exit codes:
- 0 → Manifest valid
- 1 → Validation failed
```

---

## ✅ Acceptance Criteria

| Criterion | Description |
|------------|-------------|
| ✔ CLI Command | `cap manifest validate` works offline with clear output |
| ✔ Schema Version | Manifest builds automatically with `version=manifest.v1.0` |
| ✔ JSON Schema | Validation uses Draft 2020-12 standard |
| ✔ Tests | Positive & negative validation cases pass (`cargo test`) |
| ✔ Docs | Command documented in README + linked in architecture docs |

---

## 🧩 References
- Schema File: `docs/manifest.schema.json`
- Example Manifest: `agent/build/manifest.json`
- Project Context: **Confidential Assurance Protocol (CAP)**
- Rust Edition: 2021
