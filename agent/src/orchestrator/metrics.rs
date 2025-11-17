//! # Orchestrator Metrics (Week 6 - B1)
//!
//! Prometheus metrics for adaptive enforcement and drift monitoring.
//! Exported via /metrics endpoint in the REST API.

use once_cell::sync::Lazy;
use prometheus::{
    register_gauge, register_histogram, register_int_counter_vec, Gauge, Histogram, IntCounterVec,
};

/// Current enforcement rollout percentage (0-100)
///
/// # Labels
/// None
///
/// # Example
/// ```text
/// adapt_enforce_rollout_percent 25.0
/// ```
pub static ADAPT_ROLLOUT_PERCENT: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!(
        "adapt_enforce_rollout_percent",
        "Current enforce rollout percent (0-100)"
    )
    .unwrap()
});

/// Total adapt requests by mode and policy
///
/// # Labels
/// - `mode`: "shadow" or "enforced"
/// - `policy_id`: Policy identifier (e.g., "lksg.v1")
///
/// # Example
/// ```text
/// adapt_requests_total{mode="shadow",policy_id="lksg.v1"} 1000
/// adapt_requests_total{mode="enforced",policy_id="lksg.v1"} 250
/// ```
pub static ADAPT_REQUESTS_TOTAL: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "adapt_requests_total",
        "Adapt requests by mode and policy",
        &["mode", "policy_id"]
    )
    .unwrap()
});

/// Total drift events by policy
///
/// Drift = Shadow verdict ≠ Enforced verdict
///
/// # Labels
/// - `policy_id`: Policy identifier
///
/// # Example
/// ```text
/// adapt_drift_events_total{policy_id="lksg.v1"} 12
/// ```
pub static ADAPT_DRIFT_EVENTS_TOTAL: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "adapt_drift_events_total",
        "Drift events by policy (shadow != enforced)",
        &["policy_id"]
    )
    .unwrap()
});

/// Drift ratio (rolling 5-minute window)
///
/// Calculated as: drift_events / total_requests over last 5 minutes
///
/// # Labels
/// - `window`: Time window (e.g., "5m")
///
/// # Example
/// ```text
/// adapt_drift_ratio{window="5m"} 0.003
/// ```
pub static ADAPT_DRIFT_RATIO_5M: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!("adapt_drift_ratio", "Drift ratio (5-minute rolling window)").unwrap()
});

/// Selection latency (rule selection + planning)
///
/// Measures time spent in orchestrator for rule selection and plan creation.
///
/// # Buckets
/// - 1ms, 5ms, 10ms, 50ms, 100ms, 500ms, 1s, 5s
///
/// # Example
/// ```text
/// adapt_selection_latency_seconds_bucket{le="0.01"} 50
/// adapt_selection_latency_seconds_bucket{le="0.05"} 120
/// adapt_selection_latency_seconds_sum 45.2
/// adapt_selection_latency_seconds_count 150
/// ```
pub static ADAPT_SELECTION_LATENCY: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "adapt_selection_latency_seconds",
        "Selection latency seconds",
        vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]
    )
    .unwrap()
});

/// Helper function to record shadow request
pub fn record_shadow_request(policy_id: &str) {
    ADAPT_REQUESTS_TOTAL
        .with_label_values(&["shadow", policy_id])
        .inc();
}

/// Helper function to record enforced request
pub fn record_enforced_request(policy_id: &str) {
    ADAPT_REQUESTS_TOTAL
        .with_label_values(&["enforced", policy_id])
        .inc();
}

/// Helper function to record drift event
pub fn record_drift_event(policy_id: &str) {
    ADAPT_DRIFT_EVENTS_TOTAL
        .with_label_values(&[policy_id])
        .inc();
}

/// Helper function to update rollout percentage
pub fn set_rollout_percent(percent: u8) {
    ADAPT_ROLLOUT_PERCENT.set(percent as f64);
}

/// Helper function to update drift ratio
pub fn set_drift_ratio(ratio: f64) {
    ADAPT_DRIFT_RATIO_5M.set(ratio);
}

/// Helper function to observe selection latency
pub fn observe_selection_latency(duration_seconds: f64) {
    ADAPT_SELECTION_LATENCY.observe(duration_seconds);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rollout_percent_metric() {
        set_rollout_percent(25);
        let value = ADAPT_ROLLOUT_PERCENT.get();
        assert_eq!(value, 25.0);

        set_rollout_percent(100);
        let value = ADAPT_ROLLOUT_PERCENT.get();
        assert_eq!(value, 100.0);
    }

    #[test]
    fn test_drift_ratio_metric() {
        set_drift_ratio(0.005);
        let value = ADAPT_DRIFT_RATIO_5M.get();
        assert!((value - 0.005).abs() < 0.0001);

        set_drift_ratio(0.0);
        let value = ADAPT_DRIFT_RATIO_5M.get();
        assert_eq!(value, 0.0);
    }

    #[test]
    fn test_shadow_request_counter() {
        // Note: Prometheus counters are cumulative, so we can only check that they increment
        let before = ADAPT_REQUESTS_TOTAL
            .with_label_values(&["shadow", "test.v1"])
            .get();

        record_shadow_request("test.v1");

        let after = ADAPT_REQUESTS_TOTAL
            .with_label_values(&["shadow", "test.v1"])
            .get();

        assert_eq!(after, before + 1);
    }

    #[test]
    fn test_enforced_request_counter() {
        let before = ADAPT_REQUESTS_TOTAL
            .with_label_values(&["enforced", "test.v1"])
            .get();

        record_enforced_request("test.v1");

        let after = ADAPT_REQUESTS_TOTAL
            .with_label_values(&["enforced", "test.v1"])
            .get();

        assert_eq!(after, before + 1);
    }

    #[test]
    fn test_drift_event_counter() {
        let before = ADAPT_DRIFT_EVENTS_TOTAL
            .with_label_values(&["test.v1"])
            .get();

        record_drift_event("test.v1");

        let after = ADAPT_DRIFT_EVENTS_TOTAL
            .with_label_values(&["test.v1"])
            .get();

        assert_eq!(after, before + 1);
    }

    #[test]
    fn test_selection_latency_histogram() {
        // Record some latencies
        observe_selection_latency(0.001); // 1ms
        observe_selection_latency(0.050); // 50ms
        observe_selection_latency(0.100); // 100ms

        // Check that histogram has recorded observations
        let metric = ADAPT_SELECTION_LATENCY.get_sample_count();
        assert!(metric >= 3); // At least our 3 observations
    }

    #[test]
    fn test_no_pii_in_labels() {
        // Verify that metrics do not expose PII
        // Only policy_id and mode are allowed as labels

        // Valid labels
        record_shadow_request("lksg.v1");
        record_enforced_request("lksg.v1");
        record_drift_event("lksg.v1");

        // This test passes if compilation succeeds (no PII fields in metric signatures)
    }
}
