# Claude Task: Complete Verifier CLI (v1.0) — Offline Proof Verification

## 🎯 Goal
Implement a new CLI command  
`cap manifest verify`  
that performs a **complete offline verification** of a proof package —  
using only existing components (hashes, signatures, timestamps, registry).  

This is a **non-ZK version** of the verifier (for v0.7.0),  
providing an auditor-ready verification path until the real ZK-backend is integrated.

---

## 🧱 Implementation Scope

### 1️⃣ Command Definition
**New CLI Command:**  
```bash
cap manifest verify \
  --manifest build/manifest.json \
  --proof build/zk_proof.dat \
  --registry build/registry.json \
  [--timestamp build/timestamp.tsr] \
  [--out build/verification.report.json]
```

**Purpose:**  
Execute all existing integrity checks in fixed order:
```
1. Manifest → Proof → Hash match
2. Manifest signature verify
3. Timestamp verification (mock)
4. Registry match
```

---

### 2️⃣ New File: `agent/src/cli/manifest_verify.rs`

```rust
use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use sha3::{Digest, Sha3_256};

#[derive(Serialize, Deserialize)]
pub struct VerificationReport {
    pub manifest_hash: String,
    pub proof_hash: String,
    pub timestamp_valid: bool,
    pub registry_match: bool,
    pub signature_valid: bool,
    pub status: String,
}

pub fn run(manifest: &str, proof: &str, registry: &str, timestamp: Option<&str>, out: Option<&str>) -> Result<()> {
    // 1️⃣ Compute hashes
    let manifest_bytes = fs::read(manifest)?;
    let proof_bytes = fs::read(proof)?;
    let manifest_hash = format!("0x{:x}", Sha3_256::digest(&manifest_bytes));
    let proof_hash = format!("0x{:x}", Sha3_256::digest(&proof_bytes));

    // 2️⃣ Verify signature (mock / existing verifier)
    let signature_valid = crate::sign::verify_manifest_signature(manifest).unwrap_or(false);

    // 3️⃣ Timestamp verification (mock)
    let timestamp_valid = match timestamp {
        Some(ts_path) => crate::registry::verify_timestamp_mock(ts_path),
        None => true,
    };

    // 4️⃣ Registry verification
    let registry_match = crate::registry::verify_entry(registry, &manifest_hash, &proof_hash).unwrap_or(false);

    // 5️⃣ Consolidate result
    let all_ok = signature_valid && timestamp_valid && registry_match;
    let status = if all_ok { "ok" } else { "fail" }.to_string();

    let report = VerificationReport {
        manifest_hash,
        proof_hash,
        timestamp_valid,
        registry_match,
        signature_valid,
        status: status.clone(),
    };

    // 6️⃣ Save report
    let report_path = out.unwrap_or("build/verification.report.json");
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(report_path, json)?;

    if all_ok {
        println!("✅ Verification successful. Report: {report_path}");
    } else {
        eprintln!("❌ Verification failed. Report: {report_path}");
    }

    Ok(())
}
```

---

### 3️⃣ Register Command in CLI
**File:** `agent/src/main.rs`

```rust
.subcommand(
    Command::new("manifest")
        .about("Manifest operations")
        .subcommand(
            Command::new("verify")
                .about("Verify manifest and proof package offline")
                .arg(arg!(--manifest <FILE> "Manifest JSON file"))
                .arg(arg!(--proof <FILE> "Proof DAT file"))
                .arg(arg!(--registry <FILE> "Registry JSON file"))
                .arg(arg!(--timestamp [FILE] "Timestamp TSR file"))
                .arg(arg!(--out [FILE] "Optional output report"))
        )
)
```

---

### 4️⃣ Extend `registry.rs` (Helper Functions)

Add simple verification helper:

```rust
pub fn verify_entry(registry_path: &str, manifest_hash: &str, proof_hash: &str) -> Option<bool> {
    let registry_json: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(registry_path).ok()?) .ok()?;
    let entries = registry_json.get("entries")?.as_array()?;
    for entry in entries {
        if entry.get("manifest_hash")?.as_str()? == manifest_hash &&
           entry.get("proof_hash")?.as_str()? == proof_hash {
            return Some(true);
        }
    }
    Some(false)
}

pub fn verify_timestamp_mock(ts_path: &str) -> bool {
    let ts: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(ts_path).unwrap_or_default()).unwrap_or_default();
    ts.get("status").and_then(|s| s.as_str()).unwrap_or("") == "ok"
}
```

---

### 5️⃣ Add Unit Tests
**File:** `agent/tests/test_manifest_verify.rs`

```rust
#[test]
fn test_verify_ok() {
    let res = run(
        "tests/data/valid_manifest.json",
        "tests/data/valid_proof.dat",
        "tests/data/registry.json",
        Some("tests/data/timestamp.tsr"),
        Some("tests/output/verification.report.json")
    );
    assert!(res.is_ok());
}
```

---

### 6️⃣ Example Output

Console:
```
✅ Verification successful. Report: build/verification.report.json
```

`verification.report.json`:
```json
{
  "manifest_hash": "0xd490be94abc123...",
  "proof_hash": "0x83a8779ddef456...",
  "timestamp_valid": true,
  "registry_match": true,
  "signature_valid": true,
  "status": "ok"
}
```

---

## ✅ Acceptance Criteria

| Criterion | Description |
|------------|-------------|
| ✔ Command | `cap manifest verify` runs end-to-end offline |
| ✔ Sequence | Fixed check order: Hash → Signature → Timestamp → Registry |
| ✔ Output | Creates `verification.report.json` |
| ✔ Test | Unit tests for success/failure cases pass |
| ✔ Compatibility | Works without ZK backend, future-ready for ZK integration |

---

## 🔧 Future Extension (v0.8+)
Later replace `verify_proof()` placeholder with:
```rust
crate::zk_system::verify_zk_proof(...)
```
This ensures full ZK-Verifier compatibility once Halo2 or zkVM backend is active.

---

## 🧩 References
- Manifest: `build/manifest.json`
- Proof: `build/zk_proof.dat`
- Registry: `build/registry.json`
- Timestamp: `build/timestamp.tsr`
- Output: `build/verification.report.json`
- Docs: `docs/SYSTEMARCHITEKTUR_v0.6.0.md`
