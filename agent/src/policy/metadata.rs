use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Policy Status Lifecycle
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PolicyStatus {
    /// Policy is active and can be used for verification
    Active,
    /// Policy is deprecated but still accessible (for old proofs)
    Deprecated,
    /// Policy is in draft state (not yet active)
    Draft,
}

/// Policy Metadata (stored in Registry)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyMetadata {
    /// Unique Policy ID (UUID v4)
    pub id: Uuid,

    /// Human-readable name
    #[serde(default)]
    pub name: String,

    /// Policy version (e.g., "lksg.v1")
    pub version: String,

    /// SHA3-256 hash of policy content (0x-prefixed hex)
    pub hash: String,

    /// Current status
    pub status: PolicyStatus,

    /// Creation timestamp (ISO 8601)
    pub created_at: String,

    /// Last update timestamp (ISO 8601)
    pub updated_at: String,

    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Compiled Policy (Policy + Metadata + Compiled Bytes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledPolicy {
    /// Metadata
    pub metadata: PolicyMetadata,

    /// Original Policy JSON
    pub policy: crate::policy::Policy,

    /// Compiled policy bytes (for zkVM)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compiled_bytes: Option<Vec<u8>>,
}
