//! Shared types for Taurin Desktop App
//!
//! This module defines all request/response types used by Tauri commands.
//! Types follow camelCase for JSON serialization (Tauri convention).

use serde::{Deserialize, Serialize};

// ============================================================================
// Enums
// ============================================================================

/// Type of CSV file for import
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CsvType {
    Suppliers,
    Ubos,
    Sanctions,
    Jurisdictions,
}

/// Workflow step status
#[allow(dead_code)] // Used for workflow state serialization
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StepStatus {
    Pending,
    InProgress,
    Completed,
    Error,
}

// ============================================================================
// Project Types
// ============================================================================

/// Project information
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInfo {
    pub path: String,
    pub name: String,
    pub created_at: String,
}

/// Project status with workflow state
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectStatus {
    pub info: ProjectInfo,
    pub has_suppliers_csv: bool,
    pub has_ubos_csv: bool,
    pub has_policy: bool,
    pub has_commitments: bool,
    pub has_manifest: bool,
    pub has_proof: bool,
    pub current_step: String,
}

/// Project metadata stored in taurin.project.json
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMeta {
    pub schema: String,
    pub name: String,
    pub created_at: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub cap_version: Option<String>,
}

impl ProjectMeta {
    pub const SCHEMA_VERSION: &'static str = "taurin.project.v1";
}

// ============================================================================
// Import Types
// ============================================================================

/// Result of CSV import
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub csv_type: String,
    pub record_count: usize,
    pub hash: String,
    pub destination: String,
}

// ============================================================================
// Commitments Types
// ============================================================================

/// Result of commitment creation
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitmentsResult {
    pub supplier_root: String,
    pub ubo_root: String,
    pub company_root: String,
    pub path: String,
}

// ============================================================================
// Policy Types
// ============================================================================

/// Policy information
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyInfo {
    pub name: String,
    pub version: String,
    pub hash: String,
    pub rules_count: usize,
    pub path: String,
}

// ============================================================================
// Manifest Types
// ============================================================================

/// Result of manifest build
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestResult {
    pub manifest_hash: String,
    pub path: String,
    pub supplier_root: String,
    pub ubo_root: String,
    pub policy_hash: String,
}

// ============================================================================
// Proof Types
// ============================================================================

/// Result of proof build
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProofResult {
    pub proof_hash: String,
    pub path: String,
    pub backend: String,
}

/// Progress event for proof building
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProofProgress {
    pub percent: u8,
    pub message: String,
}

// ============================================================================
// Export Types
// ============================================================================

/// Result of bundle export
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub bundle_path: String,
    pub size_bytes: u64,
    pub hash: String,
    pub files: Vec<String>,
}

// ============================================================================
// Verify Types (existing, moved here)
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyBundleRequest {
    pub bundle_path: String,
    #[serde(default)]
    pub options: Option<VerifyOptionsInput>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyOptionsInput {
    #[serde(default)]
    pub check_timestamp: bool,
    #[serde(default)]
    pub check_registry: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyBundleResponse {
    pub status: String,
    pub bundle_id: String,
    pub manifest_hash: String,
    pub proof_hash: String,
    pub signature_valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp_valid: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry_match: Option<bool>,
    pub details: serde_json::Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleInfo {
    pub bundle_id: String,
    pub schema: String,
    pub created_at: String,
    pub proof_units: Vec<ProofUnitInfo>,
    pub file_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProofUnitInfo {
    pub id: String,
    pub policy_id: String,
    pub backend: String,
}

// ============================================================================
// Audit Types
// ============================================================================

/// Audit event result status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum AuditEventResult {
    Ok,
    Warn,
    Fail,
}

/// Unified audit event (supports both V1.0 and V2.0 formats)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEvent {
    /// Sequence number (V1.0) - optional for V2.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq: Option<u64>,

    /// Timestamp (ISO 8601)
    pub ts: String,

    /// Event type
    pub event: String,

    /// Event details (V1.0 format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,

    /// Policy ID (V2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_id: Option<String>,

    /// IR hash (V2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ir_hash: Option<String>,

    /// Manifest hash (V2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest_hash: Option<String>,

    /// Result status (V2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<AuditEventResult>,

    /// Run ID (V2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,

    /// Previous hash (V1.0: prev_digest, V2.0: prev_hash)
    pub prev_hash: String,

    /// Self hash (V1.0: digest, V2.0: self_hash)
    pub self_hash: String,
}

/// Audit log with events and chain status
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditLog {
    /// List of audit events
    pub events: Vec<AuditEvent>,

    /// Total event count (for pagination)
    pub total_count: usize,

    /// Whether the hash chain is valid
    pub chain_valid: bool,

    /// Offset used for pagination
    pub offset: usize,

    /// Limit used for pagination
    pub limit: usize,
}

/// Error in hash chain
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainError {
    /// Index of the tampered event
    pub index: usize,

    /// Event timestamp
    pub timestamp: String,

    /// Error type
    pub error_type: String,

    /// Expected hash
    pub expected: String,

    /// Found hash
    pub found: String,
}

/// Result of hash chain verification
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainVerifyResult {
    /// Whether the chain is valid
    pub valid: bool,

    /// Number of events verified
    pub verified_count: usize,

    /// List of errors found
    pub errors: Vec<ChainError>,

    /// Tail hash (last event hash)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tail_hash: Option<String>,
}

// ============================================================================
// Signing Types
// ============================================================================

/// Information about a generated or loaded key
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyInfo {
    /// Unique key identifier (derived from public key)
    pub kid: String,

    /// Signer name (user-provided)
    pub signer_name: String,

    /// Path to public key file
    pub public_key_path: String,

    /// SHA-256 fingerprint of public key
    pub fingerprint: String,

    /// Creation timestamp (ISO 8601)
    pub created_at: String,
}

/// Result of manifest signing operation
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignResult {
    /// Whether signing was successful
    pub success: bool,

    /// Signer name
    pub signer: String,

    /// Signature hash (first 16 chars)
    pub signature_hash: String,

    /// Full signature in hex
    pub signature_hex: String,

    /// Path to updated manifest
    pub manifest_path: String,
}

/// Result of signature verification
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignatureVerifyResult {
    /// Whether signature is valid
    pub valid: bool,

    /// Signer name from signature
    pub signer: String,

    /// Algorithm used
    pub algorithm: String,

    /// Error message if invalid
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
