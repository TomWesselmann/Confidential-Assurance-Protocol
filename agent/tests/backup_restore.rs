//! # Backup & Restore Integration Tests (Week 6 - D1)
//!
//! Integration tests for backup/restore procedures:
//! - Backup manifest generation and verification
//! - Restore hash integrity validation
//! - No PII in backups
//! - Full backup/restore cycle (requires cluster)
//!
//! **IMPORTANT**: Some tests require a Kubernetes cluster and are marked with `#[ignore]`.
//! Run with: `cargo test --test backup_restore -- --ignored --nocapture`

use serde_json::Value;
use std::fs;
use std::process::Command;

/// Helper: Execute shell script and return output
fn run_script(script: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(script)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute {}: {}", script, e))?;

    if !output.status.success() {
        return Err(format!(
            "{} exited with non-zero status: {}\nStdout: {}\nStderr: {}",
            script,
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Helper: Create test registry file (minimal SQLite)
fn create_test_registry(path: &str) {
    use std::fs;

    // Create minimal test registry JSON (simpler than SQLite for tests)
    let registry_json = serde_json::json!({
        "registry_version": "1.0",
        "entries": [
            {
                "id": "entry_001",
                "manifest_hash": "0x1da941f7026bae3cf8b1bcdc3a8e01e76ea678c32ec6bc2c374fb67b3744571f",
                "proof_hash": "0x83a8779ddef4567890123456789012345678901234567890123456789012345678",
                "timestamp": "2025-11-10T12:00:00Z",
                "signature": "test_signature_base64",
                "public_key": "test_pubkey_base64",
                "kid": "a010ac65166984697b93b867c36e9c94",
                "signature_scheme": "ed25519"
            }
        ]
    });

    fs::write(path, registry_json.to_string()).expect("Failed to write test registry");
}

/// Helper: Create test policy store file
fn create_test_policy_store(path: &str) {
    let policy_store_json = serde_json::json!({
        "policies": [
            {
                "policy_hash": "0x0afcb402e74ff6a11601863fc4ae2f2d756124db71bc703f5889ecefcd371ff4",
                "policy_id": "lksg.v1",
                "version": "lksg.v1",
                "created_at": "2025-11-10T10:00:00Z"
            }
        ]
    });

    fs::write(path, policy_store_json.to_string()).expect("Failed to write test policy store");
}

/// Helper: Compute SHA3-256 hash using openssl (fallback)
#[allow(dead_code)]
fn _compute_sha3_fallback(file_path: &str) -> String {
    let output = Command::new("openssl")
        .args(["dgst", "-sha3-256", "-hex", file_path])
        .output()
        .expect("Failed to execute openssl");

    let output_str = String::from_utf8_lossy(&output.stdout);
    // Parse "SHA3-256(file)= abc123..." format
    output_str.split('=').nth(1).unwrap().trim().to_string()
}

#[test]
fn test_backup_manifest_generation() {
    println!("🧪 Test: Backup manifest generation");

    // Setup: Create test files
    let temp_dir = std::env::temp_dir().join("cap_backup_test");
    fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

    let registry_path = temp_dir.join("registry.json");
    let policy_store_path = temp_dir.join("policy_store.json");
    let output_path = temp_dir.join("backup.tar.gz");
    let manifest_path = temp_dir.join("backup.manifest.json");

    create_test_registry(registry_path.to_str().unwrap());
    create_test_policy_store(policy_store_path.to_str().unwrap());

    // Execute backup script
    println!("📦 Running backup script...");
    let result = run_script(
        "./scripts/backup.sh",
        &[
            "--output",
            output_path.to_str().unwrap(),
            "--registry",
            registry_path.to_str().unwrap(),
            "--policy-store",
            policy_store_path.to_str().unwrap(),
            "--manifest",
            manifest_path.to_str().unwrap(),
        ],
    );

    assert!(result.is_ok(), "Backup script failed: {:?}", result.err());
    println!("✅ Backup script executed successfully");

    // Verify manifest exists
    assert!(manifest_path.exists(), "Manifest file not created");
    println!("✅ Manifest file created");

    // Parse manifest
    let manifest_content = fs::read_to_string(&manifest_path).expect("Failed to read manifest");
    let manifest: Value =
        serde_json::from_str(&manifest_content).expect("Failed to parse manifest JSON");

    // Validate manifest structure
    assert_eq!(
        manifest["version"], "backup.manifest.v1",
        "Manifest version mismatch"
    );
    assert!(manifest["created_at"].is_string(), "created_at missing");
    assert!(manifest["backup_id"].is_string(), "backup_id missing");
    assert!(manifest["files"].is_array(), "files array missing");

    let files = manifest["files"].as_array().unwrap();
    assert!(
        files.len() >= 2,
        "Expected at least 2 files (registry + policy_store), got {}",
        files.len()
    );
    println!("✅ Manifest structure valid ({} files)", files.len());

    // Validate hashes are present
    for file in files {
        assert!(
            file["sha3_256"].is_string(),
            "sha3_256 hash missing for file: {:?}",
            file["path"]
        );
        let hash = file["sha3_256"].as_str().unwrap();
        assert!(
            hash.starts_with("0x"),
            "Hash should be 0x-prefixed: {}",
            hash
        );
        assert!(
            hash.len() == 66,
            "Hash should be 66 chars (0x + 64 hex), got {}",
            hash.len()
        );
    }
    println!("✅ All file hashes present and valid format");

    // Cleanup
    fs::remove_dir_all(&temp_dir).ok();
    println!("🎉 Test passed");
}

#[test]
#[ignore] // Integration test requiring external scripts (backup.sh, restore.sh) and tools (jq, tar) - may fail in CI
fn test_restore_hash_integrity() {
    println!("🧪 Test: Restore hash integrity verification");

    // Setup: Create test backup
    let temp_dir = std::env::temp_dir().join("cap_restore_test");
    fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

    let registry_path = temp_dir.join("registry.json");
    let policy_store_path = temp_dir.join("policy_store.json");
    let output_path = temp_dir.join("backup.tar.gz");

    create_test_registry(registry_path.to_str().unwrap());
    create_test_policy_store(policy_store_path.to_str().unwrap());

    // Create backup
    println!("📦 Creating backup...");
    run_script(
        "./scripts/backup.sh",
        &[
            "--output",
            output_path.to_str().unwrap(),
            "--registry",
            registry_path.to_str().unwrap(),
            "--policy-store",
            policy_store_path.to_str().unwrap(),
        ],
    )
    .expect("Backup failed");

    // Extract backup for restore
    let restore_dir = temp_dir.join("restore");
    fs::create_dir_all(&restore_dir).expect("Failed to create restore dir");

    println!("📂 Extracting backup...");
    let extract_result = Command::new("tar")
        .args([
            "-xzf",
            output_path.to_str().unwrap(),
            "-C",
            restore_dir.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to extract backup");

    assert!(extract_result.success(), "Extract failed");

    // Run restore script in verify-only mode
    println!("🔍 Running restore verification...");
    let manifest_path = restore_dir.join("backup.manifest.json");

    let result = run_script(
        "./scripts/restore.sh",
        &[
            "--backup-dir",
            restore_dir.to_str().unwrap(),
            "--manifest",
            manifest_path.to_str().unwrap(),
            "--verify-only",
        ],
    );

    assert!(
        result.is_ok(),
        "Restore verification failed: {:?}",
        result.err()
    );
    println!("✅ Restore verification passed");

    // Cleanup
    fs::remove_dir_all(&temp_dir).ok();
    println!("🎉 Test passed");
}

#[test]
fn test_no_pii_in_backup() {
    println!("🧪 Test: No PII in backup");

    // Setup: Create test files
    let temp_dir = std::env::temp_dir().join("cap_pii_test");
    fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

    let registry_path = temp_dir.join("registry.json");
    let policy_store_path = temp_dir.join("policy_store.json");
    let output_path = temp_dir.join("backup.tar.gz");

    create_test_registry(registry_path.to_str().unwrap());
    create_test_policy_store(policy_store_path.to_str().unwrap());

    // Create backup
    println!("📦 Creating backup...");
    run_script(
        "./scripts/backup.sh",
        &[
            "--output",
            output_path.to_str().unwrap(),
            "--registry",
            registry_path.to_str().unwrap(),
            "--policy-store",
            policy_store_path.to_str().unwrap(),
        ],
    )
    .expect("Backup failed");

    // Extract backup
    let extract_dir = temp_dir.join("extract");
    fs::create_dir_all(&extract_dir).expect("Failed to create extract dir");

    Command::new("tar")
        .args([
            "-xzf",
            output_path.to_str().unwrap(),
            "-C",
            extract_dir.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to extract backup");

    // Check for PII patterns in all extracted files
    println!("🔍 Scanning for PII...");
    let pii_patterns = vec![
        "email",
        "phone",
        "address",
        "firstname",
        "lastname",
        "ssn",
        "dob",
        "birthdate",
        "passport",
        "credit_card",
    ];

    let registry_content =
        fs::read_to_string(extract_dir.join("registry.json")).expect("Failed to read registry");
    let policy_content = fs::read_to_string(extract_dir.join("policy_store.json"))
        .expect("Failed to read policy store");

    for pattern in &pii_patterns {
        assert!(
            !registry_content.to_lowercase().contains(pattern),
            "PII pattern '{}' found in registry backup",
            pattern
        );
        assert!(
            !policy_content.to_lowercase().contains(pattern),
            "PII pattern '{}' found in policy store backup",
            pattern
        );
    }

    println!("✅ No PII patterns detected");

    // Verify only hashes/commitments present
    assert!(
        registry_content.contains("0x"),
        "Registry should contain hashes (0x-prefixed)"
    );
    assert!(
        policy_content.contains("0x"),
        "Policy store should contain hashes (0x-prefixed)"
    );

    println!("✅ Only hashes/commitments present in backup");

    // Cleanup
    fs::remove_dir_all(&temp_dir).ok();
    println!("🎉 Test passed");
}

#[test]
fn test_restored_policy_hash_determinism() {
    println!("🧪 Test: Restored policy hash determinism");

    // Setup: Create test files
    let temp_dir = std::env::temp_dir().join("cap_hash_test");
    fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

    let policy_store_path = temp_dir.join("policy_store.json");

    // Create policy store with known hash
    let policy_hash = "0x0afcb402e74ff6a11601863fc4ae2f2d756124db71bc703f5889ecefcd371ff4";
    let policy_store_json = serde_json::json!({
        "policies": [
            {
                "policy_hash": policy_hash,
                "policy_id": "lksg.v1",
                "version": "lksg.v1",
                "created_at": "2025-11-10T10:00:00Z",
                "policy": {
                    "version": "lksg.v1",
                    "name": "Test Policy",
                    "constraints": {
                        "require_at_least_one_ubo": true
                    }
                }
            }
        ]
    });

    fs::write(&policy_store_path, policy_store_json.to_string())
        .expect("Failed to write policy store");

    // Create backup
    let registry_path = temp_dir.join("registry.json");
    create_test_registry(registry_path.to_str().unwrap());

    let output_path = temp_dir.join("backup.tar.gz");
    run_script(
        "./scripts/backup.sh",
        &[
            "--output",
            output_path.to_str().unwrap(),
            "--registry",
            registry_path.to_str().unwrap(),
            "--policy-store",
            policy_store_path.to_str().unwrap(),
        ],
    )
    .expect("Backup failed");

    // Extract and verify
    let restore_dir = temp_dir.join("restore");
    fs::create_dir_all(&restore_dir).expect("Failed to create restore dir");

    Command::new("tar")
        .args([
            "-xzf",
            output_path.to_str().unwrap(),
            "-C",
            restore_dir.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to extract");

    // Read restored policy store
    let restored_content = fs::read_to_string(restore_dir.join("policy_store.json"))
        .expect("Failed to read restored policy store");
    let restored_json: Value =
        serde_json::from_str(&restored_content).expect("Failed to parse restored JSON");

    let restored_hash = restored_json["policies"][0]["policy_hash"]
        .as_str()
        .unwrap();

    // CRITICAL: Hash must match exactly
    assert_eq!(
        restored_hash, policy_hash,
        "Policy hash changed after backup/restore!"
    );
    println!("✅ Policy hash deterministic: {}", restored_hash);

    // Cleanup
    fs::remove_dir_all(&temp_dir).ok();
    println!("🎉 Test passed");
}

#[test]
#[ignore] // Requires Kubernetes cluster
fn test_full_backup_restore_cycle() {
    println!("🧪 Test: Full backup/restore cycle (requires cluster)");

    // This test requires:
    // 1. Running Kubernetes cluster
    // 2. CAP Verifier API deployed
    // 3. kubectl configured

    // Setup: Create real backup from running system
    let temp_dir = std::env::temp_dir().join("cap_e2e_test");
    fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

    // Step 1: Get current registry from running pod
    println!("📥 Step 1: Fetching current registry from cluster...");
    let get_pod_cmd =
        "kubectl -n cap get pod -l app=cap-verifier-api -o jsonpath='{.items[0].metadata.name}'";
    let pod_output = Command::new("sh")
        .args(["-c", get_pod_cmd])
        .output()
        .expect("Failed to get pod name");

    let pod_name = String::from_utf8_lossy(&pod_output.stdout)
        .trim()
        .to_string();
    assert!(!pod_name.is_empty(), "No CAP API pod found");

    // Copy registry from pod
    let registry_path = temp_dir.join("registry.sqlite");
    let cp_cmd = format!(
        "kubectl -n cap cp {}:/app/build/registry.sqlite {}",
        pod_name,
        registry_path.to_str().unwrap()
    );

    Command::new("sh")
        .args(["-c", &cp_cmd])
        .status()
        .expect("Failed to copy registry from pod");

    assert!(registry_path.exists(), "Registry not copied");
    println!("✅ Registry copied from cluster");

    // Step 2: Create backup
    println!("📦 Step 2: Creating backup...");
    let policy_store_path = temp_dir.join("policy_store.json");
    create_test_policy_store(policy_store_path.to_str().unwrap()); // Use test policy for simplicity

    let output_path = temp_dir.join("backup.tar.gz");
    run_script(
        "./scripts/backup.sh",
        &[
            "--output",
            output_path.to_str().unwrap(),
            "--registry",
            registry_path.to_str().unwrap(),
            "--policy-store",
            policy_store_path.to_str().unwrap(),
        ],
    )
    .expect("Backup failed");

    println!("✅ Backup created");

    // Step 3: Extract backup
    println!("📂 Step 3: Extracting backup...");
    let restore_dir = temp_dir.join("restore");
    fs::create_dir_all(&restore_dir).expect("Failed to create restore dir");

    Command::new("tar")
        .args([
            "-xzf",
            output_path.to_str().unwrap(),
            "-C",
            restore_dir.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to extract");

    // Step 4: Verify backup
    println!("🔍 Step 4: Verifying backup...");
    let manifest_path = restore_dir.join("backup.manifest.json");

    run_script(
        "./scripts/restore.sh",
        &[
            "--backup-dir",
            restore_dir.to_str().unwrap(),
            "--manifest",
            manifest_path.to_str().unwrap(),
            "--verify-only",
        ],
    )
    .expect("Verification failed");

    println!("✅ Backup verified");

    // Step 5: Restore to new namespace (requires cluster)
    println!("♻️  Step 5: Restoring to cap-test namespace...");
    let restore_result = run_script(
        "./scripts/restore.sh",
        &[
            "--backup-dir",
            restore_dir.to_str().unwrap(),
            "--manifest",
            manifest_path.to_str().unwrap(),
            "--target-namespace",
            "cap-test",
            "--skip-smoke", // Skip smoke tests in CI
        ],
    );

    assert!(
        restore_result.is_ok(),
        "Restore failed: {:?}",
        restore_result.err()
    );
    println!("✅ Restore completed");

    // Step 6: Validate restored system
    println!("✅ Step 6: Validating restored system...");
    let health_cmd = "kubectl -n cap-test exec $(kubectl -n cap-test get pod -l app=cap-verifier-api -o jsonpath='{.items[0].metadata.name}') -- curl -s http://localhost:8080/healthz";

    let health_output = Command::new("sh")
        .args(["-c", health_cmd])
        .output()
        .expect("Failed to check health");

    let health_json: Value =
        serde_json::from_slice(&health_output.stdout).expect("Failed to parse health response");
    assert_eq!(health_json["status"], "OK", "Health check failed");

    println!("✅ Health check passed");

    // Cleanup namespace
    println!("🧹 Cleanup: Deleting test namespace...");
    Command::new("kubectl")
        .args(["delete", "namespace", "cap-test"])
        .status()
        .ok();

    // Cleanup local files
    fs::remove_dir_all(&temp_dir).ok();

    println!("🎉 Full E2E test passed");
}

#[test]
#[ignore] // Requires cluster
fn test_smoke_ready_after_restore() {
    println!("🧪 Test: Smoke tests pass after restore");

    // This test verifies that:
    // 1. /healthz returns 200 OK
    // 2. /readyz returns 200 OK (with auth)
    // 3. Registry is accessible
    // 4. Policy store is loaded

    // Prerequisite: cap-test namespace exists with restored system
    let namespace = "cap-test";

    // Get pod name
    let get_pod_cmd = format!(
        "kubectl -n {} get pod -l app=cap-verifier-api -o jsonpath='{{.items[0].metadata.name}}'",
        namespace
    );
    let pod_output = Command::new("sh")
        .args(["-c", &get_pod_cmd])
        .output()
        .expect("Failed to get pod");

    let pod_name = String::from_utf8_lossy(&pod_output.stdout)
        .trim()
        .to_string();
    assert!(
        !pod_name.is_empty(),
        "No pod found in namespace {}",
        namespace
    );

    // Test 1: Health check
    println!("🔍 Test 1: Health check...");
    let health_cmd = format!(
        "kubectl -n {} exec {} -- curl -s http://localhost:8080/healthz",
        namespace, pod_name
    );

    let health_output = Command::new("sh")
        .args(["-c", &health_cmd])
        .output()
        .expect("Health check failed");

    let health_json: Value =
        serde_json::from_slice(&health_output.stdout).expect("Failed to parse health JSON");
    assert_eq!(health_json["status"], "OK");
    println!("✅ Health check passed");

    // Test 2: Registry query
    println!("🔍 Test 2: Registry query...");
    let registry_cmd = format!(
        "kubectl -n {} exec {} -- sqlite3 /app/build/registry.sqlite 'SELECT COUNT(*) FROM registry_entries;'",
        namespace, pod_name
    );

    let registry_output = Command::new("sh")
        .args(["-c", &registry_cmd])
        .output()
        .expect("Registry query failed");

    let count_str = String::from_utf8_lossy(&registry_output.stdout)
        .trim()
        .to_string();
    let count: i32 = count_str.parse().unwrap_or(0);
    assert!(count > 0, "Registry has no entries");
    println!("✅ Registry accessible ({} entries)", count);

    println!("🎉 Smoke tests passed");
}
