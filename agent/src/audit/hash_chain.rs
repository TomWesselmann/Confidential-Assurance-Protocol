//! Structured Audit Hash-Chain (Track A)
//!
//! Append-only audit log with cryptographic hash chain, tamper detection, and export.

use anyhow::{anyhow, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write as IoWrite};
use std::path::Path;

/// Audit event result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum AuditEventResult {
    Ok,
    Warn,
    Fail,
}

impl std::fmt::Display for AuditEventResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ok => write!(f, "OK"),
            Self::Warn => write!(f, "WARN"),
            Self::Fail => write!(f, "FAIL"),
        }
    }
}

/// Structured audit event (Track A)
///
/// Contains cryptographic hash chain with tamper detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// ISO 8601 timestamp (UTC, Z-suffix)
    pub ts: String,

    /// Event type (e.g. "verify_response", "policy_compile", "registry_add")
    pub event: String,

    /// Policy ID (e.g. "lksg.v1")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_id: Option<String>,

    /// IR hash (intermediate representation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ir_hash: Option<String>,

    /// Manifest hash
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest_hash: Option<String>,

    /// Result status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<AuditEventResult>,

    /// Run ID (UUID for correlation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,

    /// Previous event hash (hash chain)
    pub prev_hash: String,

    /// Self hash (computed from canonical JSON)
    pub self_hash: String,
}

impl AuditEvent {
    /// Creates a new audit event with hash chain
    ///
    /// # Arguments
    /// * `event` - Event type
    /// * `prev_hash` - Previous event hash (or genesis "0x00...00")
    /// * `policy_id` - Optional policy ID
    /// * `ir_hash` - Optional IR hash
    /// * `manifest_hash` - Optional manifest hash
    /// * `result` - Optional result status
    /// * `run_id` - Optional run ID
    ///
    /// # Returns
    /// New audit event with computed self_hash
    pub fn new(
        event: String,
        prev_hash: String,
        policy_id: Option<String>,
        ir_hash: Option<String>,
        manifest_hash: Option<String>,
        result: Option<AuditEventResult>,
        run_id: Option<String>,
    ) -> Self {
        let ts = Utc::now().to_rfc3339();

        // Compute self_hash from canonical JSON (without self_hash field)
        let mut temp_event = AuditEvent {
            ts: ts.clone(),
            event: event.clone(),
            policy_id: policy_id.clone(),
            ir_hash: ir_hash.clone(),
            manifest_hash: manifest_hash.clone(),
            result: result.clone(),
            run_id: run_id.clone(),
            prev_hash: prev_hash.clone(),
            self_hash: String::new(), // Placeholder
        };

        let self_hash = temp_event.compute_hash();

        temp_event.self_hash = self_hash;
        temp_event
    }

    /// Computes SHA3-256 hash of event (without self_hash field)
    ///
    /// Uses canonical JSON serialization for determinism.
    ///
    /// # Returns
    /// Hex-encoded SHA3-256 hash (0x-prefixed)
    fn compute_hash(&self) -> String {
        use sha3::{Digest, Sha3_256};

        // Canonical JSON: sorted keys, no self_hash
        #[derive(Serialize)]
        struct CanonicalEvent<'a> {
            ts: &'a str,
            event: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            policy_id: &'a Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            ir_hash: &'a Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            manifest_hash: &'a Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            result: &'a Option<AuditEventResult>,
            #[serde(skip_serializing_if = "Option::is_none")]
            run_id: &'a Option<String>,
            prev_hash: &'a str,
        }

        let canonical = CanonicalEvent {
            ts: &self.ts,
            event: &self.event,
            policy_id: &self.policy_id,
            ir_hash: &self.ir_hash,
            manifest_hash: &self.manifest_hash,
            result: &self.result,
            run_id: &self.run_id,
            prev_hash: &self.prev_hash,
        };

        let json = serde_json::to_string(&canonical).expect("Serialization should never fail");

        let mut hasher = Sha3_256::new();
        hasher.update(json.as_bytes());
        let hash_bytes = hasher.finalize();

        format!("0x{}", hex::encode(hash_bytes))
    }

    /// Verifies that self_hash is correctly computed
    ///
    /// # Returns
    /// True if self_hash matches computed hash, false otherwise
    pub fn verify_self_hash(&self) -> bool {
        let computed = self.compute_hash();
        self.self_hash == computed
    }
}

/// Audit chain verification report
#[derive(Debug, Clone)]
pub struct VerifyReport {
    /// Total events verified
    pub total_events: usize,

    /// Verification result
    pub ok: bool,

    /// Index of first tampered event (if any)
    pub tamper_index: Option<usize>,

    /// Error message (if any)
    pub error: Option<String>,
}

impl VerifyReport {
    /// Creates a successful verification report
    pub fn ok(total_events: usize) -> Self {
        Self {
            total_events,
            ok: true,
            tamper_index: None,
            error: None,
        }
    }

    /// Creates a failed verification report
    pub fn fail(total_events: usize, tamper_index: usize, error: String) -> Self {
        Self {
            total_events,
            ok: false,
            tamper_index: Some(tamper_index),
            error: Some(error),
        }
    }
}

/// Audit chain manager (JSONL storage)
pub struct AuditChain {
    path: String,
    last_hash: String,
}

impl AuditChain {
    /// Genesis hash for first event
    pub const GENESIS_HASH: &'static str =
        "0x0000000000000000000000000000000000000000000000000000000000000000";

    /// Creates or opens an audit chain
    ///
    /// # Arguments
    /// * `path` - Path to JSONL file
    ///
    /// # Returns
    /// New AuditChain instance
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_str = path
            .as_ref()
            .to_str()
            .ok_or_else(|| anyhow!("Invalid path"))?
            .to_string();

        let last_hash = if path.as_ref().exists() {
            Self::read_last_hash(&path_str)?
        } else {
            Self::GENESIS_HASH.to_string()
        };

        Ok(Self {
            path: path_str,
            last_hash,
        })
    }

    /// Reads last event hash from chain
    fn read_last_hash(path: &str) -> Result<String> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut last_hash = Self::GENESIS_HASH.to_string();

        for line in reader.lines() {
            let line = line?;
            if !line.trim().is_empty() {
                let event: AuditEvent = serde_json::from_str(&line)?;
                last_hash = event.self_hash.clone();
            }
        }

        Ok(last_hash)
    }

    /// Appends an event to the chain
    ///
    /// # Arguments
    /// * `event` - Event type
    /// * `policy_id` - Optional policy ID
    /// * `ir_hash` - Optional IR hash
    /// * `manifest_hash` - Optional manifest hash
    /// * `result` - Optional result status
    /// * `run_id` - Optional run ID
    ///
    /// # Returns
    /// The appended event
    pub fn append(
        &mut self,
        event: String,
        policy_id: Option<String>,
        ir_hash: Option<String>,
        manifest_hash: Option<String>,
        result: Option<AuditEventResult>,
        run_id: Option<String>,
    ) -> Result<AuditEvent> {
        let audit_event = AuditEvent::new(
            event,
            self.last_hash.clone(),
            policy_id,
            ir_hash,
            manifest_hash,
            result,
            run_id,
        );

        // Write to JSONL
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        let json = serde_json::to_string(&audit_event)?;
        writeln!(file, "{}", json)?;

        // Update last hash
        self.last_hash = audit_event.self_hash.clone();

        Ok(audit_event)
    }

    /// Returns the current tail hash
    pub fn tail_hash(&self) -> &str {
        &self.last_hash
    }
}

/// Verifies the integrity of an audit chain
///
/// # Arguments
/// * `path` - Path to JSONL file
///
/// # Returns
/// VerifyReport with tamper detection
pub fn verify_chain<P: AsRef<Path>>(path: P) -> Result<VerifyReport> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut prev_hash = AuditChain::GENESIS_HASH.to_string();
    let mut index = 0;

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let event: AuditEvent = serde_json::from_str(&line)?;

        // Verify hash chain
        if event.prev_hash != prev_hash {
            return Ok(VerifyReport::fail(
                index + 1,
                index,
                format!(
                    "Hash chain broken: expected prev_hash {}, got {}",
                    prev_hash, event.prev_hash
                ),
            ));
        }

        // Verify self_hash
        if !event.verify_self_hash() {
            return Ok(VerifyReport::fail(
                index + 1,
                index,
                format!("Self-hash mismatch at event {}", index),
            ));
        }

        prev_hash = event.self_hash.clone();
        index += 1;
    }

    Ok(VerifyReport::ok(index))
}

/// Exports events filtered by time range and/or policy
///
/// # Arguments
/// * `path` - Path to JSONL file
/// * `from_ts` - Start timestamp (inclusive, optional)
/// * `to_ts` - End timestamp (inclusive, optional)
/// * `policy_id` - Policy ID filter (optional)
///
/// # Returns
/// Vector of filtered events
pub fn export_events<P: AsRef<Path>>(
    path: P,
    from_ts: Option<&str>,
    to_ts: Option<&str>,
    policy_id: Option<&str>,
) -> Result<Vec<AuditEvent>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut events = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let event: AuditEvent = serde_json::from_str(&line)?;

        // Filter by timestamp
        if let Some(from) = from_ts {
            if event.ts.as_str() < from {
                continue;
            }
        }

        if let Some(to) = to_ts {
            if event.ts.as_str() > to {
                continue;
            }
        }

        // Filter by policy_id
        if let Some(policy) = policy_id {
            if event.policy_id.as_deref() != Some(policy) {
                continue;
            }
        }

        events.push(event);
    }

    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_event_hash_determinism() {
        let event = AuditEvent::new(
            "test_event".to_string(),
            AuditChain::GENESIS_HASH.to_string(),
            Some("lksg.v1".to_string()),
            None,
            None,
            Some(AuditEventResult::Ok),
            None,
        );

        // Hash should be deterministic
        assert!(event.verify_self_hash());
    }

    #[test]
    fn test_audit_chain_append() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut chain = AuditChain::new(temp_file.path()).unwrap();

        // Append first event
        let event1 = chain
            .append(
                "event_1".to_string(),
                Some("lksg.v1".to_string()),
                None,
                None,
                Some(AuditEventResult::Ok),
                None,
            )
            .unwrap();

        assert_eq!(event1.prev_hash, AuditChain::GENESIS_HASH);
        assert_eq!(chain.tail_hash(), &event1.self_hash);

        // Append second event
        let event2 = chain
            .append("event_2".to_string(), None, None, None, None, None)
            .unwrap();

        assert_eq!(event2.prev_hash, event1.self_hash);
        assert_eq!(chain.tail_hash(), &event2.self_hash);
    }

    #[test]
    fn test_verify_chain_ok() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut chain = AuditChain::new(temp_file.path()).unwrap();

        // Append events
        chain
            .append("event_1".to_string(), None, None, None, None, None)
            .unwrap();
        chain
            .append("event_2".to_string(), None, None, None, None, None)
            .unwrap();
        chain
            .append("event_3".to_string(), None, None, None, None, None)
            .unwrap();

        // Verify
        let report = verify_chain(temp_file.path()).unwrap();
        assert!(report.ok);
        assert_eq!(report.total_events, 3);
        assert!(report.tamper_index.is_none());
    }

    #[test]
    fn test_export_events_by_policy() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut chain = AuditChain::new(temp_file.path()).unwrap();

        // Append events with different policies
        chain
            .append(
                "event_1".to_string(),
                Some("lksg.v1".to_string()),
                None,
                None,
                None,
                None,
            )
            .unwrap();
        chain
            .append(
                "event_2".to_string(),
                Some("other.v1".to_string()),
                None,
                None,
                None,
                None,
            )
            .unwrap();
        chain
            .append(
                "event_3".to_string(),
                Some("lksg.v1".to_string()),
                None,
                None,
                None,
                None,
            )
            .unwrap();

        // Export filtered by policy
        let events = export_events(temp_file.path(), None, None, Some("lksg.v1")).unwrap();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event, "event_1");
        assert_eq!(events[1].event, "event_3");
    }
}
