/// OAuth2 Authentication Errors (Week 5)
///
/// Fail-closed error handling without PII leakage
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum AuthError {
    /// Missing Authorization header
    MissingAuthHeader,

    /// Invalid Authorization header format
    InvalidAuthFormat,

    /// JWT decode error (generic, no details leaked)
    InvalidToken,

    /// Token signature verification failed
    InvalidSignature,

    /// Token expired (exp claim)
    TokenExpired,

    /// Token not yet valid (nbf claim)
    TokenNotYetValid,

    /// Issuer mismatch
    IssuerMismatch,

    /// Audience mismatch
    AudienceMismatch,

    /// Required scope missing
    InsufficientScope,

    /// JWKS fetch failed
    JwksFetchFailed,

    /// JWK key ID (kid) not found in JWKS
    KeyIdNotFound,

    /// JWK parse error
    JwkParseError,

    /// Internal error (catch-all)
    InternalError,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Fail-closed: No PII, no internal details
        match self {
            AuthError::MissingAuthHeader => write!(f, "Missing Authorization header"),
            AuthError::InvalidAuthFormat => write!(f, "Invalid Authorization header format"),
            AuthError::InvalidToken => write!(f, "Invalid token"),
            AuthError::InvalidSignature => write!(f, "Invalid token signature"),
            AuthError::TokenExpired => write!(f, "Token expired"),
            AuthError::TokenNotYetValid => write!(f, "Token not yet valid"),
            AuthError::IssuerMismatch => write!(f, "Token issuer mismatch"),
            AuthError::AudienceMismatch => write!(f, "Token audience mismatch"),
            AuthError::InsufficientScope => write!(f, "Insufficient scope"),
            AuthError::JwksFetchFailed => write!(f, "Failed to fetch JWKS"),
            AuthError::KeyIdNotFound => write!(f, "Key ID not found"),
            AuthError::JwkParseError => write!(f, "Failed to parse JWK"),
            AuthError::InternalError => write!(f, "Internal authentication error"),
        }
    }
}

impl std::error::Error for AuthError {}

/// HTTP status code mapping
impl AuthError {
    pub fn status_code(&self) -> u16 {
        match self {
            AuthError::MissingAuthHeader
            | AuthError::InvalidAuthFormat
            | AuthError::InvalidToken
            | AuthError::InvalidSignature
            | AuthError::TokenExpired
            | AuthError::TokenNotYetValid
            | AuthError::IssuerMismatch
            | AuthError::AudienceMismatch
            | AuthError::KeyIdNotFound
            | AuthError::JwkParseError => 401, // Unauthorized

            AuthError::InsufficientScope => 403, // Forbidden

            AuthError::JwksFetchFailed | AuthError::InternalError => 500, // Internal Error
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_messages_no_pii() {
        // Verify error messages don't leak PII or internal details
        assert_eq!(AuthError::InvalidToken.to_string(), "Invalid token");
        assert_eq!(AuthError::TokenExpired.to_string(), "Token expired");
        assert!(!AuthError::IssuerMismatch.to_string().contains("http"));
    }

    #[test]
    fn test_status_codes() {
        assert_eq!(AuthError::MissingAuthHeader.status_code(), 401);
        assert_eq!(AuthError::InsufficientScope.status_code(), 403);
        assert_eq!(AuthError::JwksFetchFailed.status_code(), 500);
    }

    #[test]
    fn test_all_error_display_messages() {
        // Test all error variants have proper Display messages
        assert_eq!(
            AuthError::MissingAuthHeader.to_string(),
            "Missing Authorization header"
        );
        assert_eq!(
            AuthError::InvalidAuthFormat.to_string(),
            "Invalid Authorization header format"
        );
        assert_eq!(AuthError::InvalidToken.to_string(), "Invalid token");
        assert_eq!(
            AuthError::InvalidSignature.to_string(),
            "Invalid token signature"
        );
        assert_eq!(AuthError::TokenExpired.to_string(), "Token expired");
        assert_eq!(
            AuthError::TokenNotYetValid.to_string(),
            "Token not yet valid"
        );
        assert_eq!(
            AuthError::IssuerMismatch.to_string(),
            "Token issuer mismatch"
        );
        assert_eq!(
            AuthError::AudienceMismatch.to_string(),
            "Token audience mismatch"
        );
        assert_eq!(
            AuthError::InsufficientScope.to_string(),
            "Insufficient scope"
        );
        assert_eq!(
            AuthError::JwksFetchFailed.to_string(),
            "Failed to fetch JWKS"
        );
        assert_eq!(AuthError::KeyIdNotFound.to_string(), "Key ID not found");
        assert_eq!(AuthError::JwkParseError.to_string(), "Failed to parse JWK");
        assert_eq!(
            AuthError::InternalError.to_string(),
            "Internal authentication error"
        );
    }

    #[test]
    fn test_all_status_codes_401() {
        // All authentication failures return 401 Unauthorized
        assert_eq!(AuthError::MissingAuthHeader.status_code(), 401);
        assert_eq!(AuthError::InvalidAuthFormat.status_code(), 401);
        assert_eq!(AuthError::InvalidToken.status_code(), 401);
        assert_eq!(AuthError::InvalidSignature.status_code(), 401);
        assert_eq!(AuthError::TokenExpired.status_code(), 401);
        assert_eq!(AuthError::TokenNotYetValid.status_code(), 401);
        assert_eq!(AuthError::IssuerMismatch.status_code(), 401);
        assert_eq!(AuthError::AudienceMismatch.status_code(), 401);
        assert_eq!(AuthError::KeyIdNotFound.status_code(), 401);
        assert_eq!(AuthError::JwkParseError.status_code(), 401);
    }

    #[test]
    fn test_status_code_403_forbidden() {
        // Insufficient scope returns 403 Forbidden
        assert_eq!(AuthError::InsufficientScope.status_code(), 403);
    }

    #[test]
    fn test_status_codes_500_internal_error() {
        // Server errors return 500 Internal Server Error
        assert_eq!(AuthError::JwksFetchFailed.status_code(), 500);
        assert_eq!(AuthError::InternalError.status_code(), 500);
    }

    #[test]
    fn test_error_debug_format() {
        // Test Debug formatting works
        let err = AuthError::InvalidToken;
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("InvalidToken"));
    }

    #[test]
    fn test_error_clone() {
        // Test Clone trait
        let err1 = AuthError::TokenExpired;
        let err2 = err1.clone();
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_error_partial_eq() {
        // Test PartialEq trait
        assert_eq!(AuthError::InvalidToken, AuthError::InvalidToken);
        assert_ne!(AuthError::InvalidToken, AuthError::TokenExpired);
        assert_ne!(AuthError::MissingAuthHeader, AuthError::InvalidAuthFormat);
    }

    #[test]
    fn test_error_trait_implementation() {
        // Test std::error::Error trait
        let err: Box<dyn std::error::Error> = Box::new(AuthError::InvalidToken);
        assert_eq!(err.to_string(), "Invalid token");
    }
}
