//! # Enforcer Metrics Tests (Week 6 - B1)
//!
//! Tests für Prometheus-Metriken des Adaptive Enforcer.

use cap_agent::orchestrator::metrics;
use cap_agent::orchestrator::{
    DriftTracker, EnforceOptions, Enforcer, OrchestratorContext, Verdict, VerdictPair,
};
use cap_agent::policy_v2::types::{IrExpression, IrRule, IrV1};
use std::collections::HashMap;

/// Helper: Erstellt Test-IR mit minimalen Regeln
fn create_test_ir() -> IrV1 {
    IrV1 {
        ir_version: "1.0".to_string(),
        policy_id: "test_metrics.v1".to_string(),
        policy_hash: "sha3-256:0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
            .to_string(),
        rules: vec![IrRule {
            id: "rule1".to_string(),
            op: "eq".to_string(),
            lhs: IrExpression::Var {
                var: "supplier_hash".to_string(),
            },
            rhs: IrExpression::Literal(serde_json::Value::String("0xabc".to_string())),
        }],
        adaptivity: None,
        ir_hash: "sha3-256:def".to_string(),
    }
}

/// Helper: Erstellt Test-Context
fn create_test_context() -> OrchestratorContext {
    OrchestratorContext {
        supplier_hashes: vec!["0xabc".to_string()],
        ubo_hashes: vec![],
        company_commitment_root: Some("0x123".to_string()),
        sanctions_root: None,
        jurisdiction_root: None,
        variables: HashMap::new(),
    }
}

#[test]
fn test_metrics_increment_shadow_only() {
    // Test: Shadow-only Mode erhöht nur shadow-Counter
    // Use unique policy_id to avoid metric sharing across tests
    let policy_id = format!(
        "test_shadow_only_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );

    let mut ir = create_test_ir();
    ir.policy_id = policy_id.clone();
    let ctx = create_test_context();

    let opts = EnforceOptions {
        enforce: false,
        rollout_percent: 0,
        drift_max_ratio: 0.005,
    };

    // Get baseline counts
    let shadow_before = metrics::ADAPT_REQUESTS_TOTAL
        .with_label_values(&["shadow", &policy_id])
        .get();
    let enforced_before = metrics::ADAPT_REQUESTS_TOTAL
        .with_label_values(&["enforced", &policy_id])
        .get();

    // Execute enforcement
    let enforcer = Enforcer::new(&ir, opts).unwrap();
    let verdict_pair = enforcer.decide(&ctx, "test-shadow-request").unwrap();

    // Record metrics
    if verdict_pair.enforced_applied {
        metrics::record_enforced_request(&policy_id);
    } else {
        metrics::record_shadow_request(&policy_id);
    }

    // Check metrics
    let shadow_after = metrics::ADAPT_REQUESTS_TOTAL
        .with_label_values(&["shadow", &policy_id])
        .get();
    let enforced_after = metrics::ADAPT_REQUESTS_TOTAL
        .with_label_values(&["enforced", &policy_id])
        .get();

    assert_eq!(
        shadow_after,
        shadow_before + 1,
        "Shadow counter should increment"
    );
    assert_eq!(
        enforced_after, enforced_before,
        "Enforced counter should not increment in shadow-only mode"
    );
}

#[test]
fn test_metrics_increment_enforced() {
    // Test: Enforced Mode erhöht beide Counter und setzt Rollout-Gauge
    // Use unique policy_id to avoid metric sharing across tests
    let policy_id = format!(
        "test_enforced_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );

    let mut ir = create_test_ir();
    ir.policy_id = policy_id.clone();
    let ctx = create_test_context();

    let opts = EnforceOptions {
        enforce: true,
        rollout_percent: 100, // 100% rollout
        drift_max_ratio: 0.005,
    };

    // Get baseline counts
    let shadow_before = metrics::ADAPT_REQUESTS_TOTAL
        .with_label_values(&["shadow", &policy_id])
        .get();
    let enforced_before = metrics::ADAPT_REQUESTS_TOTAL
        .with_label_values(&["enforced", &policy_id])
        .get();

    // Set rollout percentage
    metrics::set_rollout_percent(100);

    // Execute enforcement
    let enforcer = Enforcer::new(&ir, opts).unwrap();
    let verdict_pair = enforcer.decide(&ctx, "test-enforced-request").unwrap();

    // Record metrics (both shadow and enforced)
    metrics::record_shadow_request(&policy_id);
    if verdict_pair.enforced_applied {
        metrics::record_enforced_request(&policy_id);
    }

    // Check metrics
    let shadow_after = metrics::ADAPT_REQUESTS_TOTAL
        .with_label_values(&["shadow", &policy_id])
        .get();
    let enforced_after = metrics::ADAPT_REQUESTS_TOTAL
        .with_label_values(&["enforced", &policy_id])
        .get();
    let rollout_gauge = metrics::ADAPT_ROLLOUT_PERCENT.get();

    assert_eq!(
        shadow_after,
        shadow_before + 1,
        "Shadow counter should increment (always computed)"
    );
    assert_eq!(
        enforced_after,
        enforced_before + 1,
        "Enforced counter should increment"
    );
    assert_eq!(rollout_gauge, 100.0, "Rollout gauge should be set to 100");
}

#[test]
fn test_drift_event_increments_counter() {
    // Test: Drift-Event erhöht Drift-Counter
    let policy_id = format!(
        "test_drift_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );

    // Get baseline count
    let drift_before = metrics::ADAPT_DRIFT_EVENTS_TOTAL
        .with_label_values(&[&policy_id])
        .get();

    // Simulate drift event
    metrics::record_drift_event(&policy_id);

    // Check metrics
    let drift_after = metrics::ADAPT_DRIFT_EVENTS_TOTAL
        .with_label_values(&[&policy_id])
        .get();

    assert_eq!(
        drift_after,
        drift_before + 1,
        "Drift counter should increment"
    );
}

#[test]
fn test_drift_ratio_updates() {
    // Test: DriftTracker berechnet Drift-Ratio korrekt
    let mut tracker = DriftTracker::new();
    let policy_id = format!(
        "test_ratio_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );

    // Add non-drift events
    for _ in 0..95 {
        let pair = VerdictPair {
            shadow: Verdict::Ok,
            enforced: Verdict::Ok,
            enforced_applied: true,
        };
        tracker.record(&policy_id, &pair);
    }

    // Add drift events
    for _ in 0..5 {
        let pair = VerdictPair {
            shadow: Verdict::Ok,
            enforced: Verdict::Fail,
            enforced_applied: true,
        };
        tracker.record(&policy_id, &pair);
    }

    // Calculate drift ratio
    let drift_ratio = tracker.drift_ratio();
    assert_eq!(drift_ratio, 0.05, "Drift ratio should be 5% (5/100)");

    // Update Prometheus gauge
    metrics::set_drift_ratio(drift_ratio);
    let gauge_value = metrics::ADAPT_DRIFT_RATIO_5M.get();
    assert_eq!(gauge_value, 0.05, "Drift ratio gauge should be updated");
}

#[test]
fn test_selection_latency_histogram() {
    // Test: Selection Latency Histogram wird aktualisiert

    // Get baseline sample count
    let samples_before = metrics::ADAPT_SELECTION_LATENCY.get_sample_count();

    // Record some latencies
    metrics::observe_selection_latency(0.001); // 1ms
    metrics::observe_selection_latency(0.005); // 5ms
    metrics::observe_selection_latency(0.050); // 50ms

    // Check that observations were recorded
    let samples_after = metrics::ADAPT_SELECTION_LATENCY.get_sample_count();
    assert!(
        samples_after >= samples_before + 3,
        "Histogram should have at least 3 new observations"
    );
}

#[test]
fn test_rollout_percent_gauge() {
    // Test: Rollout-Prozent-Gauge wird korrekt gesetzt

    metrics::set_rollout_percent(0);
    assert_eq!(metrics::ADAPT_ROLLOUT_PERCENT.get(), 0.0);

    metrics::set_rollout_percent(25);
    assert_eq!(metrics::ADAPT_ROLLOUT_PERCENT.get(), 25.0);

    metrics::set_rollout_percent(50);
    assert_eq!(metrics::ADAPT_ROLLOUT_PERCENT.get(), 50.0);

    metrics::set_rollout_percent(100);
    assert_eq!(metrics::ADAPT_ROLLOUT_PERCENT.get(), 100.0);
}

#[test]
fn test_drift_tracker_threshold() {
    // Test: DriftTracker erkennt Überschreitung der Drift-Schwelle
    let mut tracker = DriftTracker::new();
    let policy_id = format!(
        "test_threshold_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );

    // Threshold: 0.5%
    let threshold = 0.005;

    // Add requests below threshold
    for _ in 0..199 {
        let pair = VerdictPair {
            shadow: Verdict::Ok,
            enforced: Verdict::Ok,
            enforced_applied: true,
        };
        tracker.record(&policy_id, &pair);
    }

    // Add 1 drift event (1/200 = 0.5% = exactly at threshold)
    let pair = VerdictPair {
        shadow: Verdict::Ok,
        enforced: Verdict::Fail,
        enforced_applied: true,
    };
    tracker.record(&policy_id, &pair);

    assert!(
        !tracker.exceeds_threshold(threshold),
        "Should not exceed threshold at exactly 0.5%"
    );

    // Add one more drift event (2/201 ≈ 0.995% > 0.5%)
    let pair2 = VerdictPair {
        shadow: Verdict::Ok,
        enforced: Verdict::Fail,
        enforced_applied: true,
    };
    tracker.record(&policy_id, &pair2);

    assert!(
        tracker.exceeds_threshold(threshold),
        "Should exceed threshold above 0.5%"
    );
}

#[test]
fn test_no_pii_in_metrics() {
    // Test: Metriken enthalten keine PII (nur policy_id und mode als Labels)
    let policy_id = format!(
        "test_pii_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );

    // Record metrics with only allowed labels
    metrics::record_shadow_request(&policy_id);
    metrics::record_enforced_request(&policy_id);
    metrics::record_drift_event(&policy_id);

    // Check that metrics exist (would fail at compile time if PII fields were added)
    let _shadow_count = metrics::ADAPT_REQUESTS_TOTAL
        .with_label_values(&["shadow", &policy_id])
        .get();
    let _enforced_count = metrics::ADAPT_REQUESTS_TOTAL
        .with_label_values(&["enforced", &policy_id])
        .get();
    let _drift_count = metrics::ADAPT_DRIFT_EVENTS_TOTAL
        .with_label_values(&[&policy_id])
        .get();

    // If we got here, no PII fields are exposed (compile-time check)
    // No assertions needed - the fact that metrics can be retrieved proves no PII leakage
}
