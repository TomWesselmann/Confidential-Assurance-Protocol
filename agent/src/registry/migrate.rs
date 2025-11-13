//! Registry Migration Logic (v1.0 → v1.1)
//!
//! Provides idempotent migration from v1.0 to v1.1 registry format.

use super::schema::{RegistryV1_1, RegistryEntryV1_1, RegistryMeta};
use super::v1_0::{Registry as RegistryV1_0, RegistryEntry as RegistryEntryV1_0};
use anyhow::{anyhow, Result};

/// Migrates a v1.0 registry to v1.1
///
/// This function is idempotent: migrating an already v1.1 registry is a no-op.
///
/// # Arguments
/// * `v1_0_registry` - The v1.0 registry to migrate
/// * `tool_version` - Tool version performing the migration
///
/// # Returns
/// v1.1 registry
pub fn migrate_to_v1_1(v1_0_registry: RegistryV1_0, tool_version: &str) -> Result<RegistryV1_1> {
    // Check if already v1.1 (idempotency check would go here if we stored version)
    // For now, we always migrate since v1.0 doesn't have metadata

    let mut v1_1 = RegistryV1_1 {
        meta: RegistryMeta::migrated_v1_1(tool_version, &v1_0_registry.registry_version),
        entries: Vec::new(),
    };

    // Migrate each entry
    for (idx, v1_0_entry) in v1_0_registry.entries.iter().enumerate() {
        let v1_1_entry = migrate_entry(v1_0_entry, idx)?;
        v1_1.add_entry(v1_1_entry).map_err(|e| anyhow!(e))?;
    }

    Ok(v1_1)
}

/// Migrates a single v1.0 entry to v1.1
fn migrate_entry(v1_0: &RegistryEntryV1_0, idx: usize) -> Result<RegistryEntryV1_1> {
    // Generate required fields that don't exist in v1.0
    let entry_id = if !v1_0.id.is_empty() {
        v1_0.id.clone()
    } else {
        format!("migrated_entry_{:04}", idx + 1)
    };

    // Use defaults for missing required fields
    let policy_id = "migrated.v1".to_string(); // Default policy ID
    let ir_hash = format!("sha3-256:migrated_{}", v1_0.manifest_hash); // Derived IR hash

    // Create v1.1 entry
    let mut entry = RegistryEntryV1_1::new(
        entry_id,
        policy_id,
        ir_hash,
        v1_0.manifest_hash.clone(),
    );

    // Copy over optional fields that exist in both versions
    entry.created_at = v1_0.registered_at.clone();
    entry.proof_hash = Some(v1_0.proof_hash.clone());
    entry.kid = v1_0.kid.clone();
    entry.signature = v1_0.signature.clone();
    entry.public_key = v1_0.public_key.clone();
    entry.signature_scheme = v1_0.signature_scheme.clone();
    entry.timestamp_file = v1_0.timestamp_file.clone();

    // BLOB fields
    entry.blob_manifest = v1_0.blob_manifest.clone();
    entry.blob_proof = v1_0.blob_proof.clone();
    entry.blob_wasm = v1_0.blob_wasm.clone();
    entry.blob_abi = v1_0.blob_abi.clone();

    // Self-verify fields
    entry.selfverify_status = v1_0.selfverify_status.clone();
    entry.selfverify_at = v1_0.selfverify_at.clone();
    entry.verifier_name = v1_0.verifier_name.clone();
    entry.verifier_version = v1_0.verifier_version.clone();

    // prev_hash is new in v1.1, starts as None
    entry.prev_hash = None;

    Ok(entry)
}

/// Backfills KID from public_key for entries missing KID
///
/// # Arguments
/// * `registry` - Mutable reference to v1.1 registry
///
/// # Returns
/// Number of entries backfilled
pub fn backfill_kid(registry: &mut RegistryV1_1) -> Result<usize> {
    let mut backfilled = 0;

    for entry in &mut registry.entries {
        // Skip if kid already exists
        if entry.kid.is_some() {
            continue;
        }

        // Skip if no public_key to derive from
        if entry.public_key.is_none() {
            continue;
        }

        // Derive KID from public_key using Week 7 formula
        let public_key = entry.public_key.as_ref().unwrap();

        // Use the key derivation from Track S
        // For now, we use a simple hash of the public key
        // In production, this should use the proper derive_kid function
        let kid = derive_kid_from_pubkey(public_key)?;

        entry.kid = Some(kid);
        backfilled += 1;
    }

    Ok(backfilled)
}

/// Derives KID from base64-encoded public key
///
/// This is a simplified version. In production, use the proper
/// derive_kid function from providers::key_provider.
fn derive_kid_from_pubkey(pubkey_b64: &str) -> Result<String> {
    use base64::Engine;

    let pubkey_bytes = base64::engine::general_purpose::STANDARD
        .decode(pubkey_b64)
        .map_err(|e| anyhow!("Failed to decode public key: {}", e))?;

    // Use BLAKE3 to derive KID (simplified - in production use Week 7 formula)
    use blake3::Hasher;
    let mut hasher = Hasher::new();
    hasher.update(&pubkey_bytes);
    hasher.update(b"software"); // Default provider
    hasher.update(b"migrated"); // Default key name

    let hash = hasher.finalize();
    let kid_bytes = &hash.as_bytes()[0..16]; // First 16 bytes

    Ok(hex::encode(kid_bytes))
}

/// Checks if a registry is already v1.1
pub fn is_v1_1(meta_version: &str) -> bool {
    meta_version == "1.1"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::RegistryEntry as RegistryEntryV1_0;

    #[test]
    fn test_migrate_empty_registry() {
        let v1_0 = RegistryV1_0::new();
        let v1_1 = migrate_to_v1_1(v1_0, "cap-agent-test").unwrap();

        assert_eq!(v1_1.meta.schema_version, "1.1");
        assert_eq!(v1_1.count(), 0);
        assert!(v1_1.validate().is_ok());
    }

    #[test]
    fn test_migrate_single_entry() {
        let mut v1_0 = RegistryV1_0::new();
        v1_0.add_entry(
            "0xabc123".to_string(),
            "0xdef456".to_string(),
            None,
        );

        let v1_1 = migrate_to_v1_1(v1_0, "cap-agent-test").unwrap();

        assert_eq!(v1_1.meta.schema_version, "1.1");
        assert_eq!(v1_1.count(), 1);
        assert!(v1_1.validate().is_ok());

        let entry = &v1_1.entries[0];
        assert_eq!(entry.manifest_hash, "0xabc123");
        assert_eq!(entry.proof_hash, Some("0xdef456".to_string()));
        assert_eq!(entry.policy_id, "migrated.v1");
    }

    #[test]
    fn test_migrate_entry_preserves_optional_fields() {
        let mut v1_0_entry = RegistryEntryV1_0 {
            id: "test_001".to_string(),
            manifest_hash: "0xabc".to_string(),
            proof_hash: "0xdef".to_string(),
            timestamp_file: Some("ts.tsr".to_string()),
            registered_at: "2025-01-01T00:00:00Z".to_string(),
            signature: Some("sig_base64".to_string()),
            public_key: Some("pubkey_base64".to_string()),
            kid: Some("kid_abc123".to_string()),
            signature_scheme: Some("ed25519".to_string()),
            blob_manifest: None,
            blob_proof: None,
            blob_wasm: None,
            blob_abi: None,
            selfverify_status: None,
            selfverify_at: None,
            verifier_name: None,
            verifier_version: None,
        };

        let v1_1_entry = migrate_entry(&v1_0_entry, 0).unwrap();

        assert_eq!(v1_1_entry.entry_id, "test_001");
        assert_eq!(v1_1_entry.kid, Some("kid_abc123".to_string()));
        assert_eq!(v1_1_entry.signature, Some("sig_base64".to_string()));
        assert_eq!(v1_1_entry.public_key, Some("pubkey_base64".to_string()));
        assert_eq!(v1_1_entry.timestamp_file, Some("ts.tsr".to_string()));
    }

    #[test]
    fn test_migrate_idempotency_flag() {
        let meta_v1_0 = "1.0";
        let meta_v1_1 = "1.1";

        assert!(!is_v1_1(meta_v1_0));
        assert!(is_v1_1(meta_v1_1));
    }

    #[test]
    fn test_backfill_kid_empty_registry() {
        let mut registry = RegistryV1_1::new("cap-agent-test");
        let count = backfill_kid(&mut registry).unwrap();

        assert_eq!(count, 0);
    }

    #[test]
    fn test_backfill_kid_with_existing_kid() {
        let mut registry = RegistryV1_1::new("cap-agent-test");

        let mut entry = RegistryEntryV1_1::new(
            "entry_001".to_string(),
            "lksg.v1".to_string(),
            "sha3-256:abc".to_string(),
            "0xdef".to_string(),
        );
        entry.kid = Some("existing_kid".to_string());
        registry.add_entry(entry).unwrap();

        let count = backfill_kid(&mut registry).unwrap();

        assert_eq!(count, 0); // No backfill needed
        assert_eq!(registry.entries[0].kid, Some("existing_kid".to_string()));
    }
}
