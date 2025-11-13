/// Integration test for ZIP archive creation
///
/// Tests that bundle-v2 --zip creates a valid ZIP archive
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

/// Test: Create bundle with ZIP archive
#[test]
fn test_create_bundle_with_zip() {
    let test_dir = "tests/out/zip_creation";
    let manifest_path = format!("{}/test_manifest.json", test_dir);
    let proof_path = format!("{}/test_proof.capz", test_dir);
    let bundle_path = format!("{}/test_bundle", test_dir);
    let zip_path = format!("{}.zip", bundle_path);

    // Setup
    fs::create_dir_all(test_dir).ok();
    create_test_manifest(&manifest_path).expect("Failed to create manifest");
    create_test_capz(&proof_path).expect("Failed to create proof");

    // Clean old bundle and ZIP
    fs::remove_dir_all(&bundle_path).ok();
    fs::remove_file(&zip_path).ok();

    // Create bundle with ZIP
    let output = std::process::Command::new("cargo")
        .args([
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
            "--zip",
        ])
        .output()
        .expect("Failed to execute bundle-v2");

    // Check command succeeded
    assert!(
        output.status.success(),
        "bundle-v2 --zip failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify ZIP file exists
    assert!(Path::new(&zip_path).exists(), "ZIP file should be created");

    // Verify ZIP is not empty
    let zip_metadata = fs::metadata(&zip_path).expect("Failed to get ZIP metadata");
    assert!(
        zip_metadata.len() > 100,
        "ZIP file should not be empty (size: {} bytes)",
        zip_metadata.len()
    );

    // Cleanup
    fs::remove_dir_all(&bundle_path).ok();
    fs::remove_file(&zip_path).ok();
}

/// Test: ZIP contains all required files
#[test]
fn test_zip_contains_all_files() {
    let test_dir = "tests/out/zip_creation";
    let manifest_path = format!("{}/test_manifest.json", test_dir);
    let proof_path = format!("{}/test_proof.capz", test_dir);
    let bundle_path = format!("{}/test_bundle_complete", test_dir);
    let zip_path = format!("{}.zip", bundle_path);

    // Setup
    fs::create_dir_all(test_dir).ok();
    create_test_manifest(&manifest_path).expect("Failed to create manifest");
    create_test_capz(&proof_path).expect("Failed to create proof");

    // Clean old bundle and ZIP
    fs::remove_dir_all(&bundle_path).ok();
    fs::remove_file(&zip_path).ok();

    // Create bundle with ZIP
    std::process::Command::new("cargo")
        .args([
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
            "--zip",
        ])
        .output()
        .expect("Failed to execute bundle-v2");

    // Open and inspect ZIP
    let zip_file = fs::File::open(&zip_path).expect("Failed to open ZIP");
    let mut archive = zip::ZipArchive::new(zip_file).expect("Failed to read ZIP");

    // Check required files are in ZIP
    let required_files = vec!["manifest.json", "proof.capz", "_meta.json", "README.txt"];
    let mut found_files = Vec::new();

    for i in 0..archive.len() {
        let file = archive.by_index(i).expect("Failed to get ZIP entry");
        found_files.push(file.name().to_string());
    }

    for required in &required_files {
        assert!(
            found_files.contains(&required.to_string()),
            "ZIP should contain {}, found: {:?}",
            required,
            found_files
        );
    }

    // Cleanup
    fs::remove_dir_all(&bundle_path).ok();
    fs::remove_file(&zip_path).ok();
}

/// Test: ZIP can be extracted and verified
#[test]
fn test_zip_extract_and_verify() {
    let test_dir = "tests/out/zip_creation";
    let manifest_path = format!("{}/test_manifest.json", test_dir);
    let proof_path = format!("{}/test_proof.capz", test_dir);
    let bundle_path = format!("{}/test_bundle_extract", test_dir);
    let zip_path = format!("{}.zip", bundle_path);
    let extract_path = format!("{}_extracted", bundle_path);

    // Setup
    fs::create_dir_all(test_dir).ok();
    create_test_manifest(&manifest_path).expect("Failed to create manifest");
    create_test_capz(&proof_path).expect("Failed to create proof");

    // Clean
    fs::remove_dir_all(&bundle_path).ok();
    fs::remove_file(&zip_path).ok();
    fs::remove_dir_all(&extract_path).ok();

    // Create bundle with ZIP
    std::process::Command::new("cargo")
        .args([
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
            "--zip",
        ])
        .output()
        .expect("Failed to execute bundle-v2");

    // Extract ZIP
    fs::create_dir_all(&extract_path).ok();
    let zip_file = fs::File::open(&zip_path).expect("Failed to open ZIP");
    let mut archive = zip::ZipArchive::new(zip_file).expect("Failed to read ZIP");
    archive
        .extract(&extract_path)
        .expect("Failed to extract ZIP");

    // Verify extracted files exist
    assert!(Path::new(&format!("{}/manifest.json", extract_path)).exists());
    assert!(Path::new(&format!("{}/proof.capz", extract_path)).exists());
    assert!(Path::new(&format!("{}/_meta.json", extract_path)).exists());
    assert!(Path::new(&format!("{}/README.txt", extract_path)).exists());

    // Cleanup
    fs::remove_dir_all(&bundle_path).ok();
    fs::remove_file(&zip_path).ok();
    fs::remove_dir_all(&extract_path).ok();
}
