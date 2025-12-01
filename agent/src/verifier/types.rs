//! Verifier Types - Core data structures for verification
//!
//! Contains:
//! - ProofStatement: Cryptographic commitments from manifest
//! - VerifyOptions: Verification check configuration
//! - VerifyReport: Structured verification results

use serde::{Deserialize, Serialize};

/// Proof statement extracted from manifest
///
/// Represents the cryptographic commitments and policy requirements
/// that the proof must satisfy.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProofStatement {
    /// Policy hash (SHA3-256, 0x-prefixed)
    pub policy_hash: String,

    /// Company commitment root (BLAKE3 Merkle root, 0x-prefixed)
    pub company_commitment_root: String,

    /// Optional sanctions list root (BLAKE3, 0x-prefixed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sanctions_root: Option<String>,

    /// Optional jurisdiction list root (BLAKE3, 0x-prefixed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jurisdiction_root: Option<String>,

    /// Optional extensions (future use)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Value>,
}

/// Verification options
///
/// Controls which verification checks should be performed.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerifyOptions {
    /// Check timestamp validity (requires timestamp data in manifest)
    pub check_timestamp: bool,

    /// Check registry match (requires registry entry data)
    pub check_registry: bool,
}

impl Default for VerifyOptions {
    /// Default options for offline-first verification (REQ-07)
    ///
    /// Timestamp and registry checks are disabled by default to support
    /// offline verification workflows (e.g., desktop proofer).
    fn default() -> Self {
        Self {
            check_timestamp: false,
            check_registry: false,
        }
    }
}

/// Verification report
///
/// Contains structured results of verification checks, including
/// detailed error information for failed checks.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerifyReport {
    /// Overall status: "ok" or "fail"
    pub status: String,

    /// Manifest hash (SHA3-256, 0x-prefixed)
    pub manifest_hash: String,

    /// Proof hash (SHA3-256, 0x-prefixed)
    pub proof_hash: String,

    /// Signature validation result
    pub signature_valid: bool,

    /// Timestamp validation result (None if check disabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp_valid: Option<bool>,

    /// Registry match result (None if check disabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry_match: Option<bool>,

    /// Structured details about verification findings
    pub details: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_options_default() {
        let opts = VerifyOptions::default();
        assert!(!opts.check_timestamp);
        assert!(!opts.check_registry);
    }

    #[test]
    fn test_proof_statement_serialization() {
        let stmt = ProofStatement {
            policy_hash: "0x1234".to_string(),
            company_commitment_root: "0xabcd".to_string(),
            sanctions_root: None,
            jurisdiction_root: None,
            extensions: None,
        };

        let json = serde_json::to_string(&stmt).unwrap();
        assert!(json.contains("policy_hash"));
        assert!(!json.contains("sanctions_root")); // skip_serializing_if works
    }
}
