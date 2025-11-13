/// Prometheus Metrics Module (Week 5)
///
/// Exports metrics for monitoring and SLO tracking
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Global metrics registry
pub struct MetricsRegistry {
    // Counters
    requests_total_ok: AtomicU64,
    requests_total_warn: AtomicU64,
    requests_total_fail: AtomicU64,
    auth_failures_total: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,

    // Histograms (simplified - stores individual durations)
    request_durations: Arc<Mutex<Vec<f64>>>,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self {
            requests_total_ok: AtomicU64::new(0),
            requests_total_warn: AtomicU64::new(0),
            requests_total_fail: AtomicU64::new(0),
            auth_failures_total: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            request_durations: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Increment requests_total counter
    pub fn inc_requests_total(&self, result: &str) {
        match result {
            "ok" => self.requests_total_ok.fetch_add(1, Ordering::Relaxed),
            "warn" => self.requests_total_warn.fetch_add(1, Ordering::Relaxed),
            "fail" => self.requests_total_fail.fetch_add(1, Ordering::Relaxed),
            _ => 0,
        };
    }

    /// Increment auth_failures_total counter
    pub fn inc_auth_failures(&self) {
        self.auth_failures_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Record cache hit
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record cache miss
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Record request duration (in seconds)
    pub fn record_request_duration(&self, duration_secs: f64) {
        let mut durations = self.request_durations.lock().unwrap();
        durations.push(duration_secs);

        // Keep only last 10000 samples to prevent unbounded growth
        if durations.len() > 10000 {
            durations.drain(0..5000);
        }
    }

    /// Get cache hit ratio (0.0 to 1.0)
    pub fn cache_hit_ratio(&self) -> f64 {
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);
        let total = hits + misses;

        if total == 0 {
            return 0.0;
        }

        hits as f64 / total as f64
    }

    /// Export metrics in Prometheus text format
    pub fn export_prometheus(&self) -> String {
        let mut output = String::new();

        // cap_verifier_requests_total
        output
            .push_str("# HELP cap_verifier_requests_total Total number of verification requests\n");
        output.push_str("# TYPE cap_verifier_requests_total counter\n");
        output.push_str(&format!(
            "cap_verifier_requests_total{{result=\"ok\"}} {}\n",
            self.requests_total_ok.load(Ordering::Relaxed)
        ));
        output.push_str(&format!(
            "cap_verifier_requests_total{{result=\"warn\"}} {}\n",
            self.requests_total_warn.load(Ordering::Relaxed)
        ));
        output.push_str(&format!(
            "cap_verifier_requests_total{{result=\"fail\"}} {}\n",
            self.requests_total_fail.load(Ordering::Relaxed)
        ));

        // cap_auth_token_validation_failures_total
        output.push_str(
            "# HELP cap_auth_token_validation_failures_total Total authentication failures\n",
        );
        output.push_str("# TYPE cap_auth_token_validation_failures_total counter\n");
        output.push_str(&format!(
            "cap_auth_token_validation_failures_total {}\n",
            self.auth_failures_total.load(Ordering::Relaxed)
        ));

        // cap_cache_hit_ratio
        output.push_str("# HELP cap_cache_hit_ratio Cache hit ratio (0.0 to 1.0)\n");
        output.push_str("# TYPE cap_cache_hit_ratio gauge\n");
        output.push_str(&format!(
            "cap_cache_hit_ratio {:.4}\n",
            self.cache_hit_ratio()
        ));

        // cap_verifier_request_duration_seconds (histogram)
        let durations = self.request_durations.lock().unwrap();
        if !durations.is_empty() {
            output.push_str(
                "# HELP cap_verifier_request_duration_seconds Request duration in seconds\n",
            );
            output.push_str("# TYPE cap_verifier_request_duration_seconds histogram\n");

            // Calculate histogram buckets
            let buckets = [0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0];
            let mut counts = vec![0u64; buckets.len()];
            let mut sum = 0.0;

            for &duration in durations.iter() {
                sum += duration;
                for (i, &bucket) in buckets.iter().enumerate() {
                    if duration <= bucket {
                        counts[i] += 1;
                    }
                }
            }

            // Cumulative counts for histogram
            let mut cumulative = 0;
            for (bucket, count) in buckets.iter().zip(counts.iter()) {
                cumulative += count;
                output.push_str(&format!(
                    "cap_verifier_request_duration_seconds_bucket{{le=\"{}\"}} {}\n",
                    bucket, cumulative
                ));
            }

            output.push_str(&format!(
                "cap_verifier_request_duration_seconds_bucket{{le=\"+Inf\"}} {}\n",
                durations.len()
            ));
            output.push_str(&format!(
                "cap_verifier_request_duration_seconds_sum {:.6}\n",
                sum
            ));
            output.push_str(&format!(
                "cap_verifier_request_duration_seconds_count {}\n",
                durations.len()
            ));
        }

        output
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global metrics instance
static mut METRICS: Option<MetricsRegistry> = None;
static INIT: std::sync::Once = std::sync::Once::new();

/// Initialize global metrics registry
pub fn init_metrics() {
    INIT.call_once(|| unsafe {
        METRICS = Some(MetricsRegistry::new());
    });
}

/// Get global metrics registry
#[allow(static_mut_refs)]
pub fn get_metrics() -> &'static MetricsRegistry {
    unsafe {
        METRICS
            .as_ref()
            .expect("Metrics not initialized. Call init_metrics() first.")
    }
}

/// Helper: Record request with timing
pub struct RequestTimer {
    start: Instant,
}

impl RequestTimer {
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn finish(self, result: &str) {
        let duration_secs = self.start.elapsed().as_secs_f64();
        let metrics = get_metrics();
        metrics.inc_requests_total(result);
        metrics.record_request_duration(duration_secs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_counters() {
        let metrics = MetricsRegistry::new();

        metrics.inc_requests_total("ok");
        metrics.inc_requests_total("ok");
        metrics.inc_requests_total("fail");

        assert_eq!(metrics.requests_total_ok.load(Ordering::Relaxed), 2);
        assert_eq!(metrics.requests_total_fail.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_cache_hit_ratio() {
        let metrics = MetricsRegistry::new();

        metrics.record_cache_hit();
        metrics.record_cache_hit();
        metrics.record_cache_hit();
        metrics.record_cache_miss();

        let ratio = metrics.cache_hit_ratio();
        assert!((ratio - 0.75).abs() < 0.01); // 3/4 = 0.75
    }

    #[test]
    fn test_prometheus_export() {
        let metrics = MetricsRegistry::new();

        metrics.inc_requests_total("ok");
        metrics.inc_auth_failures();
        metrics.record_cache_hit();
        metrics.record_request_duration(0.123);

        let output = metrics.export_prometheus();

        assert!(output.contains("cap_verifier_requests_total{result=\"ok\"} 1"));
        assert!(output.contains("cap_auth_token_validation_failures_total 1"));
        assert!(output.contains("cap_cache_hit_ratio"));
        assert!(output.contains("cap_verifier_request_duration_seconds"));
    }

    #[test]
    fn test_request_timer() {
        init_metrics();
        let timer = RequestTimer::start();
        std::thread::sleep(std::time::Duration::from_millis(10));
        timer.finish("ok");

        let metrics = get_metrics();
        assert_eq!(metrics.requests_total_ok.load(Ordering::Relaxed), 1);
    }
}
