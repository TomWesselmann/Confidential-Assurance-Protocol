//! Integration Tests für Registry Migration (v1.0 → v1.1)
//!
//! Testet die End-to-End-Migration von v1.0 zu v1.1 Format.

use cap_agent::registry::{
    Registry as RegistryV1_0, RegistryEntry as RegistryEntryV1_0,
    UnifiedRegistry, migrate_to_v1_1
};
use tempfile::NamedTempFile;
use std::io::Write;

#[test]
fn test_migrate_empty_v1_0_registry() {
    // Create empty v1.0 registry
    let v1_0 = RegistryV1_0::new();

    // Migrate to v1.1
    let v1_1 = migrate_to_v1_1(v1_0, "test-tool").unwrap();

    // Verify migration
    assert_eq!(v1_1.meta.schema_version, "1.1");
    assert_eq!(v1_1.meta.tool_version, "test-tool");
    assert_eq!(v1_1.count(), 0);
    assert!(v1_1.meta.migrated_from.is_some());
    assert_eq!(v1_1.meta.migrated_from.unwrap(), "1.0");
}

#[test]
fn test_migrate_v1_0_with_entries() {
    // Create v1.0 registry with entries
    let mut v1_0 = RegistryV1_0::new();
    v1_0.add_entry(
        "0xabc123".to_string(),
        "0xdef456".to_string(),
        None,
    );
    v1_0.add_entry(
        "0x111222".to_string(),
        "0x333444".to_string(),
        None,
    );

    // Migrate to v1.1
    let v1_1 = migrate_to_v1_1(v1_0, "test-tool").unwrap();

    // Verify migration
    assert_eq!(v1_1.count(), 2);
    assert_eq!(v1_1.meta.schema_version, "1.1");

    // Check first entry
    let entry0 = &v1_1.entries[0];
    assert_eq!(entry0.manifest_hash, "0xabc123");
    assert_eq!(entry0.proof_hash, Some("0xdef456".to_string()));
    assert_eq!(entry0.policy_id, "migrated.v1");
    assert!(entry0.ir_hash.starts_with("sha3-256:migrated_"));

    // Check second entry
    let entry1 = &v1_1.entries[1];
    assert_eq!(entry1.manifest_hash, "0x111222");
    assert_eq!(entry1.proof_hash, Some("0x333444".to_string()));
}

#[test]
fn test_unified_registry_loads_v1_0_json() {
    // Create v1.0 JSON file
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

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(v1_0_json.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    // Load with UnifiedRegistry (should auto-migrate)
    let registry = UnifiedRegistry::load(temp_file.path()).unwrap();

    // Verify
    assert_eq!(registry.source_version(), "1.0");
    assert!(registry.was_migrated());
    assert_eq!(registry.count(), 1);

    // Check migrated entry
    let v1_1 = registry.as_v1_1();
    assert_eq!(v1_1.entries[0].entry_id, "entry_001");
    assert_eq!(v1_1.entries[0].manifest_hash, "0xabc");
}

#[test]
fn test_unified_registry_loads_v1_1_json() {
    // Create v1.1 JSON file
    let v1_1_json = r#"{
        "meta": {
            "schema_version": "1.1",
            "tool_version": "test",
            "created_at": "2025-01-01T00:00:00Z"
        },
        "entries": [
            {
                "entry_id": "entry_001",
                "created_at": "2025-01-01T00:00:00Z",
                "policy_id": "lksg.v1",
                "ir_hash": "sha3-256:abc123",
                "manifest_hash": "0xdef456"
            }
        ]
    }"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(v1_1_json.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    // Load with UnifiedRegistry
    let registry = UnifiedRegistry::load(temp_file.path()).unwrap();

    // Verify
    assert_eq!(registry.source_version(), "1.1");
    assert!(!registry.was_migrated());
    assert_eq!(registry.count(), 1);

    // Check entry
    let v1_1 = registry.as_v1_1();
    assert_eq!(v1_1.entries[0].entry_id, "entry_001");
    assert_eq!(v1_1.entries[0].policy_id, "lksg.v1");
}

#[test]
fn test_migration_preserves_optional_fields() {
    // Create v1.0 entry with optional fields
    let mut v1_0 = RegistryV1_0::new();
    let entry = RegistryEntryV1_0 {
        id: "test_001".to_string(),
        manifest_hash: "0xabc".to_string(),
        proof_hash: "0xdef".to_string(),
        timestamp_file: Some("ts.tsr".to_string()),
        registered_at: "2025-01-01T00:00:00Z".to_string(),
        signature: Some("sig_base64".to_string()),
        public_key: Some("pubkey_base64".to_string()),
        kid: Some("kid_abc123".to_string()),
        signature_scheme: Some("ed25519".to_string()),
        blob_manifest: Some("blob_001".to_string()),
        blob_proof: Some("blob_002".to_string()),
        blob_wasm: Some("blob_003".to_string()),
        blob_abi: Some("blob_004".to_string()),
        selfverify_status: Some("ok".to_string()),
        selfverify_at: Some("2025-01-01T00:05:00Z".to_string()),
        verifier_name: Some("test-verifier".to_string()),
        verifier_version: Some("1.0.0".to_string()),
    };

    v1_0.entries.push(entry);

    // Migrate
    let v1_1 = migrate_to_v1_1(v1_0, "test-tool").unwrap();

    // Verify all optional fields preserved
    let migrated = &v1_1.entries[0];
    assert_eq!(migrated.entry_id, "test_001");
    assert_eq!(migrated.kid, Some("kid_abc123".to_string()));
    assert_eq!(migrated.signature, Some("sig_base64".to_string()));
    assert_eq!(migrated.public_key, Some("pubkey_base64".to_string()));
    assert_eq!(migrated.timestamp_file, Some("ts.tsr".to_string()));
    assert_eq!(migrated.blob_manifest, Some("blob_001".to_string()));
    assert_eq!(migrated.blob_proof, Some("blob_002".to_string()));
    assert_eq!(migrated.blob_wasm, Some("blob_003".to_string()));
    assert_eq!(migrated.blob_abi, Some("blob_004".to_string()));
    assert_eq!(migrated.selfverify_status, Some("ok".to_string()));
}

#[test]
fn test_save_and_reload_preserves_migration_metadata() {
    // Create and migrate v1.0 registry
    let mut v1_0 = RegistryV1_0::new();
    v1_0.add_entry("0xabc".to_string(), "0xdef".to_string(), None);

    let temp_file = NamedTempFile::new().unwrap();

    // Load as v1.0, save as v1.1
    let v1_0_json = serde_json::to_string(&v1_0).unwrap();
    let mut temp_v1_0 = NamedTempFile::new().unwrap();
    temp_v1_0.write_all(v1_0_json.as_bytes()).unwrap();
    temp_v1_0.flush().unwrap();

    let registry = UnifiedRegistry::load(temp_v1_0.path()).unwrap();
    registry.save(temp_file.path()).unwrap();

    // Reload and verify migration metadata preserved
    let reloaded = UnifiedRegistry::load(temp_file.path()).unwrap();
    assert_eq!(reloaded.source_version(), "1.1");
    assert!(reloaded.was_migrated());

    let meta = &reloaded.as_v1_1().meta;
    assert_eq!(meta.migrated_from, Some("1.0".to_string()));
    assert!(meta.migrated_at.is_some());
}

#[test]
fn test_backfill_kid_from_public_key() {
    // Create v1.1 registry with entry that has public_key but no KID
    let mut registry = UnifiedRegistry::new("test-tool");

    let mut entry = cap_agent::registry::RegistryEntryV1_1::new(
        "entry_001".to_string(),
        "lksg.v1".to_string(),
        "sha3-256:abc".to_string(),
        "0xdef".to_string(),
    );

    // Add public_key but no KID
    entry.public_key = Some("dGVzdF9wdWJsaWNfa2V5XzMyX2J5dGVzX2xvbmchIQ==".to_string()); // base64
    entry.kid = None;

    registry.add_entry(entry).unwrap();

    // Backfill KIDs
    let backfilled = registry.backfill_kids().unwrap();

    assert_eq!(backfilled, 1);

    // Verify KID was added
    let entries = &registry.as_v1_1().entries;
    assert!(entries[0].kid.is_some());
    assert_eq!(entries[0].kid.as_ref().unwrap().len(), 32); // 32 hex chars
}

#[test]
fn test_backfill_kid_skips_existing() {
    // Create entry with existing KID
    let mut registry = UnifiedRegistry::new("test-tool");

    let mut entry = cap_agent::registry::RegistryEntryV1_1::new(
        "entry_001".to_string(),
        "lksg.v1".to_string(),
        "sha3-256:abc".to_string(),
        "0xdef".to_string(),
    );

    entry.public_key = Some("dGVzdF9wdWJsaWNfa2V5XzMyX2J5dGVzX2xvbmchIQ==".to_string());
    entry.kid = Some("existing_kid_1234567890abcdef".to_string());

    registry.add_entry(entry).unwrap();

    // Backfill should skip this entry
    let backfilled = registry.backfill_kids().unwrap();

    assert_eq!(backfilled, 0);

    // Verify original KID unchanged
    let entries = &registry.as_v1_1().entries;
    assert_eq!(entries[0].kid.as_ref().unwrap(), "existing_kid_1234567890abcdef");
}

#[test]
fn test_migration_idempotency() {
    // Create two identical v1.0 registries
    let mut v1_0_first = RegistryV1_0::new();
    v1_0_first.add_entry("0xabc".to_string(), "0xdef".to_string(), None);

    let mut v1_0_second = RegistryV1_0::new();
    v1_0_second.add_entry("0xabc".to_string(), "0xdef".to_string(), None);

    // Migrate both
    let v1_1_first = migrate_to_v1_1(v1_0_first, "test-tool").unwrap();
    let v1_1_second = migrate_to_v1_1(v1_0_second, "test-tool").unwrap();

    // Compare structure (timestamps will differ, so compare key fields only)
    assert_eq!(v1_1_first.count(), v1_1_second.count());
    assert_eq!(v1_1_first.entries[0].entry_id, v1_1_second.entries[0].entry_id);
    assert_eq!(v1_1_first.entries[0].manifest_hash, v1_1_second.entries[0].manifest_hash);
    assert_eq!(v1_1_first.entries[0].policy_id, v1_1_second.entries[0].policy_id);
}

#[test]
fn test_validation_after_migration() {
    // Create v1.0 registry
    let mut v1_0 = RegistryV1_0::new();
    v1_0.add_entry("0xabc".to_string(), "0xdef".to_string(), None);

    // Migrate
    let v1_1 = migrate_to_v1_1(v1_0, "test-tool").unwrap();

    // Validate migrated registry
    assert!(v1_1.validate().is_ok());
}

#[test]
fn test_large_registry_migration() {
    // Create v1.0 registry with many entries
    let mut v1_0 = RegistryV1_0::new();

    for i in 0..100 {
        v1_0.add_entry(
            format!("0xmanifest_{:04}", i),
            format!("0xproof_{:04}", i),
            None,
        );
    }

    // Migrate
    let v1_1 = migrate_to_v1_1(v1_0, "test-tool").unwrap();

    // Verify all entries migrated
    assert_eq!(v1_1.count(), 100);
    assert!(v1_1.validate().is_ok());

    // Spot check a few entries
    assert_eq!(v1_1.entries[0].manifest_hash, "0xmanifest_0000");
    assert_eq!(v1_1.entries[50].manifest_hash, "0xmanifest_0050");
    assert_eq!(v1_1.entries[99].manifest_hash, "0xmanifest_0099");
}
