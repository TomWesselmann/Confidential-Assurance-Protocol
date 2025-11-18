use async_trait::async_trait;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::policy::Policy;
use super::metadata::{CompiledPolicy, PolicyMetadata, PolicyStatus};
use super::store::{compute_policy_hash, now_iso8601, PolicyStore};

/// In-Memory Policy Store (for testing and development)
#[derive(Clone)]
pub struct InMemoryPolicyStore {
    // HashMap<policy_id, CompiledPolicy>
    policies: Arc<Mutex<HashMap<String, CompiledPolicy>>>,
    // HashMap<hash, policy_id> for fast hash lookup
    hash_index: Arc<Mutex<HashMap<String, String>>>,
}

impl InMemoryPolicyStore {
    pub fn new() -> Self {
        Self {
            policies: Arc::new(Mutex::new(HashMap::new())),
            hash_index: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryPolicyStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PolicyStore for InMemoryPolicyStore {
    async fn save(&self, policy: Policy) -> Result<PolicyMetadata> {
        let hash = compute_policy_hash(&policy)?;
        let now = now_iso8601();

        // Check if policy with same hash already exists
        let existing_id = {
            let hash_idx = self.hash_index.lock().unwrap();
            hash_idx.get(&hash).cloned()
        };

        if let Some(existing_id) = existing_id {
            // Update existing policy's timestamp
            let mut policies = self.policies.lock().unwrap();
            if let Some(existing) = policies.get_mut(&existing_id) {
                existing.metadata.updated_at = now.clone();
                return Ok(existing.metadata.clone());
            }
        }

        // Create new policy
        let id = Uuid::new_v4();
        let metadata = PolicyMetadata {
            id,
            name: policy.name.clone(),
            version: policy.version.clone(),
            hash: hash.clone(),
            status: PolicyStatus::Active,
            created_at: now.clone(),
            updated_at: now,
            description: Some(policy.notes.clone()),
        };

        let compiled = CompiledPolicy {
            metadata: metadata.clone(),
            policy,
            compiled_bytes: None, // Mock backend doesn't compile
        };

        let id_str = id.to_string();
        self.policies
            .lock()
            .unwrap()
            .insert(id_str.clone(), compiled);
        self.hash_index.lock().unwrap().insert(hash, id_str);

        Ok(metadata)
    }

    async fn get(&self, id: &str) -> Result<Option<CompiledPolicy>> {
        let policies = self.policies.lock().unwrap();
        Ok(policies.get(id).cloned())
    }

    async fn get_by_hash(&self, hash: &str) -> Result<Option<CompiledPolicy>> {
        let id = {
            let hash_idx = self.hash_index.lock().unwrap();
            hash_idx.get(hash).cloned()
        };

        if let Some(id) = id {
            return self.get(&id).await;
        }
        Ok(None)
    }

    async fn list(&self, status_filter: Option<PolicyStatus>) -> Result<Vec<PolicyMetadata>> {
        let policies = self.policies.lock().unwrap();
        let mut result: Vec<_> = policies
            .values()
            .filter(|p| {
                if let Some(status) = status_filter {
                    p.metadata.status == status
                } else {
                    true
                }
            })
            .map(|p| p.metadata.clone())
            .collect();

        // Sort by created_at descending
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(result)
    }

    async fn set_status(&self, id: &str, status: PolicyStatus) -> Result<()> {
        let mut policies = self.policies.lock().unwrap();
        if let Some(policy) = policies.get_mut(id) {
            policy.metadata.status = status;
            policy.metadata.updated_at = now_iso8601();
            Ok(())
        } else {
            Err(anyhow!("Policy not found: {}", id))
        }
    }
}
