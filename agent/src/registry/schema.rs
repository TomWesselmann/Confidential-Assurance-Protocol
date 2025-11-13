//! Registry Schema v1.1
//!
//! Defines the v1.1 registry structure with required fields and metadata.

use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Registry Metadata (v1.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryMeta {
    /// Schema version (e.g., "1.1")
    pub schema_version: String,

    /// Tool version that created/migrated this registry
    pub tool_version: String,

    /// Timestamp when created (RFC3339)
    pub created_at: String,

    /// Timestamp when last migrated (RFC3339, optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub migrated_at: Option<String>,

    /// Previous schema version (if migrated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub migrated_from: Option<String>,
}

impl RegistryMeta {
    /// Creates new v1.1 metadata
    #[allow(dead_code)]
    pub fn new_v1_1(tool_version: &str) -> Self {
        Self {
            schema_version: "1.1".to_string(),
            tool_version: tool_version.to_string(),
            created_at: Utc::now().to_rfc3339(),
            migrated_at: None,
            migrated_from: None,
        }
    }

    /// Creates metadata for migrated registry
    pub fn migrated_v1_1(tool_version: &str, from_version: &str) -> Self {
        Self {
            schema_version: "1.1".to_string(),
            tool_version: tool_version.to_string(),
            created_at: Utc::now().to_rfc3339(),
            migrated_at: Some(Utc::now().to_rfc3339()),
            migrated_from: Some(from_version.to_string()),
        }
    }
}

/// Registry Entry v1.1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntryV1_1 {
    // ===== REQUIRED FIELDS (v1.1) =====
    /// Unique entry identifier
    pub entry_id: String,

    /// Entry creation timestamp (RFC3339)
    pub created_at: String,

    /// Policy identifier
    pub policy_id: String,

    /// IR hash (PolicyV2 intermediate representation)
    pub ir_hash: String,

    /// Manifest hash (SHA3-256)
    pub manifest_hash: String,

    // ===== OPTIONAL FIELDS =====
    /// Proof hash (SHA3-256, optional for pending proofs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_hash: Option<String>,

    /// Previous entry hash (for hash-chain, Track A integration)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_hash: Option<String>,

    /// Key Identifier (Week 7 Track S integration)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kid: Option<String>,

    /// Signature over entry (base64-encoded)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,

    /// Legacy: public key (base64-encoded, fallback if kid not available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,

    /// Signature scheme (e.g., "ed25519")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_scheme: Option<String>,

    /// Timestamp file reference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp_file: Option<String>,

    /// BLOB references (v0.9 compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob_manifest: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob_proof: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob_wasm: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob_abi: Option<String>,

    /// Self-verification fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selfverify_status: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub selfverify_at: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub verifier_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub verifier_version: Option<String>,
}

impl RegistryEntryV1_1 {
    /// Creates a new v1.1 entry with required fields
    pub fn new(
        entry_id: String,
        policy_id: String,
        ir_hash: String,
        manifest_hash: String,
    ) -> Self {
        Self {
            entry_id,
            created_at: Utc::now().to_rfc3339(),
            policy_id,
            ir_hash,
            manifest_hash,
            proof_hash: None,
            prev_hash: None,
            kid: None,
            signature: None,
            public_key: None,
            signature_scheme: None,
            timestamp_file: None,
            blob_manifest: None,
            blob_proof: None,
            blob_wasm: None,
            blob_abi: None,
            selfverify_status: None,
            selfverify_at: None,
            verifier_name: None,
            verifier_version: None,
        }
    }

    /// Validates that all required fields are present
    pub fn validate(&self) -> Result<(), String> {
        if self.entry_id.is_empty() {
            return Err("entry_id is required".to_string());
        }
        if self.created_at.is_empty() {
            return Err("created_at is required".to_string());
        }
        if self.policy_id.is_empty() {
            return Err("policy_id is required".to_string());
        }
        if self.ir_hash.is_empty() {
            return Err("ir_hash is required".to_string());
        }
        if self.manifest_hash.is_empty() {
            return Err("manifest_hash is required".to_string());
        }
        Ok(())
    }
}

/// Registry v1.1 Structure
#[derive(Debug, Serialize, Deserialize)]
pub struct RegistryV1_1 {
    /// Registry metadata
    pub meta: RegistryMeta,

    /// Registry entries
    pub entries: Vec<RegistryEntryV1_1>,
}

impl RegistryV1_1 {
    /// Creates a new empty v1.1 registry
    #[allow(dead_code)]
    pub fn new(tool_version: &str) -> Self {
        Self {
            meta: RegistryMeta::new_v1_1(tool_version),
            entries: Vec::new(),
        }
    }

    /// Adds an entry to the registry
    pub fn add_entry(&mut self, entry: RegistryEntryV1_1) -> Result<(), String> {
        entry.validate()?;
        self.entries.push(entry);
        Ok(())
    }

    /// Validates the entire registry
    pub fn validate(&self) -> Result<(), String> {
        if self.meta.schema_version != "1.1" {
            return Err(format!(
                "Invalid schema version: {} (expected 1.1)",
                self.meta.schema_version
            ));
        }

        for (idx, entry) in self.entries.iter().enumerate() {
            entry
                .validate()
                .map_err(|e| format!("Entry {} validation failed: {}", idx, e))?;
        }

        Ok(())
    }

    /// Returns the number of entries
    pub fn count(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_meta_new_v1_1() {
        let meta = RegistryMeta::new_v1_1("cap-agent-0.1.0");

        assert_eq!(meta.schema_version, "1.1");
        assert_eq!(meta.tool_version, "cap-agent-0.1.0");
        assert!(meta.migrated_at.is_none());
        assert!(meta.migrated_from.is_none());
    }

    #[test]
    fn test_registry_meta_migrated() {
        let meta = RegistryMeta::migrated_v1_1("cap-agent-0.1.0", "1.0");

        assert_eq!(meta.schema_version, "1.1");
        assert_eq!(meta.migrated_from.unwrap(), "1.0");
        assert!(meta.migrated_at.is_some());
    }

    #[test]
    fn test_entry_v1_1_new() {
        let entry = RegistryEntryV1_1::new(
            "entry_001".to_string(),
            "lksg.v1".to_string(),
            "sha3-256:abc123".to_string(),
            "0xdef456".to_string(),
        );

        assert_eq!(entry.entry_id, "entry_001");
        assert_eq!(entry.policy_id, "lksg.v1");
        assert_eq!(entry.ir_hash, "sha3-256:abc123");
        assert_eq!(entry.manifest_hash, "0xdef456");
        assert!(entry.validate().is_ok());
    }

    #[test]
    fn test_entry_v1_1_validation_fails() {
        let mut entry = RegistryEntryV1_1::new(
            "".to_string(), // Empty entry_id
            "lksg.v1".to_string(),
            "sha3-256:abc123".to_string(),
            "0xdef456".to_string(),
        );
        entry.entry_id = "".to_string();

        assert!(entry.validate().is_err());
    }

    #[test]
    fn test_registry_v1_1_new() {
        let registry = RegistryV1_1::new("cap-agent-0.1.0");

        assert_eq!(registry.meta.schema_version, "1.1");
        assert_eq!(registry.count(), 0);
        assert!(registry.validate().is_ok());
    }

    #[test]
    fn test_registry_v1_1_add_entry() {
        let mut registry = RegistryV1_1::new("cap-agent-0.1.0");

        let entry = RegistryEntryV1_1::new(
            "entry_001".to_string(),
            "lksg.v1".to_string(),
            "sha3-256:abc123".to_string(),
            "0xdef456".to_string(),
        );

        assert!(registry.add_entry(entry).is_ok());
        assert_eq!(registry.count(), 1);
        assert!(registry.validate().is_ok());
    }

    #[test]
    fn test_registry_v1_1_json_roundtrip() {
        let mut registry = RegistryV1_1::new("cap-agent-0.1.0");

        let entry = RegistryEntryV1_1::new(
            "entry_001".to_string(),
            "lksg.v1".to_string(),
            "sha3-256:abc123".to_string(),
            "0xdef456".to_string(),
        );
        registry.add_entry(entry).unwrap();

        let json = serde_json::to_string_pretty(&registry).unwrap();
        let deserialized: RegistryV1_1 = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.meta.schema_version, "1.1");
        assert_eq!(deserialized.count(), 1);
        assert!(deserialized.validate().is_ok());
    }
}
