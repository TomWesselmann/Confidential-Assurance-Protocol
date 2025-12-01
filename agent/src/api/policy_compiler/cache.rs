//! Policy IR Cache - LRU Cache for compiled policies
//!
//! Provides thread-safe LRU cache for PolicyV2 → IR mappings.

use lru::LruCache;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex, OnceLock};

use crate::policy_v2::{IrV1, PolicyV2};

/// Cache entry containing policy and compiled IR
#[derive(Debug, Clone)]
pub struct PolicyEntry {
    pub policy: PolicyV2,
    pub policy_hash: String,
    pub ir: IrV1,
    pub ir_hash: String,
}

/// LRU Cache for policy_hash → IR mapping
/// Key: policy_hash (SHA3-256)
/// Size: 1000 entries (Week 4 spec)
#[allow(clippy::type_complexity)]
static POLICY_IR_CACHE: OnceLock<Arc<Mutex<LruCache<String, Arc<PolicyEntry>>>>> = OnceLock::new();

/// Policy ID → policy_hash index for lookups
static POLICY_ID_INDEX: OnceLock<Arc<Mutex<HashMap<String, String>>>> = OnceLock::new();

/// Returns the global policy IR cache
pub fn get_cache() -> Arc<Mutex<LruCache<String, Arc<PolicyEntry>>>> {
    POLICY_IR_CACHE
        .get_or_init(|| {
            let cache = LruCache::new(NonZeroUsize::new(1000).unwrap());
            Arc::new(Mutex::new(cache))
        })
        .clone()
}

/// Returns the global policy ID → hash index
pub fn get_id_index() -> Arc<Mutex<HashMap<String, String>>> {
    POLICY_ID_INDEX
        .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
        .clone()
}
