//! Audit Chain Integration Tests (Track A)
//!
//! End-to-end tests for audit chain workflow.

use cap_agent::audit::{export_events, verify_chain, AuditChain, AuditEventResult};
use tempfile::NamedTempFile;

#[test]
fn test_full_workflow_append_verify_export() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut chain = AuditChain::new(temp_file.path()).unwrap();

    // Append multiple events
    for i in 0..10 {
        chain
            .append(
                format!("event_{}", i),
                Some("lksg.v1".to_string()),
                Some(format!("sha3-256:ir_{}", i)),
                Some(format!("0xmanifest_{}", i)),
                Some(if i % 2 == 0 {
                    AuditEventResult::Ok
                } else {
                    AuditEventResult::Warn
                }),
                Some(format!("run-{}", i / 3)),
            )
            .unwrap();
    }

    // Verify chain
    let report = verify_chain(temp_file.path()).unwrap();
    assert!(report.ok);
    assert_eq!(report.total_events, 10);

    // Export all
    let all_events = export_events(temp_file.path(), None, None, None).unwrap();
    assert_eq!(all_events.len(), 10);

    // Export by policy
    let lksg_events = export_events(temp_file.path(), None, None, Some("lksg.v1")).unwrap();
    assert_eq!(lksg_events.len(), 10);
}

#[test]
fn test_chain_persistence_reload() {
    let temp_file = NamedTempFile::new().unwrap();

    // First session: append events
    {
        let mut chain = AuditChain::new(temp_file.path()).unwrap();
        chain
            .append("event_1".to_string(), None, None, None, None, None)
            .unwrap();
        chain
            .append("event_2".to_string(), None, None, None, None, None)
            .unwrap();
    }

    // Second session: reload and append more
    {
        let mut chain = AuditChain::new(temp_file.path()).unwrap();

        // Tail should be preserved from previous session
        let _event3 = chain
            .append("event_3".to_string(), None, None, None, None, None)
            .unwrap();

        // Export all 3 events
        let events = export_events(temp_file.path(), None, None, None).unwrap();
        assert_eq!(events.len(), 3);

        // Verify chain integrity across sessions
        assert_eq!(events[2].prev_hash, events[1].self_hash);
    }

    // Verify final chain
    let report = verify_chain(temp_file.path()).unwrap();
    assert!(report.ok);
    assert_eq!(report.total_events, 3);
}

#[test]
fn test_large_chain_performance() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut chain = AuditChain::new(temp_file.path()).unwrap();

    // Append 1000 events
    for i in 0..1000 {
        chain
            .append(
                format!("event_{}", i),
                Some("lksg.v1".to_string()),
                None,
                None,
                Some(AuditEventResult::Ok),
                None,
            )
            .unwrap();
    }

    // Verify should be fast (DoD: 10k events ≤ 3s)
    let start = std::time::Instant::now();
    let report = verify_chain(temp_file.path()).unwrap();
    let duration = start.elapsed();

    assert!(report.ok);
    assert_eq!(report.total_events, 1000);

    // Should be well under 3s for 1000 events
    assert!(duration.as_secs() < 3, "Verify took {:?} (> 3s)", duration);

    println!("✅ Verified 1000 events in {:?}", duration);
}

#[test]
#[ignore] // Slow test, run with --ignored
fn test_dod_verify_10k_events_under_3s() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut chain = AuditChain::new(temp_file.path()).unwrap();

    // Append 10,000 events (DoD requirement)
    println!("Appending 10,000 events...");
    let append_start = std::time::Instant::now();
    for i in 0..10000 {
        chain
            .append(
                format!("event_{}", i),
                Some("lksg.v1".to_string()),
                Some(format!("sha3-256:ir_{}", i)),
                Some(format!("0xmanifest_{}", i)),
                Some(AuditEventResult::Ok),
                Some(format!("run-{}", i / 100)),
            )
            .unwrap();
    }
    let append_duration = append_start.elapsed();
    println!("✅ Appended 10,000 events in {:?}", append_duration);

    // Verify 10k events ≤ 3s (DoD criterion)
    println!("Verifying 10,000 events...");
    let verify_start = std::time::Instant::now();
    let report = verify_chain(temp_file.path()).unwrap();
    let verify_duration = verify_start.elapsed();

    assert!(report.ok);
    assert_eq!(report.total_events, 10000);

    println!("✅ Verified 10,000 events in {:?}", verify_duration);

    // DoD: Verify 10k events ≤ 3s
    assert!(
        verify_duration.as_secs() <= 3,
        "FAILED: Verify took {:?}, expected ≤ 3s",
        verify_duration
    );

    println!(
        "✅ DoD Performance Criterion MET: 10,000 events verified in {:?} (≤ 3s)",
        verify_duration
    );
}

#[test]
#[ignore] // Slow test, run with --ignored
fn test_dod_append_p95_under_0_5ms() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut chain = AuditChain::new(temp_file.path()).unwrap();

    // Measure append latency for 1000 events
    let mut append_times = Vec::with_capacity(1000);

    for i in 0..1000 {
        let start = std::time::Instant::now();
        chain
            .append(
                format!("event_{}", i),
                Some("lksg.v1".to_string()),
                None,
                None,
                Some(AuditEventResult::Ok),
                None,
            )
            .unwrap();
        let duration = start.elapsed();
        append_times.push(duration);
    }

    // Calculate p95
    append_times.sort();
    let p95_index = (append_times.len() as f64 * 0.95) as usize;
    let p95 = append_times[p95_index];

    println!("✅ Append p95: {:?}", p95);

    // DoD: Append p95 ≤ 0.5ms
    assert!(
        p95.as_micros() <= 500,
        "FAILED: Append p95 = {:?}, expected ≤ 0.5ms",
        p95
    );

    println!(
        "✅ DoD Performance Criterion MET: Append p95 = {:?} (≤ 0.5ms)",
        p95
    );
}

#[test]
fn test_export_with_complex_filters() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut chain = AuditChain::new(temp_file.path()).unwrap();

    // Append events over time with different policies
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
    std::thread::sleep(std::time::Duration::from_millis(10));

    let mid_time = chrono::Utc::now().to_rfc3339();
    std::thread::sleep(std::time::Duration::from_millis(10));

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
    chain
        .append(
            "event_4".to_string(),
            Some("lksg.v1".to_string()),
            None,
            None,
            None,
            None,
        )
        .unwrap();

    // Export: from mid_time AND policy lksg.v1
    let events = export_events(temp_file.path(), Some(&mid_time), None, Some("lksg.v1")).unwrap();

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].event, "event_3");
    assert_eq!(events[1].event, "event_4");
}
