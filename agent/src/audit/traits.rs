//! Audit Store Trait Abstraction
//!
//! Provides a unified interface for both AuditLog (v1) and AuditChain (v2).
//! This allows code to work with either implementation interchangeably.

#![allow(dead_code)]

use anyhow::Result;
use serde_json::Value;
use std::path::Path;

/// Unified audit store trait
///
/// Implementations:
/// - `AuditLog` (v1_0.rs) - Simple hash-chain with JSON details
/// - `AuditChain` (hash_chain.rs) - Structured events with typed fields
pub trait AuditStore: Send + Sync {
    /// Appends a generic event to the audit log
    ///
    /// # Arguments
    /// * `event` - Event type (e.g., "verify_response", "policy_compile")
    /// * `details` - JSON details/metadata for the event
    ///
    /// # Returns
    /// The computed hash of the appended event
    fn append_event(&mut self, event: &str, details: Value) -> Result<String>;

    /// Returns the current tail/tip hash of the chain
    fn tail_hash(&self) -> String;
}

/// Helper for creating audit stores
///
/// # Arguments
/// * `path` - Path to the audit file
/// * `version` - "v1" for AuditLog, "v2" for AuditChain
///
/// # Returns
/// Boxed trait object
pub fn create_audit_store<P: AsRef<Path>>(path: P, version: &str) -> Result<Box<dyn AuditStore>> {
    match version {
        "v1" => {
            let store = super::v1_0::AuditLog::new(path)
                .map_err(|e| anyhow::anyhow!("Failed to create AuditLog: {}", e))?;
            Ok(Box::new(AuditLogAdapter(store)))
        }
        "v2" => {
            let store = super::hash_chain::AuditChain::new(path)?;
            Ok(Box::new(AuditChainAdapter(store)))
        }
        _ => Err(anyhow::anyhow!("Unknown audit version: {}", version)),
    }
}

// ============================================================================
// Adapter implementations
// ============================================================================

/// Adapter for AuditLog (v1)
struct AuditLogAdapter(super::v1_0::AuditLog);

impl AuditStore for AuditLogAdapter {
    fn append_event(&mut self, event: &str, details: Value) -> Result<String> {
        self.0
            .log_event(event, details)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(self.0.get_tip())
    }

    fn tail_hash(&self) -> String {
        self.0.get_tip()
    }
}

/// Adapter for AuditChain (v2)
struct AuditChainAdapter(super::hash_chain::AuditChain);

impl AuditStore for AuditChainAdapter {
    fn append_event(&mut self, event: &str, details: Value) -> Result<String> {
        // Extract optional fields from details for structured logging
        let policy_id = details
            .get("policy_id")
            .and_then(|v| v.as_str())
            .map(String::from);
        let ir_hash = details
            .get("ir_hash")
            .and_then(|v| v.as_str())
            .map(String::from);
        let manifest_hash = details
            .get("manifest_hash")
            .and_then(|v| v.as_str())
            .map(String::from);
        let run_id = details
            .get("run_id")
            .and_then(|v| v.as_str())
            .map(String::from);

        // Parse result if present
        let result = details.get("result").and_then(|v| {
            v.as_str().and_then(|s| match s.to_uppercase().as_str() {
                "OK" => Some(super::hash_chain::AuditEventResult::Ok),
                "WARN" => Some(super::hash_chain::AuditEventResult::Warn),
                "FAIL" => Some(super::hash_chain::AuditEventResult::Fail),
                _ => None,
            })
        });

        let audit_event =
            self.0
                .append(event.to_string(), policy_id, ir_hash, manifest_hash, result, run_id)?;

        Ok(audit_event.self_hash)
    }

    fn tail_hash(&self) -> String {
        self.0.tail_hash().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_v1_adapter() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("audit_v1.jsonl");

        let mut store = create_audit_store(&path, "v1").unwrap();

        // Initial tail hash should be genesis
        let initial_hash = store.tail_hash();
        assert!(initial_hash.starts_with("0x"));

        // Append event
        let hash1 = store
            .append_event("test_event", json!({"foo": "bar"}))
            .unwrap();
        assert!(hash1.starts_with("0x"));
        assert_ne!(hash1, initial_hash);

        // Append another event
        let hash2 = store
            .append_event("another_event", json!({"baz": 123}))
            .unwrap();
        assert!(hash2.starts_with("0x"));
        assert_ne!(hash2, hash1);

        // Verify file was created
        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("test_event"));
        assert!(content.contains("another_event"));
    }

    #[test]
    fn test_v2_adapter() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("audit_v2.jsonl");

        let mut store = create_audit_store(&path, "v2").unwrap();

        // Initial tail hash should be genesis
        let initial_hash = store.tail_hash();
        assert!(initial_hash.starts_with("0x"));

        // Append event with structured fields
        let hash1 = store
            .append_event(
                "verify_response",
                json!({
                    "policy_id": "lksg.v1",
                    "manifest_hash": "0xabc123",
                    "result": "OK",
                    "run_id": "test-run-001"
                }),
            )
            .unwrap();
        assert!(hash1.starts_with("0x"));
        assert_ne!(hash1, initial_hash);

        // Verify file was created with structured content
        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("verify_response"));
        assert!(content.contains("lksg.v1"));
        assert!(content.contains("0xabc123"));
    }

    #[test]
    fn test_unknown_version_error() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("audit.jsonl");

        let result = create_audit_store(&path, "v3");
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.to_string().contains("Unknown audit version"));
    }

    #[test]
    fn test_trait_object_polymorphism() {
        let dir = tempdir().unwrap();

        // Create both versions
        let path_v1 = dir.path().join("audit_v1.jsonl");
        let path_v2 = dir.path().join("audit_v2.jsonl");

        let mut stores: Vec<Box<dyn AuditStore>> = vec![
            create_audit_store(&path_v1, "v1").unwrap(),
            create_audit_store(&path_v2, "v2").unwrap(),
        ];

        // Use them polymorphically
        for store in stores.iter_mut() {
            let hash = store
                .append_event("polymorphic_test", json!({"version": "both"}))
                .unwrap();
            assert!(hash.starts_with("0x"));
        }

        // Both files should exist
        assert!(path_v1.exists());
        assert!(path_v2.exists());
    }
}
