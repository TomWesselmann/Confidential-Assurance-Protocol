//! Policy Compiler Test Helpers - Utilities for integration tests
//!
//! Provides functions to manipulate the LRU cache for testing purposes.

use std::sync::Arc;

use crate::policy_v2::{IrV1, PolicyV2};

use super::cache::{get_cache, get_id_index, PolicyEntry};

/// Clear the LRU cache and ID index (for testing)
pub fn test_clear_cache() {
    let cache = get_cache();
    let mut lru = cache.lock().expect("Failed to lock cache");
    lru.clear();

    let id_index = get_id_index();
    let mut index = id_index.lock().expect("Failed to lock index");
    index.clear();
}

/// Get current cache size (for testing)
pub fn test_get_cache_size() -> usize {
    let cache = get_cache();
    let lru = cache.lock().expect("Failed to lock cache");
    lru.len()
}

/// Check if a policy_hash exists in cache (for testing)
pub fn test_cache_contains(policy_hash: &str) -> bool {
    let cache = get_cache();
    let lru = cache.lock().expect("Failed to lock cache");
    lru.peek(policy_hash).is_some()
}

/// Insert a test policy into cache (for testing)
pub fn test_insert_policy(policy: PolicyV2, policy_hash: String, ir: IrV1, ir_hash: String) {
    let cache = get_cache();
    let mut lru = cache.lock().expect("Failed to lock cache");

    let entry = Arc::new(PolicyEntry {
        policy: policy.clone(),
        policy_hash: policy_hash.clone(),
        ir,
        ir_hash,
    });

    lru.put(policy_hash.clone(), entry);

    // Update ID index
    let id_index = get_id_index();
    let mut index = id_index.lock().expect("Failed to lock index");
    index.insert(policy.id.clone(), policy_hash);
}

/// Access (touch) a policy in cache to update LRU order (for testing)
pub fn test_touch_policy(policy_hash: &str) -> bool {
    let cache = get_cache();
    let mut lru = cache.lock().expect("Failed to lock cache");
    lru.get(policy_hash).is_some()
}
