//! Registry Module v1.1
//!
//! Modular registry structure with schema versioning and migration support.

pub mod v1_0;
pub mod schema;
pub mod migrate;
pub mod api;

// Re-export v1.0 types (for backward compatibility and migration)
pub use v1_0::{
    Registry, RegistryEntry, RegistryStore, RegistryBackend,
    JsonRegistryStore, SqliteRegistryStore, Timestamp, TimestampProvider,
    open_store, compute_file_hash, sign_entry, validate_key_status,
    verify_entry_signature, verify_timestamp_from_file, verify_entry_from_file
};

// Re-export v1.1 types
pub use schema::{RegistryV1_1, RegistryEntryV1_1, RegistryMeta};
pub use migrate::{migrate_to_v1_1, backfill_kid};

// Re-export unified API
pub use api::UnifiedRegistry;
