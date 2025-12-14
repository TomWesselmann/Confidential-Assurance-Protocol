#![allow(dead_code)]
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use super::metadata::{CompiledPolicy, PolicyMetadata, PolicyStatus};
use super::store::{compute_policy_hash, now_iso8601, PolicyStore};
use crate::policy::Policy;

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
            let hash_idx = self.hash_index.lock().expect("Failed to acquire in-memory policy store lock");
            hash_idx.get(&hash).cloned()
        };

        if let Some(existing_id) = existing_id {
            // Update existing policy's timestamp
            let mut policies = self.policies.lock().expect("Failed to acquire in-memory policy store lock");
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
        self.hash_index.lock().expect("Failed to acquire in-memory policy store lock").insert(hash, id_str);

        Ok(metadata)
    }

    async fn get(&self, id: &str) -> Result<Option<CompiledPolicy>> {
        let policies = self.policies.lock().expect("Failed to acquire in-memory policy store lock");
        Ok(policies.get(id).cloned())
    }

    async fn get_by_hash(&self, hash: &str) -> Result<Option<CompiledPolicy>> {
        let id = {
            let hash_idx = self.hash_index.lock().expect("Failed to acquire in-memory policy store lock");
            hash_idx.get(hash).cloned()
        };

        if let Some(id) = id {
            return self.get(&id).await;
        }
        Ok(None)
    }

    async fn list(&self, status_filter: Option<PolicyStatus>) -> Result<Vec<PolicyMetadata>> {
        let policies = self.policies.lock().expect("Failed to acquire in-memory policy store lock");
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
        let mut policies = self.policies.lock().expect("Failed to acquire in-memory policy store lock");
        if let Some(policy) = policies.get_mut(id) {
            policy.metadata.status = status;
            policy.metadata.updated_at = now_iso8601();
            Ok(())
        } else {
            Err(anyhow!("Policy not found: {}", id))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::{types::PolicyConstraints, Policy};

    fn create_test_policy(name: &str) -> Policy {
        Policy {
            version: "lksg.v1".to_string(),
            name: name.to_string(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            constraints: PolicyConstraints {
                require_at_least_one_ubo: true,
                supplier_count_max: 10,
                ubo_count_min: None,
                require_statement_roots: None,
            },
            notes: "Test policy".to_string(),
        }
    }

    #[tokio::test]
    async fn test_new() {
        let store = InMemoryPolicyStore::new();
        let policies = store.policies.lock().expect("Failed to acquire in-memory policy store lock");
        assert_eq!(policies.len(), 0);
    }

    #[tokio::test]
    async fn test_default() {
        let store = InMemoryPolicyStore::default();
        let policies = store.policies.lock().expect("Failed to acquire in-memory policy store lock");
        assert_eq!(policies.len(), 0);
    }

    #[tokio::test]
    async fn test_save() {
        let store = InMemoryPolicyStore::new();
        let policy = create_test_policy("Test Policy 1");

        let metadata = store.save(policy.clone()).await.unwrap();

        assert_eq!(metadata.name, "Test Policy 1");
        assert_eq!(metadata.version, "lksg.v1");
        assert_eq!(metadata.status, PolicyStatus::Active);
        assert!(!metadata.hash.is_empty());
    }

    #[tokio::test]
    async fn test_save_deduplication() {
        let store = InMemoryPolicyStore::new();
        let policy1 = create_test_policy("Test Policy");
        let policy2 = create_test_policy("Test Policy"); // Same content

        let metadata1 = store.save(policy1).await.unwrap();
        let metadata2 = store.save(policy2).await.unwrap();

        // Should return same ID for same hash
        assert_eq!(metadata1.id, metadata2.id);
        assert_eq!(metadata1.hash, metadata2.hash);

        // updated_at should be different
        assert!(metadata2.updated_at >= metadata1.updated_at);
    }

    #[tokio::test]
    async fn test_get() {
        let store = InMemoryPolicyStore::new();
        let policy = create_test_policy("Test Policy");

        let metadata = store.save(policy).await.unwrap();
        let id_str = metadata.id.to_string();

        let retrieved = store.get(&id_str).await.unwrap();
        assert!(retrieved.is_some());

        let compiled = retrieved.unwrap();
        assert_eq!(compiled.metadata.name, "Test Policy");
    }

    #[tokio::test]
    async fn test_get_not_found() {
        let store = InMemoryPolicyStore::new();
        let uuid = Uuid::new_v4().to_string();

        let result = store.get(&uuid).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_by_hash() {
        let store = InMemoryPolicyStore::new();
        let policy = create_test_policy("Test Policy");

        let metadata = store.save(policy).await.unwrap();

        let retrieved = store.get_by_hash(&metadata.hash).await.unwrap();
        assert!(retrieved.is_some());

        let compiled = retrieved.unwrap();
        assert_eq!(compiled.metadata.hash, metadata.hash);
    }

    #[tokio::test]
    async fn test_get_by_hash_not_found() {
        let store = InMemoryPolicyStore::new();

        let result = store.get_by_hash("nonexistent_hash").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_list_all() {
        let store = InMemoryPolicyStore::new();

        store.save(create_test_policy("Policy 1")).await.unwrap();
        store.save(create_test_policy("Policy 2")).await.unwrap();
        store.save(create_test_policy("Policy 3")).await.unwrap();

        let all = store.list(None).await.unwrap();
        assert_eq!(all.len(), 3);
    }

    #[tokio::test]
    async fn test_list_filtered_by_status() {
        let store = InMemoryPolicyStore::new();

        let _meta1 = store.save(create_test_policy("Active 1")).await.unwrap();
        let _meta2 = store.save(create_test_policy("Active 2")).await.unwrap();
        let meta3 = store
            .save(create_test_policy("Deprecated 1"))
            .await
            .unwrap();

        // Set one to Deprecated
        store
            .set_status(&meta3.id.to_string(), PolicyStatus::Deprecated)
            .await
            .unwrap();

        // Filter for Active
        let active = store.list(Some(PolicyStatus::Active)).await.unwrap();
        assert_eq!(active.len(), 2);

        // Filter for Deprecated
        let deprecated = store.list(Some(PolicyStatus::Deprecated)).await.unwrap();
        assert_eq!(deprecated.len(), 1);
    }

    #[tokio::test]
    async fn test_list_sorted_by_created_at() {
        let store = InMemoryPolicyStore::new();

        store.save(create_test_policy("Policy 1")).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        store.save(create_test_policy("Policy 2")).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        store.save(create_test_policy("Policy 3")).await.unwrap();

        let all = store.list(None).await.unwrap();

        // Should be sorted descending by created_at
        assert_eq!(all[0].name, "Policy 3");
        assert_eq!(all[1].name, "Policy 2");
        assert_eq!(all[2].name, "Policy 1");
    }

    #[tokio::test]
    async fn test_set_status() {
        let store = InMemoryPolicyStore::new();
        let metadata = store.save(create_test_policy("Test")).await.unwrap();
        let id_str = metadata.id.to_string();

        // Initially Active
        assert_eq!(metadata.status, PolicyStatus::Active);

        // Wait a bit to ensure timestamp difference
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Change to Deprecated
        store
            .set_status(&id_str, PolicyStatus::Deprecated)
            .await
            .unwrap();

        let retrieved = store.get(&id_str).await.unwrap().unwrap();
        assert_eq!(retrieved.metadata.status, PolicyStatus::Deprecated);

        // updated_at should have changed
        assert!(retrieved.metadata.updated_at > metadata.updated_at);
    }

    #[tokio::test]
    async fn test_set_status_not_found() {
        let store = InMemoryPolicyStore::new();
        let uuid = Uuid::new_v4().to_string();

        let result = store.set_status(&uuid, PolicyStatus::Deprecated).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Policy not found"));
    }
}
