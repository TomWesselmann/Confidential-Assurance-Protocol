use serde_json::Value;
/// Integration tests for Bundle v2 creation and validation
///
/// Tests bundle creation, hash consistency, and structure validation.
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
        "signatures": []
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

/// Test: Build minimal bundle successfully
#[test]
fn test_build_minimal_bundle_ok() {
    let test_dir = "tests/out/bundle_v2";
    let manifest_path = format!("{}/test_manifest.json", test_dir);
    let proof_path = format!("{}/test_proof.capz", test_dir);
    let bundle_path = format!("{}/test_bundle", test_dir);

    // Setup
    fs::create_dir_all(test_dir).ok();
    create_test_manifest(&manifest_path).expect("Failed to create manifest");
    create_test_capz(&proof_path).expect("Failed to create proof");

    // Clean old bundle
    fs::remove_dir_all(&bundle_path).ok();

    // Create bundle via CLI
    let output = std::process::Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "cap-agent",
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

    // Check command succeeded
    assert!(
        output.status.success(),
        "bundle-v2 failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify bundle structure
    assert!(Path::new(&format!("{}/manifest.json", bundle_path)).exists());
    assert!(Path::new(&format!("{}/proof.capz", bundle_path)).exists());
    assert!(Path::new(&format!("{}/_meta.json", bundle_path)).exists());

    // Verify _meta.json
    let meta_content = fs::read_to_string(format!("{}/_meta.json", bundle_path))
        .expect("Failed to read _meta.json");
    let meta: Value = serde_json::from_str(&meta_content).expect("Invalid _meta.json");

    assert_eq!(meta["bundle_version"], "cap-proof.v2.0");
    assert!(meta["hashes"]["manifest_sha3"]
        .as_str()
        .unwrap()
        .starts_with("0x"));
    assert!(meta["hashes"]["proof_sha3"]
        .as_str()
        .unwrap()
        .starts_with("0x"));

    // Cleanup
    fs::remove_dir_all(&bundle_path).ok();
}

/// Test: Bundle creation fails without --force on existing directory
#[test]
fn test_bundle_exists_without_force_fails() {
    let test_dir = "tests/out/bundle_v2";
    let manifest_path = format!("{}/test_manifest.json", test_dir);
    let proof_path = format!("{}/test_proof.capz", test_dir);
    let bundle_path = format!("{}/test_bundle_exists", test_dir);

    // Setup
    fs::create_dir_all(test_dir).ok();
    create_test_manifest(&manifest_path).expect("Failed to create manifest");
    create_test_capz(&proof_path).expect("Failed to create proof");

    // Create bundle first time with --force
    fs::create_dir_all(&bundle_path).ok();
    fs::write(format!("{}/dummy.txt", bundle_path), "exists").ok();

    // Try to create again WITHOUT --force (should fail)
    let output = std::process::Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "cap-agent",
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
        ])
        .output()
        .expect("Failed to execute bundle-v2");

    // Should fail
    assert!(
        !output.status.success(),
        "bundle-v2 should fail without --force on existing dir"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("already exists") || stderr.contains("exists"),
        "Error should mention directory exists"
    );

    // Cleanup
    fs::remove_dir_all(&bundle_path).ok();
}

/// Test: Tampered manifest detected via hash mismatch
#[test]
fn test_tamper_manifest_detected() {
    let test_dir = "tests/out/bundle_v2";
    let manifest_path = format!("{}/test_manifest.json", test_dir);
    let proof_path = format!("{}/test_proof.capz", test_dir);
    let bundle_path = format!("{}/test_bundle_tamper", test_dir);

    // Setup
    fs::create_dir_all(test_dir).ok();
    create_test_manifest(&manifest_path).expect("Failed to create manifest");
    create_test_capz(&proof_path).expect("Failed to create proof");

    // Clean and create bundle
    fs::remove_dir_all(&bundle_path).ok();
    let output = std::process::Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "cap-agent",
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
    let original_hash = meta["hashes"]["manifest_sha3"].as_str().unwrap();

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
        original_hash, tampered_hash,
        "Tampered manifest should have different hash"
    );

    // Note: Actual validation happens in verify-bundle
    // This test verifies that tampering changes the hash

    // Cleanup
    fs::remove_dir_all(&bundle_path).ok();
}

/// Test: CAPZ header validation with invalid magic
#[test]
fn test_capz_invalid_magic_fails() {
    use cap_agent::proof::CapzHeader;
    use std::io::Cursor;

    let mut invalid_header = vec![0u8; 78];
    invalid_header[0..4].copy_from_slice(b"XXXX"); // Wrong magic

    let mut cursor = Cursor::new(invalid_header);
    let result = CapzHeader::read(&mut cursor);

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid CAPZ magic"));
}

/// Test: CAPZ header validation with invalid version
#[test]
fn test_capz_invalid_version_fails() {
    use cap_agent::proof::{CapzHeader, CAPZ_MAGIC};
    use std::io::Cursor;

    let mut invalid_header = vec![0u8; 78];
    invalid_header[0..4].copy_from_slice(CAPZ_MAGIC);
    invalid_header[4..6].copy_from_slice(&0x9999u16.to_le_bytes()); // Wrong version

    let mut cursor = Cursor::new(invalid_header);
    let result = CapzHeader::read(&mut cursor);

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Unsupported CAPZ version"));
}

/// Test: Bundle structure validation
#[test]
fn test_bundle_structure_complete() {
    let test_dir = "tests/out/bundle_v2";
    let manifest_path = format!("{}/test_manifest.json", test_dir);
    let proof_path = format!("{}/test_proof.capz", test_dir);
    let bundle_path = format!("{}/test_bundle_structure", test_dir);

    // Setup
    fs::create_dir_all(test_dir).ok();
    create_test_manifest(&manifest_path).expect("Failed to create manifest");
    create_test_capz(&proof_path).expect("Failed to create proof");

    // Create bundle
    fs::remove_dir_all(&bundle_path).ok();
    std::process::Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "cap-agent",
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

    // Cleanup
    fs::remove_dir_all(&bundle_path).ok();
}
