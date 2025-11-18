use async_trait::async_trait;
use anyhow::Result;
use crate::policy::Policy;
use super::metadata::{CompiledPolicy, PolicyMetadata, PolicyStatus};

/// PolicyStore Trait - Abstraction for policy persistence
#[async_trait]
pub trait PolicyStore: Send + Sync {
    /// Save a policy and return metadata
    /// SECURITY: Hash is computed from policy content, never from client input
    async fn save(&self, policy: Policy) -> Result<PolicyMetadata>;

    /// Get policy by UUID
    async fn get(&self, id: &str) -> Result<Option<CompiledPolicy>>;

    /// Get policy by hash (0x-prefixed SHA3-256)
    async fn get_by_hash(&self, hash: &str) -> Result<Option<CompiledPolicy>>;

    /// List all policies (optionally filtered by status)
    async fn list(&self, status_filter: Option<PolicyStatus>) -> Result<Vec<PolicyMetadata>>;

    /// Update policy status
    async fn set_status(&self, id: &str, status: PolicyStatus) -> Result<()>;
}

/// Compute SHA3-256 hash of policy (deterministic)
pub fn compute_policy_hash(policy: &Policy) -> Result<String> {
    use sha3::{Digest, Sha3_256};

    // Canonical JSON serialization
    let json = serde_json::to_string(policy)?;

    let mut hasher = Sha3_256::new();
    hasher.update(json.as_bytes());
    let result = hasher.finalize();

    Ok(format!("0x{}", hex::encode(result)))
}

/// Generate ISO 8601 timestamp
pub fn now_iso8601() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}
