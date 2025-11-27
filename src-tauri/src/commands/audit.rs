//! Audit log commands
//!
//! Commands for reading and verifying audit logs.

use crate::security::{sanitize_error_message, validate_path_exists};
use crate::types::{AuditEvent, AuditLog, ChainError, ChainVerifyResult};
use sha3::{Digest, Sha3_256};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Genesis hash (first event has this as prev_hash)
const GENESIS_HASH: &str = "0x0000000000000000000000000000000000000000000000000000000000000000";

/// Maximum events to return (prevent memory exhaustion)
const MAX_EVENTS: usize = 10_000;

/// Default limit for pagination
const DEFAULT_LIMIT: usize = 100;

/// Raw V1.0 audit entry (for deserialization)
#[derive(Debug, serde::Deserialize)]
struct RawAuditEntryV1 {
    seq: u64,
    ts: String,
    event: String,
    details: serde_json::Value,
    prev_digest: String,
    digest: String,
}

/// Raw V2.0 audit entry (for deserialization)
#[derive(Debug, serde::Deserialize)]
struct RawAuditEntryV2 {
    ts: String,
    event: String,
    #[serde(default)]
    policy_id: Option<String>,
    #[serde(default)]
    ir_hash: Option<String>,
    #[serde(default)]
    manifest_hash: Option<String>,
    #[serde(default)]
    result: Option<crate::types::AuditEventResult>,
    #[serde(default)]
    run_id: Option<String>,
    prev_hash: String,
    self_hash: String,
}

/// Parses a JSONL line into a unified AuditEvent
fn parse_audit_line(line: &str) -> Result<AuditEvent, String> {
    // Try V1.0 format first (has 'seq' and 'digest' fields)
    if let Ok(v1) = serde_json::from_str::<RawAuditEntryV1>(line) {
        return Ok(AuditEvent {
            seq: Some(v1.seq),
            ts: v1.ts,
            event: v1.event,
            details: Some(v1.details),
            policy_id: None,
            ir_hash: None,
            manifest_hash: None,
            result: None,
            run_id: None,
            prev_hash: v1.prev_digest,
            self_hash: v1.digest,
        });
    }

    // Try V2.0 format (has 'self_hash' and 'prev_hash' fields)
    if let Ok(v2) = serde_json::from_str::<RawAuditEntryV2>(line) {
        return Ok(AuditEvent {
            seq: None,
            ts: v2.ts,
            event: v2.event,
            details: None,
            policy_id: v2.policy_id,
            ir_hash: v2.ir_hash,
            manifest_hash: v2.manifest_hash,
            result: v2.result,
            run_id: v2.run_id,
            prev_hash: v2.prev_hash,
            self_hash: v2.self_hash,
        });
    }

    Err(format!("Failed to parse audit line: {}", &line[..line.len().min(100)]))
}

/// Computes SHA3-256 hash for V1.0 format
fn compute_hash_v1(
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

/// Computes SHA3-256 hash for V2.0 format (canonical JSON)
fn compute_hash_v2(event: &AuditEvent) -> String {
    use serde::Serialize;

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
        result: &'a Option<crate::types::AuditEventResult>,
        #[serde(skip_serializing_if = "Option::is_none")]
        run_id: &'a Option<String>,
        prev_hash: &'a str,
    }

    let canonical = CanonicalEvent {
        ts: &event.ts,
        event: &event.event,
        policy_id: &event.policy_id,
        ir_hash: &event.ir_hash,
        manifest_hash: &event.manifest_hash,
        result: &event.result,
        run_id: &event.run_id,
        prev_hash: &event.prev_hash,
    };

    let json = serde_json::to_string(&canonical).expect("Serialization should never fail");

    let mut hasher = Sha3_256::new();
    hasher.update(json.as_bytes());
    let hash_bytes = hasher.finalize();

    format!("0x{}", hex::encode(hash_bytes))
}

/// Verifies a single event's self hash
fn verify_event_hash(event: &AuditEvent) -> bool {
    if event.seq.is_some() {
        // V1.0 format
        let details = event.details.as_ref().unwrap_or(&serde_json::Value::Null);
        let computed = compute_hash_v1(
            event.seq.unwrap(),
            &event.ts,
            &event.event,
            details,
            &event.prev_hash,
        );
        computed == event.self_hash
    } else {
        // V2.0 format
        let computed = compute_hash_v2(event);
        computed == event.self_hash
    }
}

/// Gets the audit log path for a project
fn get_audit_path(project_path: &Path) -> std::path::PathBuf {
    project_path.join("audit").join("agent.audit.jsonl")
}

/// Reads and parses the audit log file
fn read_audit_events(audit_path: &Path) -> Result<Vec<AuditEvent>, String> {
    if !audit_path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(audit_path)
        .map_err(|e| sanitize_error_message(&format!("Failed to open audit log: {}", e)))?;

    let reader = BufReader::new(file);
    let mut events = Vec::new();

    for (line_num, line_result) in reader.lines().enumerate() {
        if events.len() >= MAX_EVENTS {
            break;
        }

        let line = line_result
            .map_err(|e| sanitize_error_message(&format!("Failed to read line {}: {}", line_num + 1, e)))?;

        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        match parse_audit_line(&line) {
            Ok(event) => events.push(event),
            Err(e) => {
                // Log warning but continue (partial read is better than no read)
                eprintln!("Warning: Skipping malformed audit line {}: {}", line_num + 1, e);
            }
        }
    }

    Ok(events)
}

/// Gets audit log for a project
///
/// # Arguments
/// * `project` - Path to the project directory
/// * `limit` - Maximum number of events to return (default: 100)
/// * `offset` - Number of events to skip (default: 0)
///
/// # Returns
/// AuditLog with events and chain status
#[tauri::command]
pub async fn get_audit_log(
    project: String,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<AuditLog, String> {
    let project_path = Path::new(&project);
    validate_path_exists(project_path)?;

    let audit_path = get_audit_path(project_path);
    let all_events = read_audit_events(&audit_path)?;

    let total_count = all_events.len();
    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_EVENTS);

    // Apply pagination
    let events: Vec<AuditEvent> = all_events
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    // Quick chain validation (just check if all hashes match)
    let chain_valid = verify_chain_internal(&audit_path).map(|r| r.valid).unwrap_or(false);

    Ok(AuditLog {
        events,
        total_count,
        chain_valid,
        offset,
        limit,
    })
}

/// Internal chain verification
fn verify_chain_internal(audit_path: &Path) -> Result<ChainVerifyResult, String> {
    if !audit_path.exists() {
        return Ok(ChainVerifyResult {
            valid: true,
            verified_count: 0,
            errors: Vec::new(),
            tail_hash: None,
        });
    }

    let events = read_audit_events(audit_path)?;

    if events.is_empty() {
        return Ok(ChainVerifyResult {
            valid: true,
            verified_count: 0,
            errors: Vec::new(),
            tail_hash: None,
        });
    }

    let mut errors = Vec::new();
    let mut prev_hash = GENESIS_HASH.to_string();

    for (idx, event) in events.iter().enumerate() {
        // Check 1: prev_hash matches previous event's self_hash
        if event.prev_hash != prev_hash {
            errors.push(ChainError {
                index: idx,
                timestamp: event.ts.clone(),
                error_type: "chain_break".to_string(),
                expected: prev_hash.clone(),
                found: event.prev_hash.clone(),
            });
        }

        // Check 2: self_hash is correctly computed
        if !verify_event_hash(event) {
            let expected = if event.seq.is_some() {
                let details = event.details.as_ref().unwrap_or(&serde_json::Value::Null);
                compute_hash_v1(event.seq.unwrap(), &event.ts, &event.event, details, &event.prev_hash)
            } else {
                compute_hash_v2(event)
            };

            errors.push(ChainError {
                index: idx,
                timestamp: event.ts.clone(),
                error_type: "hash_mismatch".to_string(),
                expected,
                found: event.self_hash.clone(),
            });
        }

        prev_hash = event.self_hash.clone();
    }

    let tail_hash = events.last().map(|e| e.self_hash.clone());

    Ok(ChainVerifyResult {
        valid: errors.is_empty(),
        verified_count: events.len(),
        errors,
        tail_hash,
    })
}

/// Verifies the audit chain integrity
///
/// # Arguments
/// * `project` - Path to the project directory
///
/// # Returns
/// ChainVerifyResult with validation status and errors
#[tauri::command]
pub async fn verify_audit_chain(project: String) -> Result<ChainVerifyResult, String> {
    let project_path = Path::new(&project);
    validate_path_exists(project_path)?;

    let audit_path = get_audit_path(project_path);
    verify_chain_internal(&audit_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_project() -> TempDir {
        let temp = TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join("audit")).unwrap();
        temp
    }

    fn write_audit_log(temp: &TempDir, content: &str) {
        let audit_path = temp.path().join("audit").join("agent.audit.jsonl");
        let mut file = File::create(audit_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    #[tokio::test]
    async fn test_get_audit_log_empty() {
        let temp = create_test_project();
        let project = temp.path().to_string_lossy().to_string();

        let result = get_audit_log(project, None, None).await;
        assert!(result.is_ok());

        let log = result.unwrap();
        assert_eq!(log.events.len(), 0);
        assert_eq!(log.total_count, 0);
        assert!(log.chain_valid);
    }

    #[tokio::test]
    async fn test_get_audit_log_v1_format() {
        let temp = create_test_project();

        // V1.0 format event
        let event = r#"{"seq":1,"ts":"2025-11-24T13:49:14.487148+00:00","event":"test_event","details":{"key":"value"},"prev_digest":"0x0000000000000000000000000000000000000000000000000000000000000000","digest":"0xabc123"}"#;
        write_audit_log(&temp, event);

        let project = temp.path().to_string_lossy().to_string();
        let result = get_audit_log(project, None, None).await;

        assert!(result.is_ok());
        let log = result.unwrap();
        assert_eq!(log.events.len(), 1);
        assert_eq!(log.events[0].event, "test_event");
        assert_eq!(log.events[0].seq, Some(1));
    }

    #[tokio::test]
    async fn test_get_audit_log_pagination() {
        let temp = create_test_project();

        // Create 5 events
        let mut events = Vec::new();
        for i in 1..=5 {
            events.push(format!(
                r#"{{"seq":{},"ts":"2025-11-24T13:49:{:02}.000000+00:00","event":"event_{}","details":{{}},"prev_digest":"0x0000","digest":"0x{:04x}"}}"#,
                i, i, i, i
            ));
        }
        write_audit_log(&temp, &events.join("\n"));

        let project = temp.path().to_string_lossy().to_string();

        // Get with limit=2, offset=1
        let result = get_audit_log(project, Some(2), Some(1)).await;
        assert!(result.is_ok());

        let log = result.unwrap();
        assert_eq!(log.events.len(), 2);
        assert_eq!(log.total_count, 5);
        assert_eq!(log.offset, 1);
        assert_eq!(log.limit, 2);
        assert_eq!(log.events[0].event, "event_2");
        assert_eq!(log.events[1].event, "event_3");
    }

    #[tokio::test]
    async fn test_verify_chain_empty() {
        let temp = create_test_project();
        let project = temp.path().to_string_lossy().to_string();

        let result = verify_audit_chain(project).await;
        assert!(result.is_ok());

        let verify = result.unwrap();
        assert!(verify.valid);
        assert_eq!(verify.verified_count, 0);
        assert!(verify.errors.is_empty());
    }

    #[tokio::test]
    async fn test_verify_chain_valid_v1() {
        let temp = create_test_project();

        // Create a valid V1.0 chain
        // First event: prev_digest = genesis, compute digest
        let seq1 = 1u64;
        let ts1 = "2025-11-24T13:49:14.000000+00:00";
        let event1 = "test_event";
        let details1 = serde_json::json!({"key": "value"});
        let prev1 = GENESIS_HASH;
        let digest1 = compute_hash_v1(seq1, ts1, event1, &details1, prev1);

        let event1_json = format!(
            r#"{{"seq":{},"ts":"{}","event":"{}","details":{},"prev_digest":"{}","digest":"{}"}}"#,
            seq1, ts1, event1, details1, prev1, digest1
        );

        write_audit_log(&temp, &event1_json);

        let project = temp.path().to_string_lossy().to_string();
        let result = verify_audit_chain(project).await;

        assert!(result.is_ok());
        let verify = result.unwrap();
        assert!(verify.valid, "Chain should be valid: {:?}", verify.errors);
        assert_eq!(verify.verified_count, 1);
        assert!(verify.errors.is_empty());
        assert_eq!(verify.tail_hash, Some(digest1));
    }

    #[tokio::test]
    async fn test_verify_chain_tampered() {
        let temp = create_test_project();

        // Create an event with wrong digest
        let event = format!(
            r#"{{"seq":1,"ts":"2025-11-24T13:49:14.000000+00:00","event":"test","details":{{}},"prev_digest":"{}","digest":"0xwrongdigest"}}"#,
            GENESIS_HASH
        );
        write_audit_log(&temp, &event);

        let project = temp.path().to_string_lossy().to_string();
        let result = verify_audit_chain(project).await;

        assert!(result.is_ok());
        let verify = result.unwrap();
        assert!(!verify.valid);
        assert_eq!(verify.errors.len(), 1);
        assert_eq!(verify.errors[0].error_type, "hash_mismatch");
    }

    #[tokio::test]
    async fn test_verify_chain_broken_link() {
        let temp = create_test_project();

        // Create two events with broken chain (second event has wrong prev_digest)
        let seq1 = 1u64;
        let ts1 = "2025-11-24T13:49:14.000000+00:00";
        let details1 = serde_json::json!({});
        let digest1 = compute_hash_v1(seq1, ts1, "event1", &details1, GENESIS_HASH);

        let event1 = format!(
            r#"{{"seq":{},"ts":"{}","event":"event1","details":{},"prev_digest":"{}","digest":"{}"}}"#,
            seq1, ts1, details1, GENESIS_HASH, digest1
        );

        // Second event points to wrong prev_digest
        let seq2 = 2u64;
        let ts2 = "2025-11-24T13:49:15.000000+00:00";
        let details2 = serde_json::json!({});
        let wrong_prev = "0xwrongprevhash";
        let digest2 = compute_hash_v1(seq2, ts2, "event2", &details2, wrong_prev);

        let event2 = format!(
            r#"{{"seq":{},"ts":"{}","event":"event2","details":{},"prev_digest":"{}","digest":"{}"}}"#,
            seq2, ts2, details2, wrong_prev, digest2
        );

        write_audit_log(&temp, &format!("{}\n{}", event1, event2));

        let project = temp.path().to_string_lossy().to_string();
        let result = verify_audit_chain(project).await;

        assert!(result.is_ok());
        let verify = result.unwrap();
        assert!(!verify.valid);
        assert!(verify.errors.iter().any(|e| e.error_type == "chain_break"));
    }

    #[tokio::test]
    async fn test_get_audit_log_not_found() {
        let temp = TempDir::new().unwrap();
        // Don't create audit directory
        let project = temp.path().to_string_lossy().to_string();

        let result = get_audit_log(project, None, None).await;
        // Should return empty log, not error
        assert!(result.is_ok());
        let log = result.unwrap();
        assert_eq!(log.events.len(), 0);
    }
}
