//! Unified Registry API (v1.0 + v1.1 Compatible)
//!
//! Provides a single API for reading v1.0/v1.1 registries and always writing v1.1.

use super::migrate::{backfill_kid, migrate_to_v1_1};
use super::schema::{RegistryEntryV1_1, RegistryV1_1};
use super::v1_0::Registry as RegistryV1_0;
use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::path::Path;

/// Unified Registry API (version-agnostic)
///
/// This wrapper automatically handles v1.0 and v1.1 registries:
/// - Reads both v1.0 and v1.1 formats
/// - Always writes v1.1 format
/// - Transparently migrates v1.0 on load
pub struct UnifiedRegistry {
    inner: RegistryV1_1,
    source_version: String,
}

impl UnifiedRegistry {
    /// Creates a new empty v1.1 registry
    #[allow(dead_code)]
    pub fn new(tool_version: &str) -> Self {
        Self {
            inner: RegistryV1_1::new(tool_version),
            source_version: "1.1".to_string(),
        }
    }

    /// Loads a registry from file (auto-detects v1.0 or v1.1)
    pub fn load(path: &Path) -> Result<Self> {
        let json_str =
            std::fs::read_to_string(path).map_err(|e| anyhow!("Failed to read registry: {}", e))?;

        // Try to detect version
        let version = detect_version(&json_str)?;

        match version.as_str() {
            "1.0" => {
                // Load as v1.0 and migrate
                let v1_0: RegistryV1_0 = serde_json::from_str(&json_str)
                    .map_err(|e| anyhow!("Failed to parse v1.0 registry: {}", e))?;

                let v1_1 = migrate_to_v1_1(v1_0, "cap-agent")?;

                Ok(Self {
                    inner: v1_1,
                    source_version: "1.0".to_string(),
                })
            }
            "1.1" => {
                // Load as v1.1 directly
                let v1_1: RegistryV1_1 = serde_json::from_str(&json_str)
                    .map_err(|e| anyhow!("Failed to parse v1.1 registry: {}", e))?;

                Ok(Self {
                    inner: v1_1,
                    source_version: "1.1".to_string(),
                })
            }
            _ => Err(anyhow!("Unsupported registry version: {}", version)),
        }
    }

    /// Saves registry as v1.1 format
    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.inner)
            .map_err(|e| anyhow!("Failed to serialize registry: {}", e))?;

        std::fs::write(path, json).map_err(|e| anyhow!("Failed to write registry: {}", e))?;

        Ok(())
    }

    /// Adds an entry to the registry
    #[allow(dead_code)]
    pub fn add_entry(&mut self, entry: RegistryEntryV1_1) -> Result<()> {
        self.inner.add_entry(entry).map_err(|e| anyhow!(e))
    }

    /// Returns the source version (1.0 or 1.1)
    pub fn source_version(&self) -> &str {
        &self.source_version
    }

    /// Returns true if registry was migrated from v1.0
    pub fn was_migrated(&self) -> bool {
        self.inner.meta.migrated_from.is_some()
    }

    /// Returns the number of entries
    pub fn count(&self) -> usize {
        self.inner.count()
    }

    /// Validates the registry
    pub fn validate(&self) -> Result<()> {
        self.inner.validate().map_err(|e| anyhow!(e))
    }

    /// Returns a reference to the underlying v1.1 registry
    pub fn as_v1_1(&self) -> &RegistryV1_1 {
        &self.inner
    }

    /// Returns a mutable reference to the underlying v1.1 registry
    #[allow(dead_code)]
    pub fn as_v1_1_mut(&mut self) -> &mut RegistryV1_1 {
        &mut self.inner
    }

    /// Backfills KIDs from public_key fields
    pub fn backfill_kids(&mut self) -> Result<usize> {
        backfill_kid(&mut self.inner)
    }
}

/// Detects registry version from JSON string
fn detect_version(json_str: &str) -> Result<String> {
    #[derive(Deserialize)]
    struct VersionDetector {
        #[serde(default)]
        registry_version: Option<String>,
        #[serde(default)]
        meta: Option<MetaVersion>,
    }

    #[derive(Deserialize)]
    struct MetaVersion {
        schema_version: String,
    }

    let detector: VersionDetector =
        serde_json::from_str(json_str).map_err(|e| anyhow!("Failed to detect version: {}", e))?;

    // Check for v1.1 metadata
    if let Some(meta) = detector.meta {
        return Ok(meta.schema_version);
    }

    // Check for v1.0 version field
    if let Some(version) = detector.registry_version {
        return Ok(version);
    }

    // Default to v1.0 if no version field found
    Ok("1.0".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_new_registry_is_v1_1() {
        let reg = UnifiedRegistry::new("test");
        assert_eq!(reg.source_version(), "1.1");
        assert!(!reg.was_migrated());
        assert_eq!(reg.count(), 0);
    }

    #[test]
    fn test_load_v1_0_registry() {
        let v1_0_json = r#"{
            "registry_version": "1.0",
            "entries": []
        }"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(v1_0_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let reg = UnifiedRegistry::load(temp_file.path()).unwrap();

        assert_eq!(reg.source_version(), "1.0");
        assert!(reg.was_migrated());
        assert_eq!(reg.count(), 0);
    }

    #[test]
    fn test_load_v1_1_registry() {
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

        let reg = UnifiedRegistry::load(temp_file.path()).unwrap();

        assert_eq!(reg.source_version(), "1.1");
        assert!(!reg.was_migrated());
        assert_eq!(reg.count(), 0);
    }

    #[test]
    fn test_save_always_v1_1() {
        let mut reg = UnifiedRegistry::new("test");

        // Add an entry
        let entry = RegistryEntryV1_1::new(
            "entry_001".to_string(),
            "test.v1".to_string(),
            "sha3-256:abc".to_string(),
            "0xdef".to_string(),
        );
        reg.add_entry(entry).unwrap();

        // Save and reload
        let temp_file = NamedTempFile::new().unwrap();
        reg.save(temp_file.path()).unwrap();

        let reloaded = UnifiedRegistry::load(temp_file.path()).unwrap();
        assert_eq!(reloaded.source_version(), "1.1");
        assert_eq!(reloaded.count(), 1);
    }

    #[test]
    fn test_roundtrip_v1_0_to_v1_1() {
        // Create v1.0 registry
        let v1_0_json = r#"{
            "registry_version": "1.0",
            "entries": [{
                "id": "test_001",
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
            }]
        }"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(v1_0_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        // Load v1.0, save as v1.1
        let reg = UnifiedRegistry::load(temp_file.path()).unwrap();

        let output_file = NamedTempFile::new().unwrap();
        reg.save(output_file.path()).unwrap();

        // Reload and verify
        let reloaded = UnifiedRegistry::load(output_file.path()).unwrap();
        assert_eq!(reloaded.source_version(), "1.1");
        assert_eq!(reloaded.count(), 1);
        assert!(reloaded.was_migrated()); // Migration metadata is preserved
    }
}
