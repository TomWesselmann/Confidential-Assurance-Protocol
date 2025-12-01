//! Registry v1.0 - Re-export layer for backward compatibility
//!
//! This module re-exports types and functions from the new modular structure:
//! - entry.rs: RegistryEntry struct
//! - signing.rs: Entry signing/verification
//! - timestamp.rs: Timestamp and providers
//! - store.rs: Registry, RegistryStore, backends

// Re-export from entry module
pub use super::entry::RegistryEntry;

// Re-export from signing module
pub use super::signing::{sign_entry, validate_key_status, verify_entry_signature};

// Re-export from timestamp module
pub use super::timestamp::{
    make_provider, provider_from_cli, verify_timestamp_from_file, MockRfc3161Provider,
    ProviderKind, RealRfc3161Provider, Timestamp, TimestampProvider,
};

// Re-export from store module
pub use super::store::{
    compute_file_hash, open_store, verify_entry_from_file, JsonRegistryStore, Registry,
    RegistryBackend, RegistryStore, SqliteRegistryStore,
};

#[cfg(test)]
mod tests {
    use super::*;

    // Integration tests ensuring all re-exports work correctly

    #[test]
    fn test_registry_reexport() {
        let registry = Registry::new();
        assert_eq!(registry.registry_version, "1.0");
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_registry_entry_reexport() {
        let entry = RegistryEntry::new(
            "proof_001".to_string(),
            "0xabc123".to_string(),
            "0xdef456".to_string(),
            chrono::Utc::now().to_rfc3339(),
        );
        assert_eq!(entry.id, "proof_001");
    }

    #[test]
    fn test_timestamp_reexport() {
        let ts = Timestamp::create_mock("0x1234".to_string());
        assert_eq!(ts.version, "tsr.v1");
        assert!(ts.verify("0x1234"));
    }

    #[test]
    fn test_signing_reexport() {
        let mut entry = RegistryEntry::new(
            "proof_001".to_string(),
            "0xabc123".to_string(),
            "0xdef456".to_string(),
            chrono::Utc::now().to_rfc3339(),
        );
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&[42u8; 32]);

        sign_entry(&mut entry, &signing_key).unwrap();
        assert!(entry.signature.is_some());

        let valid = verify_entry_signature(&entry).unwrap();
        assert!(valid);
    }

    #[test]
    fn test_store_backend_reexport() {
        assert_eq!(RegistryBackend::Json, RegistryBackend::Json);
        assert_ne!(RegistryBackend::Json, RegistryBackend::Sqlite);
    }
}
