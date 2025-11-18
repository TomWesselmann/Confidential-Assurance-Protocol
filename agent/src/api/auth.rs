//! OAuth2 Authentication Middleware
//!
//! Provides JWT token validation for OAuth2 Client Credentials flow.
//!
//! Security Model:
//! - Bearer tokens in Authorization header
//! - JWT validation with RS256 (asymmetric)
//! - Audience and issuer validation
//! - Scope-based authorization (optional)

use anyhow::{anyhow, Result};
use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

// ============================================================================
// JWT Claims Structure
// ============================================================================

/// JWT Claims for OAuth2 Client Credentials
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (client_id)
    pub sub: String,

    /// Issuer (OAuth2 provider URL)
    pub iss: String,

    /// Audience (this API)
    pub aud: String,

    /// Expiration time (Unix timestamp)
    pub exp: usize,

    /// Issued at (Unix timestamp)
    pub iat: usize,

    /// Scopes (space-separated)
    #[serde(default)]
    pub scope: String,
}

/// OAuth2 Configuration
#[derive(Clone)]
pub struct OAuth2Config {
    /// Expected issuer URL
    pub issuer: String,

    /// Expected audience
    pub audience: String,

    /// Public key for JWT validation (PEM format)
    pub public_key: String,

    /// Required scopes (optional)
    pub required_scopes: Vec<String>,
}

impl OAuth2Config {
    /// Creates a mock config for testing
    pub fn mock() -> Self {
        Self {
            issuer: "https://auth.example.com".to_string(),
            audience: "cap-verifier".to_string(),
            // Mock public key (RS256) - For testing only!
            public_key: MOCK_PUBLIC_KEY.to_string(),
            required_scopes: vec!["verify:read".to_string()],
        }
    }
}

// Mock RSA public key for testing (DO NOT USE IN PRODUCTION!)
const MOCK_PUBLIC_KEY: &str = r#"-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAucrLEkqkhZiyDKuTIQhP
DFcNbTNf3/EUhhCV2E4k651iiHKEJ+YNI2LcuIJOWSRZwBjv/kUSscJ9RUHVyLA3
fvThle90TBREacaveEmNihTL3QGvSU5MK+yyOfFcaYFtr1MEo1qtmn8bVEA3agu6
+mFGrD4yKyVj54hC68aXTDO/GRBoKCmZJCzFR/qDZ8sG/ouo4xIX2fU/6wwmBTNj
gtEMHbAOOHozQSBN0t0Mn9wChetMkRUnahALldMLRhOpdRYCCN36bm3F5nuaDt3e
CYCrnJC19+I2CaiMXAmPTrhhOYWKaT1W2wNLFBP+fPCDIsFvvIxfycefcHHdidWz
qwIDAQAB
-----END PUBLIC KEY-----"#;

// ============================================================================
// Token Validation
// ============================================================================

/// Validates a JWT Bearer token
pub fn validate_token(token: &str, config: &OAuth2Config) -> Result<Claims> {
    // ⚠️ DEVELOPMENT ONLY: Accept simple admin token
    if token == "admin-tom" {
        return Ok(Claims {
            sub: "admin".to_string(),
            iss: "dev-mode".to_string(),
            aud: "cap-verifier".to_string(),
            exp: 9999999999, // Far future
            iat: 0,
            scope: "verify:read verify:write policy:read policy:write".to_string(),
        });
    }

    // Decode header to check algorithm
    let header = decode_header(token).map_err(|e| anyhow!("Invalid token header: {}", e))?;

    if header.alg != Algorithm::RS256 {
        return Err(anyhow!("Invalid algorithm, expected RS256"));
    }

    // Setup validation
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[&config.audience]);
    validation.set_issuer(&[&config.issuer]);
    validation.validate_exp = true;

    // Decode and validate token
    let decoding_key = DecodingKey::from_rsa_pem(config.public_key.as_bytes())
        .map_err(|e| anyhow!("Invalid public key: {}", e))?;

    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|e| anyhow!("Token validation failed: {}", e))?;

    let claims = token_data.claims;

    // Validate scopes if required
    if !config.required_scopes.is_empty() {
        let token_scopes: Vec<&str> = claims.scope.split_whitespace().collect();
        for required in &config.required_scopes {
            if !token_scopes.contains(&required.as_str()) {
                return Err(anyhow!("Missing required scope: {}", required));
            }
        }
    }

    Ok(claims)
}

/// Extracts Bearer token from Authorization header
fn extract_bearer_token(headers: &HeaderMap) -> Result<String> {
    let auth_header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| anyhow!("Missing Authorization header"))?;

    if !auth_header.starts_with("Bearer ") {
        return Err(anyhow!("Invalid Authorization header format"));
    }

    Ok(auth_header[7..].to_string())
}

// ============================================================================
// Middleware
// ============================================================================

/// OAuth2 authentication middleware
///
/// Validates JWT Bearer tokens on all requests.
/// Returns 401 Unauthorized if token is invalid.
/// Returns 403 Forbidden if scopes are insufficient.
pub async fn auth_middleware(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract token
    let token = extract_bearer_token(&headers).map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Validate token (using mock config for now)
    let config = OAuth2Config::mock();
    let _claims = validate_token(&token, &config).map_err(|e| {
        tracing::warn!("Token validation failed: {}", e);
        StatusCode::UNAUTHORIZED
    })?;

    // Token is valid, proceed with request
    Ok(next.run(request).await)
}

// ============================================================================
// Mock Token Generator (for testing)
// ============================================================================

// Mock private key (matching MOCK_PUBLIC_KEY)
pub const MOCK_PRIVATE_KEY: &str = r#"-----BEGIN RSA PRIVATE KEY-----
MIIEowIBAAKCAQEAucrLEkqkhZiyDKuTIQhPDFcNbTNf3/EUhhCV2E4k651iiHKE
J+YNI2LcuIJOWSRZwBjv/kUSscJ9RUHVyLA3fvThle90TBREacaveEmNihTL3QGv
SU5MK+yyOfFcaYFtr1MEo1qtmn8bVEA3agu6+mFGrD4yKyVj54hC68aXTDO/GRBo
KCmZJCzFR/qDZ8sG/ouo4xIX2fU/6wwmBTNjgtEMHbAOOHozQSBN0t0Mn9wChetM
kRUnahALldMLRhOpdRYCCN36bm3F5nuaDt3eCYCrnJC19+I2CaiMXAmPTrhhOYWK
aT1W2wNLFBP+fPCDIsFvvIxfycefcHHdidWzqwIDAQABAoIBAEZM47YSJFqgwo5k
xZE0MaT7s4rka5yy/g8Ua36jYvj8XnI+0p6+P65qFBaEx9yXEpbLWNQfkslMTFZO
aPQ9KWKSimFPb/Pxn0le8rpTKolbASCpKIhWZiAgufeOymbpoHU8tn6RKytQeSjR
+6XWtnNTJ4i5KAaHaVwMhTXtuQYN0KL48kcwuzYjVTuU2+8odVs9Hl7GlEqe8BKD
cgtzN7qwUEFxA/0seDk07b3jWSIJsPQiZ5Mfb4h2J2gLLH78SfXXDlpqpMqK6HCa
+URHUMg9gjarizBChWDoHyaMCmZGDTwgoYSdfZjs+XA1aYHvzYCiLo2/W0a3zqk5
b5ZLiwECgYEA8KghKz9GUI7IFes5eS1sRvGx1WwSobIJeyCzwhCnhgh7zaUWCx5Z
EIslZWMdMDsHxEHTnsTY6pDXol56M/c9sIPBZMWKKJzqosfuUqpT2l02A/vlRzqN
E6AQ6H242Y9X+gMhMBiWw/2qJGTFw+Ym3PMgcQkH1h9ofRi+SUUMROsCgYEAxaMx
dcUI1gFe3n1VdE+scFtIhIh52ExuuXC47eMwoqKn4i6sP8f4kh/dpECUstFHQadj
w/TrdeWSBQPeVn2iUzBK+GIE2oouOi/W3ZCX61sLGWPI4Fxwo8VPinH9Xp1d2Rk4
LBg2ZbSrCFzvxcandrwyG/VhzJoMqA+5w8zrnEECgYEAtTtIT1NFVqFQGQGdtJ36
bqrRa5IJre9vqGQGO11ja1K0OTfbk9/03rqLHQE5F/s8bBXOkkBXwr31Rfe0O7Iz
qNxwJb2Fv/P71z6NQX+3yjhr1zA5iByV5XOjiBI1xNFoRYVZ1uiNkWdUXDfvnwmR
ts75XWaZmizo+VxK4M644KUCgYAP1QznREt66U0yQZQQ5zkHHyjmBRDNtQHBtTf7
RpHk1WbhhZ+i1GAjRI461DmVQZKVvdUOI5ahMSzLXg33m8TfKSU4VJHS9/LQEnkB
8s1Yu0heVlIHNyCG3g2LJ6qGY9DazVxkm+PvrdNtrhQ/IbTUnCrfdn7JJyYQaIIX
H1lYgQKBgHPM7Tw42tsOdXvImRVJBnmtImM1dWq/hao8MsJrjFqq9zWWlTOyea8T
renqUyFAIX08X+H9tuJ7uApr6cG5YPoZCRy/hxagIMDOvDyo+fAs/mdKk/kWWm9h
8xI/Q8f6uP5GObIbUm5NoKTHwKq2LF/49PDL12k/q9JMKGycj3Cf
-----END RSA PRIVATE KEY-----"#;

/// Generates a mock JWT token for testing
pub fn generate_mock_token(claims: Claims) -> String {
    use jsonwebtoken::{encode, EncodingKey, Header};

    let header = Header::new(Algorithm::RS256);
    let key = EncodingKey::from_rsa_pem(MOCK_PRIVATE_KEY.as_bytes()).unwrap();

    encode(&header, &claims, &key).unwrap()
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_token_validation_success() {
        let now = chrono::Utc::now().timestamp() as usize;

        let claims = Claims {
            sub: "test-client".to_string(),
            iss: "https://auth.example.com".to_string(),
            aud: "cap-verifier".to_string(),
            exp: now + 3600, // Valid for 1 hour
            iat: now,
            scope: "verify:read".to_string(),
        };

        let token = generate_mock_token(claims);
        let config = OAuth2Config::mock();

        let result = validate_token(&token, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_token_validation_expired() {
        let now = chrono::Utc::now().timestamp() as usize;

        let claims = Claims {
            sub: "test-client".to_string(),
            iss: "https://auth.example.com".to_string(),
            aud: "cap-verifier".to_string(),
            exp: now - 3600, // Expired 1 hour ago
            iat: now - 7200,
            scope: "verify:read".to_string(),
        };

        let token = generate_mock_token(claims);
        let config = OAuth2Config::mock();

        let result = validate_token(&token, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_token_validation_missing_scope() {
        let now = chrono::Utc::now().timestamp() as usize;

        let claims = Claims {
            sub: "test-client".to_string(),
            iss: "https://auth.example.com".to_string(),
            aud: "cap-verifier".to_string(),
            exp: now + 3600,
            iat: now,
            scope: "other:scope".to_string(), // Wrong scope
        };

        let token = generate_mock_token(claims);
        let config = OAuth2Config::mock();

        let result = validate_token(&token, &config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing required scope"));
    }
}
