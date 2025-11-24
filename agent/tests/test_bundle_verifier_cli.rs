/// Integration tests for BundleVerifier CLI (cap-bundle.v1 Format)
///
/// Tests the `verifier run` command with cap-bundle.v1 packages,
/// including multi-proof-unit support, hash validation, and cycle detection.

use serde_json::{json, Value};
use std::fs;

/// Helper: Create test bundle directory with _meta.json
fn create_test_bundle(
    bundle_dir: &str,
    proof_units: Vec<TestProofUnit>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create bundle directory
    fs::create_dir_all(bundle_dir)?;

    // Create _meta.json
    let mut files = std::collections::HashMap::new();
    let mut proof_units_meta = Vec::new();

    for (i, unit) in proof_units.iter().enumerate() {
        let manifest_file = format!("manifest_{}.json", i);
        let proof_file = format!("proof_{}.capz", i);

        // Create manifest
        let manifest_path = format!("{}/{}", bundle_dir, manifest_file);
        create_test_manifest(&manifest_path, &unit.policy_id)?;

        // Compute TWO hashes:
        // 1. Hash of FILE bytes (for _meta.json, what BundleVerifier checks)
        let manifest_bytes = fs::read(&manifest_path)?;
        let manifest_file_hash = cap_agent::crypto::hex_lower_prefixed32(
            cap_agent::crypto::sha3_256(&manifest_bytes)
        );

        // 2. Hash of RE-SERIALIZED struct (for Proof, what proof_engine checks)
        let manifest = cap_agent::manifest::Manifest::load(&manifest_path)?;
        let manifest_json = serde_json::to_string(&manifest)?;
        let manifest_proof_hash = cap_agent::crypto::hex_lower_prefixed32(
            cap_agent::crypto::sha3_256(manifest_json.as_bytes())
        );

        // Create proof with the STRUCT hash (what proof_engine expects)
        let proof_path = format!("{}/{}", bundle_dir, proof_file);
        create_test_proof(&proof_path, &manifest_proof_hash)?;

        // Compute proof hash
        let proof_bytes = fs::read(&proof_path)?;
        let proof_hash = cap_agent::crypto::hex_lower_prefixed32(
            cap_agent::crypto::sha3_256(&proof_bytes)
        );

        // Add to files map with FILE hash (what BundleVerifier checks)
        files.insert(
            manifest_file.clone(),
            json!({
                "role": "manifest",
                "hash": manifest_file_hash,
                "size": manifest_bytes.len(),
                "content_type": "application/json",
                "optional": false
            }),
        );
        files.insert(
            proof_file.clone(),
            json!({
                "role": "proof",
                "hash": proof_hash,
                "size": proof_bytes.len(),
                "optional": false
            }),
        );

        // Add to proof_units
        proof_units_meta.push(json!({
            "id": unit.id,
            "manifest_file": manifest_file,
            "proof_file": proof_file,
            "policy_id": unit.policy_id,
            "policy_hash": unit.policy_hash,
            "backend": "mock",
            "depends_on": unit.depends_on
        }));
    }

    // Create _meta.json
    let meta = json!({
        "schema": "cap-bundle.v1",
        "bundle_id": "test-bundle-123",
        "created_at": "2025-11-24T12:00:00Z",
        "files": files,
        "proof_units": proof_units_meta
    });

    fs::write(
        format!("{}/_meta.json", bundle_dir),
        serde_json::to_string_pretty(&meta)?,
    )?;

    Ok(())
}

/// Helper: Create test manifest
fn create_test_manifest(
    path: &str,
    policy_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest = json!({
        "version": "manifest.v1.0",
        "created_at": "2025-11-24T12:00:00Z",
        "supplier_root": "0xabc1234567890123456789012345678901234567890123456789012345678901",
        "ubo_root": "0xdef1234567890123456789012345678901234567890123456789012345678901",
        "company_commitment_root": "0x1231234567890123456789012345678901234567890123456789012345678901",
        "policy": {
            "name": policy_id,
            "version": "lksg.v1",
            "hash": "0xabc1234567890123456789012345678901234567890123456789012345678901"
        },
        "audit": {
            "tail_digest": "0xdef1234567890123456789012345678901234567890123456789012345678901",
            "events_count": 10
        },
        "proof": {
            "type": "mock",
            "status": "ok"
        },
        "signatures": []
    });

    // Write with NON-PRETTY JSON to match what the verifier expects
    fs::write(path, serde_json::to_string(&manifest)?)?;
    Ok(())
}

/// Helper: Create test proof (JSON bytes for bundle.v1, Base64 for legacy)
///
/// Note: cap-bundle.v1 expects raw JSON bytes in .capz files,
/// but legacy .dat format expects Base64-encoded JSON.
fn create_test_proof(path: &str, manifest_hash: &str) -> Result<(), Box<dyn std::error::Error>> {
    let proof = json!({
        "version": "proof.v0",
        "type": "mock",
        "statement": "policy:lksg.v1",
        "manifest_hash": manifest_hash,
        "policy_hash": "0xabc1234567890123456789012345678901234567890123456789012345678901",
        "proof_data": {
            "checked_constraints": []
        },
        "status": "ok"
    });

    // Determine format based on file extension
    if path.ends_with(".capz") {
        // cap-bundle.v1: Write raw JSON bytes
        fs::write(path, serde_json::to_vec(&proof)?)?;
    } else if path.ends_with(".dat") {
        // Legacy: Write Base64-encoded JSON
        use base64::{engine::general_purpose, Engine as _};
        let json_bytes = serde_json::to_vec(&proof)?;
        let encoded = general_purpose::STANDARD.encode(&json_bytes);
        fs::write(path, encoded)?;
    } else {
        return Err("Unknown proof file extension (expected .capz or .dat)".into());
    }

    Ok(())
}

#[derive(Clone)]
struct TestProofUnit {
    id: String,
    policy_id: String,
    policy_hash: String,
    depends_on: Vec<String>,
}

/// Test 1: Verify valid cap-bundle.v1 with single proof unit
#[test]
fn test_verify_bundle_v1_single_unit_ok() {
    let bundle_dir = "tests/out/bundle_verifier_cli/test1_single";

    // Setup
    fs::remove_dir_all(bundle_dir).ok();
    fs::create_dir_all(bundle_dir).ok();

    let units = vec![TestProofUnit {
        id: "main".to_string(),
        policy_id: "lksg.demo.v1".to_string(),
        policy_hash: "0xabc1234567890123456789012345678901234567890123456789012345678901".to_string(),
        depends_on: vec![],
    }];

    create_test_bundle(bundle_dir, units).expect("Failed to create test bundle");

    // Run verifier
    let output = std::process::Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "cap-agent",
            "--",
            "verifier",
            "run",
            "--package",
            bundle_dir,
        ])
        .output()
        .expect("Failed to execute verifier run");

    // Verify success
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("STDOUT:\n{}", stdout);
    println!("STDERR:\n{}", stderr);

    assert!(
        output.status.success(),
        "verifier run should succeed for valid bundle"
    );
    assert!(stdout.contains("cap-bundle.v1"), "Should detect bundle.v1 format");
    assert!(stdout.contains("Bundle-Verifikation abgeschlossen"), "Should complete verification");

    // Cleanup
    fs::remove_dir_all(bundle_dir).ok();
}

/// Test 2: Fallback to legacy verifier when _meta.json missing
#[test]
fn test_verify_bundle_legacy_fallback() {
    let bundle_dir = "tests/out/bundle_verifier_cli/test2_legacy";

    // Setup: Create legacy package WITHOUT _meta.json
    fs::remove_dir_all(bundle_dir).ok();
    fs::create_dir_all(bundle_dir).ok();

    let manifest_path = format!("{}/manifest.json", bundle_dir);
    create_test_manifest(&manifest_path, "legacy.policy")
        .expect("Failed to create manifest");

    // Compute manifest hash THE SAME WAY as verifier: load as Manifest struct, re-serialize
    let manifest = cap_agent::manifest::Manifest::load(std::path::Path::new(&manifest_path))
        .expect("Failed to load manifest");
    let manifest_json = serde_json::to_string(&manifest)
        .expect("Failed to serialize manifest");
    let manifest_hash = cap_agent::crypto::hex_lower_prefixed32(
        cap_agent::crypto::sha3_256(manifest_json.as_bytes())
    );

    create_test_proof(&format!("{}/proof.dat", bundle_dir), &manifest_hash)
        .expect("Failed to create proof");

    // Run verifier
    let output = std::process::Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "cap-agent",
            "--",
            "verifier",
            "run",
            "--package",
            bundle_dir,
        ])
        .output()
        .expect("Failed to execute verifier run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("STDOUT:\n{}", stdout);
    println!("STDERR:\n{}", stderr);

    assert!(
        output.status.success(),
        "verifier run should fallback to legacy verifier"
    );
    assert!(stdout.contains("Legacy") || stdout.contains("pre-bundle.v1"),
            "Should detect legacy format");

    // Cleanup
    fs::remove_dir_all(bundle_dir).ok();
}

/// Test 3: Verify bundle with multiple proof units
#[test]
fn test_verify_bundle_multiple_units() {
    let bundle_dir = "tests/out/bundle_verifier_cli/test3_multi";

    // Setup
    fs::remove_dir_all(bundle_dir).ok();
    fs::create_dir_all(bundle_dir).ok();

    let units = vec![
        TestProofUnit {
            id: "unit_a".to_string(),
            policy_id: "policy.a.v1".to_string(),
            policy_hash: "0xabc1234567890123456789012345678901234567890123456789012345678901".to_string(),
            depends_on: vec![],
        },
        TestProofUnit {
            id: "unit_b".to_string(),
            policy_id: "policy.b.v1".to_string(),
            policy_hash: "0xdef1234567890123456789012345678901234567890123456789012345678901".to_string(),
            depends_on: vec![],
        },
    ];

    create_test_bundle(bundle_dir, units).expect("Failed to create test bundle");

    // Run verifier
    let output = std::process::Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "cap-agent",
            "--",
            "verifier",
            "run",
            "--package",
            bundle_dir,
        ])
        .output()
        .expect("Failed to execute verifier run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("STDOUT:\n{}", stdout);

    assert!(output.status.success(), "Should verify bundle with multiple units");
    assert!(stdout.contains("Proof Units: 2"), "Should report 2 proof units");

    // Cleanup
    fs::remove_dir_all(bundle_dir).ok();
}

/// Test 4: Detect hash mismatch in bundle
#[test]
fn test_verify_bundle_hash_mismatch() {
    let bundle_dir = "tests/out/bundle_verifier_cli/test4_hash_mismatch";

    // Setup
    fs::remove_dir_all(bundle_dir).ok();
    fs::create_dir_all(bundle_dir).ok();

    let units = vec![TestProofUnit {
        id: "main".to_string(),
        policy_id: "lksg.demo.v1".to_string(),
        policy_hash: "0xabc1234567890123456789012345678901234567890123456789012345678901".to_string(),
        depends_on: vec![],
    }];

    create_test_bundle(bundle_dir, units).expect("Failed to create test bundle");

    // Tamper with manifest
    let manifest_path = format!("{}/manifest_0.json", bundle_dir);
    let mut manifest: Value = serde_json::from_str(
        &fs::read_to_string(&manifest_path).unwrap()
    ).unwrap();
    manifest["audit"]["events_count"] = json!(999); // Tamper
    fs::write(&manifest_path, serde_json::to_string_pretty(&manifest).unwrap())
        .expect("Failed to write tampered manifest");

    // Run verifier
    let output = std::process::Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "cap-agent",
            "--",
            "verifier",
            "run",
            "--package",
            bundle_dir,
        ])
        .output()
        .expect("Failed to execute verifier run");

    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("STDERR:\n{}", stderr);

    // Should fail due to hash mismatch
    assert!(
        !output.status.success(),
        "Should fail when manifest hash doesn't match"
    );
    assert!(
        stderr.contains("hash") || stderr.contains("mismatch"),
        "Error should mention hash mismatch"
    );

    // Cleanup
    fs::remove_dir_all(bundle_dir).ok();
}

/// Test 5: Detect circular dependencies in proof units
#[test]
fn test_verify_bundle_circular_dependency() {
    let bundle_dir = "tests/out/bundle_verifier_cli/test5_circular";

    // Setup
    fs::remove_dir_all(bundle_dir).ok();
    fs::create_dir_all(bundle_dir).ok();

    // Create circular dependency: A → B → C → A
    let units = vec![
        TestProofUnit {
            id: "unit_a".to_string(),
            policy_id: "policy.a.v1".to_string(),
            policy_hash: "0xabc1234567890123456789012345678901234567890123456789012345678901".to_string(),
            depends_on: vec!["unit_b".to_string()],
        },
        TestProofUnit {
            id: "unit_b".to_string(),
            policy_id: "policy.b.v1".to_string(),
            policy_hash: "0xdef1234567890123456789012345678901234567890123456789012345678901".to_string(),
            depends_on: vec!["unit_c".to_string()],
        },
        TestProofUnit {
            id: "unit_c".to_string(),
            policy_id: "policy.c.v1".to_string(),
            policy_hash: "0x1231234567890123456789012345678901234567890123456789012345678901".to_string(),
            depends_on: vec!["unit_a".to_string()], // Cycle!
        },
    ];

    create_test_bundle(bundle_dir, units).expect("Failed to create test bundle");

    // Run verifier
    let output = std::process::Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "cap-agent",
            "--",
            "verifier",
            "run",
            "--package",
            bundle_dir,
        ])
        .output()
        .expect("Failed to execute verifier run");

    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("STDERR:\n{}", stderr);

    // Should fail due to circular dependency
    assert!(
        !output.status.success(),
        "Should fail when circular dependency detected"
    );
    assert!(
        stderr.contains("Circular") || stderr.contains("cycle") || stderr.contains("dependency"),
        "Error should mention circular dependency"
    );

    // Cleanup
    fs::remove_dir_all(bundle_dir).ok();
}

/// Test 6: Fail when proof file missing in bundle
#[test]
fn test_verify_bundle_missing_proof_file() {
    let bundle_dir = "tests/out/bundle_verifier_cli/test6_missing_proof";

    // Setup
    fs::remove_dir_all(bundle_dir).ok();
    fs::create_dir_all(bundle_dir).ok();

    let units = vec![TestProofUnit {
        id: "main".to_string(),
        policy_id: "lksg.demo.v1".to_string(),
        policy_hash: "0xabc1234567890123456789012345678901234567890123456789012345678901".to_string(),
        depends_on: vec![],
    }];

    create_test_bundle(bundle_dir, units).expect("Failed to create test bundle");

    // Delete proof file
    fs::remove_file(format!("{}/proof_0.capz", bundle_dir))
        .expect("Failed to remove proof file");

    // Run verifier
    let output = std::process::Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "cap-agent",
            "--",
            "verifier",
            "run",
            "--package",
            bundle_dir,
        ])
        .output()
        .expect("Failed to execute verifier run");

    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("STDERR:\n{}", stderr);

    // Should fail due to missing proof file
    assert!(
        !output.status.success(),
        "Should fail when proof file missing"
    );
    assert!(
        stderr.contains("not found") || stderr.contains("missing") || stderr.contains("No such file"),
        "Error should mention missing file"
    );

    // Cleanup
    fs::remove_dir_all(bundle_dir).ok();
}
