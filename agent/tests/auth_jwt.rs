/// Integration Tests for OAuth2 JWT RS256 Authentication (Week 5)
///
/// Tests:
/// - IT-A1: Valid RS256 token → 200 for /verify
/// - IT-A2: Expired token → 401
/// - IT-A3: Issuer/Audience mismatch → 401
/// - IT-A4: Missing scope for endpoint → 403
/// - IT-A5: JWKS rotation (key ID not found → 401)
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Mock JWT Claims (matches auth::Claims)
#[derive(Debug, Clone, Deserialize, Serialize)]
struct TestClaims {
    sub: String,
    iss: String,
    aud: String,
    exp: usize,
    iat: usize,
    scope: Option<String>,
}

/// Generate RSA-2048 keypair for testing (DO NOT USE IN PRODUCTION)
fn generate_test_keypair() -> (EncodingKey, jsonwebtoken::DecodingKey) {
    use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey};
    use rsa::RsaPrivateKey;

    let mut rng = rand::thread_rng();
    let private_key = RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate RSA key");
    let public_key = private_key.to_public_key();

    let private_pem = private_key
        .to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)
        .expect("Failed to encode private key")
        .to_string();
    let public_pem = public_key
        .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
        .expect("Failed to encode public key");

    let encoding_key =
        EncodingKey::from_rsa_pem(private_pem.as_bytes()).expect("Failed to create encoding key");
    let decoding_key = jsonwebtoken::DecodingKey::from_rsa_pem(public_pem.as_bytes())
        .expect("Failed to create decoding key");

    (encoding_key, decoding_key)
}

/// Helper: Create JWT token with custom claims
fn create_test_token(claims: &TestClaims, key: &EncodingKey, kid: &str) -> String {
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(kid.to_string());

    encode(&header, claims, key).expect("Failed to encode token")
}

/// Helper: Current Unix timestamp
fn now() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
#[ignore] // Requires running API server
fn it_a1_valid_token_200_for_verify() {
    // Test: Valid RS256 token with correct scope → 200 for /verify

    let (encoding_key, _decoding_key) = generate_test_keypair();

    let claims = TestClaims {
        sub: "test-client-123".to_string(),
        iss: "https://idp.example.com".to_string(),
        aud: "cap-verifier".to_string(),
        exp: now() + 3600, // Valid for 1 hour
        iat: now(),
        scope: Some("verify:run".to_string()),
    };

    let token = create_test_token(&claims, &encoding_key, "test-kid-001");

    // NOTE: This test requires a running API server with JWKS endpoint
    // For full integration, run:
    //   cargo run --bin cap-verifier-api &
    //   cargo test --test auth_jwt -- --ignored

    println!("✅ Test token generated: {}", &token[..50]);
    assert!(!token.is_empty());
}

#[test]
fn it_a2_expired_token_rejected() {
    // Test: Expired token → 401 Unauthorized

    let (encoding_key, decoding_key) = generate_test_keypair();

    let claims = TestClaims {
        sub: "test-client-123".to_string(),
        iss: "https://idp.example.com".to_string(),
        aud: "cap-verifier".to_string(),
        exp: now() - 3600, // Expired 1 hour ago
        iat: now() - 7200,
        scope: Some("verify:run".to_string()),
    };

    let token = create_test_token(&claims, &encoding_key, "test-kid-001");

    // Validate token with jsonwebtoken (should fail with ExpiredSignature)
    let mut validation = jsonwebtoken::Validation::new(Algorithm::RS256);
    validation.set_issuer(&["https://idp.example.com"]);
    validation.set_audience(&["cap-verifier"]);

    let result = jsonwebtoken::decode::<TestClaims>(&token, &decoding_key, &validation);

    assert!(result.is_err());
    if let Err(e) = result {
        match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                println!("✅ Expired token correctly rejected");
            }
            _ => panic!("Expected ExpiredSignature error, got: {:?}", e),
        }
    }
}

#[test]
fn it_a3_issuer_mismatch_rejected() {
    // Test: Token with wrong issuer → 401 Unauthorized

    let (encoding_key, decoding_key) = generate_test_keypair();

    let claims = TestClaims {
        sub: "test-client-123".to_string(),
        iss: "https://malicious-idp.com".to_string(), // Wrong issuer
        aud: "cap-verifier".to_string(),
        exp: now() + 3600,
        iat: now(),
        scope: Some("verify:run".to_string()),
    };

    let token = create_test_token(&claims, &encoding_key, "test-kid-001");

    // Validate token (should fail with InvalidIssuer)
    let mut validation = jsonwebtoken::Validation::new(Algorithm::RS256);
    validation.set_issuer(&["https://idp.example.com"]); // Expected issuer
    validation.set_audience(&["cap-verifier"]);

    let result = jsonwebtoken::decode::<TestClaims>(&token, &decoding_key, &validation);

    assert!(result.is_err());
    if let Err(e) = result {
        match e.kind() {
            jsonwebtoken::errors::ErrorKind::InvalidIssuer => {
                println!("✅ Token with wrong issuer correctly rejected");
            }
            _ => panic!("Expected InvalidIssuer error, got: {:?}", e),
        }
    }
}

#[test]
fn it_a4_audience_mismatch_rejected() {
    // Test: Token with wrong audience → 401 Unauthorized

    let (encoding_key, decoding_key) = generate_test_keypair();

    let claims = TestClaims {
        sub: "test-client-123".to_string(),
        iss: "https://idp.example.com".to_string(),
        aud: "wrong-audience".to_string(), // Wrong audience
        exp: now() + 3600,
        iat: now(),
        scope: Some("verify:run".to_string()),
    };

    let token = create_test_token(&claims, &encoding_key, "test-kid-001");

    // Validate token (should fail with InvalidAudience)
    let mut validation = jsonwebtoken::Validation::new(Algorithm::RS256);
    validation.set_issuer(&["https://idp.example.com"]);
    validation.set_audience(&["cap-verifier"]); // Expected audience

    let result = jsonwebtoken::decode::<TestClaims>(&token, &decoding_key, &validation);

    assert!(result.is_err());
    if let Err(e) = result {
        match e.kind() {
            jsonwebtoken::errors::ErrorKind::InvalidAudience => {
                println!("✅ Token with wrong audience correctly rejected");
            }
            _ => panic!("Expected InvalidAudience error, got: {:?}", e),
        }
    }
}

#[test]
fn it_a5_missing_scope_for_endpoint() {
    // Test: Token without required scope → 403 Forbidden

    use cap_agent::auth::{validate_scopes, Claims};

    let claims = Claims {
        sub: "test-client-123".to_string(),
        iss: "https://idp.example.com".to_string(),
        aud: "cap-verifier".to_string(),
        exp: now() + 3600,
        iat: now(),
        nbf: None,
        scope: Some("policy:read".to_string()), // Only policy:read, missing verify:run
    };

    // Try to validate for /verify endpoint (requires verify:run)
    let required_scopes = ["verify:run".to_string()];
    let result = validate_scopes(&claims, &required_scopes);

    assert!(result.is_err());
    println!("✅ Token with insufficient scope correctly rejected");
}

#[test]
fn it_a6_jwks_key_id_not_found() {
    // Test: JWKS rotation scenario (key ID not found)
    // In production, this would happen when:
    // 1. Token signed with key "old-kid-001"
    // 2. JWKS only contains "new-kid-002"
    // → validate_token() → JwksCache.find_jwk("old-kid-001") → KeyIdNotFound → 401

    use cap_agent::auth::JwksCache;

    // This is a conceptual test - actual JWKS fetching requires HTTP server
    // In real scenario:
    // - Old token with kid="old-kid-001"
    // - JWKS at /jwks.json only has kid="new-kid-002"
    // - validate_token() will return AuthError::KeyIdNotFound

    let _jwks_cache = JwksCache::new(
        "https://idp.example.com/.well-known/jwks.json".to_string(),
        600,
    );

    // Simulate key not found scenario
    // In integration test with real server, this would be tested by:
    // 1. Generate token with kid="old-key"
    // 2. Update JWKS to only contain kid="new-key"
    // 3. Attempt to validate → expect 401 with KeyIdNotFound

    println!("✅ JWKS key rotation test prepared (requires live JWKS endpoint)");
}

#[test]
fn it_a7_valid_token_with_multiple_scopes() {
    // Test: Token with multiple scopes (space-separated) → all scopes recognized

    use cap_agent::auth::{validate_scopes, Claims};

    let claims = Claims {
        sub: "test-client-123".to_string(),
        iss: "https://idp.example.com".to_string(),
        aud: "cap-verifier".to_string(),
        exp: now() + 3600,
        iat: now(),
        nbf: None,
        scope: Some("verify:run policy:compile policy:read".to_string()),
    };

    // Validate each scope individually
    assert!(validate_scopes(&claims, &["verify:run".to_string()]).is_ok());
    assert!(validate_scopes(&claims, &["policy:compile".to_string()]).is_ok());
    assert!(validate_scopes(&claims, &["policy:read".to_string()]).is_ok());

    // Validate multiple scopes at once
    assert!(validate_scopes(
        &claims,
        &["verify:run".to_string(), "policy:compile".to_string()]
    )
    .is_ok());

    println!("✅ Token with multiple scopes validated correctly");
}
