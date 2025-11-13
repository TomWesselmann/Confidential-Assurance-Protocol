//! Audit Chain Tamper Detection Tests (Track A)
//!
//! Tests for tamper detection and chain integrity verification.

use cap_agent::audit::{AuditChain, AuditEvent, verify_chain};
use tempfile::NamedTempFile;
use std::io::Write;
use std::fs::OpenOptions;

#[test]
fn test_tamper_modify_event_detected() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut chain = AuditChain::new(temp_file.path()).unwrap();

    // Append events
    chain.append("event_1".to_string(), None, None, None, None, None).unwrap();
    chain.append("event_2".to_string(), None, None, None, None, None).unwrap();
    chain.append("event_3".to_string(), None, None, None, None, None).unwrap();

    // Read all events
    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    let lines: Vec<&str> = content.lines().collect();

    // Tamper: modify second event
    let mut event2: AuditEvent = serde_json::from_str(lines[1]).unwrap();
    event2.event = "tampered_event".to_string(); // Modify content

    // Rewrite file with tampered event
    std::fs::write(temp_file.path(), "").unwrap(); // Clear file
    let mut file = OpenOptions::new().create(true).write(true).open(temp_file.path()).unwrap();

    writeln!(file, "{}", lines[0]).unwrap(); // Original event 1
    writeln!(file, "{}", serde_json::to_string(&event2).unwrap()).unwrap(); // Tampered event 2
    writeln!(file, "{}", lines[2]).unwrap(); // Original event 3

    // Verify should detect tamper
    let report = verify_chain(temp_file.path()).unwrap();

    assert!(!report.ok);
    assert_eq!(report.tamper_index, Some(1)); // Second event (index 1)
    assert!(report.error.is_some());
}

#[test]
fn test_tamper_break_chain_detected() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut chain = AuditChain::new(temp_file.path()).unwrap();

    // Append events
    chain.append("event_1".to_string(), None, None, None, None, None).unwrap();
    chain.append("event_2".to_string(), None, None, None, None, None).unwrap();
    chain.append("event_3".to_string(), None, None, None, None, None).unwrap();

    // Read all events
    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    let lines: Vec<&str> = content.lines().collect();

    // Tamper: break chain by modifying prev_hash of event 3
    let mut event3: AuditEvent = serde_json::from_str(lines[2]).unwrap();
    event3.prev_hash = "0xWRONG_HASH_BREAKS_CHAIN".to_string();

    // Recompute self_hash for tampered event (to pass self-hash check)
    let tampered_json = serde_json::to_string(&event3).unwrap();

    // Rewrite file
    std::fs::write(temp_file.path(), "").unwrap();
    let mut file = OpenOptions::new().create(true).write(true).open(temp_file.path()).unwrap();

    writeln!(file, "{}", lines[0]).unwrap();
    writeln!(file, "{}", lines[1]).unwrap();
    writeln!(file, "{}", tampered_json).unwrap();

    // Verify should detect broken chain
    let report = verify_chain(temp_file.path()).unwrap();

    assert!(!report.ok);
    assert_eq!(report.tamper_index, Some(2)); // Third event (index 2)
    assert!(report.error.unwrap().contains("Hash chain broken"));
}

#[test]
fn test_tamper_reorder_events_detected() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut chain = AuditChain::new(temp_file.path()).unwrap();

    // Append events with distinct content
    chain.append("event_A".to_string(), None, None, None, None, None).unwrap();
    chain.append("event_B".to_string(), None, None, None, None, None).unwrap();
    chain.append("event_C".to_string(), None, None, None, None, None).unwrap();

    // Read all events
    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    let lines: Vec<&str> = content.lines().collect();

    // Tamper: reorder events (swap event_B and event_C)
    std::fs::write(temp_file.path(), "").unwrap();
    let mut file = OpenOptions::new().create(true).write(true).open(temp_file.path()).unwrap();

    writeln!(file, "{}", lines[0]).unwrap(); // event_A
    writeln!(file, "{}", lines[2]).unwrap(); // event_C (was 3rd)
    writeln!(file, "{}", lines[1]).unwrap(); // event_B (was 2nd)

    // Verify should detect broken chain
    let report = verify_chain(temp_file.path()).unwrap();

    assert!(!report.ok);
    // Chain will break at the reordered event
    assert!(report.tamper_index.is_some());
}

#[test]
fn test_tamper_delete_middle_event_detected() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut chain = AuditChain::new(temp_file.path()).unwrap();

    // Append events
    chain.append("event_1".to_string(), None, None, None, None, None).unwrap();
    chain.append("event_2".to_string(), None, None, None, None, None).unwrap();
    chain.append("event_3".to_string(), None, None, None, None, None).unwrap();

    // Read all events
    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    let lines: Vec<&str> = content.lines().collect();

    // Tamper: delete middle event
    std::fs::write(temp_file.path(), "").unwrap();
    let mut file = OpenOptions::new().create(true).write(true).open(temp_file.path()).unwrap();

    writeln!(file, "{}", lines[0]).unwrap(); // event_1
    // Skip event_2 (deleted)
    writeln!(file, "{}", lines[2]).unwrap(); // event_3

    // Verify should detect broken chain
    let report = verify_chain(temp_file.path()).unwrap();

    assert!(!report.ok);
    assert_eq!(report.tamper_index, Some(1)); // Second remaining event (index 1 = event_3)
    assert!(report.error.unwrap().contains("Hash chain broken"));
}

#[test]
fn test_no_tamper_verification_passes() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut chain = AuditChain::new(temp_file.path()).unwrap();

    // Append events
    for i in 0..50 {
        chain.append(format!("event_{}", i), None, None, None, None, None).unwrap();
    }

    // Verify should pass
    let report = verify_chain(temp_file.path()).unwrap();

    assert!(report.ok);
    assert_eq!(report.total_events, 50);
    assert!(report.tamper_index.is_none());
    assert!(report.error.is_none());
}
