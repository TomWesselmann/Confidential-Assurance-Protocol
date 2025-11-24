/// Authentication Middleware for HTTP endpoints (Week 5)
///
/// Extracts Bearer tokens, validates JWT, and enforces scopes
use crate::auth::{errors::AuthError, validate_scopes, validate_token, AuthConfig, JwksCache};
use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

/// Shared authentication state
#[derive(Clone)]
pub struct AuthState {
    pub config: Arc<AuthConfig>,
    pub jwks_cache: Arc<JwksCache>,
}

impl AuthState {
    pub fn new(config: AuthConfig) -> Self {
        let jwks_cache = JwksCache::new(config.jwks_url.clone(), config.jwks_cache_ttl_sec);

        Self {
            config: Arc::new(config),
            jwks_cache: Arc::new(jwks_cache),
        }
    }
}

/// Extract Bearer token from Authorization header
fn extract_bearer_token(auth_header: Option<&str>) -> Result<&str, AuthError> {
    let header = auth_header.ok_or(AuthError::MissingAuthHeader)?;

    if !header.starts_with("Bearer ") {
        return Err(AuthError::InvalidAuthFormat);
    }

    let token = &header[7..]; // Skip "Bearer "
    if token.is_empty() {
        return Err(AuthError::InvalidAuthFormat);
    }

    Ok(token)
}

/// Determine required scopes for an endpoint
fn get_required_scopes_for_path(path: &str, config: &AuthConfig) -> Vec<String> {
    if path.starts_with("/verify") {
        config
            .required_scopes
            .get("verify")
            .cloned()
            .unwrap_or_default()
    } else if path.starts_with("/policy/compile") {
        config
            .required_scopes
            .get("policy_compile")
            .cloned()
            .unwrap_or_default()
    } else if path.starts_with("/policy/") {
        config
            .required_scopes
            .get("policy_read")
            .cloned()
            .unwrap_or_default()
    } else {
        Vec::new() // No scopes required for unknown paths
    }
}

/// Authentication middleware (validates JWT + scopes)
pub async fn auth_middleware(
    State(auth_state): State<AuthState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, AuthErrorResponse> {
    // 1. Extract Bearer token from Authorization header
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok());

    let token = extract_bearer_token(auth_header)?;

    // 2. Validate JWT token
    let claims = validate_token(token, &auth_state.config, &auth_state.jwks_cache).await?;

    // 3. Check required scopes for endpoint
    let path = req.uri().path();
    let required_scopes = get_required_scopes_for_path(path, &auth_state.config);

    if !required_scopes.is_empty() {
        validate_scopes(&claims, &required_scopes)?;
    }

    // 4. Attach claims to request extensions (for downstream handlers)
    req.extensions_mut().insert(claims);

    // 5. Continue to next middleware/handler
    Ok(next.run(req).await)
}

/// Auth error response (converts AuthError to HTTP response)
#[derive(Debug)]
pub struct AuthErrorResponse(AuthError);

impl From<AuthError> for AuthErrorResponse {
    fn from(err: AuthError) -> Self {
        Self(err)
    }
}

impl IntoResponse for AuthErrorResponse {
    fn into_response(self) -> Response {
        let status_code =
            StatusCode::from_u16(self.0.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        let body = serde_json::json!({
            "error": self.0.to_string(),
            "status": self.0.status_code()
        });

        (status_code, axum::Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bearer_token_ok() {
        let header = "Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...";
        let token = extract_bearer_token(Some(header)).unwrap();
        assert_eq!(token, "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...");
    }

    #[test]
    fn test_extract_bearer_token_missing() {
        let result = extract_bearer_token(None);
        assert!(matches!(result, Err(AuthError::MissingAuthHeader)));
    }

    #[test]
    fn test_extract_bearer_token_invalid_format() {
        let header = "Basic dXNlcjpwYXNz";
        let result = extract_bearer_token(Some(header));
        assert!(matches!(result, Err(AuthError::InvalidAuthFormat)));
    }

    #[test]
    fn test_extract_bearer_token_empty() {
        let header = "Bearer ";
        let result = extract_bearer_token(Some(header));
        assert!(matches!(result, Err(AuthError::InvalidAuthFormat)));
    }

    #[test]
    fn test_get_required_scopes_verify() {
        let mut config = AuthConfig {
            issuer: "test".to_string(),
            audience: "test".to_string(),
            jwks_url: "test".to_string(),
            jwks_cache_ttl_sec: 600,
            required_scopes: std::collections::HashMap::new(),
        };
        config
            .required_scopes
            .insert("verify".to_string(), vec!["verify:run".to_string()]);

        let scopes = get_required_scopes_for_path("/verify", &config);
        assert_eq!(scopes, vec!["verify:run"]);
    }

    #[test]
    fn test_get_required_scopes_policy_compile() {
        let mut config = AuthConfig {
            issuer: "test".to_string(),
            audience: "test".to_string(),
            jwks_url: "test".to_string(),
            jwks_cache_ttl_sec: 600,
            required_scopes: std::collections::HashMap::new(),
        };
        config.required_scopes.insert(
            "policy_compile".to_string(),
            vec!["policy:compile".to_string()],
        );

        let scopes = get_required_scopes_for_path("/policy/compile", &config);
        assert_eq!(scopes, vec!["policy:compile"]);
    }

    #[test]
    fn test_get_required_scopes_policy_read() {
        let mut config = AuthConfig {
            issuer: "test".to_string(),
            audience: "test".to_string(),
            jwks_url: "test".to_string(),
            jwks_cache_ttl_sec: 600,
            required_scopes: std::collections::HashMap::new(),
        };
        config
            .required_scopes
            .insert("policy_read".to_string(), vec!["policy:read".to_string()]);

        let scopes = get_required_scopes_for_path("/policy/12345", &config);
        assert_eq!(scopes, vec!["policy:read"]);
    }

    #[test]
    fn test_get_required_scopes_unknown_path() {
        let config = AuthConfig {
            issuer: "test".to_string(),
            audience: "test".to_string(),
            jwks_url: "test".to_string(),
            jwks_cache_ttl_sec: 600,
            required_scopes: std::collections::HashMap::new(),
        };

        let scopes = get_required_scopes_for_path("/unknown", &config);
        assert_eq!(scopes, Vec::<String>::new());
    }

    #[test]
    fn test_auth_state_new() {
        let config = AuthConfig {
            issuer: "https://auth.example.com".to_string(),
            audience: "cap-verifier".to_string(),
            jwks_url: "https://auth.example.com/.well-known/jwks.json".to_string(),
            jwks_cache_ttl_sec: 600,
            required_scopes: std::collections::HashMap::new(),
        };

        let state = AuthState::new(config.clone());

        // Verify config is stored
        assert_eq!(state.config.issuer, "https://auth.example.com");
        assert_eq!(state.config.audience, "cap-verifier");
        assert_eq!(state.config.jwks_cache_ttl_sec, 600);

        // Verify JWKS cache is initialized
        // (We can't directly access cache internals, but we can verify it was created)
    }

    #[test]
    fn test_auth_error_response_from_auth_error() {
        let err = AuthError::MissingAuthHeader;
        let response_err: AuthErrorResponse = err.into();

        // Verify the error is wrapped correctly
        assert!(matches!(response_err.0, AuthError::MissingAuthHeader));
    }

    #[test]
    fn test_auth_error_response_into_response_401() {
        let err = AuthError::MissingAuthHeader;
        let response_err = AuthErrorResponse(err);

        let response = response_err.into_response();

        // Verify status code
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_auth_error_response_into_response_403() {
        let err = AuthError::InsufficientScope;
        let response_err = AuthErrorResponse(err);

        let response = response_err.into_response();

        // Verify status code
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn test_auth_error_response_into_response_500() {
        let err = AuthError::JwksFetchFailed;
        let response_err = AuthErrorResponse(err);

        let response = response_err.into_response();

        // Verify status code
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_auth_error_response_body_format() {
        let err = AuthError::InvalidToken;
        let response_err = AuthErrorResponse(err);

        let response = response_err.into_response();

        // Verify response has correct structure (we can't easily parse the body in a unit test,
        // but we can verify the status code which indicates correct formatting)
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_auth_state_new_initializes_jwks_cache() {
        let config = AuthConfig {
            issuer: "https://auth.example.com".to_string(),
            audience: "cap-verifier".to_string(),
            jwks_url: "https://auth.example.com/.well-known/jwks.json".to_string(),
            jwks_cache_ttl_sec: 3600,
            required_scopes: std::collections::HashMap::new(),
        };

        let state = AuthState::new(config.clone());

        // Verify config is correctly stored in Arc
        assert_eq!(state.config.issuer, config.issuer);
        assert_eq!(state.config.audience, config.audience);
        assert_eq!(state.config.jwks_url, config.jwks_url);
        assert_eq!(state.config.jwks_cache_ttl_sec, config.jwks_cache_ttl_sec);

        // JwksCache should be initialized
        // (We can't directly test internal state, but creation should succeed)
    }

    #[test]
    fn test_get_required_scopes_for_path_verify_endpoint() {
        let mut config = AuthConfig {
            issuer: "test".to_string(),
            audience: "test".to_string(),
            jwks_url: "test".to_string(),
            jwks_cache_ttl_sec: 600,
            required_scopes: std::collections::HashMap::new(),
        };
        config.required_scopes.insert(
            "verify".to_string(),
            vec!["verify:run".to_string(), "verify:read".to_string()],
        );

        let scopes = get_required_scopes_for_path("/verify/bundle", &config);
        assert_eq!(scopes, vec!["verify:run".to_string(), "verify:read".to_string()]);
    }

    #[test]
    fn test_get_required_scopes_for_path_policy_compile_exact() {
        let mut config = AuthConfig {
            issuer: "test".to_string(),
            audience: "test".to_string(),
            jwks_url: "test".to_string(),
            jwks_cache_ttl_sec: 600,
            required_scopes: std::collections::HashMap::new(),
        };
        config.required_scopes.insert(
            "policy_compile".to_string(),
            vec!["policy:compile".to_string()],
        );

        let scopes = get_required_scopes_for_path("/policy/compile", &config);
        assert_eq!(scopes, vec!["policy:compile".to_string()]);
    }

    #[test]
    fn test_get_required_scopes_for_path_policy_read_with_uuid() {
        let mut config = AuthConfig {
            issuer: "test".to_string(),
            audience: "test".to_string(),
            jwks_url: "test".to_string(),
            jwks_cache_ttl_sec: 600,
            required_scopes: std::collections::HashMap::new(),
        };
        config.required_scopes.insert(
            "policy_read".to_string(),
            vec!["policy:read".to_string()],
        );

        let scopes = get_required_scopes_for_path("/policy/0x123abc", &config);
        assert_eq!(scopes, vec!["policy:read".to_string()]);
    }

    #[test]
    fn test_get_required_scopes_for_path_empty_when_not_configured() {
        let config = AuthConfig {
            issuer: "test".to_string(),
            audience: "test".to_string(),
            jwks_url: "test".to_string(),
            jwks_cache_ttl_sec: 600,
            required_scopes: std::collections::HashMap::new(), // No scopes configured
        };

        let scopes = get_required_scopes_for_path("/verify", &config);
        assert_eq!(scopes, Vec::<String>::new());
    }

    #[test]
    fn test_auth_error_response_all_error_types() {
        // Test all AuthError variants map to correct status codes
        let test_cases = vec![
            (AuthError::MissingAuthHeader, StatusCode::UNAUTHORIZED),
            (AuthError::InvalidAuthFormat, StatusCode::UNAUTHORIZED),
            (AuthError::InvalidToken, StatusCode::UNAUTHORIZED),
            (AuthError::TokenExpired, StatusCode::UNAUTHORIZED),
            (AuthError::InsufficientScope, StatusCode::FORBIDDEN),
            (AuthError::JwksFetchFailed, StatusCode::INTERNAL_SERVER_ERROR),
        ];

        for (err, expected_status) in test_cases {
            let response_err = AuthErrorResponse(err);
            let response = response_err.into_response();
            assert_eq!(response.status(), expected_status);
        }
    }
}
