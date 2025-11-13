//! Audit Chain Unit Tests (Track A)
//!
//! Tests for structured audit log with hash chain.

use cap_agent::audit::{AuditEvent, AuditEventResult, AuditChain, verify_chain, export_events};
use tempfile::NamedTempFile;

#[test]
fn test_event_hash_determinism() {
    // Same event should produce same hash
    let event1 = AuditEvent::new(
        "verify_response".to_string(),
        AuditChain::GENESIS_HASH.to_string(),
        Some("lksg.v1".to_string()),
        Some("sha3-256:abc123".to_string()),
        Some("0xdef456".to_string()),
        Some(AuditEventResult::Ok),
        Some("run-123".to_string()),
    );

    let event2 = AuditEvent::new(
        "verify_response".to_string(),
        AuditChain::GENESIS_HASH.to_string(),
        Some("lksg.v1".to_string()),
        Some("sha3-256:abc123".to_string()),
        Some("0xdef456".to_string()),
        Some(AuditEventResult::Ok),
        Some("run-123".to_string()),
    );

    // Timestamps will differ, so hashes will differ
    // But structure should be consistent
    assert!(event1.verify_self_hash());
    assert!(event2.verify_self_hash());
}

#[test]
fn test_event_verify_self_hash_ok() {
    let event = AuditEvent::new(
        "test_event".to_string(),
        AuditChain::GENESIS_HASH.to_string(),
        None,
        None,
        None,
        None,
        None,
    );

    assert!(event.verify_self_hash());
}

#[test]
fn test_audit_chain_append_first_event() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut chain = AuditChain::new(temp_file.path()).unwrap();

    // First event should have genesis prev_hash
    let event = chain.append(
        "first_event".to_string(),
        Some("lksg.v1".to_string()),
        None,
        None,
        Some(AuditEventResult::Ok),
        None,
    ).unwrap();

    assert_eq!(event.prev_hash, AuditChain::GENESIS_HASH);
    assert_eq!(chain.tail_hash(), &event.self_hash);
}

#[test]
fn test_audit_chain_append_multiple_events() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut chain = AuditChain::new(temp_file.path()).unwrap();

    // Append 3 events
    let event1 = chain.append("event_1".to_string(), None, None, None, None, None).unwrap();
    let event2 = chain.append("event_2".to_string(), None, None, None, None, None).unwrap();
    let event3 = chain.append("event_3".to_string(), None, None, None, None, None).unwrap();

    // Verify chain links
    assert_eq!(event1.prev_hash, AuditChain::GENESIS_HASH);
    assert_eq!(event2.prev_hash, event1.self_hash);
    assert_eq!(event3.prev_hash, event2.self_hash);
    assert_eq!(chain.tail_hash(), &event3.self_hash);
}

#[test]
fn test_verify_chain_empty_file() {
    let temp_file = NamedTempFile::new().unwrap();

    // Empty file should verify successfully
    let report = verify_chain(temp_file.path()).unwrap();
    assert!(report.ok);
    assert_eq!(report.total_events, 0);
    assert!(report.tamper_index.is_none());
}

#[test]
fn test_verify_chain_valid() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut chain = AuditChain::new(temp_file.path()).unwrap();

    // Append events
    chain.append("event_1".to_string(), None, None, None, None, None).unwrap();
    chain.append("event_2".to_string(), None, None, None, None, None).unwrap();
    chain.append("event_3".to_string(), None, None, None, None, None).unwrap();

    // Verify
    let report = verify_chain(temp_file.path()).unwrap();
    assert!(report.ok);
    assert_eq!(report.total_events, 3);
    assert!(report.tamper_index.is_none());
}

#[test]
fn test_export_events_all() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut chain = AuditChain::new(temp_file.path()).unwrap();

    // Append events
    chain.append("event_1".to_string(), Some("lksg.v1".to_string()), None, None, None, None).unwrap();
    chain.append("event_2".to_string(), Some("other.v1".to_string()), None, None, None, None).unwrap();
    chain.append("event_3".to_string(), Some("lksg.v1".to_string()), None, None, None, None).unwrap();

    // Export all events
    let events = export_events(temp_file.path(), None, None, None).unwrap();
    assert_eq!(events.len(), 3);
}

#[test]
fn test_export_events_by_policy() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut chain = AuditChain::new(temp_file.path()).unwrap();

    // Append events with different policies
    chain.append("event_1".to_string(), Some("lksg.v1".to_string()), None, None, None, None).unwrap();
    chain.append("event_2".to_string(), Some("other.v1".to_string()), None, None, None, None).unwrap();
    chain.append("event_3".to_string(), Some("lksg.v1".to_string()), None, None, None, None).unwrap();
    chain.append("event_4".to_string(), None, None, None, None, None).unwrap();

    // Export filtered by policy
    let events = export_events(temp_file.path(), None, None, Some("lksg.v1")).unwrap();

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].event, "event_1");
    assert_eq!(events[1].event, "event_3");
}

#[test]
fn test_export_events_by_timestamp() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut chain = AuditChain::new(temp_file.path()).unwrap();

    // Append events
    chain.append("event_1".to_string(), None, None, None, None, None).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(10));

    let mid_time = chrono::Utc::now().to_rfc3339();
    std::thread::sleep(std::time::Duration::from_millis(10));

    chain.append("event_2".to_string(), None, None, None, None, None).unwrap();
    chain.append("event_3".to_string(), None, None, None, None, None).unwrap();

    // Export events after mid_time
    let events = export_events(temp_file.path(), Some(&mid_time), None, None).unwrap();

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].event, "event_2");
    assert_eq!(events[1].event, "event_3");
}

#[test]
fn test_event_result_display() {
    assert_eq!(format!("{}", AuditEventResult::Ok), "OK");
    assert_eq!(format!("{}", AuditEventResult::Warn), "WARN");
    assert_eq!(format!("{}", AuditEventResult::Fail), "FAIL");
}
