//! Verify API Types - Request/Response structures
//!
//! Provides REST API types for POST /verify endpoint.
//! Supports two modes:
//! - Mode A: policy_id (reference to stored policy)
//! - Mode B: ir (embedded IR v1 object)

use crate::policy_v2::IrV1;
use crate::verifier::core::VerifyReport;
use serde::{Deserialize, Serialize};

// ============================================================================
// Request Types
// ============================================================================

/// POST /verify - Request Body (Week 4: Supports embedded IR)
///
/// Mode A (Policy ID Reference):
/// ```json
/// {
///   "policy_id": "lksg.v1",
///   "context": {...},
///   "backend": "mock"
/// }
/// ```
///
/// Mode B (Embedded IR):
/// ```json
/// {
///   "ir": { "ir_version": "1.0", ... },
///   "context": {...},
///   "backend": "mock"
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    /// Policy ID (Mode A - reference to stored policy)
    #[serde(default)]
    pub policy_id: Option<String>,

    /// Embedded IR v1 (Mode B - direct IR verification)
    #[serde(default)]
    pub ir: Option<IrV1>,

    /// Context containing proof data
    pub context: VerifyContext,

    /// Backend type (mock, zkvm, halo2)
    #[serde(default = "default_backend")]
    pub backend: String,

    /// Optional verification options
    #[serde(default)]
    pub options: VerifyRequestOptions,
}

fn default_backend() -> String {
    "mock".to_string()
}

/// Verification context (proof data)
#[derive(Debug, Deserialize)]
pub struct VerifyContext {
    /// Supplier hashes (BLAKE3, 0x-prefixed)
    #[serde(default)]
    pub supplier_hashes: Vec<String>,

    /// UBO hashes (BLAKE3, 0x-prefixed)
    #[serde(default)]
    pub ubo_hashes: Vec<String>,

    /// Company commitment root (BLAKE3, 0x-prefixed)
    pub company_commitment_root: Option<String>,

    /// Sanctions list root (BLAKE3, 0x-prefixed)
    pub sanctions_root: Option<String>,

    /// Jurisdiction list root (BLAKE3, 0x-prefixed)
    pub jurisdiction_root: Option<String>,
}

/// Request options
#[derive(Debug, Deserialize, Default)]
pub struct VerifyRequestOptions {
    /// Adaptive mode (activates rules based on context)
    #[serde(default)]
    pub adaptive: bool,

    /// Check timestamp validity
    #[serde(default = "default_true")]
    pub check_timestamp: bool,

    /// Check registry match
    #[serde(default = "default_true")]
    pub check_registry: bool,
}

fn default_true() -> bool {
    true
}

// ============================================================================
// Response Types
// ============================================================================

/// POST /verify - Response Body
#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    /// Result: "OK", "FAIL", or "WARN"
    pub result: String,

    /// Manifest hash (SHA3-256, 0x-prefixed)
    pub manifest_hash: String,

    /// Proof hash (SHA3-256, 0x-prefixed)
    pub proof_hash: String,

    /// Rule trace (which rules were checked)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<serde_json::Value>,

    /// Ed25519 signature (base64)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,

    /// RFC3161 timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,

    /// Detailed verification report
    pub report: VerifyReport,
}

// ============================================================================
// Helper functions for defaults (exposed for testing)
// ============================================================================

#[cfg(test)]
pub fn test_default_backend() -> String {
    default_backend()
}

#[cfg(test)]
pub fn test_default_true() -> bool {
    default_true()
}
