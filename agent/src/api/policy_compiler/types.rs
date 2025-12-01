//! Policy Compiler Types - Request/Response structures
//!
//! Provides REST API types for policy compilation endpoints.

use crate::policy_v2::{IrV1, LintDiagnostic, PolicyV2};
use serde::{Deserialize, Serialize};

/// Request for compiling a PolicyV2
#[derive(Debug, Deserialize)]
pub struct PolicyV2CompileRequest {
    /// Policy YAML (base64-encoded) or direct PolicyV2 JSON
    #[serde(default)]
    pub policy_yaml: Option<String>,

    /// Direct PolicyV2 JSON (alternative to policy_yaml)
    #[serde(default)]
    pub policy: Option<PolicyV2>,

    /// Lint mode (strict or relaxed)
    #[serde(default = "default_lint_mode")]
    pub lint_mode: String,

    /// Persist policy and IR in store
    #[serde(default)]
    pub persist: bool,
}

fn default_lint_mode() -> String {
    "strict".to_string()
}

/// Response after PolicyV2 compilation
#[derive(Debug, Serialize)]
pub struct PolicyV2CompileResponse {
    /// Policy ID from PolicyV2.id
    pub policy_id: String,

    /// Policy hash (SHA3-256)
    pub policy_hash: String,

    /// Compiled IR v1
    pub ir: IrV1,

    /// IR hash (SHA3-256)
    pub ir_hash: String,

    /// Lint diagnostics (warnings and errors)
    pub lints: Vec<LintDiagnostic>,

    /// Whether policy was stored
    pub stored: bool,

    /// ETag for caching (format: "ir:sha3-256:...")
    pub etag: String,
}

/// Response for policy retrieval with ETag
#[derive(Debug, Serialize)]
pub struct PolicyV2GetResponse {
    /// Policy ID
    pub policy_id: String,

    /// Policy version
    pub version: String,

    /// Policy hash
    pub policy_hash: String,

    /// Compiled IR v1
    pub ir: IrV1,

    /// IR hash
    pub ir_hash: String,

    /// ETag
    pub etag: String,
}
