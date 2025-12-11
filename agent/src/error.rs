//! Custom Error Types for cap-agent
//!
//! Provides structured error handling across all modules.
//! Uses thiserror for derive macros and better error messages.

use thiserror::Error;

/// Main error type for cap-agent operations
#[derive(Error, Debug)]
pub enum CapAgentError {
    /// I/O errors (file operations, network)
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Cryptographic operation errors
    #[error("Crypto error: {0}")]
    Crypto(String),

    /// Verification failures
    #[error("Verification failed: {0}")]
    Verification(String),

    /// Policy-related errors
    #[error("Policy error: {0}")]
    Policy(String),

    /// Registry errors
    #[error("Registry error: {0}")]
    Registry(String),

    /// Audit log errors
    #[error("Audit error: {0}")]
    Audit(String),

    /// Key management errors
    #[error("Key error: {0}")]
    Key(String),

    /// Bundle/package errors
    #[error("Bundle error: {0}")]
    Bundle(String),

    /// Configuration errors
    #[error("Config error: {0}")]
    Config(String),

    /// Invalid input/argument errors
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Resource not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Permission/authorization errors
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Internal/unexpected errors
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type alias using CapAgentError
pub type Result<T> = std::result::Result<T, CapAgentError>;

// Conversion from anyhow::Error for gradual migration
impl From<anyhow::Error> for CapAgentError {
    fn from(err: anyhow::Error) -> Self {
        CapAgentError::Internal(err.to_string())
    }
}

// Conversion from Box<dyn Error> for legacy compatibility
impl From<Box<dyn std::error::Error>> for CapAgentError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        CapAgentError::Internal(err.to_string())
    }
}

// Conversion from rusqlite errors
impl From<rusqlite::Error> for CapAgentError {
    fn from(err: rusqlite::Error) -> Self {
        CapAgentError::Registry(format!("SQLite error: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = CapAgentError::Verification("hash mismatch".to_string());
        assert_eq!(err.to_string(), "Verification failed: hash mismatch");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: CapAgentError = io_err.into();
        assert!(err.to_string().contains("IO error"));
    }

    #[test]
    fn test_error_debug() {
        let err = CapAgentError::Policy("invalid constraints".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Policy"));
    }

    #[test]
    fn test_result_type() {
        fn example_fn() -> Result<String> {
            Ok("success".to_string())
        }

        let result = example_fn();
        assert!(result.is_ok());
    }
}
