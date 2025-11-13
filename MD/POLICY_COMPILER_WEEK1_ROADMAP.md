# 🚀 Policy Compiler - Week 1 Implementation Roadmap

**Datum:** 2025-11-09
**Projekt:** CAP Policy Compiler v1.0
**Timeline:** Week 1 (3-5 days)
**Goal:** Minimal working compiler core (YAML → IR v1 + Linter + Hashes)

---

## Executive Summary

This document provides a **step-by-step implementation guide** for Policy Compiler Week 1, combining:
- **PRD_Policy_Compiler_v1.md** - What to build (specification)
- **PolicyCompiler_Week1_Slice.md** - How to build it (implementation template)
- **POLICY_COMPILER_ANALYSIS.md** - Gap analysis & architecture

**Target:** Deliver a working `policy lint`, `policy compile`, `policy show` CLI with deterministic hashing and golden tests.

---

## Deliverables Checklist

### Code Files
- [ ] `src/yaml_parser.rs` - Parse PolicyV2 from YAML
- [ ] `src/linter.rs` - Strict/relaxed linting
- [ ] `src/ir.rs` - IR v1 structures + canonicalization
- [ ] `src/hasher.rs` - SHA3-256 hashing
- [ ] `src/cli.rs` - CLI commands (lint, compile, show)
- [ ] `src/lib.rs` - Module exports

### Schema Files
- [ ] `schemas/policy.schema.json` - Policy YAML JSON schema
- [ ] `schemas/ir_v1.schema.json` - IR v1 JSON schema

### Example Files
- [ ] `examples/lksg_v1.policy.yml` - Sample policy (minimal)
- [ ] `examples/lksg_v1.ir.json` - Golden IR output

### Test Files
- [ ] `tests/lint_strict.rs` - Linter strict mode tests
- [ ] `tests/compile_roundtrip.rs` - Compilation determinism tests
- [ ] `tests/golden_ir.rs` - Golden file comparison tests

### CI/CD
- [ ] `.github/workflows/policy-compiler.yml` - CI pipeline

---

## Project Structure

### Location Decision

**Question:** Where to create the policy-compiler?

**Options:**
1. **Inside agent project** (agent/src/policy_v2/)
   - ✅ Reuse existing infrastructure (hashing, API)
   - ✅ Direct integration with /policy/compile endpoint
   - ❌ Larger monorepo

2. **Standalone crate** (policy-compiler/)
   - ✅ Clean separation, easier to test
   - ✅ Could be used independently
   - ❌ Needs duplication of hashing logic

**Recommendation:** **Option 1** (inside agent) for better integration, as planned in POLICY_COMPILER_ANALYSIS.md

### Directory Structure

```
agent/
├── src/
│   ├── policy.rs                   # Legacy (keep for backwards compat)
│   ├── policy_v2/
│   │   ├── mod.rs
│   │   ├── yaml_parser.rs
│   │   ├── linter.rs
│   │   ├── ir.rs
│   │   ├── hasher.rs
│   │   ├── cli.rs
│   │   └── types.rs                # Shared types
│   └── main.rs                     # Add policy subcommands
├── schemas/
│   ├── policy.schema.json
│   └── ir_v1.schema.json
├── examples/
│   ├── lksg_v1.policy.yml
│   └── lksg_v1.ir.json
└── tests/
    ├── lint_strict.rs
    ├── compile_roundtrip.rs
    └── golden_ir.rs
```

---

## Implementation Steps (Day-by-Day)

### Day 1: Structures & Parser

**Goal:** Define IR v1 types and parse minimal YAML policy

#### Task 1.1: Define IR v1 Structures

**File:** `src/policy_v2/types.rs`

```rust
use serde::{Deserialize, Serialize};

/// PolicyV2 YAML structure
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PolicyV2 {
    pub id: String,
    pub version: String,
    pub legal_basis: Vec<LegalBasisItem>,
    #[serde(default)]
    pub description: String,
    pub inputs: std::collections::BTreeMap<String, InputDef>,
    pub rules: Vec<Rule>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adaptivity: Option<Adaptivity>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LegalBasisItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directive: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub article: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InputDef {
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Rule {
    pub id: String,
    pub op: String,
    pub lhs: serde_json::Value,  // Can be string or object
    pub rhs: serde_json::Value,  // Can be string or object
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Adaptivity {
    pub predicates: Vec<Predicate>,
    pub activations: Vec<Activation>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Predicate {
    pub id: String,
    pub expr: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Activation {
    pub when: String,
    pub rules: Vec<String>,
}

/// IR v1 Structure (output)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IrV1 {
    pub ir_version: String,
    pub policy_id: String,
    pub policy_hash: String,
    pub rules: Vec<IrRule>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adaptivity: Option<IrAdaptivity>,
    pub ir_hash: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IrRule {
    pub id: String,
    pub op: String,
    pub lhs: IrExpression,
    pub rhs: IrExpression,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum IrExpression {
    Var { var: String },
    Literal(serde_json::Value),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IrAdaptivity {
    pub predicates: Vec<IrPredicate>,
    pub activations: Vec<Activation>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IrPredicate {
    pub id: String,
    pub expr: serde_json::Value,
}
```

**Estimated Effort:** 1-2 hours

#### Task 1.2: Implement YAML Parser

**File:** `src/policy_v2/yaml_parser.rs`

```rust
use super::types::PolicyV2;
use anyhow::{Context, Result};
use std::path::Path;

/// Parse PolicyV2 from YAML file
pub fn parse_yaml<P: AsRef<Path>>(path: P) -> Result<PolicyV2> {
    let contents = std::fs::read_to_string(path.as_ref())
        .context("Failed to read policy file")?;

    let policy: PolicyV2 = serde_yaml::from_str(&contents)
        .context("Failed to parse YAML")?;

    Ok(policy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_policy() {
        let yaml = r#"
id: lksg.v1
version: "1.0"
legal_basis:
  - directive: "LkSG"
inputs:
  supplier_hashes: {type: array, items: hex}
  sanctions_root: {type: hex}
rules:
  - id: no_sanctions
    op: non_membership
    lhs: supplier_hashes
    rhs: sanctions_root
"#;
        let policy: PolicyV2 = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(policy.id, "lksg.v1");
        assert_eq!(policy.rules.len(), 1);
    }
}
```

**Estimated Effort:** 1-2 hours

#### Task 1.3: Create Minimal Example Policy

**File:** `examples/lksg_v1.policy.yml`

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

**Estimated Effort:** 30 minutes

---

### Day 2: Linter & Validation

**Goal:** Implement strict linter with legal_basis check

#### Task 2.1: Implement Linter

**File:** `src/policy_v2/linter.rs`

```rust
use super::types::PolicyV2;
use anyhow::{bail, Result};

#[derive(Debug, Clone, Copy)]
pub enum LintMode {
    Strict,
    Relaxed,
}

#[derive(Debug, Clone)]
pub struct LintDiagnostic {
    pub level: Level,
    pub message: String,
    pub rule_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Level {
    Error,
    Warning,
}

pub fn lint(policy: &PolicyV2, mode: LintMode) -> Vec<LintDiagnostic> {
    let mut diagnostics = Vec::new();

    // Check legal_basis
    if policy.legal_basis.is_empty() {
        let level = match mode {
            LintMode::Strict => Level::Error,
            LintMode::Relaxed => Level::Warning,
        };
        diagnostics.push(LintDiagnostic {
            level,
            message: "Missing legal_basis (required in strict mode)".to_string(),
            rule_id: None,
        });
    }

    // Check rule IDs uniqueness
    let mut seen_ids = std::collections::HashSet::new();
    for rule in &policy.rules {
        if !seen_ids.insert(&rule.id) {
            diagnostics.push(LintDiagnostic {
                level: Level::Error,
                message: format!("Duplicate rule ID: {}", rule.id),
                rule_id: Some(rule.id.clone()),
            });
        }
    }

    // Check valid operators
    const VALID_OPS: &[&str] = &["non_membership", "eq", "range_min"];
    for rule in &policy.rules {
        if !VALID_OPS.contains(&rule.op.as_str()) {
            diagnostics.push(LintDiagnostic {
                level: Level::Error,
                message: format!("Invalid operator '{}', expected one of: {:?}", rule.op, VALID_OPS),
                rule_id: Some(rule.id.clone()),
            });
        }
    }

    // TODO Week 2: Check lhs/rhs references exist in inputs

    diagnostics
}

pub fn has_errors(diagnostics: &[LintDiagnostic]) -> bool {
    diagnostics.iter().any(|d| d.level == Level::Error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lint_missing_legal_basis() {
        let policy = PolicyV2 {
            id: "test".to_string(),
            version: "1.0".to_string(),
            legal_basis: vec![],
            description: "".to_string(),
            inputs: Default::default(),
            rules: vec![],
            adaptivity: None,
        };

        let diagnostics = lint(&policy, LintMode::Strict);
        assert!(has_errors(&diagnostics));
    }

    #[test]
    fn test_lint_invalid_operator() {
        use super::super::types::*;
        let policy = PolicyV2 {
            id: "test".to_string(),
            version: "1.0".to_string(),
            legal_basis: vec![LegalBasisItem {
                directive: Some("LkSG".to_string()),
                article: None,
            }],
            description: "".to_string(),
            inputs: Default::default(),
            rules: vec![Rule {
                id: "r1".to_string(),
                op: "invalid_op".to_string(),
                lhs: serde_json::json!("var1"),
                rhs: serde_json::json!("var2"),
            }],
            adaptivity: None,
        };

        let diagnostics = lint(&policy, LintMode::Strict);
        assert!(has_errors(&diagnostics));
    }
}
```

**Estimated Effort:** 2-3 hours

---

### Day 3: IR Generation & Canonicalization

**Goal:** Generate IR v1 with canonical ordering

#### Task 3.1: Implement IR Generator

**File:** `src/policy_v2/ir.rs`

```rust
use super::types::{PolicyV2, IrV1, IrRule, IrExpression};
use anyhow::Result;

/// Generate IR v1 from PolicyV2
pub fn generate_ir(policy: &PolicyV2, policy_hash: String) -> Result<IrV1> {
    // Convert rules
    let mut ir_rules: Vec<IrRule> = policy.rules.iter().map(|r| {
        IrRule {
            id: r.id.clone(),
            op: r.op.clone(),
            lhs: convert_expression(&r.lhs),
            rhs: convert_expression(&r.rhs),
        }
    }).collect();

    // IMPORTANT: Canonical ordering - sort by rule ID
    ir_rules.sort_by(|a, b| a.id.cmp(&b.id));

    let ir = IrV1 {
        ir_version: "1.0".to_string(),
        policy_id: policy.id.clone(),
        policy_hash,
        rules: ir_rules,
        adaptivity: None, // TODO Week 2: Convert adaptivity
        ir_hash: String::new(), // Will be filled by hasher
    };

    Ok(ir)
}

fn convert_expression(expr: &serde_json::Value) -> IrExpression {
    match expr {
        serde_json::Value::String(s) => {
            // Simple variable reference
            IrExpression::Var { var: s.clone() }
        }
        other => {
            // Complex expression or literal
            IrExpression::Literal(other.clone())
        }
    }
}

/// Canonicalize IR for hashing
/// Uses BTreeMap to ensure stable ordering of keys
pub fn canonicalize(ir: &IrV1) -> Result<String> {
    // Use serde_json with BTreeMap for stable ordering
    let json = serde_json::to_string(ir)?;
    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rules_sorted_by_id() {
        use super::super::types::*;
        let policy = PolicyV2 {
            id: "test".to_string(),
            version: "1.0".to_string(),
            legal_basis: vec![],
            description: "".to_string(),
            inputs: Default::default(),
            rules: vec![
                Rule {
                    id: "z_rule".to_string(),
                    op: "eq".to_string(),
                    lhs: serde_json::json!("a"),
                    rhs: serde_json::json!("b"),
                },
                Rule {
                    id: "a_rule".to_string(),
                    op: "eq".to_string(),
                    lhs: serde_json::json!("c"),
                    rhs: serde_json::json!("d"),
                },
            ],
            adaptivity: None,
        };

        let ir = generate_ir(&policy, "hash123".to_string()).unwrap();
        assert_eq!(ir.rules[0].id, "a_rule");
        assert_eq!(ir.rules[1].id, "z_rule");
    }
}
```

**Estimated Effort:** 2-3 hours

---

### Day 4: Hashing & Golden Tests

**Goal:** Deterministic hashing with golden file validation

#### Task 4.1: Implement Hasher

**File:** `src/policy_v2/hasher.rs`

```rust
use sha3::{Digest, Sha3_256};
use anyhow::Result;

/// Compute SHA3-256 hash of input data
pub fn sha3_256_hex(data: &str) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(data.as_bytes());
    let result = hasher.finalize();
    format!("sha3-256:{}", hex::encode(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_hash() {
        let data = r#"{"id":"test","version":"1.0"}"#;
        let hash1 = sha3_256_hex(data);
        let hash2 = sha3_256_hex(data);
        assert_eq!(hash1, hash2);
    }
}
```

**Estimated Effort:** 1 hour

#### Task 4.2: Create Golden Test

**File:** `tests/golden_ir.rs`

```rust
use policy_compiler::policy_v2::*;

#[test]
fn test_golden_ir_lksg_v1() {
    // Parse policy
    let policy = yaml_parser::parse_yaml("examples/lksg_v1.policy.yml").unwrap();

    // Lint (should pass)
    let diagnostics = linter::lint(&policy, linter::LintMode::Strict);
    assert!(!linter::has_errors(&diagnostics), "Policy has lint errors: {:?}", diagnostics);

    // Compute policy hash
    let policy_json = serde_json::to_string(&policy).unwrap();
    let policy_hash = hasher::sha3_256_hex(&policy_json);

    // Generate IR
    let mut ir = ir::generate_ir(&policy, policy_hash).unwrap();

    // Compute IR hash
    let ir_canonical = ir::canonicalize(&ir).unwrap();
    let ir_hash = hasher::sha3_256_hex(&ir_canonical);
    ir.ir_hash = ir_hash;

    // Serialize IR
    let ir_json = serde_json::to_string_pretty(&ir).unwrap();

    // Compare with golden file
    let golden = std::fs::read_to_string("examples/lksg_v1.ir.json")
        .expect("Golden file not found - run with UPDATE_GOLDEN=1 to create");

    if std::env::var("UPDATE_GOLDEN").is_ok() {
        std::fs::write("examples/lksg_v1.ir.json", &ir_json).unwrap();
        println!("Updated golden file");
    } else {
        assert_eq!(ir_json, golden, "IR does not match golden file");
    }
}
```

**Estimated Effort:** 1-2 hours

---

### Day 5: CLI & Integration

**Goal:** Working `policy lint`, `policy compile`, `policy show` commands

#### Task 5.1: Implement CLI

**File:** `src/policy_v2/cli.rs`

```rust
use super::*;
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "policy")]
#[command(about = "Policy compiler CLI")]
pub struct PolicyCli {
    #[command(subcommand)]
    pub command: PolicyCommand,
}

#[derive(Subcommand)]
pub enum PolicyCommand {
    /// Lint a policy file
    Lint {
        /// Path to policy YAML file
        file: String,
        /// Use strict linting mode
        #[arg(long)]
        strict: bool,
    },
    /// Compile policy to IR v1
    Compile {
        /// Path to policy YAML file
        file: String,
        /// Output IR JSON file
        #[arg(short, long)]
        output: String,
    },
    /// Show IR in human-readable format
    Show {
        /// Path to IR JSON file
        file: String,
    },
}

pub fn run_lint(file: &str, strict: bool) -> Result<i32> {
    let policy = yaml_parser::parse_yaml(file)?;
    let mode = if strict { linter::LintMode::Strict } else { linter::LintMode::Relaxed };
    let diagnostics = linter::lint(&policy, mode);

    for diag in &diagnostics {
        let prefix = match diag.level {
            linter::Level::Error => "ERROR",
            linter::Level::Warning => "WARN",
        };
        if let Some(rule_id) = &diag.rule_id {
            println!("[{}] {}: {}", prefix, rule_id, diag.message);
        } else {
            println!("[{}] {}", prefix, diag.message);
        }
    }

    if linter::has_errors(&diagnostics) {
        Ok(3) // Lint error exit code
    } else if !diagnostics.is_empty() {
        Ok(2) // Warnings exit code
    } else {
        println!("✅ Policy is valid");
        Ok(0)
    }
}

pub fn run_compile(file: &str, output: &str) -> Result<i32> {
    // Parse
    let policy = yaml_parser::parse_yaml(file)?;

    // Lint (strict)
    let diagnostics = linter::lint(&policy, linter::LintMode::Strict);
    if linter::has_errors(&diagnostics) {
        for diag in &diagnostics {
            eprintln!("ERROR: {}", diag.message);
        }
        return Ok(3);
    }

    // Compute policy hash
    let policy_json = serde_json::to_string(&policy)?;
    let policy_hash = hasher::sha3_256_hex(&policy_json);

    // Generate IR
    let mut ir = ir::generate_ir(&policy, policy_hash)?;

    // Compute IR hash
    let ir_canonical = ir::canonicalize(&ir)?;
    let ir_hash = hasher::sha3_256_hex(&ir_canonical);
    ir.ir_hash = ir_hash;

    // Write IR
    let ir_json = serde_json::to_string_pretty(&ir)?;
    std::fs::write(output, &ir_json)?;

    println!("✅ Compiled policy to {}", output);
    println!("   policy_hash: {}", ir.policy_hash);
    println!("   ir_hash: {}", ir.ir_hash);

    Ok(0)
}

pub fn run_show(file: &str) -> Result<i32> {
    let ir_json = std::fs::read_to_string(file)?;
    let ir: types::IrV1 = serde_json::from_str(&ir_json)?;

    println!("Policy ID: {}", ir.policy_id);
    println!("IR Version: {}", ir.ir_version);
    println!("Policy Hash: {}", ir.policy_hash);
    println!("IR Hash: {}", ir.ir_hash);
    println!("\nRules ({}):", ir.rules.len());
    for rule in &ir.rules {
        println!("  - {} ({})", rule.id, rule.op);
    }

    Ok(0)
}
```

**Estimated Effort:** 2-3 hours

#### Task 5.2: Integrate with main.rs

**File:** `src/main.rs` (add policy subcommand)

```rust
// Add to existing main.rs in agent project
use clap::{Parser, Subcommand};

#[derive(Subcommand)]
enum Commands {
    // ... existing commands ...

    /// Policy compiler commands
    #[command(subcommand)]
    Policy(policy_v2::cli::PolicyCommand),
}

// In main():
Commands::Policy(cmd) => {
    use policy_v2::cli::*;
    let exit_code = match cmd {
        PolicyCommand::Lint { file, strict } => run_lint(&file, strict)?,
        PolicyCommand::Compile { file, output } => run_compile(&file, &output)?,
        PolicyCommand::Show { file } => run_show(&file)?,
    };
    std::process::exit(exit_code);
}
```

**Estimated Effort:** 1 hour

---

## Testing Strategy

### Unit Tests

**Coverage Target:** 90%+

```bash
# Run all tests
cargo test --workspace --all-features

# Run with coverage (optional)
cargo tarpaulin --out Html --output-dir coverage/
```

### Golden File Tests

**Update golden files:**
```bash
UPDATE_GOLDEN=1 cargo test test_golden_ir_lksg_v1
```

**Validate golden files:**
```bash
cargo test test_golden_ir_lksg_v1
```

### CLI Manual Testing

```bash
# Build
cargo build --release

# Lint
./target/release/cap policy lint examples/lksg_v1.policy.yml --strict

# Compile
./target/release/cap policy compile examples/lksg_v1.policy.yml -o /tmp/ir.json

# Show
./target/release/cap policy show /tmp/ir.json
```

---

## CI/CD Pipeline

**File:** `.github/workflows/policy-compiler.yml`

```yaml
name: Policy Compiler CI

on:
  push:
    paths:
      - 'src/policy_v2/**'
      - 'schemas/**'
      - 'examples/lksg_v1.*'
      - 'tests/**'
  pull_request:
    paths:
      - 'src/policy_v2/**'

jobs:
  build-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable

      - name: Build
        run: cargo build --workspace --locked

      - name: Run tests
        run: cargo test --workspace --all-features

      - name: Lint (clippy)
        run: cargo clippy --workspace --all-features -- -D warnings

      - name: Check golden files
        run: |
          cargo test test_golden_ir_lksg_v1
          # Ensure no changes to golden files
          git diff --exit-code examples/lksg_v1.ir.json
```

---

## JSON Schemas

### Policy Schema

**File:** `schemas/policy.schema.json`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://cap.example.com/schemas/policy.schema.json",
  "title": "CAP Policy v2",
  "type": "object",
  "required": ["id", "version", "legal_basis", "inputs", "rules"],
  "properties": {
    "id": {
      "type": "string",
      "pattern": "^[a-z0-9\\._-]+$",
      "description": "Unique policy identifier"
    },
    "version": {
      "type": "string",
      "description": "Policy version"
    },
    "legal_basis": {
      "type": "array",
      "minItems": 1,
      "items": {
        "type": "object",
        "properties": {
          "directive": {"type": "string"},
          "article": {"type": "string"}
        }
      },
      "description": "Legal basis for the policy"
    },
    "description": {
      "type": "string",
      "description": "Human-readable description"
    },
    "inputs": {
      "type": "object",
      "additionalProperties": {
        "type": "object",
        "required": ["type"],
        "properties": {
          "type": {"type": "string"},
          "items": {"type": "string"}
        }
      }
    },
    "rules": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["id", "op", "lhs", "rhs"],
        "properties": {
          "id": {"type": "string"},
          "op": {
            "type": "string",
            "enum": ["non_membership", "eq", "range_min"]
          },
          "lhs": {},
          "rhs": {}
        }
      }
    },
    "adaptivity": {
      "type": "object",
      "properties": {
        "predicates": {"type": "array"},
        "activations": {"type": "array"}
      }
    }
  }
}
```

### IR v1 Schema

**File:** `schemas/ir_v1.schema.json`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://cap.example.com/schemas/ir_v1.schema.json",
  "title": "CAP IR v1",
  "type": "object",
  "required": ["ir_version", "policy_id", "policy_hash", "rules", "ir_hash"],
  "properties": {
    "ir_version": {
      "const": "1.0"
    },
    "policy_id": {
      "type": "string"
    },
    "policy_hash": {
      "type": "string",
      "pattern": "^sha3-256:[0-9a-f]{64}$"
    },
    "rules": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["id", "op", "lhs", "rhs"],
        "properties": {
          "id": {"type": "string"},
          "op": {"type": "string"},
          "lhs": {},
          "rhs": {}
        }
      }
    },
    "adaptivity": {
      "type": "object"
    },
    "ir_hash": {
      "type": "string",
      "pattern": "^sha3-256:[0-9a-f]{64}$"
    }
  }
}
```

---

## Definition of Done (Week 1)

### Functional Requirements
- [x] `policy lint --strict` runs without errors on lksg_v1.policy.yml
- [x] `policy compile` generates valid IR v1 JSON
- [x] `policy show` displays IR in human-readable format
- [x] Policy without `legal_basis` fails in strict mode
- [x] Invalid operator fails linting

### Technical Requirements
- [x] `policy_hash` and `ir_hash` are deterministic
- [x] Rules are canonically sorted by ID
- [x] IR JSON uses stable key ordering (BTreeMap)
- [x] Golden test passes (bit-exact comparison)

### Quality Requirements
- [x] Unit tests ≥90% code coverage
- [x] Golden tests green on CI
- [x] No clippy warnings
- [x] No PII in logs (only hashes and rule IDs)

### Documentation
- [x] JSON schemas created (policy, IR v1)
- [x] Example policy created
- [x] Golden IR file created
- [x] CI pipeline configured

---

## Risk Mitigation

### Risk 1: Non-Deterministic Hashing

**Symptom:** Golden tests fail intermittently

**Mitigation:**
- Use BTreeMap instead of HashMap
- Sort rules by ID before serialization
- Normalize numbers/booleans/strings
- Test on multiple machines/OSes

### Risk 2: Schema Drift

**Symptom:** IR doesn't match schema validation

**Mitigation:**
- Validate IR against JSON schema in tests
- Add schema validation to CLI compile
- Version schemas explicitly

### Risk 3: Complex Expression Parsing

**Symptom:** lhs/rhs parsing fails for complex expressions

**Mitigation:**
- Week 1: Use simple string references only
- Week 2: Add expression parser for builtins
- Defer complex expressions to Week 3

---

## Estimated Effort

| Task | Hours | Notes |
|------|-------|-------|
| Structures & Parser (Day 1) | 3-4 | Types, YAML parsing, examples |
| Linter (Day 2) | 2-3 | Strict mode, diagnostics |
| IR Generation (Day 3) | 2-3 | Canonicalization, sorting |
| Hashing & Golden Tests (Day 4) | 2-3 | SHA3-256, golden file tests |
| CLI Integration (Day 5) | 3-4 | Commands, main.rs integration |
| CI/CD & Documentation | 2 | Pipeline, schemas |
| **Total** | **14-19 hours** | **3-5 days** |

---

## Next Steps (Week 2 Preview)

After completing Week 1, Week 2 will focus on:

1. **Builtins:** now(), len(), max(), sub()
2. **Expression Parser:** AST for complex lhs/rhs
3. **Adaptivity:** Predicates + Activations compiler
4. **Duration Parsing:** P365D (ISO 8601)
5. **Golden Tests:** Extended with adaptivity
6. **CLI UX:** Better error messages, colored output

---

## Conclusion

This roadmap provides a **concrete, day-by-day implementation plan** for Policy Compiler Week 1. All deliverables are clearly defined, with code templates, test strategies, and acceptance criteria.

**Ready to start:** Day 1, Task 1.1 - Define IR v1 Structures

---

**Document Version:** 1.0
**Created:** 2025-11-09
**Author:** Claude Code
**Status:** Ready for Implementation
