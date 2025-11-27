//! Audit Logger for Taurin Desktop App
//!
//! Logs workflow events to the project's audit log (agent.audit.jsonl).
//! Uses V1.0 format compatible with cap_agent.

use chrono::Utc;
use serde::Serialize;
use sha3::{Digest, Sha3_256};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

/// Genesis hash for first event in chain
const GENESIS_HASH: &str = "0x0000000000000000000000000000000000000000000000000000000000000000";

/// V1.0 Audit Entry
#[derive(Debug, Serialize)]
struct AuditEntry {
    seq: u64,
    ts: String,
    event: String,
    details: serde_json::Value,
    prev_digest: String,
    digest: String,
}

/// Computes SHA3-256 digest for V1.0 format
fn compute_digest(
    seq: u64,
    ts: &str,
    event: &str,
    details: &serde_json::Value,
    prev_digest: &str,
) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(seq.to_string().as_bytes());
    hasher.update(ts.as_bytes());
    hasher.update(event.as_bytes());
    hasher.update(details.to_string().as_bytes());
    hasher.update(prev_digest.as_bytes());

    let result = hasher.finalize();
    format!("0x{}", hex::encode(result))
}

/// Gets the last entry from the audit log
fn get_last_entry(audit_path: &Path) -> Option<(u64, String)> {
    if !audit_path.exists() {
        return None;
    }

    let content = fs::read_to_string(audit_path).ok()?;
    let last_line = content.lines().filter(|l| !l.trim().is_empty()).last()?;

    let entry: serde_json::Value = serde_json::from_str(last_line).ok()?;
    let seq = entry.get("seq")?.as_u64()?;
    let digest = entry.get("digest")?.as_str()?.to_string();

    Some((seq, digest))
}

/// Logs an event to the project's audit log
///
/// # Arguments
/// * `project_path` - Path to the project directory
/// * `event` - Event type (e.g., "csv_imported", "commitments_created")
/// * `details` - Event details as JSON
///
/// # Returns
/// Result with the event digest or error
pub fn log_event(
    project_path: &Path,
    event: &str,
    details: serde_json::Value,
) -> Result<String, String> {
    let audit_path = project_path.join("audit").join("agent.audit.jsonl");

    // Ensure audit directory exists
    if let Some(parent) = audit_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create audit directory: {}", e))?;
    }

    // Get previous entry info
    let (seq, prev_digest) = get_last_entry(&audit_path)
        .map(|(s, d)| (s + 1, d))
        .unwrap_or((1, GENESIS_HASH.to_string()));

    // Create timestamp
    let ts = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true);

    // Compute digest
    let digest = compute_digest(seq, &ts, event, &details, &prev_digest);

    // Create entry
    let entry = AuditEntry {
        seq,
        ts,
        event: event.to_string(),
        details,
        prev_digest,
        digest: digest.clone(),
    };

    // Serialize to JSON
    let json = serde_json::to_string(&entry)
        .map_err(|e| format!("Failed to serialize audit entry: {}", e))?;

    // Append to audit log
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&audit_path)
        .map_err(|e| format!("Failed to open audit log: {}", e))?;

    writeln!(file, "{}", json)
        .map_err(|e| format!("Failed to write audit entry: {}", e))?;

    Ok(digest)
}

/// Convenience macros for logging specific events
pub mod events {
    use super::*;
    use serde_json::json;

    pub fn project_created(project_path: &Path, name: &str) -> Result<String, String> {
        log_event(project_path, "project_created", json!({
            "name": name,
            "schema": "taurin.project.v1"
        }))
    }

    pub fn csv_imported(
        project_path: &Path,
        csv_type: &str,
        record_count: usize,
        hash: &str,
    ) -> Result<String, String> {
        log_event(project_path, "csv_imported", json!({
            "csv_type": csv_type,
            "record_count": record_count,
            "hash": hash
        }))
    }

    pub fn commitments_created(
        project_path: &Path,
        supplier_root: &str,
        ubo_root: &str,
    ) -> Result<String, String> {
        log_event(project_path, "commitments_created", json!({
            "supplier_root": supplier_root,
            "ubo_root": ubo_root
        }))
    }

    pub fn policy_loaded(
        project_path: &Path,
        name: &str,
        version: &str,
        hash: &str,
    ) -> Result<String, String> {
        log_event(project_path, "policy_loaded", json!({
            "name": name,
            "version": version,
            "hash": hash
        }))
    }

    pub fn manifest_built(
        project_path: &Path,
        manifest_hash: &str,
    ) -> Result<String, String> {
        log_event(project_path, "manifest_built", json!({
            "manifest_hash": manifest_hash
        }))
    }

    pub fn proof_built(
        project_path: &Path,
        proof_hash: &str,
        backend: &str,
    ) -> Result<String, String> {
        log_event(project_path, "proof_built", json!({
            "proof_hash": proof_hash,
            "backend": backend
        }))
    }

    pub fn bundle_exported(
        project_path: &Path,
        bundle_path: &str,
        hash: &str,
        size_bytes: u64,
    ) -> Result<String, String> {
        log_event(project_path, "bundle_exported", json!({
            "bundle_path": bundle_path,
            "hash": hash,
            "size_bytes": size_bytes
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_log_event_creates_file() {
        let temp = TempDir::new().unwrap();
        let project = temp.path();
        fs::create_dir_all(project.join("audit")).unwrap();

        let result = log_event(project, "test_event", serde_json::json!({"key": "value"}));

        assert!(result.is_ok());
        assert!(project.join("audit/agent.audit.jsonl").exists());
    }

    #[test]
    fn test_log_event_chain() {
        let temp = TempDir::new().unwrap();
        let project = temp.path();
        fs::create_dir_all(project.join("audit")).unwrap();

        // First event
        let digest1 = log_event(project, "event1", serde_json::json!({})).unwrap();

        // Second event
        let digest2 = log_event(project, "event2", serde_json::json!({})).unwrap();

        // Read and verify chain
        let content = fs::read_to_string(project.join("audit/agent.audit.jsonl")).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        assert_eq!(lines.len(), 2);

        let entry1: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        let entry2: serde_json::Value = serde_json::from_str(lines[1]).unwrap();

        assert_eq!(entry1["seq"], 1);
        assert_eq!(entry2["seq"], 2);
        assert_eq!(entry1["prev_digest"], GENESIS_HASH);
        assert_eq!(entry2["prev_digest"], digest1);
        assert_eq!(entry2["digest"], digest2);
    }
}
