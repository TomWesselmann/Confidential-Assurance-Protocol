// Policy Module - Policy Store & Management System
//
// This module provides persistent policy storage with multiple backend implementations

pub mod types;        // Core Policy structures (Policy, PolicyConstraints, PolicyInfo)
pub mod metadata;     // PolicyMetadata, PolicyStatus, CompiledPolicy
pub mod store;        // PolicyStore Trait
pub mod in_memory;    // InMemoryPolicyStore
pub mod sqlite;       // SqlitePolicyStore
// pub mod filesystem;   // FileSystemPolicyStore (planned)

// Re-export core types for backward compatibility
pub use types::{Policy, PolicyConstraints, PolicyInfo};

// Re-export new types
pub use metadata::{CompiledPolicy, PolicyMetadata, PolicyStatus};
pub use store::{compute_policy_hash, now_iso8601, PolicyStore};
pub use in_memory::InMemoryPolicyStore;
pub use sqlite::SqlitePolicyStore;
