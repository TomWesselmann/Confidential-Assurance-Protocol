//! Registry Entry - Data structure for a single proof entry
//!
//! Contains all fields for a registry entry including:
//! - Core fields (id, hashes, timestamp)
//! - Signature fields (v0.8)
//! - BLOB fields (v0.9)
//! - Key management fields (v0.10)

use serde::{Deserialize, Serialize};

/// Registry-Eintrag für einen einzelnen Proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub id: String,
    pub manifest_hash: String,
    pub proof_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp_file: Option<String>,
    pub registered_at: String, // RFC3339
    /// Ed25519 signature over entry_hash (optional for backward compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    /// Base64-encoded Ed25519 public key (optional for backward compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,

    // BLOB Store Fields (v0.9)
    /// BLAKE3 hash of manifest BLOB
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob_manifest: Option<String>,
    /// BLAKE3 hash of proof BLOB
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob_proof: Option<String>,
    /// BLAKE3 hash of WASM verifier BLOB
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob_wasm: Option<String>,
    /// SHA3-256 hash of ABI JSON BLOB
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob_abi: Option<String>,

    // Self-Verification Fields (v0.9)
    /// Self-verification status: unknown, ok, fail
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selfverify_status: Option<String>,
    /// RFC3339 timestamp of last self-verification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selfverify_at: Option<String>,
    /// Verifier name (e.g., "cap-wasm-verifier")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verifier_name: Option<String>,
    /// Verifier version (e.g., "1.0.0")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verifier_version: Option<String>,

    // Key Management Fields (v0.10)
    /// Key Identifier (16 bytes = 32 hex chars, derived from public key)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kid: Option<String>,
    /// Signature scheme (e.g., "ed25519")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_scheme: Option<String>,
}

impl RegistryEntry {
    /// Creates a new entry with required fields, all optional fields set to None
    pub fn new(id: String, manifest_hash: String, proof_hash: String, registered_at: String) -> Self {
        Self {
            id,
            manifest_hash,
            proof_hash,
            timestamp_file: None,
            registered_at,
            signature: None,
            public_key: None,
            blob_manifest: None,
            blob_proof: None,
            blob_wasm: None,
            blob_abi: None,
            selfverify_status: None,
            selfverify_at: None,
            verifier_name: None,
            verifier_version: None,
            kid: None,
            signature_scheme: None,
        }
    }

    /// Builder method to set timestamp_file
    pub fn with_timestamp_file(mut self, file: Option<String>) -> Self {
        self.timestamp_file = file;
        self
    }
}
