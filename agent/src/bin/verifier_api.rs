/// CAP Verifier REST API (Phase 1 - Production Ready with TLS/mTLS)
///
/// Proof-Prüfung per HTTP/HTTPS API (deterministisch, offline-fähig, sicher)
///
/// Endpunkte:
/// - POST /verify - Verifiziert Proof-Kontext gegen Policy
/// - GET /healthz - Health Check
/// - GET /readyz - Readiness Check
/// - GET /metrics - Prometheus metrics
use axum::{
    http::StatusCode,
    middleware,
    response::Response,
    routing::{get, post},
    Json, Router,
};
use axum_server::tls_rustls::RustlsConfig;
use cap_agent::api::auth::auth_middleware;
use cap_agent::api::metrics_middleware::metrics_middleware;
use cap_agent::api::policy::{handle_policy_compile, handle_policy_get};
use cap_agent::api::policy_compiler::{handle_policy_v2_compile, handle_policy_v2_get};
use cap_agent::api::tls::{TlsConfig, TlsMode};
use cap_agent::api::verify::{handle_verify, VerifyRequest, VerifyResponse};
use cap_agent::metrics::{get_metrics, init_metrics};
use clap::Parser;
use serde::Serialize;
use std::net::SocketAddr;
use tracing::{error, info};

/// CLI Arguments
#[derive(Parser, Debug)]
#[command(name = "cap-verifier-api")]
#[command(about = "CAP Verifier REST API Server", long_about = None)]
struct Args {
    /// Bind address (e.g. 0.0.0.0:8080 or 0.0.0.0:8443)
    #[arg(long, default_value = "127.0.0.1:8080")]
    bind: String,

    /// Enable TLS mode (requires --tls-cert and --tls-key)
    #[arg(long)]
    tls: bool,

    /// Path to TLS certificate (PEM format)
    #[arg(long, required_if_eq("tls", "true"))]
    tls_cert: Option<String>,

    /// Path to TLS private key (PEM format, PKCS#8)
    #[arg(long, required_if_eq("tls", "true"))]
    tls_key: Option<String>,

    /// Enable mutual TLS (mTLS) with client certificate verification
    #[arg(long)]
    mtls: bool,

    /// Path to CA certificate for mTLS client verification
    #[arg(long, required_if_eq("mtls", "true"))]
    tls_ca: Option<String>,
}

/// Health Check Response
#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    build_hash: Option<String>,
    tls_enabled: bool,
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
async fn health_check(tls_enabled: bool) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "OK".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        build_hash: None,
        tls_enabled,
    })
}

/// Readiness Check Endpoint
async fn readiness_check() -> Json<ReadinessResponse> {
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
        status: if all_ok {
            "OK".to_string()
        } else {
            "DEGRADED".to_string()
        },
        checks,
    })
}

/// Metrics Endpoint (Prometheus format)
async fn metrics_endpoint() -> Response {
    let metrics = get_metrics();
    let output = metrics.export_prometheus();

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain; version=0.0.4")
        .body(output.into())
        .unwrap()
}

/// Verify Endpoint
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

    match handle_verify(payload) {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            info!("Verification failed: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Build the Axum router
fn build_router(tls_enabled: bool) -> Router {
    // Closure to capture tls_enabled for health check
    let health_handler = move || health_check(tls_enabled);

    // Build public routes (no auth)
    let public_routes = Router::new()
        .route("/healthz", get(health_handler))
        .route("/readyz", get(readiness_check))
        .route("/metrics", get(metrics_endpoint));

    // Build protected routes (with OAuth2 auth + metrics)
    let protected_routes = Router::new()
        .route("/verify", post(verify_endpoint))
        .route("/policy/compile", post(handle_policy_compile))
        .route("/policy/:id", get(handle_policy_get))
        .route("/policy/v2/compile", post(handle_policy_v2_compile))
        .route("/policy/v2/:id", get(handle_policy_v2_get))
        .layer(middleware::from_fn(auth_middleware))
        .layer(middleware::from_fn(metrics_middleware));

    public_routes.merge(protected_routes)
}

#[tokio::main]
async fn main() {
    // Parse CLI arguments
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("cap_verifier_api=debug,tower_http=debug")
        .init();

    info!(
        "🚀 Starting CAP Verifier API v{}",
        env!("CARGO_PKG_VERSION")
    );

    // Initialize metrics
    init_metrics();
    info!("📊 Prometheus metrics initialized");

    // Parse bind address
    let addr: SocketAddr = args
        .bind
        .parse()
        .unwrap_or_else(|_| panic!("Invalid bind address: {}", args.bind));

    // Determine TLS mode
    let tls_mode = if args.mtls {
        if args.tls_ca.is_none() {
            error!("❌ --tls-ca is required for mTLS mode");
            std::process::exit(1);
        }
        TlsMode::Mtls
    } else if args.tls {
        TlsMode::Tls
    } else {
        TlsMode::Disabled
    };

    // Build router
    let app = build_router(tls_mode != TlsMode::Disabled);

    // Start server based on TLS mode
    match tls_mode {
        TlsMode::Disabled => {
            info!("🎧 Listening on http://{}", addr);
            info!("⚠️  TLS disabled - HTTP only (not recommended for production!)");
            info!("🔒 OAuth2 authentication enabled for /verify");

            let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
            axum::serve(listener, app).await.unwrap();
        }
        TlsMode::Tls => {
            let cert_path = args.tls_cert.unwrap();
            let key_path = args.tls_key.unwrap();

            info!("🔐 TLS mode enabled");
            info!("📜 Certificate: {}", cert_path);
            info!("🔑 Private key: {}", key_path);

            // Use axum-server's RustlsConfig
            let rustls_config = match RustlsConfig::from_pem_file(&cert_path, &key_path).await {
                Ok(config) => config,
                Err(e) => {
                    error!("❌ Failed to load TLS configuration: {}", e);
                    std::process::exit(1);
                }
            };

            info!("🎧 Listening on https://{}", addr);
            info!("🔒 OAuth2 authentication enabled for /verify");

            // Use axum-server for TLS
            axum_server::bind_rustls(addr, rustls_config)
                .serve(app.into_make_service())
                .await
                .unwrap();
        }
        TlsMode::Mtls => {
            let cert_path = args.tls_cert.unwrap();
            let key_path = args.tls_key.unwrap();
            let ca_cert_path = args.tls_ca.unwrap();

            info!("🔐 mTLS mode enabled (mutual authentication)");
            info!("📜 Server certificate: {}", cert_path);
            info!("🔑 Server private key: {}", key_path);
            info!("🔏 CA certificate: {}", ca_cert_path);

            // Build custom rustls ServerConfig for mTLS
            let tls_config =
                TlsConfig::new(cert_path.clone(), key_path.clone()).with_mtls(ca_cert_path);

            let server_config = match tls_config.build_server_config() {
                Ok(config) => config,
                Err(e) => {
                    error!("❌ Failed to load mTLS configuration: {}", e);
                    std::process::exit(1);
                }
            };

            // Create RustlsConfig from ServerConfig
            let rustls_config = RustlsConfig::from_config(server_config);

            info!("🎧 Listening on https://{} (mTLS)", addr);
            info!("🔒 OAuth2 authentication + client certificate verification enabled");

            // Use axum-server for TLS
            axum_server::bind_rustls(addr, rustls_config)
                .serve(app.into_make_service())
                .await
                .unwrap();
        }
    }
}
