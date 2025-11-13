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
}
