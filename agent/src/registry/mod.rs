//! Registry Module v1.1
//!
//! Modular registry structure with schema versioning and migration support.

pub mod api;
pub mod migrate;
pub mod schema;
pub mod v1_0;

// Re-export v1.0 types (for backward compatibility and migration)
#[allow(unused_imports)]
pub use v1_0::{
    compute_file_hash, open_store, sign_entry, validate_key_status, verify_entry_from_file,
    verify_entry_signature, verify_timestamp_from_file, Registry, RegistryBackend, RegistryEntry,
    Timestamp,
};

// Re-export v1.1 types (used in tests)
#[allow(unused_imports)]
pub use migrate::{backfill_kid, migrate_to_v1_1};
#[allow(unused_imports)]
pub use schema::{RegistryEntryV1_1, RegistryMeta, RegistryV1_1};

// Re-export unified API
pub use api::UnifiedRegistry;
