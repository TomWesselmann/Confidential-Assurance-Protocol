/// Integration tests for Dual-Anchor Schema (v0.9.0)
///
/// Tests CLI commands for private and public anchor operations.
use std::fs;
use std::process::Command;

/// Helper: Creates a test manifest with time_anchor
fn create_test_manifest_with_anchor(path: &str, audit_tip: &str) {
    let manifest_json = format!(
        r#"{{
  "version": "manifest.v1.0",
  "created_at": "2025-10-30T10:00:00Z",
  "supplier_root": "0xabc1234567890123456789012345678901234567890123456789012345678901",
  "ubo_root": "0xdef1234567890123456789012345678901234567890123456789012345678901",
  "company_commitment_root": "0x1231234567890123456789012345678901234567890123456789012345678901",
  "policy": {{
    "name": "Test Policy",
    "version": "lksg.v1",
    "hash": "0xabc1234567890123456789012345678901234567890123456789012345678901"
  }},
  "audit": {{
    "tail_digest": "0xdef1234567890123456789012345678901234567890123456789012345678901",
    "events_count": 10
  }},
  "proof": {{
    "type": "none",
    "status": "none"
  }},
  "signatures": [],
  "time_anchor": {{
    "kind": "tsa",
    "reference": "./tsa/test.tsr",
    "audit_tip_hex": "{}",
    "created_at": "2025-10-30T10:00:00Z"
  }}
}}"#,
        audit_tip
    );

    fs::write(path, manifest_json).expect("Failed to create test manifest");
}

/// Test: CLI set-private-anchor command succeeds with matching audit_tip
#[test]
fn cli_set_private_anchor_ok() {
    fs::create_dir_all("tests/out").ok();
    let manifest_path = "tests/out/test_private_anchor.json";
    let audit_tip = "0x83a8779d12345678901234567890123456789012345678901234567890123456";

    // Create test manifest
    create_test_manifest_with_anchor(manifest_path, audit_tip);

    // Run CLI command
    let output = Command::new("cargo")
        .args([
            "run", "--bin", "cap-agent",
            "--",
            "audit",
            "set-private-anchor",
            "--manifest",
            manifest_path,
            "--audit-tip",
            audit_tip,
        ])
        .output()
        .expect("Failed to execute command");

    // Check command succeeded
    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify manifest was updated
    let manifest_content = fs::read_to_string(manifest_path).expect("Failed to read manifest");
    assert!(manifest_content.contains("\"private\":"));
    assert!(manifest_content.contains(audit_tip));

    // Cleanup
    fs::remove_file(manifest_path).ok();
}

/// Test: CLI set-private-anchor fails with mismatched audit_tip
#[test]
fn cli_set_private_anchor_mismatch_fails() {
    fs::create_dir_all("tests/out").ok();
    let manifest_path = "tests/out/test_private_mismatch.json";
    let anchor_tip = "0x83a8779d12345678901234567890123456789012345678901234567890123456";
    let wrong_tip = "0xdeadbeef12345678901234567890123456789012345678901234567890123456";

    // Create test manifest with anchor_tip
    create_test_manifest_with_anchor(manifest_path, anchor_tip);

    // Try to set private anchor with wrong_tip (should fail)
    let output = Command::new("cargo")
        .args([
            "run", "--bin", "cap-agent",
            "--",
            "audit",
            "set-private-anchor",
            "--manifest",
            manifest_path,
            "--audit-tip",
            wrong_tip,
        ])
        .output()
        .expect("Failed to execute command");

    // Check command failed
    assert!(
        !output.status.success(),
        "Command should have failed with mismatched audit_tip"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("does not match"),
        "Error should mention mismatch"
    );

    // Cleanup
    fs::remove_file(manifest_path).ok();
}

/// Test: CLI set-public-anchor command with ethereum chain
#[test]
fn cli_set_public_anchor_ethereum_ok() {
    fs::create_dir_all("tests/out").ok();
    let manifest_path = "tests/out/test_public_anchor.json";
    let audit_tip = "0x83a8779d12345678901234567890123456789012345678901234567890123456";

    // Create test manifest
    create_test_manifest_with_anchor(manifest_path, audit_tip);

    // Run CLI command
    let output = Command::new("cargo")
        .args([
            "run", "--bin", "cap-agent",
            "--",
            "audit",
            "set-public-anchor",
            "--manifest",
            manifest_path,
            "--chain",
            "ethereum",
            "--txid",
            "0xabc123def456",
            "--digest",
            "0x1234567890123456789012345678901234567890123456789012345678901234",
        ])
        .output()
        .expect("Failed to execute command");

    // Check command succeeded
    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify manifest was updated
    let manifest_content = fs::read_to_string(manifest_path).expect("Failed to read manifest");
    assert!(manifest_content.contains("\"public\":"));
    assert!(manifest_content.contains("\"ethereum\""));
    assert!(manifest_content.contains("0xabc123def456"));

    // Cleanup
    fs::remove_file(manifest_path).ok();
}

/// Test: CLI verify-anchor command with valid dual-anchor
#[test]
fn cli_verify_anchor_ok() {
    fs::create_dir_all("tests/out").ok();
    let manifest_path = "tests/out/test_verify_anchor.json";
    let audit_tip = "0x83a8779d12345678901234567890123456789012345678901234567890123456";

    // Create test manifest
    create_test_manifest_with_anchor(manifest_path, audit_tip);

    // Set private anchor
    Command::new("cargo")
        .args([
            "run", "--bin", "cap-agent",
            "--",
            "audit",
            "set-private-anchor",
            "--manifest",
            manifest_path,
            "--audit-tip",
            audit_tip,
        ])
        .output()
        .expect("Failed to set private anchor");

    // Set public anchor
    Command::new("cargo")
        .args([
            "run", "--bin", "cap-agent",
            "--",
            "audit",
            "set-public-anchor",
            "--manifest",
            manifest_path,
            "--chain",
            "ethereum",
            "--txid",
            "0xabc123",
            "--digest",
            "0x1234567890123456789012345678901234567890123456789012345678901234",
        ])
        .output()
        .expect("Failed to set public anchor");

    // Verify dual-anchor
    let output = Command::new("cargo")
        .args([
            "run", "--bin", "cap-agent",
            "--",
            "audit",
            "verify-anchor",
            "--manifest",
            manifest_path,
        ])
        .output()
        .expect("Failed to execute verify command");

    // Check command succeeded
    assert!(
        output.status.success(),
        "Verification failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Status:") && (stdout.contains("ok") || stdout.contains("✅")));

    // Cleanup
    fs::remove_file(manifest_path).ok();
}
