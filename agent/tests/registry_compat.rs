//! Compatibility Tests für Registry v1.0 ↔ v1.1
//!
//! Testet die Kompatibilität zwischen v1.0 und v1.1 Registries.

use cap_agent::registry::{
    Registry as RegistryV1_0,
    UnifiedRegistry, RegistryEntryV1_1
};
use tempfile::NamedTempFile;
use std::io::Write;

#[test]
fn test_read_v1_0_write_v1_1() {
    // Create v1.0 JSON
    let v1_0_json = r#"{
        "registry_version": "1.0",
        "entries": [
            {
                "id": "entry_001",
                "manifest_hash": "0xabc",
                "proof_hash": "0xdef",
                "registered_at": "2025-01-01T00:00:00Z",
                "signature": null,
                "public_key": null,
                "kid": null,
                "signature_scheme": null,
                "timestamp_file": null,
                "blob_manifest": null,
                "blob_proof": null,
                "blob_wasm": null,
                "blob_abi": null,
                "selfverify_status": null,
                "selfverify_at": null,
                "verifier_name": null,
                "verifier_version": null
            }
        ]
    }"#;

    let mut temp_v1_0 = NamedTempFile::new().unwrap();
    temp_v1_0.write_all(v1_0_json.as_bytes()).unwrap();
    temp_v1_0.flush().unwrap();

    // Load as UnifiedRegistry (auto-migrates to v1.1)
    let registry = UnifiedRegistry::load(temp_v1_0.path()).unwrap();
    assert_eq!(registry.source_version(), "1.0");

    // Save as v1.1
    let temp_v1_1 = NamedTempFile::new().unwrap();
    registry.save(temp_v1_1.path()).unwrap();

    // Reload and verify it's now v1.1
    let reloaded = UnifiedRegistry::load(temp_v1_1.path()).unwrap();
    assert_eq!(reloaded.source_version(), "1.1");
    assert_eq!(reloaded.count(), 1);
}

#[test]
fn test_v1_1_always_saves_as_v1_1() {
    // Create new v1.1 registry
    let mut registry = UnifiedRegistry::new("test-tool");

    let entry = RegistryEntryV1_1::new(
        "entry_001".to_string(),
        "lksg.v1".to_string(),
        "sha3-256:abc".to_string(),
        "0xdef".to_string(),
    );
    registry.add_entry(entry).unwrap();

    // Save
    let temp_file = NamedTempFile::new().unwrap();
    registry.save(temp_file.path()).unwrap();

    // Reload and verify
    let reloaded = UnifiedRegistry::load(temp_file.path()).unwrap();
    assert_eq!(reloaded.source_version(), "1.1");
    assert!(!reloaded.was_migrated());
}

#[test]
fn test_v1_0_json_format_recognized() {
    // Minimal v1.0 JSON
    let v1_0_json = r#"{"registry_version": "1.0", "entries": []}"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(v1_0_json.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    // Should be recognized as v1.0
    let registry = UnifiedRegistry::load(temp_file.path()).unwrap();
    assert_eq!(registry.source_version(), "1.0");
}

#[test]
fn test_v1_1_json_format_recognized() {
    // Minimal v1.1 JSON
    let v1_1_json = r#"{
        "meta": {
            "schema_version": "1.1",
            "tool_version": "test",
            "created_at": "2025-01-01T00:00:00Z"
        },
        "entries": []
    }"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(v1_1_json.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    // Should be recognized as v1.1
    let registry = UnifiedRegistry::load(temp_file.path()).unwrap();
    assert_eq!(registry.source_version(), "1.1");
}

#[test]
fn test_mixed_operations_v1_0_to_v1_1() {
    // Start with v1.0
    let mut v1_0 = RegistryV1_0::new();
    v1_0.add_entry("0xaaa".to_string(), "0xbbb".to_string(), None);
    v1_0.add_entry("0xccc".to_string(), "0xddd".to_string(), None);

    // Save v1.0
    let temp_v1_0 = NamedTempFile::new().unwrap();
    let v1_0_json = serde_json::to_string(&v1_0).unwrap();
    let mut file = temp_v1_0.reopen().unwrap();
    file.write_all(v1_0_json.as_bytes()).unwrap();
    file.flush().unwrap();

    // Load with UnifiedRegistry (migrates)
    let mut registry = UnifiedRegistry::load(temp_v1_0.path()).unwrap();
    assert_eq!(registry.count(), 2);

    // Add new entry in v1.1 format
    let new_entry = RegistryEntryV1_1::new(
        "entry_003".to_string(),
        "lksg.v1".to_string(),
        "sha3-256:new".to_string(),
        "0xnew_manifest".to_string(),
    );
    registry.add_entry(new_entry).unwrap();

    // Should now have 3 entries
    assert_eq!(registry.count(), 3);

    // Save as v1.1
    let temp_v1_1 = NamedTempFile::new().unwrap();
    registry.save(temp_v1_1.path()).unwrap();

    // Reload and verify all entries
    let final_registry = UnifiedRegistry::load(temp_v1_1.path()).unwrap();
    assert_eq!(final_registry.count(), 3);
    assert_eq!(final_registry.source_version(), "1.1");
}

#[test]
fn test_validation_works_for_both_versions() {
    // Test v1.0 validation through UnifiedRegistry
    let v1_0 = RegistryV1_0::new();
    let temp_v1_0 = NamedTempFile::new().unwrap();
    let json = serde_json::to_string(&v1_0).unwrap();
    let mut file = temp_v1_0.reopen().unwrap();
    file.write_all(json.as_bytes()).unwrap();
    file.flush().unwrap();

    let registry_v1_0 = UnifiedRegistry::load(temp_v1_0.path()).unwrap();
    assert!(registry_v1_0.validate().is_ok());

    // Test v1.1 validation
    let registry_v1_1 = UnifiedRegistry::new("test-tool");
    assert!(registry_v1_1.validate().is_ok());
}

#[test]
fn test_entry_count_preserved_across_migration() {
    // Create v1.0 with specific number of entries
    let mut v1_0 = RegistryV1_0::new();
    for i in 0..5 {
        v1_0.add_entry(
            format!("0xmanifest_{}", i),
            format!("0xproof_{}", i),
            None,
        );
    }

    let temp_file = NamedTempFile::new().unwrap();
    let json = serde_json::to_string(&v1_0).unwrap();
    let mut file = temp_file.reopen().unwrap();
    file.write_all(json.as_bytes()).unwrap();
    file.flush().unwrap();

    // Load and migrate
    let registry = UnifiedRegistry::load(temp_file.path()).unwrap();

    // Count should be preserved
    assert_eq!(registry.count(), 5);
}

#[test]
fn test_v1_1_metadata_absent_in_v1_0() {
    // v1.0 JSON should not have meta field
    let v1_0 = RegistryV1_0::new();
    let json = serde_json::to_string(&v1_0).unwrap();

    // Verify no "meta" field in JSON
    assert!(!json.contains("\"meta\""));
    assert!(json.contains("\"registry_version\""));
}

#[test]
fn test_v1_1_metadata_present_after_save() {
    // Create and save v1.1 registry
    let registry = UnifiedRegistry::new("test-tool");
    let temp_file = NamedTempFile::new().unwrap();
    registry.save(temp_file.path()).unwrap();

    // Read JSON and verify meta field present
    let json = std::fs::read_to_string(temp_file.path()).unwrap();
    assert!(json.contains("\"meta\""));
    assert!(json.contains("\"schema_version\""));
    assert!(json.contains("\"1.1\""));
}

#[test]
fn test_backward_compatibility_fields_preserved() {
    // Create v1.0 entry with all optional fields
    let mut v1_0 = RegistryV1_0::new();
    v1_0.add_entry("0xabc".to_string(), "0xdef".to_string(), Some("ts.tsr".to_string()));

    // Manually set optional fields
    if let Some(entry) = v1_0.entries.last_mut() {
        entry.signature = Some("test_signature".to_string());
        entry.public_key = Some("test_public_key".to_string());
        entry.blob_manifest = Some("blob_manifest_id".to_string());
    }

    let temp_file = NamedTempFile::new().unwrap();
    let json = serde_json::to_string(&v1_0).unwrap();
    let mut file = temp_file.reopen().unwrap();
    file.write_all(json.as_bytes()).unwrap();
    file.flush().unwrap();

    // Load and migrate
    let registry = UnifiedRegistry::load(temp_file.path()).unwrap();

    // Verify backward compatibility fields preserved
    let v1_1 = registry.as_v1_1();
    let entry = &v1_1.entries[0];

    assert_eq!(entry.signature, Some("test_signature".to_string()));
    assert_eq!(entry.public_key, Some("test_public_key".to_string()));
    assert_eq!(entry.blob_manifest, Some("blob_manifest_id".to_string()));
    assert_eq!(entry.timestamp_file, Some("ts.tsr".to_string()));
}

#[test]
fn test_empty_registry_roundtrip() {
    // v1.0 empty
    let v1_0 = RegistryV1_0::new();
    let temp_file = NamedTempFile::new().unwrap();
    let json = serde_json::to_string(&v1_0).unwrap();
    let mut file = temp_file.reopen().unwrap();
    file.write_all(json.as_bytes()).unwrap();
    file.flush().unwrap();

    // Load, migrate, save
    let registry = UnifiedRegistry::load(temp_file.path()).unwrap();
    let output_file = NamedTempFile::new().unwrap();
    registry.save(output_file.path()).unwrap();

    // Reload as v1.1
    let final_registry = UnifiedRegistry::load(output_file.path()).unwrap();
    assert_eq!(final_registry.count(), 0);
    assert_eq!(final_registry.source_version(), "1.1");
}
