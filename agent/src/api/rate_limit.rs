// Rate Limiting Middleware
//
// Provides IP-based rate limiting using the token bucket algorithm (GCRA).
// Prevents API abuse and ensures fair resource allocation.
//
// Features:
// - IP-based rate limiting (via X-Forwarded-For or socket address)
// - Configurable burst and replenishment rate
// - Standard rate limit headers (X-RateLimit-*)
// - Per-endpoint rate limits
//
// Default Limits:
// - Global: 100 requests per minute
// - Verify endpoint: 20 requests per minute
// - Policy compile: 10 requests per minute

use axum::{http::StatusCode, response::IntoResponse};
use governor::middleware::StateInformationMiddleware;
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorError,
    GovernorLayer,
};

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Requests per minute
    pub requests_per_minute: u32,
    /// Burst size (max requests in burst)
    pub burst_size: u32,
}

impl RateLimitConfig {
    /// Default global rate limit (100 req/min, burst 120)
    pub fn default_global() -> Self {
        Self {
            requests_per_minute: 100,
            burst_size: 120,
        }
    }

    /// Strict rate limit for expensive operations (10 req/min, burst 15)
    pub fn strict() -> Self {
        Self {
            requests_per_minute: 10,
            burst_size: 15,
        }
    }

    /// Moderate rate limit for normal operations (20 req/min, burst 25)
    pub fn moderate() -> Self {
        Self {
            requests_per_minute: 20,
            burst_size: 25,
        }
    }
}

/// Create rate limiter layer with IP extraction
pub fn rate_limiter_layer(
    config: RateLimitConfig,
) -> GovernorLayer<SmartIpKeyExtractor, StateInformationMiddleware> {
    let replenish_interval = std::time::Duration::from_secs(60) / config.requests_per_minute;

    let governor_conf = std::sync::Arc::new(
        GovernorConfigBuilder::default()
            .per_millisecond(replenish_interval.as_millis() as u64)
            .burst_size(config.burst_size)
            .use_headers()
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .expect("Failed to build GovernorConfig"),
    );

    GovernorLayer {
        config: governor_conf,
    }
}

/// Rate limit error handler
///
/// Converts GovernorError to HTTP 429 Too Many Requests with retry-after header
pub async fn handle_rate_limit_error(_err: GovernorError) -> impl IntoResponse {
    (
        StatusCode::TOO_MANY_REQUESTS,
        [
            ("X-RateLimit-Limit", "100"),
            ("X-RateLimit-Remaining", "0"),
            ("Retry-After", "60"),
        ],
        "Rate limit exceeded. Please retry after 60 seconds.",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default_global();
        assert_eq!(config.requests_per_minute, 100);
        assert_eq!(config.burst_size, 120);
    }

    #[test]
    fn test_rate_limit_config_strict() {
        let config = RateLimitConfig::strict();
        assert_eq!(config.requests_per_minute, 10);
        assert_eq!(config.burst_size, 15);
    }

    #[test]
    fn test_rate_limit_config_moderate() {
        let config = RateLimitConfig::moderate();
        assert_eq!(config.requests_per_minute, 20);
        assert_eq!(config.burst_size, 25);
    }

    #[test]
    fn test_rate_limiter_layer() {
        let config = RateLimitConfig::default_global();
        let _layer = rate_limiter_layer(config);
        // If we get here without panicking, the layer was created successfully
    }

    #[tokio::test]
    async fn test_handle_rate_limit_error() {
        // Create a mock governor error (wait_time is in milliseconds as u64)
        let err = GovernorError::TooManyRequests {
            wait_time: 60_000, // 60 seconds in milliseconds
            headers: None,
        };

        // Call the error handler
        let response = handle_rate_limit_error(err).await.into_response();

        // Verify status code
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

        // Verify headers
        let headers = response.headers();
        assert_eq!(headers.get("X-RateLimit-Limit").unwrap(), "100");
        assert_eq!(headers.get("X-RateLimit-Remaining").unwrap(), "0");
        assert_eq!(headers.get("Retry-After").unwrap(), "60");
    }
}
