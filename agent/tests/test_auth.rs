//! Integration Tests für auth.rs
//!
//! Diese Tests wurden als Integration Tests erstellt um Tarpaulin Coverage-Tracking zu ermöglichen.
//! Tarpaulin hat eine bekannte Limitation mit #[cfg(test)] inline modules.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware,
    response::IntoResponse,
    routing::post,
    Router,
};
use cap_agent::api::auth::*;
use tower::ServiceExt;

#[test]
fn test_claims_creation() {
    let claims = Claims {
        sub: "test-client-123".to_string(),
        iss: "https://auth.example.com".to_string(),
        aud: "cap-verifier".to_string(),
        exp: 1700000000,
        iat: 1699900000,
        scope: "verify:read verify:write".to_string(),
    };

    assert_eq!(claims.sub, "test-client-123");
    assert_eq!(claims.iss, "https://auth.example.com");
    assert_eq!(claims.scope, "verify:read verify:write");
}

#[test]
fn test_oauth2_config_mock() {
    let config = OAuth2Config::mock();

    assert_eq!(config.issuer, "https://auth.example.com");
    assert_eq!(config.audience, "cap-verifier");
    assert!(config.public_key.contains("BEGIN PUBLIC KEY"));
    assert_eq!(config.required_scopes, vec!["verify:read"]);
}

#[test]
fn test_generate_mock_token() {
    let claims = Claims {
        sub: "test-client".to_string(),
        iss: "https://auth.example.com".to_string(),
        aud: "cap-verifier".to_string(),
        exp: 9999999999,
        iat: 1699900000,
        scope: "verify:read".to_string(),
    };

    let token = generate_mock_token(claims);

    // JWT tokens have three parts separated by dots
    let parts: Vec<&str> = token.split('.').collect();
    assert_eq!(parts.len(), 3, "JWT should have three parts");
}

#[test]
fn test_validate_token_with_admin_bypass() {
    let config = OAuth2Config::mock();

    // Test admin-tom bypass token
    let result = validate_token("admin-tom", &config);

    assert!(result.is_ok(), "admin-tom should bypass validation");
    let claims = result.unwrap();
    assert_eq!(claims.sub, "admin");
    assert_eq!(claims.iss, "dev-mode");
    assert!(claims.scope.contains("verify:read"));
    assert!(claims.scope.contains("verify:write"));
    assert!(claims.scope.contains("policy:read"));
    assert!(claims.scope.contains("policy:write"));
}

#[test]
fn test_validate_token_with_valid_jwt() {
    let config = OAuth2Config::mock();

    // Create a valid token
    let claims = Claims {
        sub: "test-client".to_string(),
        iss: "https://auth.example.com".to_string(),
        aud: "cap-verifier".to_string(),
        exp: 9999999999, // Far future
        iat: 1699900000,
        scope: "verify:read verify:write".to_string(),
    };

    let token = generate_mock_token(claims);
    let result = validate_token(&token, &config);

    assert!(result.is_ok(), "Valid token should pass validation");
    let validated_claims = result.unwrap();
    assert_eq!(validated_claims.sub, "test-client");
}

#[test]
fn test_validate_token_with_expired_jwt() {
    let config = OAuth2Config::mock();

    // Create an expired token
    let claims = Claims {
        sub: "test-client".to_string(),
        iss: "https://auth.example.com".to_string(),
        aud: "cap-verifier".to_string(),
        exp: 1000000000, // Past timestamp
        iat: 999900000,
        scope: "verify:read".to_string(),
    };

    let token = generate_mock_token(claims);
    let result = validate_token(&token, &config);

    assert!(result.is_err(), "Expired token should fail validation");
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("expired") || err_msg.contains("Expired"));
}

#[test]
fn test_validate_token_with_invalid_format() {
    let config = OAuth2Config::mock();

    let result = validate_token("not-a-valid-jwt-token", &config);

    assert!(result.is_err(), "Invalid format should fail validation");
}

#[test]
fn test_validate_token_empty_string() {
    let config = OAuth2Config::mock();

    let result = validate_token("", &config);

    assert!(result.is_err(), "Empty token should fail validation");
}

#[tokio::test]
async fn test_auth_middleware_success() {
    // Create test app with auth middleware
    async fn protected_handler() -> impl IntoResponse {
        (StatusCode::OK, "Protected resource")
    }

    let app = Router::new()
        .route("/protected", post(protected_handler))
        .layer(middleware::from_fn(auth_middleware));

    // Create request with valid admin token
    let req = Request::builder()
        .method("POST")
        .uri("/protected")
        .header("Authorization", "Bearer admin-tom")
        .body(Body::empty())
        .unwrap();

    // Send request
    let response = app.oneshot(req).await.unwrap();

    // Should succeed with 200 OK
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_auth_middleware_missing_token() {
    // Create test app with auth middleware
    async fn protected_handler() -> impl IntoResponse {
        (StatusCode::OK, "Protected resource")
    }

    let app = Router::new()
        .route("/protected", post(protected_handler))
        .layer(middleware::from_fn(auth_middleware));

    // Create request WITHOUT Authorization header
    let req = Request::builder()
        .method("POST")
        .uri("/protected")
        .body(Body::empty())
        .unwrap();

    // Send request
    let response = app.oneshot(req).await.unwrap();

    // Should fail with 401 UNAUTHORIZED
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_auth_middleware_invalid_token() {
    // Create test app with auth middleware
    async fn protected_handler() -> impl IntoResponse {
        (StatusCode::OK, "Protected resource")
    }

    let app = Router::new()
        .route("/protected", post(protected_handler))
        .layer(middleware::from_fn(auth_middleware));

    // Create request with invalid token
    let req = Request::builder()
        .method("POST")
        .uri("/protected")
        .header("Authorization", "Bearer invalid-token-xyz")
        .body(Body::empty())
        .unwrap();

    // Send request
    let response = app.oneshot(req).await.unwrap();

    // Should fail with 401 UNAUTHORIZED
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_auth_middleware_expired_token() {
    // Create test app with auth middleware
    async fn protected_handler() -> impl IntoResponse {
        (StatusCode::OK, "Protected resource")
    }

    let app = Router::new()
        .route("/protected", post(protected_handler))
        .layer(middleware::from_fn(auth_middleware));

    // Create expired token
    let expired_claims = Claims {
        sub: "test-client".to_string(),
        iss: "https://auth.example.com".to_string(),
        aud: "cap-verifier".to_string(),
        exp: 1000000000, // Past timestamp
        iat: 999900000,
        scope: "verify:read".to_string(),
    };
    let expired_token = generate_mock_token(expired_claims);

    // Create request with expired token
    let req = Request::builder()
        .method("POST")
        .uri("/protected")
        .header("Authorization", format!("Bearer {}", expired_token))
        .body(Body::empty())
        .unwrap();

    // Send request
    let response = app.oneshot(req).await.unwrap();

    // Should fail with 401 UNAUTHORIZED
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn test_oauth2_config_creation() {
    let config = OAuth2Config {
        issuer: "https://custom-auth.example.com".to_string(),
        audience: "my-api".to_string(),
        public_key: "-----BEGIN PUBLIC KEY-----\nMOCK\n-----END PUBLIC KEY-----".to_string(),
        required_scopes: vec!["custom:scope".to_string()],
    };

    assert_eq!(config.issuer, "https://custom-auth.example.com");
    assert_eq!(config.audience, "my-api");
    assert!(config.public_key.contains("BEGIN PUBLIC KEY"));
    assert_eq!(config.required_scopes.len(), 1);
    assert_eq!(config.required_scopes[0], "custom:scope");
}

#[test]
fn test_validate_token_with_wrong_algorithm() {
    let config = OAuth2Config::mock();

    // JWT with HS256 algorithm instead of RS256
    // Header: {"alg":"HS256","typ":"JWT"}
    // Payload: {"sub":"test"}
    // This is a well-formed JWT but with wrong algorithm
    let hs256_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ0ZXN0In0.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";

    let result = validate_token(hs256_token, &config);

    assert!(result.is_err(), "HS256 token should fail validation");
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Invalid algorithm") || err_msg.contains("RS256"));
}

#[tokio::test]
async fn test_auth_middleware_invalid_header_format() {
    // Create test app with auth middleware
    async fn protected_handler() -> impl IntoResponse {
        (StatusCode::OK, "Protected resource")
    }

    let app = Router::new()
        .route("/protected", post(protected_handler))
        .layer(middleware::from_fn(auth_middleware));

    // Create request with "Basic" instead of "Bearer"
    let req = Request::builder()
        .method("POST")
        .uri("/protected")
        .header("Authorization", "Basic invalid-format")
        .body(Body::empty())
        .unwrap();

    // Send request
    let response = app.oneshot(req).await.unwrap();

    // Should fail with 401 UNAUTHORIZED
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
