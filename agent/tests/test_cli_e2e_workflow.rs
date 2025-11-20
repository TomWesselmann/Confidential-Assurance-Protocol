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
        .stdout(predicate::str::contains("Verifikation erfolgreich")); // Should show verification success

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
    fs::write(&ubos_csv, "name,birthdate,citizenship\nOwner A,1990-01-01,DE\n")?;

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
    fs::write(&ubos_csv, "name,birthdate,citizenship\nTest,1990-01-01,DE\n")?;

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
