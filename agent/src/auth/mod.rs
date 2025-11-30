/// OAuth2 Authentication Module (Week 5)
///
/// RS256 JWT validation with JWKS caching
pub mod errors;

use errors::AuthError;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// OAuth2 configuration (matches config/auth.yaml)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    pub issuer: String,
    pub audience: String,
    pub jwks_url: String,
    pub jwks_cache_ttl_sec: u64,
    pub required_scopes: HashMap<String, Vec<String>>,
}

/// JWT Claims structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String, // Subject (client_id)
    pub iss: String, // Issuer
    pub aud: String, // Audience
    pub exp: usize,  // Expiration time (Unix timestamp)
    pub iat: usize,  // Issued at (Unix timestamp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<usize>, // Not before (Unix timestamp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>, // Scopes (space-separated)
}

/// JSON Web Key (JWK) structure
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // kty, key_use, alg are for future validation
struct Jwk {
    kid: String,
    kty: String,
    #[serde(rename = "use")]
    key_use: Option<String>,
    alg: Option<String>,
    n: String, // RSA modulus (base64url)
    e: String, // RSA exponent (base64url)
}

/// JWKS (JSON Web Key Set) structure
#[derive(Debug, Clone, Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

/// Cached JWKS entry with TTL
#[derive(Debug, Clone)]
struct CachedJwks {
    jwks: Jwks,
    fetched_at: Instant,
    ttl: Duration,
}

impl CachedJwks {
    fn is_expired(&self) -> bool {
        self.fetched_at.elapsed() > self.ttl
    }
}

/// JWKS Cache with TTL-based refresh
#[derive(Debug, Clone)]
pub struct JwksCache {
    cache: Arc<Mutex<Option<CachedJwks>>>,
    jwks_url: String,
    ttl: Duration,
}

impl JwksCache {
    pub fn new(jwks_url: String, ttl_sec: u64) -> Self {
        Self {
            cache: Arc::new(Mutex::new(None)),
            jwks_url,
            ttl: Duration::from_secs(ttl_sec),
        }
    }

    /// Fetch JWKS from URL (blocks on network I/O)
    async fn fetch_jwks(&self) -> Result<Jwks, AuthError> {
        let response = reqwest::get(&self.jwks_url)
            .await
            .map_err(|_| AuthError::JwksFetchFailed)?;

        if !response.status().is_success() {
            return Err(AuthError::JwksFetchFailed);
        }

        response
            .json::<Jwks>()
            .await
            .map_err(|_| AuthError::JwkParseError)
    }

    /// Get JWKS from cache or fetch if expired (private, used internally)
    async fn get_jwks(&self) -> Result<Jwks, AuthError> {
        // Check cache first
        {
            let cache_lock = self.cache.lock().expect("Failed to lock JWKS cache");
            if let Some(cached) = &*cache_lock {
                if !cached.is_expired() {
                    return Ok(cached.jwks.clone());
                }
            }
        }

        // Cache miss or expired → fetch new JWKS
        let jwks = self.fetch_jwks().await?;

        // Update cache
        {
            let mut cache_lock = self.cache.lock().expect("Failed to lock JWKS cache");
            *cache_lock = Some(CachedJwks {
                jwks: jwks.clone(),
                fetched_at: Instant::now(),
                ttl: self.ttl,
            });
        }

        Ok(jwks)
    }

    /// Find JWK by key ID (kid) (private, used internally)
    async fn find_jwk(&self, kid: &str) -> Result<Jwk, AuthError> {
        let jwks = self.get_jwks().await?;
        jwks.keys
            .into_iter()
            .find(|k| k.kid == kid)
            .ok_or(AuthError::KeyIdNotFound)
    }
}

/// Validate JWT token (RS256)
///
/// Steps:
/// 1. Decode header to extract `kid`
/// 2. Fetch JWK from JWKS cache
/// 3. Verify signature with RSA public key
/// 4. Validate issuer, audience, exp, nbf
/// 5. Return claims
pub async fn validate_token(
    token: &str,
    cfg: &AuthConfig,
    jwks_cache: &JwksCache,
) -> Result<Claims, AuthError> {
    // 1. Decode header to get key ID (kid)
    let header = decode_header(token).map_err(|_| AuthError::InvalidToken)?;

    let kid = header.kid.ok_or(AuthError::InvalidToken)?;

    // 2. Fetch JWK from cache
    let jwk = jwks_cache.find_jwk(&kid).await?;

    // 3. Create decoding key from JWK (RSA public key)
    let decoding_key =
        DecodingKey::from_rsa_components(&jwk.n, &jwk.e).map_err(|_| AuthError::JwkParseError)?;

    // 4. Setup validation rules
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[&cfg.issuer]);
    validation.set_audience(&[&cfg.audience]);

    // 5. Decode and validate token
    let token_data =
        decode::<Claims>(token, &decoding_key, &validation).map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            jsonwebtoken::errors::ErrorKind::ImmatureSignature => AuthError::TokenNotYetValid,
            jsonwebtoken::errors::ErrorKind::InvalidIssuer => AuthError::IssuerMismatch,
            jsonwebtoken::errors::ErrorKind::InvalidAudience => AuthError::AudienceMismatch,
            jsonwebtoken::errors::ErrorKind::InvalidSignature => AuthError::InvalidSignature,
            _ => AuthError::InvalidToken,
        })?;

    Ok(token_data.claims)
}

/// Validate scopes (check if required scopes are present in token)
pub fn validate_scopes(claims: &Claims, required_scopes: &[String]) -> Result<(), AuthError> {
    if required_scopes.is_empty() {
        return Ok(()); // No scopes required
    }

    let token_scopes = match &claims.scope {
        Some(s) => s.split_whitespace().collect::<Vec<_>>(),
        None => return Err(AuthError::InsufficientScope),
    };

    // Check if all required scopes are present
    for required in required_scopes {
        if !token_scopes.contains(&required.as_str()) {
            return Err(AuthError::InsufficientScope);
        }
    }

    Ok(())
}

/// Load auth config from YAML file
pub fn load_auth_config(path: &str) -> Result<AuthConfig, std::io::Error> {
    let content = std::fs::read_to_string(path)?;
    serde_yaml::from_str(&content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_config_load() {
        let config_yaml = r#"
issuer: "https://idp.example.com"
audience: "cap-verifier"
jwks_url: "https://idp.example.com/.well-known/jwks.json"
jwks_cache_ttl_sec: 600
required_scopes:
  verify: ["verify:run"]
  policy_compile: ["policy:compile"]
  policy_read: ["policy:read"]
"#;

        let config: AuthConfig = serde_yaml::from_str(config_yaml).unwrap();
        assert_eq!(config.issuer, "https://idp.example.com");
        assert_eq!(config.audience, "cap-verifier");
        assert_eq!(config.jwks_cache_ttl_sec, 600);
        assert_eq!(
            config.required_scopes.get("verify").unwrap(),
            &vec!["verify:run"]
        );
    }

    #[test]
    fn test_validate_scopes_ok() {
        let claims = Claims {
            sub: "client-123".to_string(),
            iss: "https://idp.example.com".to_string(),
            aud: "cap-verifier".to_string(),
            exp: 9999999999,
            iat: 1234567890,
            nbf: None,
            scope: Some("verify:run policy:read".to_string()),
        };

        let result = validate_scopes(&claims, &["verify:run".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_scopes_missing() {
        let claims = Claims {
            sub: "client-123".to_string(),
            iss: "https://idp.example.com".to_string(),
            aud: "cap-verifier".to_string(),
            exp: 9999999999,
            iat: 1234567890,
            nbf: None,
            scope: Some("policy:read".to_string()),
        };

        let result = validate_scopes(&claims, &["verify:run".to_string()]);
        assert!(matches!(result, Err(AuthError::InsufficientScope)));
    }

    #[test]
    fn test_validate_scopes_no_scope_claim() {
        let claims = Claims {
            sub: "client-123".to_string(),
            iss: "https://idp.example.com".to_string(),
            aud: "cap-verifier".to_string(),
            exp: 9999999999,
            iat: 1234567890,
            nbf: None,
            scope: None,
        };

        let result = validate_scopes(&claims, &["verify:run".to_string()]);
        assert!(matches!(result, Err(AuthError::InsufficientScope)));
    }

    #[test]
    fn test_jwks_cache_creation() {
        let cache = JwksCache::new(
            "https://idp.example.com/.well-known/jwks.json".to_string(),
            600,
        );

        assert_eq!(
            cache.jwks_url,
            "https://idp.example.com/.well-known/jwks.json"
        );
        assert_eq!(cache.ttl, Duration::from_secs(600));
    }

    #[test]
    fn test_cached_jwks_not_expired() {
        let jwks = Jwks { keys: vec![] };
        let cached = CachedJwks {
            jwks: jwks.clone(),
            fetched_at: Instant::now(),
            ttl: Duration::from_secs(600),
        };

        assert!(
            !cached.is_expired(),
            "Freshly cached JWKS should not be expired"
        );
    }

    #[test]
    fn test_cached_jwks_expired() {
        let jwks = Jwks { keys: vec![] };
        let cached = CachedJwks {
            jwks: jwks.clone(),
            fetched_at: Instant::now() - Duration::from_secs(601), // 601 seconds ago
            ttl: Duration::from_secs(600),                         // TTL is 600 seconds
        };

        assert!(cached.is_expired(), "JWKS older than TTL should be expired");
    }

    #[test]
    fn test_validate_scopes_empty_required() {
        let claims = Claims {
            sub: "client-123".to_string(),
            iss: "https://idp.example.com".to_string(),
            aud: "cap-verifier".to_string(),
            exp: 9999999999,
            iat: 1234567890,
            nbf: None,
            scope: None, // No scopes in token
        };

        // Empty required scopes should always pass
        let result = validate_scopes(&claims, &[]);
        assert!(result.is_ok(), "Empty required scopes should always pass");
    }

    #[test]
    fn test_validate_scopes_multiple_required() {
        let claims = Claims {
            sub: "client-123".to_string(),
            iss: "https://idp.example.com".to_string(),
            aud: "cap-verifier".to_string(),
            exp: 9999999999,
            iat: 1234567890,
            nbf: None,
            scope: Some("verify:run policy:read policy:write".to_string()),
        };

        // All required scopes present
        let result = validate_scopes(
            &claims,
            &["verify:run".to_string(), "policy:read".to_string()],
        );
        assert!(result.is_ok(), "All required scopes are present");

        // One required scope missing
        let result = validate_scopes(
            &claims,
            &["verify:run".to_string(), "missing:scope".to_string()],
        );
        assert!(matches!(result, Err(AuthError::InsufficientScope)));
    }

    #[test]
    fn test_load_auth_config_success() {
        // Create temporary config file
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("test_auth_config.yml");

        let config_yaml = r#"
issuer: "https://auth.test.com"
audience: "test-api"
jwks_url: "https://auth.test.com/.well-known/jwks.json"
jwks_cache_ttl_sec: 300
required_scopes:
  verify: ["verify:run"]
  policy_compile: ["policy:compile"]
"#;

        std::fs::write(&config_path, config_yaml).expect("Failed to write test config");

        // Load config
        let result = load_auth_config(config_path.to_str().unwrap());
        assert!(result.is_ok(), "Config loading should succeed");

        let config = result.unwrap();
        assert_eq!(config.issuer, "https://auth.test.com");
        assert_eq!(config.audience, "test-api");
        assert_eq!(config.jwks_cache_ttl_sec, 300);

        // Cleanup
        std::fs::remove_file(&config_path).ok();
    }

    #[test]
    fn test_load_auth_config_file_not_found() {
        let result = load_auth_config("/nonexistent/path/config.yml");
        assert!(result.is_err(), "Loading nonexistent file should fail");
    }

    #[test]
    fn test_load_auth_config_invalid_yaml() {
        // Create temporary config file with invalid YAML
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("test_invalid_config.yml");

        let invalid_yaml = "this is not valid yaml: [unclosed bracket";

        std::fs::write(&config_path, invalid_yaml).expect("Failed to write test config");

        // Load config
        let result = load_auth_config(config_path.to_str().unwrap());
        assert!(result.is_err(), "Loading invalid YAML should fail");

        // Cleanup
        std::fs::remove_file(&config_path).ok();
    }
}
