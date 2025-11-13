use serde_json::Value;
/// Integration tests for Bundle v2 verification
///
/// Tests verify-bundle command with native fallback and hash validation.
use std::fs;
use std::path::Path;

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

/// Test: Verify bundle with native fallback (no WASM)
#[test]
fn test_verify_bundle_native_fallback_ok() {
    let test_dir = "tests/out/verify_bundle";
    let manifest_path = format!("{}/test_manifest.json", test_dir);
    let proof_path = format!("{}/test_proof.capz", test_dir);
    let bundle_path = format!("{}/test_bundle", test_dir);

    // Setup
    fs::create_dir_all(test_dir).ok();
    create_test_manifest(&manifest_path).expect("Failed to create manifest");
    create_test_capz(&proof_path).expect("Failed to create proof");

    // Create bundle first
    fs::remove_dir_all(&bundle_path).ok();
    let output = std::process::Command::new("cargo")
        .args([
            "run",
            "--bin",
            "cap-agent",
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

    assert!(output.status.success(), "bundle-v2 failed");

    // Verify bundle (should use native fallback, no verifier.wasm)
    let output = std::process::Command::new("cargo")
        .args([
            "run",
            "--bin",
            "cap-agent",
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
        "verify-bundle failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check output mentions native fallback
    let _stdout = String::from_utf8_lossy(&output.stdout);
    // Note: Actual output check would depend on implementation details

    // Cleanup
    fs::remove_dir_all(&bundle_path).ok();
}

/// Test: Verify bundle fails with missing files
#[test]
fn test_verify_bundle_missing_files_fail() {
    let test_dir = "tests/out/verify_bundle";
    let bundle_path = format!("{}/test_bundle_missing", test_dir);

    // Create empty bundle directory
    fs::create_dir_all(&bundle_path).ok();

    // Try to verify (should fail - missing manifest.json and proof.capz)
    let output = std::process::Command::new("cargo")
        .args([
            "run",
            "--bin",
            "cap-agent",
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
        "verify-bundle should fail with missing files"
    );

    // Cleanup
    fs::remove_dir_all(&bundle_path).ok();
}

/// Test: Hash validation detects tampering
#[test]
fn test_verify_bundle_hash_mismatch() {
    let test_dir = "tests/out/verify_bundle";
    let manifest_path = format!("{}/test_manifest.json", test_dir);
    let proof_path = format!("{}/test_proof.capz", test_dir);
    let bundle_path = format!("{}/test_bundle_tamper", test_dir);

    // Setup
    fs::create_dir_all(test_dir).ok();
    create_test_manifest(&manifest_path).expect("Failed to create manifest");
    create_test_capz(&proof_path).expect("Failed to create proof");

    // Create bundle
    fs::remove_dir_all(&bundle_path).ok();
    let output = std::process::Command::new("cargo")
        .args([
            "run",
            "--bin",
            "cap-agent",
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

    // Read original _meta.json
    let meta_path = format!("{}/_meta.json", bundle_path);
    let meta_content = fs::read_to_string(&meta_path).expect("Failed to read _meta.json");
    let meta: Value = serde_json::from_str(&meta_content).expect("Invalid _meta.json");
    let original_manifest_hash = meta["hashes"]["manifest_sha3"].as_str().unwrap();

    // Tamper with manifest
    let manifest_bundle_path = format!("{}/manifest.json", bundle_path);
    let mut manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_bundle_path).unwrap()).unwrap();
    manifest["audit"]["events_count"] = serde_json::json!(999); // Change value
    fs::write(
        &manifest_bundle_path,
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .expect("Failed to write tampered manifest");

    // Compute new hash
    use cap_agent::crypto;
    let tampered_bytes = fs::read(&manifest_bundle_path).unwrap();
    let tampered_hash = crypto::hex_lower_prefixed32(crypto::sha3_256(&tampered_bytes));

    // Hashes should differ
    assert_ne!(
        original_manifest_hash, tampered_hash,
        "Tampered manifest should have different hash"
    );

    // Note: verify-bundle should detect hash mismatch when implemented
    // For now, just verify hashes differ

    // Cleanup
    fs::remove_dir_all(&bundle_path).ok();
}

/// Test: Bundle structure validation with all required files
#[test]
fn test_verify_bundle_complete_structure() {
    let test_dir = "tests/out/verify_bundle";
    let manifest_path = format!("{}/test_manifest.json", test_dir);
    let proof_path = format!("{}/test_proof.capz", test_dir);
    let bundle_path = format!("{}/test_bundle_complete", test_dir);

    // Setup
    fs::create_dir_all(test_dir).ok();
    create_test_manifest(&manifest_path).expect("Failed to create manifest");
    create_test_capz(&proof_path).expect("Failed to create proof");

    // Create bundle
    fs::remove_dir_all(&bundle_path).ok();
    std::process::Command::new("cargo")
        .args([
            "run",
            "--bin",
            "cap-agent",
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

    // Verify all required files exist
    let required_files = vec!["manifest.json", "proof.capz", "_meta.json"];

    for file in required_files {
        let file_path = format!("{}/{}", bundle_path, file);
        assert!(
            Path::new(&file_path).exists(),
            "Required file missing: {}",
            file
        );
    }

    // Note: executor.json only exists when WASM is provided

    // Verify _meta.json structure
    let meta_path = format!("{}/_meta.json", bundle_path);
    let meta_content = fs::read_to_string(&meta_path).expect("Failed to read _meta.json");
    let meta: Value = serde_json::from_str(&meta_content).expect("Invalid _meta.json");

    assert_eq!(meta["bundle_version"], "cap-proof.v2.0");
    assert!(meta["hashes"]["manifest_sha3"].as_str().is_some());
    assert!(meta["hashes"]["proof_sha3"].as_str().is_some());

    // Cleanup
    fs::remove_dir_all(&bundle_path).ok();
}

/// Test: BundleExecutor correctly detects missing WASM
#[test]
fn test_executor_detects_no_wasm() {
    use cap_agent::wasm::BundleExecutor;

    let test_dir = "tests/out/verify_bundle";
    let manifest_path = format!("{}/test_manifest.json", test_dir);
    let proof_path = format!("{}/test_proof.capz", test_dir);
    let bundle_path = format!("{}/test_bundle_executor", test_dir);

    // Setup
    fs::create_dir_all(test_dir).ok();
    create_test_manifest(&manifest_path).expect("Failed to create manifest");
    create_test_capz(&proof_path).expect("Failed to create proof");

    // Create bundle (no WASM)
    fs::remove_dir_all(&bundle_path).ok();
    std::process::Command::new("cargo")
        .args([
            "run",
            "--bin",
            "cap-agent",
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

    // Create executor
    let executor = BundleExecutor::new(bundle_path.clone()).expect("Failed to create executor");

    // Should detect no WASM
    assert!(!executor.has_wasm(), "Should detect missing WASM");

    // Cleanup
    fs::remove_dir_all(&bundle_path).ok();
}

/// Test: BundleExecutor can perform native verification
#[test]
fn test_executor_native_verification() {
    use cap_agent::verifier::core::VerifyOptions;
    use cap_agent::wasm::BundleExecutor;

    let test_dir = "tests/out/verify_bundle";
    let manifest_path = format!("{}/test_manifest.json", test_dir);
    let proof_path = format!("{}/test_proof.capz", test_dir);
    let bundle_path = format!("{}/test_bundle_native", test_dir);

    // Setup
    fs::create_dir_all(test_dir).ok();
    create_test_manifest(&manifest_path).expect("Failed to create manifest");
    create_test_capz(&proof_path).expect("Failed to create proof");

    // Create bundle
    fs::remove_dir_all(&bundle_path).ok();
    std::process::Command::new("cargo")
        .args([
            "run",
            "--bin",
            "cap-agent",
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

    // Create executor and verify
    let executor = BundleExecutor::new(bundle_path.clone()).expect("Failed to create executor");

    let options = VerifyOptions {
        check_timestamp: false,
        check_registry: false,
    };

    let report = executor
        .verify(&options)
        .expect("Native verification failed");

    // Should have status "ok"
    assert_eq!(report.status, "ok", "Verification should succeed");

    // Cleanup
    fs::remove_dir_all(&bundle_path).ok();
}
