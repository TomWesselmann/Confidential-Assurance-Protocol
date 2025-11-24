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
pub use types::{Policy, PolicyInfo};

// Allow unused imports for types that may be used in different build targets
#[allow(unused_imports)]
pub use types::PolicyConstraints;

// Policy Store types (currently unused, but kept for backward compatibility)
#[allow(unused_imports)]
pub use metadata::{CompiledPolicy, PolicyMetadata, PolicyStatus};
#[allow(unused_imports)]
pub use store::{compute_policy_hash, now_iso8601, PolicyStore};
#[allow(unused_imports)]
pub use in_memory::InMemoryPolicyStore;
#[allow(unused_imports)]
pub use sqlite::SqlitePolicyStore;
