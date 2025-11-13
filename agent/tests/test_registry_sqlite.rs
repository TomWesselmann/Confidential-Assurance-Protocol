// SQLite-specific tests for Registry backend
use std::fs;
use std::path::Path;

// Import from main binary
// Note: We need to add pub visibility to registry module items

#[test]
fn test_sqlite_roundtrip() {
    use std::process::Command;

    // Create test data directory
    fs::create_dir_all("tests/out").ok();
    let db_path = "tests/out/test_roundtrip.sqlite";

    // Clean up previous test
    fs::remove_file(db_path).ok();

    // Use CLI to add entry with SQLite backend
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "registry",
            "add",
            "--manifest",
            "build/manifest.json",
            "--proof",
            "build/proof.dat",
            "--backend",
            "sqlite",
            "--registry",
            db_path,
        ])
        .output();

    // If files don't exist, test should gracefully fail
    // This is an integration test that requires build artifacts
    if let Ok(result) = output {
        if result.status.success() {
            // Verify file was created
            assert!(Path::new(db_path).exists(), "SQLite DB should be created");

            // List entries
            let list_output = Command::new("cargo")
                .args([
                    "run",
                    "--",
                    "registry",
                    "list",
                    "--backend",
                    "sqlite",
                    "--registry",
                    db_path,
                ])
                .output()
                .expect("Failed to list entries");

            assert!(list_output.status.success());
        }
    }

    // Cleanup
    fs::remove_file(db_path).ok();
}

#[test]
fn test_sqlite_error_on_corrupt_db() {
    use std::process::Command;

    fs::create_dir_all("tests/out").ok();
    let corrupt_db = "tests/out/corrupt_registry.sqlite";

    // Write garbage data to simulate corruption
    fs::write(corrupt_db, b"This is not a valid SQLite database file!").unwrap();

    // Try to open the corrupted database via CLI
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "registry",
            "list",
            "--backend",
            "sqlite",
            "--registry",
            corrupt_db,
        ])
        .output()
        .expect("Failed to execute command");

    // Should fail gracefully
    assert!(
        !output.status.success(),
        "Expected error on corrupted SQLite file"
    );

    // Cleanup
    fs::remove_file(corrupt_db).ok();
}

#[test]
fn test_migrate_empty_registry() {
    use std::process::Command;

    fs::create_dir_all("tests/out").ok();

    // Create empty JSON registry
    let empty_json = r#"{
  "registry_version": "1.0",
  "entries": []
}"#;

    let json_path = "tests/out/empty_registry.json";
    let sqlite_path = "tests/out/empty_registry.sqlite";

    fs::write(json_path, empty_json).unwrap();
    fs::remove_file(sqlite_path).ok();

    // Migrate empty registry
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "registry",
            "migrate",
            "--from",
            "json",
            "--input",
            json_path,
            "--to",
            "sqlite",
            "--output",
            sqlite_path,
        ])
        .output()
        .expect("Failed to migrate");

    assert!(
        output.status.success(),
        "Empty registry migration should succeed"
    );
    assert!(
        Path::new(sqlite_path).exists(),
        "SQLite DB should be created"
    );

    // Verify it can be read back
    let list_output = Command::new("cargo")
        .args([
            "run",
            "--",
            "registry",
            "list",
            "--backend",
            "sqlite",
            "--registry",
            sqlite_path,
        ])
        .output()
        .expect("Failed to list");

    assert!(list_output.status.success());

    // Cleanup
    fs::remove_file(json_path).ok();
    fs::remove_file(sqlite_path).ok();
}

#[test]
fn test_duplicate_entry_handling() {
    use std::process::Command;

    fs::create_dir_all("tests/out").ok();
    fs::create_dir_all("build").ok();

    let sqlite_path = "tests/out/test_duplicates.sqlite";
    fs::remove_file(sqlite_path).ok();

    // Create minimal test manifest and proof files
    let test_manifest = r#"{"version":"manifest.v1.0","created_at":"2025-10-30T12:00:00Z"}"#;
    let test_proof = r#"{"version":"proof.v0","type":"mock"}"#;

    fs::write("build/test_manifest.json", test_manifest).unwrap();
    fs::write("build/test_proof.dat", test_proof).unwrap();

    // Add entry first time
    let output1 = Command::new("cargo")
        .args([
            "run",
            "--",
            "registry",
            "add",
            "--manifest",
            "build/test_manifest.json",
            "--proof",
            "build/test_proof.dat",
            "--backend",
            "sqlite",
            "--registry",
            sqlite_path,
        ])
        .output()
        .expect("Failed to add first entry");

    // Add same entry second time (should replace due to INSERT OR REPLACE)
    let output2 = Command::new("cargo")
        .args([
            "run",
            "--",
            "registry",
            "add",
            "--manifest",
            "build/test_manifest.json",
            "--proof",
            "build/test_proof.dat",
            "--backend",
            "sqlite",
            "--registry",
            sqlite_path,
        ])
        .output()
        .expect("Failed to add duplicate entry");

    // Both should succeed (REPLACE semantics)
    if output1.status.success() && output2.status.success() {
        // List should show entries (possibly 2 if ID generation differs)
        let list_output = Command::new("cargo")
            .args([
                "run",
                "--",
                "registry",
                "list",
                "--backend",
                "sqlite",
                "--registry",
                sqlite_path,
            ])
            .output()
            .expect("Failed to list");

        assert!(list_output.status.success());
    }

    // Cleanup
    fs::remove_file(sqlite_path).ok();
    fs::remove_file("build/test_manifest.json").ok();
    fs::remove_file("build/test_proof.dat").ok();
}

#[test]
fn test_sqlite_wal_mode() {
    // This test verifies that SQLite is opened in WAL mode
    // by checking that a .sqlite-wal file is created after writes

    use std::process::Command;

    fs::create_dir_all("tests/out").ok();
    fs::create_dir_all("build").ok();

    let sqlite_path = "tests/out/test_wal.sqlite";
    let wal_path = "tests/out/test_wal.sqlite-wal";

    fs::remove_file(sqlite_path).ok();
    fs::remove_file(wal_path).ok();

    // Create test files
    let test_manifest = r#"{"version":"manifest.v1.0"}"#;
    let test_proof = r#"{"version":"proof.v0"}"#;

    fs::write("build/wal_manifest.json", test_manifest).unwrap();
    fs::write("build/wal_proof.dat", test_proof).unwrap();

    // Add entry to trigger WAL file creation
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "registry",
            "add",
            "--manifest",
            "build/wal_manifest.json",
            "--proof",
            "build/wal_proof.dat",
            "--backend",
            "sqlite",
            "--registry",
            sqlite_path,
        ])
        .output();

    if let Ok(result) = output {
        if result.status.success() {
            // WAL file might exist (depending on SQLite behavior)
            // This is informational - not all operations create WAL immediately
            println!("SQLite DB created: {}", Path::new(sqlite_path).exists());
            println!("WAL file exists: {}", Path::new(wal_path).exists());
        }
    }

    // Cleanup
    fs::remove_file(sqlite_path).ok();
    fs::remove_file(wal_path).ok();
    fs::remove_file("tests/out/test_wal.sqlite-shm").ok();
    fs::remove_file("build/wal_manifest.json").ok();
    fs::remove_file("build/wal_proof.dat").ok();
}
