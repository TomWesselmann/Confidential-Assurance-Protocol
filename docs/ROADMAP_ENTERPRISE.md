# CAP-Agent Enterprise Roadmap

**Version:** 1.0
**Erstellt:** 2025-12-02
**Basiert auf:** Security Audit Report v0.1.0
**Ziel:** Enterprise-Ready Production Deployment

---

## Executive Summary

Diese Roadmap definiert den Weg von der aktuellen Development-Ready Version (57% Enterprise Score) zur vollständigen Enterprise-Readiness (95%+). Der Plan ist in 4 Phasen unterteilt, die aufeinander aufbauen.

### Meilensteine

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        ENTERPRISE READINESS ROADMAP                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  HEUTE          PHASE 1        PHASE 2        PHASE 3        PHASE 4        │
│    │              │              │              │              │             │
│    ▼              ▼              ▼              ▼              ▼             │
│  [57%] ───────► [72%] ───────► [85%] ───────► [92%] ───────► [95%+]        │
│    │              │              │              │              │             │
│  DEV-READY    PRODUCTION    ENTERPRISE    HA-READY     PREMIUM            │
│               DEPLOYABLE      READY                                         │
│                                                                              │
│  Aktuell       +2 Wochen     +6 Wochen    +10 Wochen   +14 Wochen          │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Security Hardening (Wochen 1-2)

**Ziel:** Production-Deployable (72% Enterprise Score)
**Fokus:** Kritische Sicherheitslücken schließen

### Sprint 1.1 - Kritische Fixes (Woche 1)

#### 1.1.1 CORS Whitelist Implementation

**Priorität:** KRITISCH
**Datei:** `src/bin/verifier_api.rs`
**Aufwand:** 4 Stunden

**Aktueller Zustand:**
```rust
let cors = CorsLayer::new()
    .allow_origin(Any)  // KRITISCH: Erlaubt alle Origins
```

**Ziel-Implementierung:**
```rust
use tower_http::cors::{AllowOrigin, CorsLayer};
use axum::http::{header, HeaderValue, Method};

fn build_cors_layer() -> CorsLayer {
    // Load allowed origins from environment or config
    let allowed_origins: Vec<HeaderValue> = std::env::var("CORS_ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "https://cap-verifier.example.com".to_string())
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed_origins))
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
        ])
        .allow_credentials(true)
        .max_age(Duration::from_secs(3600))
}
```

**Acceptance Criteria:**
- [ ] CORS Origins aus Environment Variable geladen
- [ ] Nur explizit konfigurierte Origins erlaubt
- [ ] Unit Tests für CORS Rejection
- [ ] Integration Test mit ungültiger Origin

---

#### 1.1.2 Security Headers Middleware

**Priorität:** KRITISCH
**Neue Datei:** `src/api/security_headers.rs`
**Aufwand:** 6 Stunden

**Implementierung:**
```rust
use axum::{
    http::{header, HeaderValue, Request, Response},
    middleware::Next,
};

pub async fn security_headers_middleware<B>(
    request: Request<B>,
    next: Next<B>,
) -> Response<B> {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // HSTS - Force HTTPS
    headers.insert(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains; preload")
    );

    // Prevent Clickjacking
    headers.insert(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY")
    );

    // Prevent MIME Sniffing
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff")
    );

    // XSS Protection (Legacy browsers)
    headers.insert(
        HeaderName::from_static("x-xss-protection"),
        HeaderValue::from_static("1; mode=block")
    );

    // Referrer Policy
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin")
    );

    // Content Security Policy
    headers.insert(
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static(
            "default-src 'self'; \
             script-src 'self' 'unsafe-inline'; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' data:; \
             font-src 'self'; \
             frame-ancestors 'none'; \
             form-action 'self'"
        )
    );

    // Permissions Policy
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static(
            "accelerometer=(), camera=(), geolocation=(), gyroscope=(), \
             magnetometer=(), microphone=(), payment=(), usb=()"
        )
    );

    response
}
```

**Acceptance Criteria:**
- [ ] Alle Security Headers werden gesetzt
- [ ] Headers in /healthz Response verifiziert
- [ ] Security Scanner (Mozilla Observatory) Score A+
- [ ] Dokumentation der Header-Policies

---

#### 1.1.3 OAuth2 Key Management Refactoring

**Priorität:** KRITISCH
**Datei:** `src/api/auth.rs`
**Aufwand:** 8 Stunden

**Änderungen:**

1. **Mock Keys entfernen:**
```rust
// ENTFERNEN:
// const MOCK_PUBLIC_KEY: &str = ...
// const MOCK_PRIVATE_KEY: &str = ...
```

2. **Environment-basierte Konfiguration:**
```rust
#[derive(Debug, Clone)]
pub struct OAuth2Config {
    pub issuer: String,
    pub audience: String,
    pub public_key_pem: String,
    pub jwks_url: Option<String>,
    pub dev_mode: bool,
}

impl OAuth2Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let dev_mode = std::env::var("CAP_ENVIRONMENT")
            .map(|v| v == "development")
            .unwrap_or(false);

        Ok(Self {
            issuer: std::env::var("OAUTH2_ISSUER")
                .map_err(|_| ConfigError::MissingEnv("OAUTH2_ISSUER"))?,
            audience: std::env::var("OAUTH2_AUDIENCE")
                .map_err(|_| ConfigError::MissingEnv("OAUTH2_AUDIENCE"))?,
            public_key_pem: std::env::var("OAUTH2_PUBLIC_KEY")
                .map_err(|_| ConfigError::MissingEnv("OAUTH2_PUBLIC_KEY"))?,
            jwks_url: std::env::var("OAUTH2_JWKS_URL").ok(),
            dev_mode,
        })
    }
}
```

3. **Development Token Guard:**
```rust
#[cfg(feature = "development")]
fn check_dev_token(token: &str) -> Option<Claims> {
    if let Ok(dev_token) = std::env::var("CAP_DEV_TOKEN") {
        if !dev_token.is_empty() && token == dev_token {
            tracing::warn!("⚠️ DEV MODE: Development token used");
            return Some(Claims::dev_claims());
        }
    }
    None
}

#[cfg(not(feature = "development"))]
fn check_dev_token(_token: &str) -> Option<Claims> {
    None  // Completely disabled in production builds
}
```

**Acceptance Criteria:**
- [ ] Keine hardcoded Keys im Code
- [ ] `cargo build --release` hat kein dev_token
- [ ] Environment Variables dokumentiert
- [ ] Integration Test mit echtem JWT

---

### Sprint 1.2 - Resilience Basics (Woche 2)

#### 1.2.1 Graceful Shutdown Implementation

**Priorität:** HOCH
**Datei:** `src/bin/verifier_api.rs`
**Aufwand:** 4 Stunden

**Implementierung:**
```rust
use tokio::signal;
use std::time::Duration;

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, initiating graceful shutdown");
        }
        _ = terminate => {
            tracing::info!("Received SIGTERM, initiating graceful shutdown");
        }
    }
}

// In main():
#[tokio::main]
async fn main() {
    // ... setup code ...

    let server = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal());

    tracing::info!("Server starting on {}", addr);

    if let Err(e) = server.await {
        tracing::error!("Server error: {}", e);
    }

    // Drain period for in-flight requests
    tracing::info!("Waiting for in-flight requests to complete...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    tracing::info!("Shutdown complete");
}
```

**Acceptance Criteria:**
- [ ] SIGTERM führt zu Graceful Shutdown
- [ ] In-flight Requests werden abgeschlossen
- [ ] Kubernetes Readiness Probe wird `false`
- [ ] Keine Datenverluste bei Restart

---

#### 1.2.2 Request Timeout Middleware

**Priorität:** HOCH
**Neue Datei:** `src/api/timeout.rs`
**Aufwand:** 3 Stunden

**Implementierung:**
```rust
use axum::{
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
};
use std::time::Duration;
use tower::timeout::TimeoutLayer;

pub fn request_timeout_layer() -> TimeoutLayer {
    TimeoutLayer::new(Duration::from_secs(30))
}

pub async fn timeout_error_handler() -> impl IntoResponse {
    (
        StatusCode::GATEWAY_TIMEOUT,
        "Request timeout - operation took too long",
    )
}

// Differenzierte Timeouts pro Endpoint-Typ
pub struct TimeoutConfig {
    pub default: Duration,
    pub verify: Duration,
    pub policy_compile: Duration,
    pub health_check: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            default: Duration::from_secs(30),
            verify: Duration::from_secs(60),      // WASM execution
            policy_compile: Duration::from_secs(120), // Complex policies
            health_check: Duration::from_secs(5),
        }
    }
}
```

**Acceptance Criteria:**
- [ ] Default Timeout: 30 Sekunden
- [ ] /verify Timeout: 60 Sekunden
- [ ] /policy/compile Timeout: 120 Sekunden
- [ ] Timeout-Metrik in Prometheus

---

#### 1.2.3 Per-Endpoint Rate Limiting

**Priorität:** HOCH
**Datei:** `src/api/rate_limit.rs`
**Aufwand:** 4 Stunden

**Implementierung:**
```rust
use std::collections::HashMap;

pub struct EndpointRateLimits {
    limits: HashMap<String, RateLimitConfig>,
    default: RateLimitConfig,
}

impl EndpointRateLimits {
    pub fn new() -> Self {
        let mut limits = HashMap::new();

        // Health/Metrics: Unlimited (internal)
        limits.insert("/healthz".to_string(), RateLimitConfig {
            requests_per_minute: 1000,
            burst_size: 1000,
        });

        // Verify: Moderate (CPU-intensive)
        limits.insert("/verify".to_string(), RateLimitConfig {
            requests_per_minute: 30,
            burst_size: 40,
        });

        // Policy Compile: Strict (very CPU-intensive)
        limits.insert("/policy/compile".to_string(), RateLimitConfig {
            requests_per_minute: 10,
            burst_size: 15,
        });
        limits.insert("/policy/v2/compile".to_string(), RateLimitConfig {
            requests_per_minute: 10,
            burst_size: 15,
        });

        // Upload: Moderate
        limits.insert("/proof/upload".to_string(), RateLimitConfig {
            requests_per_minute: 20,
            burst_size: 25,
        });

        Self {
            limits,
            default: RateLimitConfig::default_global(),
        }
    }

    pub fn get_limit(&self, path: &str) -> &RateLimitConfig {
        self.limits.get(path).unwrap_or(&self.default)
    }
}
```

**Acceptance Criteria:**
- [ ] /verify: 30 req/min
- [ ] /policy/compile: 10 req/min
- [ ] /proof/upload: 20 req/min
- [ ] Rate Limit Tests pro Endpoint

---

### Phase 1 Deliverables

| Deliverable | Status | Verantwortlich |
|-------------|--------|----------------|
| CORS Whitelist | ⬜ | Security |
| Security Headers | ⬜ | Security |
| OAuth2 Refactoring | ⬜ | Auth |
| Graceful Shutdown | ⬜ | Platform |
| Request Timeouts | ⬜ | Platform |
| Per-Endpoint Rate Limits | ⬜ | API |
| **Phase 1 Review** | ⬜ | Team |

**Exit Criteria Phase 1:**
- [ ] Security Scanner Score: A oder besser
- [ ] Keine kritischen Findings im Audit
- [ ] Alle Unit Tests bestehen
- [ ] Load Test: 100 req/s ohne Fehler
- [ ] Enterprise Score: ≥72%

---

## Phase 2: Enterprise Security (Wochen 3-6)

**Ziel:** Enterprise-Ready (85% Enterprise Score)
**Fokus:** Vollständige Security-Compliance

### Sprint 2.1 - Advanced Authentication (Woche 3)

#### 2.1.1 JWKS Endpoint Support

**Priorität:** HOCH
**Neue Datei:** `src/api/jwks.rs`
**Aufwand:** 16 Stunden

**Implementierung:**
```rust
use jsonwebtoken::{DecodingKey, jwk::JwkSet};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

pub struct JwksProvider {
    client: Client,
    jwks_url: String,
    cache: Arc<RwLock<JwksCache>>,
    cache_ttl: Duration,
}

struct JwksCache {
    jwks: Option<JwkSet>,
    last_refresh: Option<Instant>,
}

impl JwksProvider {
    pub fn new(jwks_url: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            jwks_url,
            cache: Arc::new(RwLock::new(JwksCache {
                jwks: None,
                last_refresh: None,
            })),
            cache_ttl: Duration::from_secs(300), // 5 minutes
        }
    }

    pub async fn get_key(&self, kid: &str) -> Result<DecodingKey, JwksError> {
        let cache = self.cache.read().await;

        // Check if cache is valid
        if let (Some(jwks), Some(last_refresh)) = (&cache.jwks, cache.last_refresh) {
            if last_refresh.elapsed() < self.cache_ttl {
                return self.find_key(jwks, kid);
            }
        }
        drop(cache);

        // Refresh JWKS
        self.refresh_jwks().await?;

        let cache = self.cache.read().await;
        if let Some(jwks) = &cache.jwks {
            self.find_key(jwks, kid)
        } else {
            Err(JwksError::NoKeys)
        }
    }

    async fn refresh_jwks(&self) -> Result<(), JwksError> {
        let response = self.client
            .get(&self.jwks_url)
            .send()
            .await
            .map_err(|e| JwksError::FetchError(e.to_string()))?;

        let jwks: JwkSet = response
            .json()
            .await
            .map_err(|e| JwksError::ParseError(e.to_string()))?;

        let mut cache = self.cache.write().await;
        cache.jwks = Some(jwks);
        cache.last_refresh = Some(Instant::now());

        tracing::info!("JWKS refreshed successfully");
        Ok(())
    }

    fn find_key(&self, jwks: &JwkSet, kid: &str) -> Result<DecodingKey, JwksError> {
        jwks.keys
            .iter()
            .find(|key| key.common.key_id.as_deref() == Some(kid))
            .map(|jwk| DecodingKey::from_jwk(jwk))
            .transpose()
            .map_err(|e| JwksError::InvalidKey(e.to_string()))?
            .ok_or(JwksError::KeyNotFound(kid.to_string()))
    }
}
```

**Acceptance Criteria:**
- [ ] JWKS von externem IdP geladen
- [ ] Key-Caching mit 5 Minuten TTL
- [ ] Automatische Rotation bei Key-Miss
- [ ] Metriken für JWKS Refreshes

---

#### 2.1.2 Token Introspection (Optional)

**Priorität:** MITTEL
**Aufwand:** 8 Stunden

Für Token-Revocation Support (RFC 7662):

```rust
pub struct TokenIntrospector {
    introspection_url: String,
    client_id: String,
    client_secret: String,
}

impl TokenIntrospector {
    pub async fn introspect(&self, token: &str) -> Result<IntrospectionResponse, AuthError> {
        let response = self.client
            .post(&self.introspection_url)
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&[("token", token)])
            .send()
            .await?;

        let intro: IntrospectionResponse = response.json().await?;

        if !intro.active {
            return Err(AuthError::TokenRevoked);
        }

        Ok(intro)
    }
}
```

---

### Sprint 2.2 - Input Validation & Error Handling (Woche 4)

#### 2.2.1 Request Validation Layer

**Priorität:** HOCH
**Neue Datei:** `src/api/validation.rs`
**Aufwand:** 12 Stunden

**Implementierung:**
```rust
use serde::Deserialize;
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Validate)]
pub struct VerifyRequestValidated {
    #[validate(length(max = 100), custom = "validate_uuid_format")]
    pub policy_id: Option<String>,

    #[validate(length(max = 10_485_760))] // 10MB max
    pub proof_base64: Option<String>,

    #[validate(length(max = 1_048_576))] // 1MB max
    pub ir: Option<String>,

    #[validate(custom = "validate_proof_context")]
    pub context: Option<ProofContext>,
}

fn validate_uuid_format(value: &str) -> Result<(), ValidationError> {
    uuid::Uuid::parse_str(value)
        .map(|_| ())
        .map_err(|_| ValidationError::new("invalid_uuid"))
}

fn validate_proof_context(ctx: &ProofContext) -> Result<(), ValidationError> {
    // Custom validation logic
    if ctx.claims.len() > 1000 {
        return Err(ValidationError::new("too_many_claims"));
    }
    Ok(())
}

// Validation middleware
pub async fn validate_request<T: Validate>(
    Json(payload): Json<T>,
) -> Result<Json<T>, (StatusCode, Json<ValidationErrorResponse>)> {
    payload.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ValidationErrorResponse {
                error: "validation_failed".to_string(),
                details: e.field_errors()
                    .into_iter()
                    .map(|(field, errors)| (field.to_string(), errors))
                    .collect(),
            }),
        )
    })?;

    Ok(Json(payload))
}
```

---

#### 2.2.2 Error Handling Refactoring

**Priorität:** HOCH
**Aufwand:** 24 Stunden

**Ziel:** Alle `unwrap()`/`expect()` durch proper Error Handling ersetzen.

**Strategie:**

1. **Custom Error Type:**
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Authentication failed: {0}")]
    Auth(#[from] AuthError),

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Policy not found: {0}")]
    PolicyNotFound(String),

    #[error("Verification failed: {0}")]
    Verification(#[from] VerifyError),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Rate limit exceeded")]
    RateLimited,

    #[error("Request timeout")]
    Timeout,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match &self {
            ApiError::Auth(_) => (StatusCode::UNAUTHORIZED, "AUTH_ERROR", self.to_string()),
            ApiError::Validation(_) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", self.to_string()),
            ApiError::PolicyNotFound(_) => (StatusCode::NOT_FOUND, "NOT_FOUND", self.to_string()),
            ApiError::Verification(_) => (StatusCode::UNPROCESSABLE_ENTITY, "VERIFY_ERROR", self.to_string()),
            ApiError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "Internal server error".to_string()),
            ApiError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMITED", self.to_string()),
            ApiError::Timeout => (StatusCode::GATEWAY_TIMEOUT, "TIMEOUT", self.to_string()),
        };

        let body = Json(ErrorResponse {
            error: error_code.to_string(),
            message,
            timestamp: Utc::now(),
        });

        (status, body).into_response()
    }
}
```

2. **Refactoring Pattern:**
```rust
// Vorher:
let config = build_config().expect("Failed to build config");

// Nachher:
let config = build_config()
    .map_err(|e| ApiError::Internal(format!("Config error: {}", e)))?;
```

**Acceptance Criteria:**
- [ ] 0 `unwrap()` in Production Code
- [ ] 0 `expect()` in Production Code
- [ ] Alle Errors haben Error Codes
- [ ] Error Responses sind JSON-konform

---

### Sprint 2.3 - TLS Hardening (Woche 5)

#### 2.3.1 TLS 1.3 Only Mode

**Priorität:** MITTEL
**Datei:** `src/api/tls.rs`
**Aufwand:** 4 Stunden

```rust
use rustls::version::TLS13;

fn build_tls_config(&self) -> Result<Arc<ServerConfig>, TlsError> {
    let certs = load_certs(&self.cert_path)?;
    let key = load_private_key(&self.key_path)?;

    // Enforce TLS 1.3 only
    let config = ServerConfig::builder_with_provider(
        rustls::crypto::ring::default_provider().into()
    )
    .with_protocol_versions(&[&TLS13])
    .map_err(|e| TlsError::InvalidCert(format!("TLS 1.3 not supported: {}", e)))?
    .with_no_client_auth()
    .with_single_cert(certs, key)
    .map_err(|e| TlsError::InvalidCert(e.to_string()))?;

    Ok(Arc::new(config))
}
```

---

#### 2.3.2 Certificate Monitoring

**Priorität:** MITTEL
**Aufwand:** 8 Stunden

```rust
pub struct CertificateMonitor {
    cert_path: PathBuf,
    last_check: Instant,
    expiry: DateTime<Utc>,
}

impl CertificateMonitor {
    pub fn check_expiry(&self) -> CertStatus {
        let days_until_expiry = (self.expiry - Utc::now()).num_days();

        match days_until_expiry {
            d if d <= 0 => CertStatus::Expired,
            d if d <= 7 => CertStatus::Critical(d),
            d if d <= 30 => CertStatus::Warning(d),
            d => CertStatus::Ok(d),
        }
    }
}

// Prometheus Metric
pub fn cert_expiry_days() -> Gauge {
    // cap_cert_expiry_days{path="/certs/server.crt"} 45
}
```

---

### Sprint 2.4 - Security Audit Logging (Woche 6)

#### 2.4.1 Security Event Logger

**Priorität:** HOCH
**Neue Datei:** `src/audit/security_log.rs`
**Aufwand:** 16 Stunden

```rust
use serde::Serialize;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize)]
pub struct SecurityEvent {
    pub timestamp: DateTime<Utc>,
    pub event_id: String,
    pub event_type: SecurityEventType,
    pub actor: Actor,
    pub resource: String,
    pub action: String,
    pub result: EventResult,
    pub metadata: serde_json::Value,
    pub request_id: String,
    pub client_ip: String,
}

#[derive(Debug, Serialize)]
pub enum SecurityEventType {
    // Authentication Events
    AuthSuccess,
    AuthFailure,
    TokenRefresh,
    TokenRevoked,

    // Authorization Events
    AuthzGranted,
    AuthzDenied,

    // Rate Limiting
    RateLimitWarning,
    RateLimitExceeded,

    // Data Events
    PolicyCreated,
    PolicyUpdated,
    PolicyDeleted,
    VerificationSuccess,
    VerificationFailure,

    // System Events
    ConfigChanged,
    ServiceStarted,
    ServiceStopped,
    CertRotated,
}

pub struct SecurityAuditLog {
    writer: Box<dyn Write + Send + Sync>,
}

impl SecurityAuditLog {
    pub fn log(&self, event: SecurityEvent) {
        let json = serde_json::to_string(&event).unwrap();
        writeln!(self.writer, "{}", json).ok();

        // Also emit as structured log
        tracing::info!(
            event_type = ?event.event_type,
            actor = %event.actor.id,
            resource = %event.resource,
            result = ?event.result,
            "Security event"
        );
    }
}
```

**Acceptance Criteria:**
- [ ] Alle Auth-Events geloggt
- [ ] Alle Authz-Decisions geloggt
- [ ] JSON-Format für SIEM-Integration
- [ ] Request-ID Correlation

---

### Phase 2 Deliverables

| Deliverable | Status | Verantwortlich |
|-------------|--------|----------------|
| JWKS Support | ⬜ | Auth |
| Token Introspection | ⬜ | Auth |
| Input Validation | ⬜ | API |
| Error Handling | ⬜ | Platform |
| TLS 1.3 Only | ⬜ | Security |
| Cert Monitoring | ⬜ | Ops |
| Security Audit Log | ⬜ | Security |
| **Phase 2 Review** | ⬜ | Team |

**Exit Criteria Phase 2:**
- [ ] SOC 2 Controls: 80% implemented
- [ ] Alle OWASP Top 10 addressed
- [ ] Security Audit Log active
- [ ] Penetration Test: No Critical/High
- [ ] Enterprise Score: ≥85%

---

## Phase 3: High Availability (Wochen 7-10)

**Ziel:** HA-Ready (92% Enterprise Score)
**Fokus:** Skalierbarkeit und Observability

### Sprint 3.1 - Distributed Rate Limiting (Woche 7)

#### 3.1.1 Redis-backed Rate Limiter

**Priorität:** HOCH
**Aufwand:** 16 Stunden

```rust
use redis::AsyncCommands;
use std::time::Duration;

pub struct RedisRateLimiter {
    client: redis::Client,
    prefix: String,
}

impl RedisRateLimiter {
    pub async fn check_rate_limit(
        &self,
        key: &str,
        limit: u32,
        window: Duration,
    ) -> Result<RateLimitResult, RateLimitError> {
        let mut conn = self.client.get_async_connection().await?;
        let full_key = format!("{}:{}", self.prefix, key);

        // Sliding window implementation
        let now = Utc::now().timestamp_millis();
        let window_start = now - window.as_millis() as i64;

        // Remove old entries
        conn.zremrangebyscore(&full_key, 0, window_start).await?;

        // Count current requests
        let count: u32 = conn.zcard(&full_key).await?;

        if count >= limit {
            return Ok(RateLimitResult::Exceeded {
                limit,
                remaining: 0,
                reset_at: now + window.as_millis() as i64,
            });
        }

        // Add new request
        conn.zadd(&full_key, now, uuid::Uuid::new_v4().to_string()).await?;
        conn.expire(&full_key, window.as_secs() as usize).await?;

        Ok(RateLimitResult::Allowed {
            limit,
            remaining: limit - count - 1,
            reset_at: now + window.as_millis() as i64,
        })
    }
}
```

---

### Sprint 3.2 - Observability Stack (Woche 8-9)

#### 3.2.1 OpenTelemetry Integration

**Priorität:** HOCH
**Aufwand:** 24 Stunden

```rust
use opentelemetry::{
    global,
    sdk::{trace, Resource},
    trace::{Tracer, TracerProvider},
};
use opentelemetry_otlp::WithExportConfig;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, Registry};

pub fn init_telemetry() -> Result<(), TelemetryError> {
    // Configure OTLP exporter
    let otlp_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:4317".to_string()));

    // Create tracer provider
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(otlp_exporter)
        .with_trace_config(
            trace::config()
                .with_resource(Resource::new(vec![
                    opentelemetry::KeyValue::new("service.name", "cap-verifier-api"),
                    opentelemetry::KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                ]))
                .with_sampler(trace::Sampler::TraceIdRatioBased(0.1)), // 10% sampling
        )
        .install_batch(opentelemetry::runtime::Tokio)?;

    // Setup tracing subscriber
    let telemetry_layer = OpenTelemetryLayer::new(tracer);

    let subscriber = Registry::default()
        .with(telemetry_layer)
        .with(tracing_subscriber::fmt::layer());

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

// Request tracing middleware
pub async fn tracing_middleware<B>(
    request: Request<B>,
    next: Next<B>,
) -> Response<B> {
    let trace_id = request
        .headers()
        .get("x-trace-id")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let span = tracing::info_span!(
        "http_request",
        trace_id = %trace_id,
        method = %request.method(),
        uri = %request.uri(),
    );

    let response = next.run(request).instrument(span).await;

    // Add trace ID to response headers
    let mut response = response;
    response.headers_mut().insert(
        "x-trace-id",
        HeaderValue::from_str(&trace_id).unwrap(),
    );

    response
}
```

---

#### 3.2.2 Enhanced Metrics

**Priorität:** HOCH
**Aufwand:** 12 Stunden

```rust
use prometheus::{
    Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramVec,
    Opts, Registry,
};

pub struct EnterpriseMetrics {
    // Request Metrics
    pub http_requests_total: CounterVec,
    pub http_request_duration_seconds: HistogramVec,
    pub http_request_size_bytes: HistogramVec,
    pub http_response_size_bytes: HistogramVec,

    // Business Metrics
    pub verifications_total: CounterVec,
    pub verification_duration_seconds: Histogram,
    pub policies_compiled_total: CounterVec,
    pub policy_compile_duration_seconds: Histogram,

    // System Metrics
    pub active_connections: Gauge,
    pub rate_limit_exceeded_total: CounterVec,
    pub auth_failures_total: CounterVec,

    // SLI Metrics
    pub sli_availability: Gauge,
    pub sli_latency_p99: Gauge,
    pub sli_error_rate: Gauge,
}

impl EnterpriseMetrics {
    pub fn new() -> Self {
        Self {
            http_requests_total: CounterVec::new(
                Opts::new("cap_http_requests_total", "Total HTTP requests"),
                &["method", "path", "status"],
            ).unwrap(),

            http_request_duration_seconds: HistogramVec::new(
                HistogramOpts::new(
                    "cap_http_request_duration_seconds",
                    "HTTP request duration in seconds"
                ).buckets(vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
                &["method", "path"],
            ).unwrap(),

            // ... more metrics
        }
    }
}
```

---

### Sprint 3.3 - Database Resilience (Woche 10)

#### 3.3.1 SQLite Connection Pooling

**Priorität:** MITTEL
**Aufwand:** 8 Stunden

```rust
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;

pub struct PooledPolicyStore {
    pool: Pool<SqliteConnectionManager>,
}

impl PooledPolicyStore {
    pub fn new(db_path: &str, pool_size: u32) -> Result<Self, PolicyError> {
        let manager = SqliteConnectionManager::file(db_path)
            .with_init(|conn| {
                conn.execute_batch("
                    PRAGMA journal_mode=WAL;
                    PRAGMA synchronous=NORMAL;
                    PRAGMA foreign_keys=ON;
                    PRAGMA busy_timeout=5000;
                ")?;
                Ok(())
            });

        let pool = Pool::builder()
            .max_size(pool_size)
            .min_idle(Some(2))
            .connection_timeout(Duration::from_secs(5))
            .idle_timeout(Some(Duration::from_secs(300)))
            .build(manager)
            .map_err(|e| PolicyError::ConnectionPool(e.to_string()))?;

        Ok(Self { pool })
    }

    fn get_conn(&self) -> Result<PooledConnection<SqliteConnectionManager>, PolicyError> {
        self.pool.get()
            .map_err(|e| PolicyError::ConnectionPool(e.to_string()))
    }
}
```

---

### Phase 3 Deliverables

| Deliverable | Status | Verantwortlich |
|-------------|--------|----------------|
| Redis Rate Limiter | ⬜ | Platform |
| OpenTelemetry | ⬜ | Observability |
| Enhanced Metrics | ⬜ | Observability |
| Connection Pooling | ⬜ | Database |
| Circuit Breaker | ⬜ | Resilience |
| **Phase 3 Review** | ⬜ | Team |

**Exit Criteria Phase 3:**
- [ ] Distributed Rate Limiting active
- [ ] Full Tracing in Jaeger/Tempo
- [ ] SLI Dashboards in Grafana
- [ ] 99.9% Availability in Load Test
- [ ] Enterprise Score: ≥92%

---

## Phase 4: Enterprise Premium (Wochen 11-14)

**Ziel:** Enterprise Premium (95%+ Score)
**Fokus:** Compliance & Advanced Features

### Sprint 4.1 - Compliance Features (Woche 11-12)

#### 4.1.1 SBOM Generation

```yaml
# .github/workflows/sbom.yml
name: Generate SBOM

on:
  push:
    branches: [main]
  release:
    types: [published]

jobs:
  sbom:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Generate CycloneDX SBOM
        run: |
          cargo install cargo-cyclonedx
          cargo cyclonedx --format json > sbom.cyclonedx.json

      - name: Upload SBOM
        uses: actions/upload-artifact@v4
        with:
          name: sbom
          path: sbom.cyclonedx.json

      - name: Publish to Dependency Track
        run: |
          curl -X POST "$DEPENDENCY_TRACK_URL/api/v1/bom" \
            -H "X-Api-Key: ${{ secrets.DTRACK_API_KEY }}" \
            -F "project=$PROJECT_UUID" \
            -F "bom=@sbom.cyclonedx.json"
```

---

#### 4.1.2 RFC 3161 Timestamp Verification

**Priorität:** MITTEL
**Aufwand:** 24 Stunden

```rust
use der::{Decode, Encode};
use pkcs7::ContentInfo;

pub struct TimestampVerifier {
    tsa_certs: Vec<Certificate>,
}

impl TimestampVerifier {
    pub fn verify_timestamp(
        &self,
        timestamp_token: &[u8],
        data_hash: &[u8],
    ) -> Result<TimestampInfo, TimestampError> {
        // Parse PKCS#7 ContentInfo
        let content_info = ContentInfo::from_der(timestamp_token)
            .map_err(|e| TimestampError::ParseError(e.to_string()))?;

        // Extract TSTInfo
        let tst_info = self.extract_tst_info(&content_info)?;

        // Verify message imprint matches
        if tst_info.message_imprint.hashed_message != data_hash {
            return Err(TimestampError::HashMismatch);
        }

        // Verify TSA signature
        self.verify_tsa_signature(&content_info)?;

        // Verify certificate chain
        self.verify_certificate_chain(&content_info)?;

        Ok(TimestampInfo {
            timestamp: tst_info.gen_time,
            serial_number: tst_info.serial_number,
            tsa_name: tst_info.tsa.map(|t| t.to_string()),
        })
    }
}
```

---

### Sprint 4.2 - Advanced Security (Woche 13-14)

#### 4.2.1 SQLCipher Integration

**Priorität:** MITTEL
**Aufwand:** 16 Stunden

```rust
use rusqlite::Connection;

pub struct EncryptedPolicyStore {
    conn: Connection,
}

impl EncryptedPolicyStore {
    pub fn new(db_path: &str) -> Result<Self, PolicyError> {
        let conn = Connection::open(db_path)?;

        // Get encryption key from secure storage
        let encryption_key = std::env::var("SQLITE_ENCRYPTION_KEY")
            .map_err(|_| PolicyError::MissingEncryptionKey)?;

        // Apply encryption key (SQLCipher)
        conn.execute(&format!("PRAGMA key = '{}'", encryption_key), [])?;

        // Verify encryption
        conn.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))
            .map_err(|_| PolicyError::EncryptionFailed)?;

        Ok(Self { conn })
    }
}
```

---

#### 4.2.2 Certificate Hot Reload

**Priorität:** NIEDRIG
**Aufwand:** 16 Stunden

```rust
use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;

pub struct CertReloader {
    cert_path: PathBuf,
    key_path: PathBuf,
    config: Arc<ArcSwap<ServerConfig>>,
}

impl CertReloader {
    pub fn start_watching(&self) -> Result<(), ReloadError> {
        let (tx, rx) = channel();

        let mut watcher = watcher(tx, Duration::from_secs(10))?;
        watcher.watch(&self.cert_path, RecursiveMode::NonRecursive)?;
        watcher.watch(&self.key_path, RecursiveMode::NonRecursive)?;

        let config = self.config.clone();
        let cert_path = self.cert_path.clone();
        let key_path = self.key_path.clone();

        tokio::spawn(async move {
            loop {
                match rx.recv() {
                    Ok(event) => {
                        tracing::info!("Certificate file changed, reloading...");

                        match load_new_config(&cert_path, &key_path) {
                            Ok(new_config) => {
                                config.store(Arc::new(new_config));
                                tracing::info!("Certificates reloaded successfully");
                            }
                            Err(e) => {
                                tracing::error!("Failed to reload certificates: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Watch error: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(())
    }
}
```

---

### Phase 4 Deliverables

| Deliverable | Status | Verantwortlich |
|-------------|--------|----------------|
| SBOM Generation | ⬜ | DevOps |
| Dependency Track | ⬜ | Security |
| RFC 3161 Timestamps | ⬜ | Crypto |
| SQLCipher | ⬜ | Database |
| Cert Hot Reload | ⬜ | Platform |
| Fuzzing Suite | ⬜ | QA |
| **Final Review** | ⬜ | Team |

**Exit Criteria Phase 4:**
- [ ] SBOM in Dependency Track
- [ ] RFC 3161 Timestamps functional
- [ ] SQLite encrypted at rest
- [ ] SOC 2 Audit passed
- [ ] Enterprise Score: ≥95%

---

## Risiken & Mitigationen

| Risiko | Wahrscheinlichkeit | Impact | Mitigation |
|--------|-------------------|--------|------------|
| JWKS Provider nicht erreichbar | Mittel | Hoch | Fallback auf cached keys |
| Redis Ausfall | Niedrig | Mittel | Local fallback rate limiter |
| Certificate Expiry | Mittel | Kritisch | Monitoring + Auto-Renewal |
| Breaking API Changes | Niedrig | Hoch | API Versioning |
| Performance Regression | Mittel | Mittel | Continuous Load Testing |

---

## Ressourcen-Planung

### Team-Anforderungen

| Rolle | Phase 1 | Phase 2 | Phase 3 | Phase 4 |
|-------|---------|---------|---------|---------|
| Backend Developer | 1.0 | 1.5 | 1.0 | 0.5 |
| Security Engineer | 0.5 | 1.0 | 0.5 | 1.0 |
| DevOps/SRE | 0.5 | 0.5 | 1.0 | 0.5 |
| QA Engineer | 0.5 | 0.5 | 0.5 | 1.0 |
| **Total FTE** | **2.5** | **3.5** | **3.0** | **3.0** |

### Infrastructure Requirements

| Resource | Phase 1 | Phase 2 | Phase 3 | Phase 4 |
|----------|---------|---------|---------|---------|
| Redis Cluster | - | - | ✅ | ✅ |
| Jaeger/Tempo | - | - | ✅ | ✅ |
| Grafana | ✅ | ✅ | ✅ | ✅ |
| Dependency Track | - | - | - | ✅ |
| HSM (Prod) | - | ✅ | ✅ | ✅ |

---

## Metriken & KPIs

### Security KPIs

| KPI | Aktuell | Ziel Phase 1 | Ziel Phase 4 |
|-----|---------|--------------|--------------|
| Security Score | 57% | 72% | 95% |
| Critical Vulns | 3 | 0 | 0 |
| High Vulns | 4 | 0 | 0 |
| Test Coverage | ~60% | 75% | 85% |
| MTTR (Security) | - | <4h | <1h |

### Operational KPIs

| KPI | Aktuell | Ziel Phase 3 | Ziel Phase 4 |
|-----|---------|--------------|--------------|
| Availability | - | 99.9% | 99.95% |
| P99 Latency | - | <500ms | <200ms |
| Error Rate | - | <0.1% | <0.01% |
| Deploy Frequency | - | Weekly | Daily |

---

## Anhang

### A. Environment Variables

```bash
# Phase 1
CORS_ALLOWED_ORIGINS=https://app.example.com,https://admin.example.com
CAP_ENVIRONMENT=production
OAUTH2_ISSUER=https://auth.example.com
OAUTH2_AUDIENCE=cap-verifier
OAUTH2_PUBLIC_KEY=<base64-encoded-pem>

# Phase 2
OAUTH2_JWKS_URL=https://auth.example.com/.well-known/jwks.json
SECURITY_AUDIT_LOG_PATH=/var/log/cap/security.jsonl

# Phase 3
REDIS_URL=redis://redis-cluster:6379
OTEL_EXPORTER_OTLP_ENDPOINT=http://tempo:4317

# Phase 4
SQLITE_ENCRYPTION_KEY=<vault-injected>
DEPENDENCY_TRACK_URL=https://dtrack.example.com
```

### B. Helm Values Updates

```yaml
# values-prod.yaml additions per Phase

# Phase 1
securityHeaders:
  enabled: true
  hsts: true
  csp: "default-src 'self'"

cors:
  allowedOrigins:
    - https://app.example.com

# Phase 2
auth:
  jwksUrl: https://auth.example.com/.well-known/jwks.json
  cacheSeconds: 300

# Phase 3
redis:
  enabled: true
  cluster: true

observability:
  tracing:
    enabled: true
    samplingRate: 0.1

# Phase 4
encryption:
  sqlite:
    enabled: true
    keySecret: cap-sqlite-key
```

---

**Dokumentversion:** 1.0
**Letzte Aktualisierung:** 2025-12-02
**Nächste Review:** Nach Phase 1 Completion
