// Allow deprecated cargo_bin for compatibility with custom build directories
#![allow(deprecated)]

/**
 * CLI-based End-to-End Workflow Integration Test
 *
 * Tests the entire CAP Agent workflow by calling the CLI binary:
 * 1. CSV Data → Commitments (prepare)
 * 2. Commitments → Manifest (manifest build)
 * 3. Manifest + Policy → Proof (proof build)
 * 4. Proof → Verification (proof verify)
 * 5. Proof → Package Export (proof export)
 * 6. Package → Verification (verifier run)
 */
use anyhow::Result;
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_cli_complete_workflow() -> Result<()> {
    // Setup test directory
    let temp_dir = TempDir::new()?;
    let test_dir = temp_dir.path();
    let build_dir = test_dir.join("build");
    fs::create_dir_all(&build_dir)?;

    // Step 1: Create test CSV files
    let suppliers_csv = test_dir.join("suppliers.csv");
    fs::write(
        &suppliers_csv,
        "name,jurisdiction,tier\nAcme Corp,Germany,1\nGlobal Inc,USA,2\n",
    )?;

    let ubos_csv = test_dir.join("ubos.csv");
    fs::write(
        &ubos_csv,
        "name,birthdate,citizenship\nJohn Doe,1980-01-01,USA\nJane Smith,1985-05-15,Germany\n",
    )?;

    // Step 2: Create test policy
    let policy_file = test_dir.join("test_policy.yml");
    fs::write(
        &policy_file,
        r#"
version: "lksg.v1"
name: "E2E Test Policy"
created_at: "2025-11-20T10:00:00Z"
constraints:
  require_at_least_one_ubo: true
  supplier_count_max: 10
notes: "End-to-End Integration Test Policy"
"#,
    )?;

    // Step 3: Run `cap-agent prepare`
    Command::cargo_bin("cap-agent")?
        .current_dir(&test_dir)
        .arg("prepare")
        .arg("--suppliers")
        .arg(&suppliers_csv)
        .arg("--ubos")
        .arg(&ubos_csv)
        .assert()
        .success();

    // Verify commitments.json was created
    let commitments_file = build_dir.join("commitments.json");
    assert!(
        commitments_file.exists(),
        "commitments.json should exist after prepare"
    );

    let commitments_content = fs::read_to_string(&commitments_file)?;
    assert!(
        commitments_content.contains("company_commitment_root"),
        "Commitments should contain company_commitment_root"
    );

    // Step 4: Run `cap-agent manifest build`
    Command::cargo_bin("cap-agent")?
        .current_dir(&test_dir)
        .arg("manifest")
        .arg("build")
        .arg("--policy")
        .arg(&policy_file)
        .assert()
        .success();

    // Verify manifest.json was created
    let manifest_file = build_dir.join("manifest.json");
    assert!(
        manifest_file.exists(),
        "manifest.json should exist after manifest build"
    );

    let manifest_content = fs::read_to_string(&manifest_file)?;
    assert!(
        manifest_content.contains("manifest.v1.0"),
        "Manifest should have version manifest.v1.0"
    );
    assert!(
        manifest_content.contains("E2E Test Policy"),
        "Manifest should contain policy name"
    );

    // Step 5: Run `cap-agent proof build`
    Command::cargo_bin("cap-agent")?
        .current_dir(&test_dir)
        .arg("proof")
        .arg("build")
        .arg("--manifest")
        .arg(&manifest_file)
        .arg("--policy")
        .arg(&policy_file)
        .assert()
        .success();

    // Verify proof.dat and proof.json were created
    let proof_dat = build_dir.join("proof.dat");
    let proof_json = build_dir.join("proof.json");
    assert!(proof_dat.exists(), "proof.dat should exist");
    assert!(proof_json.exists(), "proof.json should exist");

    let proof_content = fs::read_to_string(&proof_json)?;
    assert!(
        proof_content.contains("proof.v0"),
        "Proof should have version"
    );

    // Step 6: Run `cap-agent proof verify`
    Command::cargo_bin("cap-agent")?
        .current_dir(&test_dir)
        .arg("proof")
        .arg("verify")
        .arg("--proof")
        .arg(&proof_dat)
        .arg("--manifest")
        .arg(&manifest_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("ok")); // Should contain "status": "ok"

    // Step 7: Run `cap-agent proof export`
    let package_dir = build_dir.join("proof_package");
    Command::cargo_bin("cap-agent")?
        .current_dir(&test_dir)
        .arg("proof")
        .arg("export")
        .arg("--manifest")
        .arg(&manifest_file)
        .arg("--proof")
        .arg(&proof_dat)
        .arg("--out")
        .arg(&package_dir)
        .arg("--force")
        .assert()
        .success();

    // Verify package contents
    assert!(package_dir.exists(), "Package directory should exist");
    assert!(
        package_dir.join("manifest.json").exists(),
        "Package should contain manifest.json"
    );
    assert!(
        package_dir.join("proof.dat").exists(),
        "Package should contain proof.dat"
    );
    assert!(
        package_dir.join("_meta.json").exists(),
        "Package should contain _meta.json"
    );
    assert!(
        package_dir.join("README.txt").exists(),
        "Package should contain README.txt"
    );

    // Step 8: Run `cap-agent verifier run` on exported package
    Command::cargo_bin("cap-agent")?
        .current_dir(&test_dir)
        .arg("verifier")
        .arg("run")
        .arg("--package")
        .arg(&package_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Bundle-Verifikation abgeschlossen",
        )); // Should show verification success

    println!("✅ Complete CLI E2E Workflow Test PASSED");
    println!("   Steps completed:");
    println!("   1. ✅ prepare (CSV → Commitments)");
    println!("   2. ✅ manifest build (Commitments → Manifest)");
    println!("   3. ✅ proof build (Manifest + Policy → Proof)");
    println!("   4. ✅ proof verify (Proof Verification)");
    println!("   5. ✅ proof export (Package Export)");
    println!("   6. ✅ verifier run (Package Verification)");

    Ok(())
}

#[test]
fn test_cli_workflow_with_registry() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_dir = temp_dir.path();
    let build_dir = test_dir.join("build");
    fs::create_dir_all(&build_dir)?;

    // Create CSV files
    let suppliers_csv = test_dir.join("suppliers.csv");
    fs::write(&suppliers_csv, "name,jurisdiction,tier\nSupplier A,DE,1\n")?;
    let ubos_csv = test_dir.join("ubos.csv");
    fs::write(
        &ubos_csv,
        "name,birthdate,citizenship\nOwner A,1990-01-01,DE\n",
    )?;

    // Create policy
    let policy_file = test_dir.join("policy.yml");
    fs::write(
        &policy_file,
        "version: lksg.v1\nname: Registry Test\ncreated_at: 2025-11-20T10:00:00Z\nconstraints:\n  require_at_least_one_ubo: true\n  supplier_count_max: 10\n",
    )?;

    // Run prepare → manifest → proof
    Command::cargo_bin("cap-agent")?
        .current_dir(&test_dir)
        .arg("prepare")
        .arg("--suppliers")
        .arg(&suppliers_csv)
        .arg("--ubos")
        .arg(&ubos_csv)
        .assert()
        .success();

    Command::cargo_bin("cap-agent")?
        .current_dir(&test_dir)
        .arg("manifest")
        .arg("build")
        .arg("--policy")
        .arg(&policy_file)
        .assert()
        .success();

    Command::cargo_bin("cap-agent")?
        .current_dir(&test_dir)
        .arg("proof")
        .arg("build")
        .arg("--manifest")
        .arg(build_dir.join("manifest.json"))
        .arg("--policy")
        .arg(&policy_file)
        .assert()
        .success();

    // Add to registry
    let registry_file = build_dir.join("test_registry.json");
    Command::cargo_bin("cap-agent")?
        .current_dir(&test_dir)
        .arg("registry")
        .arg("add")
        .arg("--manifest")
        .arg(build_dir.join("manifest.json"))
        .arg("--proof")
        .arg(build_dir.join("proof.dat"))
        .arg("--registry")
        .arg(&registry_file)
        .assert()
        .success();

    // Verify registry file exists and has content
    assert!(registry_file.exists(), "Registry file should exist");
    let registry_content = fs::read_to_string(&registry_file)?;
    assert!(
        registry_content.contains("entries"),
        "Registry should contain entries"
    );
    assert!(
        registry_content.contains("manifest_hash"),
        "Registry entry should have manifest_hash"
    );

    // List registry entries
    Command::cargo_bin("cap-agent")?
        .current_dir(&test_dir)
        .arg("registry")
        .arg("list")
        .arg("--registry")
        .arg(&registry_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("0x")); // Should show hash

    println!("✅ CLI Workflow with Registry Test PASSED");

    Ok(())
}

#[test]
fn test_cli_workflow_invalid_policy_fails() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_dir = temp_dir.path();
    let build_dir = test_dir.join("build");
    fs::create_dir_all(&build_dir)?;

    // Create CSV files
    let suppliers_csv = test_dir.join("suppliers.csv");
    fs::write(&suppliers_csv, "name,jurisdiction,tier\nTest,DE,1\n")?;
    let ubos_csv = test_dir.join("ubos.csv");
    fs::write(
        &ubos_csv,
        "name,birthdate,citizenship\nTest,1990-01-01,DE\n",
    )?;

    // Run prepare
    Command::cargo_bin("cap-agent")?
        .current_dir(&test_dir)
        .arg("prepare")
        .arg("--suppliers")
        .arg(&suppliers_csv)
        .arg("--ubos")
        .arg(&ubos_csv)
        .assert()
        .success();

    // Create INVALID policy (missing required fields)
    let invalid_policy = test_dir.join("invalid.yml");
    fs::write(&invalid_policy, "this: is_not_a_valid_policy\n")?;

    // manifest build should FAIL with invalid policy
    Command::cargo_bin("cap-agent")?
        .current_dir(&test_dir)
        .arg("manifest")
        .arg("build")
        .arg("--policy")
        .arg(&invalid_policy)
        .assert()
        .failure(); // Should exit with non-zero code

    println!("✅ Invalid Policy Failure Test PASSED");

    Ok(())
}

#[test]
fn test_meta_json_generation() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_dir = temp_dir.path();
    let build_dir = test_dir.join("build");
    fs::create_dir_all(&build_dir)?;

    // Create CSV files
    let suppliers_csv = test_dir.join("suppliers.csv");
    fs::write(&suppliers_csv, "name,jurisdiction,tier\nSupplier A,DE,1\n")?;
    let ubos_csv = test_dir.join("ubos.csv");
    fs::write(
        &ubos_csv,
        "name,birthdate,citizenship\nOwner A,1990-01-01,DE\n",
    )?;

    // Create policy
    let policy_file = test_dir.join("policy.yml");
    fs::write(
        &policy_file,
        "version: lksg.v1\nname: Meta Test\ncreated_at: 2025-11-20T10:00:00Z\nconstraints:\n  require_at_least_one_ubo: true\n  supplier_count_max: 10\n",
    )?;

    // Run prepare → manifest → proof
    Command::cargo_bin("cap-agent")?
        .current_dir(&test_dir)
        .arg("prepare")
        .arg("--suppliers")
        .arg(&suppliers_csv)
        .arg("--ubos")
        .arg(&ubos_csv)
        .assert()
        .success();

    Command::cargo_bin("cap-agent")?
        .current_dir(&test_dir)
        .arg("manifest")
        .arg("build")
        .arg("--policy")
        .arg(&policy_file)
        .assert()
        .success();

    Command::cargo_bin("cap-agent")?
        .current_dir(&test_dir)
        .arg("proof")
        .arg("build")
        .arg("--manifest")
        .arg(build_dir.join("manifest.json"))
        .arg("--policy")
        .arg(&policy_file)
        .assert()
        .success();

    // Export proof package
    let package_dir = build_dir.join("proof_package");
    Command::cargo_bin("cap-agent")?
        .current_dir(&test_dir)
        .arg("proof")
        .arg("export")
        .arg("--manifest")
        .arg(build_dir.join("manifest.json"))
        .arg("--proof")
        .arg(build_dir.join("proof.dat"))
        .arg("--out")
        .arg(&package_dir)
        .arg("--force")
        .assert()
        .success();

    // Verify _meta.json exists and has correct structure
    let meta_path = package_dir.join("_meta.json");
    assert!(meta_path.exists(), "_meta.json should exist");

    let meta_content = fs::read_to_string(&meta_path)?;
    let meta_json: serde_json::Value = serde_json::from_str(&meta_content)?;

    // Verify schema version
    assert_eq!(
        meta_json["schema"].as_str().unwrap(),
        "cap-bundle.v1",
        "_meta.json should have cap-bundle.v1 schema"
    );

    // Verify bundle_id (UUID v4 format)
    let bundle_id = meta_json["bundle_id"].as_str().unwrap();
    assert!(
        bundle_id.len() == 36 && bundle_id.contains('-'),
        "bundle_id should be valid UUID v4"
    );

    // Verify created_at (RFC3339 format)
    let created_at = meta_json["created_at"].as_str().unwrap();
    eprintln!("DEBUG: created_at value: '{}'", created_at);
    assert!(
        created_at.contains('T') && (created_at.contains('Z') || created_at.contains('+')),
        "created_at should be RFC3339 format, got: {}",
        created_at
    );

    // Verify files section
    let files = meta_json["files"].as_object().unwrap();
    assert!(
        files.contains_key("manifest.json"),
        "files should contain manifest.json"
    );
    assert!(
        files.contains_key("proof.dat"),
        "files should contain proof.dat"
    );

    // Verify file hashes are SHA3-256 (0x + 64 hex chars)
    let manifest_meta = &files["manifest.json"];
    let manifest_hash = manifest_meta["hash"].as_str().unwrap();
    assert!(
        manifest_hash.starts_with("0x") && manifest_hash.len() == 66,
        "Manifest hash should be SHA3-256 (0x + 64 hex chars)"
    );

    // Verify file sizes
    let manifest_size = manifest_meta["size"].as_u64();
    assert!(manifest_size.is_some(), "Manifest should have size field");
    assert!(manifest_size.unwrap() > 0, "Manifest size should be > 0");

    // Verify proof_units section
    let proof_units = meta_json["proof_units"].as_array().unwrap();
    assert!(!proof_units.is_empty(), "proof_units should not be empty");

    let first_unit = &proof_units[0];
    assert_eq!(
        first_unit["id"].as_str().unwrap(),
        "main",
        "First proof unit should have id 'main'"
    );
    assert_eq!(
        first_unit["manifest_file"].as_str().unwrap(),
        "manifest.json"
    );
    assert_eq!(first_unit["proof_file"].as_str().unwrap(), "proof.dat");
    assert_eq!(first_unit["backend"].as_str().unwrap(), "mock");

    println!("✅ _meta.json Generation Test PASSED");

    Ok(())
}

#[test]
fn test_hash_manipulation_detection() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_dir = temp_dir.path();
    let build_dir = test_dir.join("build");
    fs::create_dir_all(&build_dir)?;

    // Create CSV files
    let suppliers_csv = test_dir.join("suppliers.csv");
    fs::write(&suppliers_csv, "name,jurisdiction,tier\nSupplier A,DE,1\n")?;
    let ubos_csv = test_dir.join("ubos.csv");
    fs::write(
        &ubos_csv,
        "name,birthdate,citizenship\nOwner A,1990-01-01,DE\n",
    )?;

    // Create policy
    let policy_file = test_dir.join("policy.yml");
    fs::write(
        &policy_file,
        "version: lksg.v1\nname: Hash Test\ncreated_at: 2025-11-20T10:00:00Z\nconstraints:\n  require_at_least_one_ubo: true\n  supplier_count_max: 10\n",
    )?;

    // Run prepare → manifest → proof → export
    Command::cargo_bin("cap-agent")?
        .current_dir(&test_dir)
        .arg("prepare")
        .arg("--suppliers")
        .arg(&suppliers_csv)
        .arg("--ubos")
        .arg(&ubos_csv)
        .assert()
        .success();

    Command::cargo_bin("cap-agent")?
        .current_dir(&test_dir)
        .arg("manifest")
        .arg("build")
        .arg("--policy")
        .arg(&policy_file)
        .assert()
        .success();

    Command::cargo_bin("cap-agent")?
        .current_dir(&test_dir)
        .arg("proof")
        .arg("build")
        .arg("--manifest")
        .arg(build_dir.join("manifest.json"))
        .arg("--policy")
        .arg(&policy_file)
        .assert()
        .success();

    let package_dir = build_dir.join("proof_package");
    Command::cargo_bin("cap-agent")?
        .current_dir(&test_dir)
        .arg("proof")
        .arg("export")
        .arg("--manifest")
        .arg(build_dir.join("manifest.json"))
        .arg("--proof")
        .arg(build_dir.join("proof.dat"))
        .arg("--out")
        .arg(&package_dir)
        .arg("--force")
        .assert()
        .success();

    // TAMPER: Modify manifest.json after export
    let manifest_path = package_dir.join("manifest.json");
    let mut manifest_content = fs::read_to_string(&manifest_path)?;
    manifest_content.push(' '); // Add whitespace to change hash
    fs::write(&manifest_path, manifest_content)?;

    // Verification should FAIL due to hash mismatch
    let verify_result = Command::cargo_bin("cap-agent")?
        .current_dir(&test_dir)
        .arg("verifier")
        .arg("run")
        .arg("--package")
        .arg(&package_dir)
        .assert()
        .failure(); // Should exit with non-zero code

    // Verify error message mentions hash mismatch
    let output = String::from_utf8_lossy(&verify_result.get_output().stderr);
    assert!(
        output.contains("Hash mismatch") || output.contains("mismatch"),
        "Error should mention hash mismatch"
    );

    println!("✅ Hash Manipulation Detection Test PASSED");

    Ok(())
}

#[test]
fn test_legacy_bundle_backward_compatibility() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_dir = temp_dir.path();
    let build_dir = test_dir.join("build");
    fs::create_dir_all(&build_dir)?;

    // Create CSV files
    let suppliers_csv = test_dir.join("suppliers.csv");
    fs::write(&suppliers_csv, "name,jurisdiction,tier\nSupplier A,DE,1\n")?;
    let ubos_csv = test_dir.join("ubos.csv");
    fs::write(
        &ubos_csv,
        "name,birthdate,citizenship\nOwner A,1990-01-01,DE\n",
    )?;

    // Create policy
    let policy_file = test_dir.join("policy.yml");
    fs::write(
        &policy_file,
        "version: lksg.v1\nname: Legacy Test\ncreated_at: 2025-11-20T10:00:00Z\nconstraints:\n  require_at_least_one_ubo: true\n  supplier_count_max: 10\n",
    )?;

    // Run prepare → manifest → proof → export
    Command::cargo_bin("cap-agent")?
        .current_dir(&test_dir)
        .arg("prepare")
        .arg("--suppliers")
        .arg(&suppliers_csv)
        .arg("--ubos")
        .arg(&ubos_csv)
        .assert()
        .success();

    Command::cargo_bin("cap-agent")?
        .current_dir(&test_dir)
        .arg("manifest")
        .arg("build")
        .arg("--policy")
        .arg(&policy_file)
        .assert()
        .success();

    Command::cargo_bin("cap-agent")?
        .current_dir(&test_dir)
        .arg("proof")
        .arg("build")
        .arg("--manifest")
        .arg(build_dir.join("manifest.json"))
        .arg("--policy")
        .arg(&policy_file)
        .assert()
        .success();

    let package_dir = build_dir.join("proof_package");
    Command::cargo_bin("cap-agent")?
        .current_dir(&test_dir)
        .arg("proof")
        .arg("export")
        .arg("--manifest")
        .arg(build_dir.join("manifest.json"))
        .arg("--proof")
        .arg(build_dir.join("proof.dat"))
        .arg("--out")
        .arg(&package_dir)
        .arg("--force")
        .assert()
        .success();

    // Remove _meta.json to simulate legacy bundle
    let meta_path = package_dir.join("_meta.json");
    fs::remove_file(&meta_path)?;

    // Verification should still SUCCEED (backward compatibility)
    Command::cargo_bin("cap-agent")?
        .current_dir(&test_dir)
        .arg("verifier")
        .arg("run")
        .arg("--package")
        .arg(&package_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Legacy")); // Should detect legacy bundle

    println!("✅ Legacy Bundle Backward Compatibility Test PASSED");

    Ok(())
}

#[test]
fn test_dependency_cycle_detection() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_dir = temp_dir.path();
    let package_dir = test_dir.join("cyclic_bundle");
    fs::create_dir_all(&package_dir)?;

    // Create minimal manifest.json
    let manifest_json = serde_json::json!({
        "version": "manifest.v1.0",
        "created_at": "2025-11-24T10:00:00Z",
        "supplier_root": "0x1234567890123456789012345678901234567890123456789012345678901234",
        "ubo_root": "0x1234567890123456789012345678901234567890123456789012345678901234",
        "company_commitment_root": "0x1234567890123456789012345678901234567890123456789012345678901234",
        "policy": {
            "name": "Cycle Test Policy",
            "version": "lksg.v1",
            "hash": "0x1234567890123456789012345678901234567890123456789012345678901234"
        },
        "audit": {
            "tail_digest": "0x1234567890123456789012345678901234567890123456789012345678901234",
            "events_count": 1
        }
    });

    fs::write(
        package_dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest_json)?,
    )?;

    // Create dummy proof.dat
    fs::write(package_dir.join("proof.dat"), "mock_proof_data")?;

    // Create _meta.json with circular dependency: A → B → C → A
    let meta_json = serde_json::json!({
        "schema": "cap-bundle.v1",
        "bundle_id": "550e8400-e29b-41d4-a716-446655440000",
        "created_at": "2025-11-24T10:00:00Z",
        "files": {
            "manifest.json": {
                "role": "manifest",
                "hash": "0x1234567890123456789012345678901234567890123456789012345678901234",
                "size": 500
            },
            "proof.dat": {
                "role": "proof",
                "hash": "0x1234567890123456789012345678901234567890123456789012345678901234",
                "size": 100
            }
        },
        "proof_units": [
            {
                "id": "A",
                "manifest_file": "manifest.json",
                "proof_file": "proof.dat",
                "policy_id": "test",
                "policy_hash": "0x1234567890123456789012345678901234567890123456789012345678901234",
                "backend": "mock",
                "depends_on": ["B"]
            },
            {
                "id": "B",
                "manifest_file": "manifest.json",
                "proof_file": "proof.dat",
                "policy_id": "test",
                "policy_hash": "0x1234567890123456789012345678901234567890123456789012345678901234",
                "backend": "mock",
                "depends_on": ["C"]
            },
            {
                "id": "C",
                "manifest_file": "manifest.json",
                "proof_file": "proof.dat",
                "policy_id": "test",
                "policy_hash": "0x1234567890123456789012345678901234567890123456789012345678901234",
                "backend": "mock",
                "depends_on": ["A"]  // CYCLE!
            }
        ]
    });

    fs::write(
        package_dir.join("_meta.json"),
        serde_json::to_string_pretty(&meta_json)?,
    )?;

    // Verification should FAIL due to circular dependency
    let verify_result = Command::cargo_bin("cap-agent")?
        .current_dir(&test_dir)
        .arg("verifier")
        .arg("run")
        .arg("--package")
        .arg(&package_dir)
        .assert()
        .failure(); // Should exit with non-zero code

    // Verify error message mentions circular dependency
    let output = String::from_utf8_lossy(&verify_result.get_output().stderr);
    assert!(
        output.contains("Circular") || output.contains("cycle") || output.contains("circular"),
        "Error should mention circular dependency"
    );

    println!("✅ Dependency Cycle Detection Test PASSED");

    Ok(())
}
