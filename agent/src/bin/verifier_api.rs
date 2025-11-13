/// CAP Verifier REST API
///
/// Proof-Prüfung per HTTP API (deterministisch, offline-fähig, sicher)
///
/// Endpunkte:
/// - POST /verify - Verifiziert Proof-Kontext gegen Policy
/// - GET /healthz - Health Check
/// - GET /readyz - Readiness Check

use axum::{
    routing::{get, post},
    Router,
    Json,
    http::StatusCode,
    middleware,
    response::Response,
};
use serde::{Serialize};
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber;
use cap_agent::api::verify::{VerifyRequest, VerifyResponse, handle_verify};
use cap_agent::api::auth::auth_middleware;
use cap_agent::api::policy::{handle_policy_compile, handle_policy_get};
use cap_agent::api::policy_compiler::{handle_policy_v2_compile, handle_policy_v2_get};
use cap_agent::api::metrics_middleware::metrics_middleware;
use cap_agent::metrics::{init_metrics, get_metrics};

/// Health Check Response
#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    build_hash: Option<String>,
}

/// Readiness Check Response
#[derive(Debug, Serialize)]
struct ReadinessResponse {
    status: String,
    checks: Vec<ReadinessCheck>,
}

#[derive(Debug, Serialize)]
struct ReadinessCheck {
    name: String,
    status: String,
}

/// Health Check Endpoint
async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "OK".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        build_hash: None, // TODO: Add git commit hash at build time
    })
}

/// Readiness Check Endpoint
async fn readiness_check() -> Json<ReadinessResponse> {
    // Check if all dependencies are ready
    let checks = vec![
        ReadinessCheck {
            name: "verifier_core".to_string(),
            status: "OK".to_string(),
        },
        ReadinessCheck {
            name: "crypto".to_string(),
            status: "OK".to_string(),
        },
    ];

    let all_ok = checks.iter().all(|c| c.status == "OK");

    Json(ReadinessResponse {
        status: if all_ok { "OK".to_string() } else { "DEGRADED".to_string() },
        checks,
    })
}

/// Metrics Endpoint (Prometheus format)
async fn metrics_endpoint() -> Response {
    let metrics = get_metrics();
    let output = metrics.export_prometheus();

    // Return as text/plain with Prometheus format
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain; version=0.0.4")
        .body(output.into())
        .unwrap()
}

/// Verify Endpoint - Full implementation with core verification
async fn verify_endpoint(
    Json(payload): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, StatusCode> {
    let mode = if payload.policy_id.is_some() {
        "policy_id"
    } else if payload.ir.is_some() {
        "embedded_ir"
    } else {
        "unknown"
    };
    info!("Received verify request (mode: {})", mode);

    // Call the handler from api::verify
    match handle_verify(payload) {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            info!("Verification failed: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("cap_verifier_api=debug,tower_http=debug")
        .init();

    info!("🚀 Starting CAP Verifier API v{}", env!("CARGO_PKG_VERSION"));

    // Initialize metrics registry
    init_metrics();
    info!("📊 Prometheus metrics initialized");

    // Build public routes (no auth)
    let public_routes = Router::new()
        .route("/healthz", get(health_check))
        .route("/readyz", get(readiness_check))
        .route("/metrics", get(metrics_endpoint));

    // Build protected routes (with OAuth2 auth + metrics)
    // IMPORTANT: Layer order matters! Middleware runs in REVERSE order:
    // 1. metrics_middleware runs first (outermost layer)
    // 2. auth_middleware runs second (innermost layer)
    let protected_routes = Router::new()
        .route("/verify", post(verify_endpoint))
        // Legacy Policy API (v1)
        .route("/policy/compile", post(handle_policy_compile))
        .route("/policy/:id", get(handle_policy_get))
        // PolicyV2 Compiler API (Week 3)
        .route("/policy/v2/compile", post(handle_policy_v2_compile))
        .route("/policy/v2/:id", get(handle_policy_v2_get))
        .layer(middleware::from_fn(auth_middleware))
        .layer(middleware::from_fn(metrics_middleware));

    // Combine routers
    let app = public_routes.merge(protected_routes);

    // Bind to port 8080 (Phase 4 will add TLS on 8443 for production)
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("🎧 Listening on http://{}", addr);
    info!("🔒 OAuth2 authentication enabled for /verify");

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .await
        .unwrap();
}
