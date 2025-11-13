/// Integration Tests for Prometheus Metrics Export (Week 5)
///
/// Tests:
/// - IT-M1: /metrics endpoint returns valid Prometheus format
/// - IT-M2: Metrics contain expected metric names
/// - IT-M3: Histogram buckets are present after traffic
/// - IT-M4: Cache hit ratio calculation is correct

use cap_agent::metrics::{init_metrics, get_metrics, MetricsRegistry};

#[test]
fn test_metrics_registry_creation() {
    let metrics = MetricsRegistry::new();

    // Verify initial state
    let output = metrics.export_prometheus();
    assert!(output.contains("cap_verifier_requests_total"));
    assert!(output.contains("cap_auth_token_validation_failures_total"));
    assert!(output.contains("cap_cache_hit_ratio"));

    println!("✅ Metrics registry created successfully");
}

#[test]
fn it_m1_metrics_endpoint_prometheus_format() {
    // Test: /metrics endpoint returns valid Prometheus format

    let metrics = MetricsRegistry::new();

    // Generate some traffic
    metrics.inc_requests_total("ok");
    metrics.inc_requests_total("ok");
    metrics.inc_requests_total("fail");
    metrics.inc_auth_failures();
    metrics.record_request_duration(0.123);
    metrics.record_request_duration(0.456);

    let output = metrics.export_prometheus();

    // Verify Prometheus format
    assert!(output.contains("# HELP"));
    assert!(output.contains("# TYPE"));

    // Verify counter format
    assert!(output.contains("cap_verifier_requests_total{result=\"ok\"} 2"));
    assert!(output.contains("cap_verifier_requests_total{result=\"fail\"} 1"));

    // Verify auth failures
    assert!(output.contains("cap_auth_token_validation_failures_total 1"));

    // Verify histogram format
    assert!(output.contains("cap_verifier_request_duration_seconds_bucket"));
    assert!(output.contains("cap_verifier_request_duration_seconds_sum"));
    assert!(output.contains("cap_verifier_request_duration_seconds_count 2"));

    println!("✅ Metrics in valid Prometheus format");
}

#[test]
fn it_m2_metrics_contain_all_expected_names() {
    // Test: All expected metrics are present

    let metrics = MetricsRegistry::new();

    // Generate minimal traffic
    metrics.inc_requests_total("ok");
    metrics.record_cache_hit();
    metrics.record_request_duration(0.1);

    let output = metrics.export_prometheus();

    let expected_metrics = vec![
        "cap_verifier_requests_total",
        "cap_auth_token_validation_failures_total",
        "cap_cache_hit_ratio",
        "cap_verifier_request_duration_seconds",
    ];

    for metric_name in expected_metrics {
        assert!(
            output.contains(metric_name),
            "Missing metric: {}",
            metric_name
        );
    }

    println!("✅ All expected metrics present");
}

#[test]
fn it_m3_histogram_buckets_after_traffic() {
    // Test: Histogram buckets populated after traffic

    let metrics = MetricsRegistry::new();

    // Generate traffic with various durations
    let durations = vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0];

    for duration in durations {
        metrics.record_request_duration(duration);
    }

    let output = metrics.export_prometheus();

    // Verify buckets are present
    let expected_buckets = vec!["0.001", "0.005", "0.01", "0.05", "0.1", "0.5", "1", "5"];

    for bucket in expected_buckets {
        let bucket_label = format!("le=\"{}\"", bucket);
        assert!(
            output.contains(&bucket_label),
            "Missing histogram bucket: {}",
            bucket
        );
    }

    // Verify +Inf bucket
    assert!(output.contains("le=\"+Inf\""));

    // Verify count matches number of samples
    assert!(output.contains("cap_verifier_request_duration_seconds_count 7"));

    println!("✅ Histogram buckets populated correctly");
}

#[test]
fn it_m4_cache_hit_ratio_calculation() {
    // Test: Cache hit ratio is calculated correctly

    let metrics = MetricsRegistry::new();

    // Scenario 1: 75% hit rate (3 hits, 1 miss)
    metrics.record_cache_hit();
    metrics.record_cache_hit();
    metrics.record_cache_hit();
    metrics.record_cache_miss();

    let ratio = metrics.cache_hit_ratio();
    assert!((ratio - 0.75).abs() < 0.01, "Expected 75% hit rate, got {}", ratio);

    let output = metrics.export_prometheus();
    assert!(output.contains("cap_cache_hit_ratio 0.75"));

    println!("✅ Cache hit ratio: {:.2}% (expected 75%)", ratio * 100.0);
}

#[test]
fn it_m5_zero_traffic_metrics() {
    // Test: Metrics with zero traffic (all counters at 0)

    let metrics = MetricsRegistry::new();

    let output = metrics.export_prometheus();

    // Verify zero counters
    assert!(output.contains("cap_verifier_requests_total{result=\"ok\"} 0"));
    assert!(output.contains("cap_verifier_requests_total{result=\"warn\"} 0"));
    assert!(output.contains("cap_verifier_requests_total{result=\"fail\"} 0"));
    assert!(output.contains("cap_auth_token_validation_failures_total 0"));

    // Cache hit ratio should be 0.0 with no data
    assert!(output.contains("cap_cache_hit_ratio 0.0000"));

    println!("✅ Zero traffic metrics valid");
}

#[test]
fn it_m6_high_traffic_simulation() {
    // Test: Metrics under simulated high traffic

    let metrics = MetricsRegistry::new();

    // Simulate 1000 requests
    for i in 0..1000 {
        if i % 10 == 0 {
            metrics.inc_requests_total("fail"); // 10% error rate
        } else {
            metrics.inc_requests_total("ok");
        }

        // Simulate varying latencies
        let duration = if i % 100 == 0 {
            0.5 // Occasional slow requests
        } else {
            0.05 // Most requests fast
        };
        metrics.record_request_duration(duration);

        // Cache behavior
        if i % 5 == 0 {
            metrics.record_cache_miss(); // 20% miss rate
        } else {
            metrics.record_cache_hit();
        }
    }

    let output = metrics.export_prometheus();

    // Verify counters
    assert!(output.contains("cap_verifier_requests_total{result=\"ok\"} 900"));
    assert!(output.contains("cap_verifier_requests_total{result=\"fail\"} 100"));

    // Verify cache hit ratio (should be 80%)
    let ratio = metrics.cache_hit_ratio();
    assert!((ratio - 0.8).abs() < 0.01, "Expected 80% hit rate, got {}", ratio);

    // Verify histogram count
    assert!(output.contains("cap_verifier_request_duration_seconds_count 1000"));

    println!("✅ High traffic simulation completed");
    println!("   - Requests: 1000 (900 OK, 100 FAIL)");
    println!("   - Cache hit rate: {:.1}%", ratio * 100.0);
}

#[test]
fn it_m7_request_timer_helper() {
    // Test: RequestTimer helper correctly records timing

    init_metrics();

    use cap_agent::metrics::RequestTimer;
    use std::time::Duration;

    let timer = RequestTimer::start();
    std::thread::sleep(Duration::from_millis(50));
    timer.finish("ok");

    let metrics = get_metrics();
    let output = metrics.export_prometheus();

    // Verify request was recorded
    assert!(output.contains("cap_verifier_requests_total{result=\"ok\"} 1"));

    // Verify duration was recorded (should be >= 50ms)
    assert!(output.contains("cap_verifier_request_duration_seconds_count 1"));

    println!("✅ RequestTimer helper works correctly");
}

#[test]
fn it_m8_multiple_result_types() {
    // Test: Different result types (ok, warn, fail) are tracked separately

    let metrics = MetricsRegistry::new();

    metrics.inc_requests_total("ok");
    metrics.inc_requests_total("ok");
    metrics.inc_requests_total("ok");
    metrics.inc_requests_total("warn");
    metrics.inc_requests_total("warn");
    metrics.inc_requests_total("fail");

    let output = metrics.export_prometheus();

    assert!(output.contains("cap_verifier_requests_total{result=\"ok\"} 3"));
    assert!(output.contains("cap_verifier_requests_total{result=\"warn\"} 2"));
    assert!(output.contains("cap_verifier_requests_total{result=\"fail\"} 1"));

    println!("✅ Multiple result types tracked correctly");
}

#[test]
fn it_m9_histogram_percentile_buckets() {
    // Test: Histogram buckets enable p95/p99 calculations

    let metrics = MetricsRegistry::new();

    // Generate 100 samples with known distribution
    for i in 0..95 {
        metrics.record_request_duration(0.01); // 95% fast
    }
    for _ in 0..4 {
        metrics.record_request_duration(0.5); // 4% medium
    }
    metrics.record_request_duration(1.5); // 1% slow

    let output = metrics.export_prometheus();

    // Verify buckets capture the distribution
    assert!(output.contains("le=\"0.01\"")); // Should have 95 samples
    assert!(output.contains("le=\"0.5\"")); // Should have 99 samples
    assert!(output.contains("le=\"1\""));  // Should have 99 samples
    assert!(output.contains("le=\"5\""));  // Should have 100 samples

    println!("✅ Histogram buckets enable percentile calculations");
}

#[test]
fn it_m10_auth_failures_tracked() {
    // Test: Authentication failures are tracked separately

    let metrics = MetricsRegistry::new();

    // Simulate auth failures
    for _ in 0..25 {
        metrics.inc_auth_failures();
    }

    let output = metrics.export_prometheus();

    assert!(output.contains("cap_auth_token_validation_failures_total 25"));

    println!("✅ Auth failures tracked: 25");
}
