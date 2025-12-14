//! Centralized path constants for CLI commands
//!
//! All hardcoded paths should be defined here to:
//! - Enable easy configuration changes
//! - Prevent typos and inconsistencies
//! - Support future environment-based configuration

#![allow(dead_code)]

// ============================================================================
// Build Directory Paths
// ============================================================================

/// Default build directory
pub const BUILD_DIR: &str = "build";

/// Audit log file (JSONL format with hash chain)
pub const AUDIT_LOG: &str = "build/agent.audit.jsonl";

/// Commitments file (supplier/UBO hashes)
pub const COMMITMENTS: &str = "build/commitments.json";

/// Registry file (JSON format)
pub const REGISTRY_JSON: &str = "build/registry.json";

/// Registry file (SQLite format)
pub const REGISTRY_SQLITE: &str = "build/registry.sqlite";

/// Sanctions list Merkle root
pub const SANCTIONS_ROOT: &str = "build/sanctions.root";

/// Jurisdictions list Merkle root
pub const JURISDICTIONS_ROOT: &str = "build/jurisdictions.root";

// ============================================================================
// Keys Directory Paths
// ============================================================================

/// Default keys directory
pub const KEYS_DIR: &str = "keys";

/// Default company signing key
pub const COMPANY_KEY: &str = "keys/company.ed25519";

// ============================================================================
// Helper Functions
// ============================================================================

/// Returns the audit log path, allowing override via environment variable
pub fn audit_log_path() -> String {
    std::env::var("CAP_AUDIT_LOG")
        .unwrap_or_else(|_| AUDIT_LOG.to_string())
}

/// Returns the registry path based on backend type
pub fn registry_path(backend: &str) -> &'static str {
    match backend {
        "sqlite" => REGISTRY_SQLITE,
        "json" => REGISTRY_JSON,
        _ => REGISTRY_JSON, // Default to JSON
    }
}

/// Returns the commitments path, allowing override via environment variable
pub fn commitments_path() -> String {
    std::env::var("CAP_COMMITMENTS")
        .unwrap_or_else(|_| COMMITMENTS.to_string())
}

/// Returns the keys directory, allowing override via environment variable
pub fn keys_dir() -> String {
    std::env::var("CAP_KEYS_DIR")
        .unwrap_or_else(|_| KEYS_DIR.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_paths() {
        assert_eq!(AUDIT_LOG, "build/agent.audit.jsonl");
        assert_eq!(COMMITMENTS, "build/commitments.json");
        assert_eq!(REGISTRY_JSON, "build/registry.json");
        assert_eq!(REGISTRY_SQLITE, "build/registry.sqlite");
    }

    #[test]
    fn test_registry_path_selection() {
        assert_eq!(registry_path("sqlite"), REGISTRY_SQLITE);
        assert_eq!(registry_path("json"), REGISTRY_JSON);
        assert_eq!(registry_path("auto"), REGISTRY_JSON);
    }

    #[test]
    fn test_audit_log_path_default() {
        // Clear env var if set
        std::env::remove_var("CAP_AUDIT_LOG");
        assert_eq!(audit_log_path(), AUDIT_LOG);
    }
}
