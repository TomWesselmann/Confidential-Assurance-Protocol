//! Manifest Types - Core data structures
//!
//! Provides the main Manifest struct and related info types.

use crate::policy::PolicyInfo;
use serde::{Deserialize, Serialize};

use super::anchor::TimeAnchor;

/// Manifest Schema Version (JSON Schema Draft 2020-12)
pub const MANIFEST_SCHEMA_VERSION: &str = "manifest.v1.0";

/// Audit-Informationen für Manifest
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditInfo {
    pub tail_digest: String,
    pub events_count: u64,
}

/// Proof-Informationen für Manifest
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProofInfo {
    #[serde(rename = "type")]
    pub proof_type: String,
    pub status: String,
}

/// Signatur-Informationen
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SignatureInfo {
    pub alg: String,
    pub signer: String,
    pub pubkey_hex: String,
    pub sig_hex: String,
}

/// Manifest-Datenstruktur
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Manifest {
    pub version: String,
    pub created_at: String,
    pub supplier_root: String,
    pub ubo_root: String,
    pub company_commitment_root: String,
    pub policy: PolicyInfo,
    pub audit: AuditInfo,
    pub proof: ProofInfo,
    pub signatures: Vec<SignatureInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_anchor: Option<TimeAnchor>,
}
