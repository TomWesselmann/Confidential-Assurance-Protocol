//! Key-Provider-Abstraktionen für HSM/TPM/KMS
//!
//! Dieses Modul bietet einheitliche Interfaces für verschiedene
//! Key-Management-Backends: Software (Ed25519), PKCS#11 (HSM), CloudKMS.

pub mod key_provider;
pub mod software;

#[cfg(feature = "pkcs11")]
pub mod pkcs11;

#[cfg(feature = "cloudkms")]
pub mod cloudkms;

// Re-exports
pub use key_provider::{create_provider, derive_kid, load_config, KeyError, KeyProvider, ProviderConfig, ProviderType};
pub use software::SoftwareProvider;

#[cfg(feature = "pkcs11")]
pub use self::pkcs11::Pkcs11Provider;

#[cfg(feature = "cloudkms")]
pub use cloudkms::CloudKmsProvider;
