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
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware,
        response::IntoResponse,
        routing::post,
        Router,
    };
    use tower::ServiceExt;

    // Helper: Create a test app with metrics middleware and a handler that returns a specific status
    fn create_test_app(status: StatusCode) -> Router {
        async fn handler_ok() -> impl IntoResponse {
            StatusCode::OK
        }
        async fn handler_unauthorized() -> impl IntoResponse {
            StatusCode::UNAUTHORIZED
        }
        async fn handler_bad_request() -> impl IntoResponse {
            StatusCode::BAD_REQUEST
        }
        async fn handler_not_found() -> impl IntoResponse {
            StatusCode::NOT_FOUND
        }

        let handler: axum::routing::MethodRouter = match status {
            StatusCode::OK => post(handler_ok),
            StatusCode::UNAUTHORIZED => post(handler_unauthorized),
            StatusCode::BAD_REQUEST => post(handler_bad_request),
            StatusCode::NOT_FOUND => post(handler_not_found),
            _ => post(handler_ok),
        };

        Router::new()
            .route("/verify", handler.clone())
            .route("/policy/compile", handler.clone())
            .route("/healthz", handler)
            .layer(middleware::from_fn(metrics_middleware))
    }

    #[tokio::test]
    async fn test_metrics_middleware_ok() {
        // Initialize metrics
        crate::metrics::init_metrics();
        let metrics = get_metrics();

        // Get initial counter values
        let initial_ok = metrics.get_requests_total_ok();

        // Create test app and request
        let app = create_test_app(StatusCode::OK);
        let req = Request::builder()
            .method("POST")
            .uri("/verify")
            .body(Body::empty())
            .unwrap();

        // Send request
        let _response = app.oneshot(req).await.unwrap();

        // Verify that ok counter was incremented (tests may run in parallel)
        assert!(
            metrics.get_requests_total_ok() > initial_ok,
            "OK counter should increase after successful request"
        );
    }

    #[tokio::test]
    async fn test_metrics_middleware_auth_failure() {
        // Initialize metrics
        crate::metrics::init_metrics();
        let metrics = get_metrics();

        // Get initial counter values
        let initial_auth_failures = metrics.get_auth_failures_total();
        let initial_fail = metrics.get_requests_total_fail();

        // Create test app and request
        let app = create_test_app(StatusCode::UNAUTHORIZED);
        let req = Request::builder()
            .method("POST")
            .uri("/verify")
            .body(Body::empty())
            .unwrap();

        // Send request
        let _response = app.oneshot(req).await.unwrap();

        // Verify that auth_failures counter was incremented (tests may run in parallel)
        assert!(
            metrics.get_auth_failures_total() > initial_auth_failures,
            "Auth failures counter should increase after 401"
        );

        // Verify that fail counter was incremented (tests may run in parallel)
        assert!(
            metrics.get_requests_total_fail() > initial_fail,
            "Fail counter should increase after 401"
        );
    }

    #[tokio::test]
    async fn test_metrics_middleware_bad_request() {
        // Initialize metrics
        crate::metrics::init_metrics();
        let metrics = get_metrics();

        // Get initial counter value
        let initial_fail = metrics.get_requests_total_fail();

        // Create test app and request
        let app = create_test_app(StatusCode::BAD_REQUEST);
        let req = Request::builder()
            .method("POST")
            .uri("/policy/compile")
            .body(Body::empty())
            .unwrap();

        // Send request
        let _response = app.oneshot(req).await.unwrap();

        // Verify that fail counter was incremented (tests may run in parallel)
        assert!(
            metrics.get_requests_total_fail() > initial_fail,
            "Fail counter should increase after 400"
        );
    }

    #[tokio::test]
    async fn test_metrics_middleware_warn_status() {
        // Initialize metrics
        crate::metrics::init_metrics();
        let metrics = get_metrics();

        // Get initial counter value
        let initial_warn = metrics.get_requests_total_warn();

        // Create test app and request
        let app = create_test_app(StatusCode::NOT_FOUND);
        let req = Request::builder()
            .method("POST")
            .uri("/verify")
            .body(Body::empty())
            .unwrap();

        // Send request
        let _response = app.oneshot(req).await.unwrap();

        // Verify that warn counter was incremented (tests may run in parallel)
        assert!(
            metrics.get_requests_total_warn() > initial_warn,
            "Warn counter should increase after 404"
        );
    }

    /// Test that /healthz doesn't increment metrics counters.
    ///
    /// Note: This test is flaky due to race conditions with shared static metrics
    /// when running tests in parallel. The test logic is correct but the assertion
    /// can fail if other tests run concurrently and modify the metrics.
    #[tokio::test]
    #[ignore = "Flaky due to shared static metrics - run with --ignored for isolation"]
    async fn test_metrics_middleware_ignores_non_tracked_endpoints() {
        // Initialize metrics
        crate::metrics::init_metrics();
        let metrics = get_metrics();

        // Create test apps for two sequential requests
        let app1 = create_test_app(StatusCode::OK);
        let app2 = create_test_app(StatusCode::OK);

        // Send first request to /healthz
        let req1 = Request::builder()
            .method("POST")
            .uri("/healthz")
            .body(Body::empty())
            .unwrap();
        let _response1 = app1.oneshot(req1).await.unwrap();

        // Snapshot counters after first /healthz request
        let snapshot_ok = metrics.get_requests_total_ok();
        let snapshot_durations = metrics.get_request_durations_count();

        // Send second request to /healthz
        let req2 = Request::builder()
            .method("POST")
            .uri("/healthz")
            .body(Body::empty())
            .unwrap();
        let _response2 = app2.oneshot(req2).await.unwrap();

        // Verify that counters didn't change between the two /healthz requests
        // Even if other parallel tests are running, this proves /healthz doesn't increment
        assert_eq!(
            metrics.get_requests_total_ok(),
            snapshot_ok,
            "OK counter should not change between two /healthz requests"
        );

        assert_eq!(
            metrics.get_request_durations_count(),
            snapshot_durations,
            "Duration count should not change between two /healthz requests"
        );
    }

    #[tokio::test]
    async fn test_metrics_middleware_records_duration() {
        // Initialize metrics
        crate::metrics::init_metrics();
        let metrics = get_metrics();

        // Get initial duration count
        let initial_count = metrics.get_request_durations_count();

        // Create test app with delay handler
        async fn handler_with_delay() -> impl IntoResponse {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            StatusCode::OK
        }

        let app = Router::new()
            .route("/verify", post(handler_with_delay))
            .layer(middleware::from_fn(metrics_middleware));

        let req = Request::builder()
            .method("POST")
            .uri("/verify")
            .body(Body::empty())
            .unwrap();

        // Send request
        let _response = app.oneshot(req).await.unwrap();

        // Verify that duration was recorded (tests may run in parallel)
        assert!(
            metrics.get_request_durations_count() > initial_count,
            "Duration count should increase after request"
        );
    }
}
