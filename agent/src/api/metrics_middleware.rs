use crate::metrics::get_metrics;
/// Metrics Middleware für automatisches Request-Tracking
///
/// Tracked:
/// - Request Count (per Endpoint + Status)
/// - Request Duration (Histogram)
/// - Auth Failures
use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use std::time::Instant;

/// Metrics Middleware
///
/// Automatisches Tracking von:
/// - cap_verifier_requests_total (mit result label)
/// - cap_verifier_request_duration_seconds (histogram)
/// - cap_auth_token_validation_failures_total (bei 401)
pub async fn metrics_middleware(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let path = req.uri().path().to_string();

    // Führe Request aus
    let response = next.run(req).await;

    // Messe Dauer
    let duration_secs = start.elapsed().as_secs_f64();
    let status = response.status();

    // Update Metrics
    let metrics = get_metrics();

    // Request Counter (mit result label)
    let result = match status {
        StatusCode::OK => "ok",
        StatusCode::BAD_REQUEST => "fail",
        StatusCode::UNAUTHORIZED => {
            // Auth failure tracking
            metrics.inc_auth_failures();
            "fail"
        }
        StatusCode::INTERNAL_SERVER_ERROR => "fail",
        _ => "warn",
    };

    // Record request duration + counter (nur für /verify und /policy Endpoints)
    if path.starts_with("/verify") || path.starts_with("/policy") {
        metrics.record_request_duration(duration_secs);
        metrics.inc_requests_total(result);
    }

    response
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_metrics_middleware_ok() {
        // TODO: Test mit Mock-Handler
    }

    #[tokio::test]
    async fn test_metrics_middleware_auth_failure() {
        // TODO: Test 401 → auth_failures_total increment
    }
}
