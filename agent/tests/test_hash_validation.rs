/// Integration test for bundle hash validation
///
/// Tests that verify-bundle detects tampering via _meta.json hash mismatches.

use std::fs;

/// Helper: Create minimal test manifest
fn create_test_manifest(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let manifest = serde_json::json!({
        "version": "manifest.v1.0",
        "created_at": "2025-10-30T10:00:00Z",
        "supplier_root": "0xabc1234567890123456789012345678901234567890123456789012345678901",
        "ubo_root": "0xdef1234567890123456789012345678901234567890123456789012345678901",
        "company_commitment_root": "0x1231234567890123456789012345678901234567890123456789012345678901",
        "policy": {
            "name": "Test Policy",
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
        "signatures": [{
            "signer_pubkey": "0xabc1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567",
            "signature": "0xdef1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789",
            "signed_at": "2025-10-30T10:00:00Z"
        }]
    });

    fs::write(path, serde_json::to_string_pretty(&manifest)?)?;
    Ok(())
}

/// Helper: Create minimal CAPZ proof
fn create_test_capz(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    use cap_agent::proof::{CapzContainer, ProofBackend};

    let proof_payload = serde_json::json!({
        "version": "proof.v0",
        "type": "mock",
        "statement": "policy:lksg.v1",
        "status": "ok"
    });

    let payload = serde_json::to_vec(&proof_payload)?;
    let container = CapzContainer::new(ProofBackend::Mock, payload);
    container.write_to_file(path)?;

    Ok(())
}

/// Test: Hash validation detects manifest tampering
#[test]
fn test_hash_validation_detects_manifest_tampering() {
    let test_dir = "tests/out/hash_validation";
    let manifest_path = format!("{}/test_manifest.json", test_dir);
    let proof_path = format!("{}/test_proof.capz", test_dir);
    let bundle_path = format!("{}/test_bundle", test_dir);

    // Setup
    fs::create_dir_all(test_dir).ok();
    create_test_manifest(&manifest_path).expect("Failed to create manifest");
    create_test_capz(&proof_path).expect("Failed to create proof");

    // Create bundle
    fs::remove_dir_all(&bundle_path).ok();
    let output = std::process::Command::new("cargo")
        .args(&[
            "run",
            "--",
            "bundle-v2",
            "--manifest",
            &manifest_path,
            "--proof",
            &proof_path,
            "--out",
            &bundle_path,
            "--force",
        ])
        .output()
        .expect("Failed to execute bundle-v2");

    assert!(output.status.success());

    // Tamper with manifest
    let manifest_bundle_path = format!("{}/manifest.json", bundle_path);
    let mut manifest: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&manifest_bundle_path).unwrap()
    ).unwrap();
    manifest["audit"]["events_count"] = serde_json::json!(999);
    fs::write(&manifest_bundle_path, serde_json::to_string_pretty(&manifest).unwrap())
        .expect("Failed to write tampered manifest");

    // Try to verify (should fail due to hash mismatch)
    let output = std::process::Command::new("cargo")
        .args(&[
            "run",
            "--",
            "verify-bundle",
            "--bundle",
            &bundle_path,
        ])
        .output()
        .expect("Failed to execute verify-bundle");

    // Should fail
    assert!(
        !output.status.success(),
        "verify-bundle should detect tampered manifest"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Manifest hash mismatch") || stderr.contains("integrity check failed"),
        "Error should mention hash mismatch, got: {}",
        stderr
    );

    // Cleanup
    fs::remove_dir_all(&bundle_path).ok();
}

/// Test: Hash validation detects proof tampering
#[test]
fn test_hash_validation_detects_proof_tampering() {
    let test_dir = "tests/out/hash_validation";
    let manifest_path = format!("{}/test_manifest.json", test_dir);
    let proof_path = format!("{}/test_proof.capz", test_dir);
    let bundle_path = format!("{}/test_bundle_proof_tamper", test_dir);

    // Setup
    fs::create_dir_all(test_dir).ok();
    create_test_manifest(&manifest_path).expect("Failed to create manifest");
    create_test_capz(&proof_path).expect("Failed to create proof");

    // Create bundle
    fs::remove_dir_all(&bundle_path).ok();
    let output = std::process::Command::new("cargo")
        .args(&[
            "run",
            "--",
            "bundle-v2",
            "--manifest",
            &manifest_path,
            "--proof",
            &proof_path,
            "--out",
            &bundle_path,
            "--force",
        ])
        .output()
        .expect("Failed to execute bundle-v2");

    assert!(output.status.success());

    // Tamper with proof
    let proof_bundle_path = format!("{}/proof.capz", bundle_path);
    let mut proof_bytes = fs::read(&proof_bundle_path).unwrap();
    // Change a byte in the payload (after the 78-byte header)
    if proof_bytes.len() > 80 {
        proof_bytes[80] = proof_bytes[80].wrapping_add(1);
        fs::write(&proof_bundle_path, proof_bytes).expect("Failed to write tampered proof");
    }

    // Try to verify (should fail due to hash mismatch)
    let output = std::process::Command::new("cargo")
        .args(&[
            "run",
            "--",
            "verify-bundle",
            "--bundle",
            &bundle_path,
        ])
        .output()
        .expect("Failed to execute verify-bundle");

    // Should fail
    assert!(
        !output.status.success(),
        "verify-bundle should detect tampered proof"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Proof hash mismatch") || stderr.contains("integrity check failed"),
        "Error should mention hash mismatch, got: {}",
        stderr
    );

    // Cleanup
    fs::remove_dir_all(&bundle_path).ok();
}

/// Test: Untampered bundle passes hash validation
#[test]
fn test_hash_validation_passes_for_valid_bundle() {
    let test_dir = "tests/out/hash_validation";
    let manifest_path = format!("{}/test_manifest.json", test_dir);
    let proof_path = format!("{}/test_proof.capz", test_dir);
    let bundle_path = format!("{}/test_bundle_valid", test_dir);

    // Setup
    fs::create_dir_all(test_dir).ok();
    create_test_manifest(&manifest_path).expect("Failed to create manifest");
    create_test_capz(&proof_path).expect("Failed to create proof");

    // Create bundle
    fs::remove_dir_all(&bundle_path).ok();
    let output = std::process::Command::new("cargo")
        .args(&[
            "run",
            "--",
            "bundle-v2",
            "--manifest",
            &manifest_path,
            "--proof",
            &proof_path,
            "--out",
            &bundle_path,
            "--force",
        ])
        .output()
        .expect("Failed to execute bundle-v2");

    assert!(output.status.success());

    // Verify without tampering (should succeed)
    let output = std::process::Command::new("cargo")
        .args(&[
            "run",
            "--",
            "verify-bundle",
            "--bundle",
            &bundle_path,
        ])
        .output()
        .expect("Failed to execute verify-bundle");

    // Should succeed
    assert!(
        output.status.success(),
        "verify-bundle should pass for valid bundle: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Manifest hash valid") && stdout.contains("Proof hash valid"),
        "Output should confirm hash validation passed"
    );

    // Cleanup
    fs::remove_dir_all(&bundle_path).ok();
}
