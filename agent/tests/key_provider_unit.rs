//! Unit Tests for KeyProvider Implementations (Week 7 Track S)
//!
//! These tests verify the KeyProvider trait implementations:
//! - Software Provider
//! - PKCS#11 Provider (stub validation)
//! - CloudKMS Provider (stub validation)
//! - Provider Factory
//! - Configuration Loading

use cap_agent::providers::key_provider::{create_provider, derive_kid};
use cap_agent::providers::{KeyError, KeyProvider, ProviderConfig, SoftwareProvider};
use tempfile::tempdir;

#[test]
fn test_software_provider_empty_dir() {
    let temp_dir = tempdir().unwrap();
    let provider = SoftwareProvider::new(temp_dir.path(), None).unwrap();

    assert_eq!(provider.provider_id(), "software");

    // Empty directory should have no keys
    let result = provider.current_kid();
    assert!(result.is_err());
    assert!(matches!(result, Err(KeyError::NotFound(_))));

    let kids = provider.list_kids().unwrap();
    assert_eq!(kids.len(), 0, "Empty directory should have no KIDs");
}

#[test]
fn test_kid_derivation_week7_formula() {
    // Week 7 formula: blake3(pubkey || provider_id || key_name)
    let pubkey = b"test_public_key_32_bytes_long!!";
    let provider_id = "software";
    let key_name = "test-key";

    let kid1 = derive_kid(pubkey, provider_id, key_name);
    let kid2 = derive_kid(pubkey, provider_id, key_name);

    assert_eq!(kid1, kid2, "KID derivation must be deterministic");
    assert!(kid1.starts_with("0x"), "KID must be 0x-prefixed");
    assert_eq!(kid1.len(), 66, "KID must be 64 hex chars + 0x prefix");
}

#[test]
fn test_kid_different_providers_unique() {
    let pubkey = b"test_public_key_32_bytes_long!!";
    let key_name = "test-key";

    let kid_software = derive_kid(pubkey, "software", key_name);
    let kid_pkcs11 = derive_kid(pubkey, "pkcs11", key_name);
    let kid_cloudkms = derive_kid(pubkey, "cloudkms-gcp", key_name);

    assert_ne!(
        kid_software, kid_pkcs11,
        "Different providers must have different KIDs"
    );
    assert_ne!(
        kid_software, kid_cloudkms,
        "Different providers must have different KIDs"
    );
    assert_ne!(
        kid_pkcs11, kid_cloudkms,
        "Different providers must have different KIDs"
    );
}

#[test]
fn test_kid_different_key_names_unique() {
    let pubkey = b"test_public_key_32_bytes_long!!";
    let provider_id = "software";

    let kid1 = derive_kid(pubkey, provider_id, "key1");
    let kid2 = derive_kid(pubkey, provider_id, "key2");

    assert_ne!(
        kid1, kid2,
        "Different key names must produce different KIDs"
    );
}

#[test]
fn test_provider_config_yaml_software() {
    let yaml = r#"
provider: software
software:
  keys_dir: /test/keys
  default_key: test-key
"#;

    let config: ProviderConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.provider, "software");
    assert!(config.software.is_some());

    let sw_config = config.software.unwrap();
    assert_eq!(sw_config.keys_dir, "/test/keys");
    assert_eq!(sw_config.default_key.unwrap(), "test-key");
}

#[test]
fn test_provider_config_yaml_pkcs11() {
    let yaml = r#"
provider: pkcs11
pkcs11:
  module: /usr/lib/softhsm/libsofthsm2.so
  slot: 0
  pin_env: PKCS11_PIN
  key_label: cap-signing
"#;

    let config: ProviderConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.provider, "pkcs11");
    assert!(config.pkcs11.is_some());

    let p11_config = config.pkcs11.unwrap();
    assert_eq!(p11_config.module, "/usr/lib/softhsm/libsofthsm2.so");
    assert_eq!(p11_config.slot, 0);
    assert_eq!(p11_config.pin_env, "PKCS11_PIN");
    assert_eq!(p11_config.key_label, "cap-signing");
}

#[test]
fn test_provider_config_yaml_cloudkms_gcp() {
    let yaml = r#"
provider: cloudkms
cloudkms:
  cloud: gcp
  project: my-project
  location: europe-west1
  keyring: my-keyring
  key: signing-key
  version: "1"
"#;

    let config: ProviderConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.provider, "cloudkms");
    assert!(config.cloudkms.is_some());

    let kms_config = config.cloudkms.unwrap();
    assert_eq!(kms_config.cloud, "gcp");
    assert_eq!(kms_config.project, "my-project");
    assert_eq!(kms_config.location.unwrap(), "europe-west1");
    assert_eq!(kms_config.keyring.unwrap(), "my-keyring");
    assert_eq!(kms_config.key, "signing-key");
    assert_eq!(kms_config.version, "1");
}

#[test]
fn test_provider_factory_software() {
    let temp_dir = tempdir().unwrap();

    let config = ProviderConfig {
        provider: "software".to_string(),
        software: Some(cap_agent::providers::key_provider::SoftwareConfig {
            keys_dir: temp_dir.path().to_str().unwrap().to_string(),
            default_key: None,
        }),
        pkcs11: None,
        cloudkms: None,
    };

    let provider = create_provider(config).unwrap();
    assert_eq!(provider.provider_id(), "software");
}

#[test]
fn test_provider_factory_missing_config() {
    let config = ProviderConfig {
        provider: "software".to_string(),
        software: None, // Missing required config
        pkcs11: None,
        cloudkms: None,
    };

    let result = create_provider(config);
    assert!(result.is_err());
    assert!(matches!(result, Err(KeyError::ConfigError(_))));
}

#[test]
fn test_key_error_display() {
    let err = KeyError::NotFound("test-key".to_string());
    assert!(err.to_string().contains("test-key"));

    let err = KeyError::AuthenticationFailed("Invalid PIN".to_string());
    assert!(err.to_string().contains("Authentication failed"));

    let err = KeyError::Timeout("KMS request timeout".to_string());
    assert!(err.to_string().contains("Timeout"));

    let err = KeyError::TokenLocked("HSM token locked".to_string());
    assert!(err.to_string().contains("Token locked"));
}

#[cfg(feature = "pkcs11")]
#[test]
fn test_pkcs11_provider_stub_not_implemented() {
    use cap_agent::providers::pkcs11::Pkcs11InternalConfig;
    use cap_agent::providers::pkcs11::Pkcs11Provider;

    let config = Pkcs11InternalConfig {
        module_path: "/usr/lib/softhsm/libsofthsm2.so".to_string(),
        slot: 0,
        pin: "1234".to_string(),
        key_label: "test-key".to_string(),
    };

    let provider = Pkcs11Provider::new(config).unwrap();
    assert_eq!(provider.provider_id(), "pkcs11");

    // Stub should return not-implemented error
    let result = provider.current_kid();
    assert!(result.is_err());
    assert!(matches!(result, Err(KeyError::ProviderError(_))));
}

#[cfg(feature = "cloudkms")]
#[test]
fn test_cloudkms_provider_stub_not_implemented() {
    use cap_agent::providers::cloudkms::{CloudKmsInternalConfig, CloudKmsProvider, CloudProvider};

    let config = CloudKmsInternalConfig {
        provider: CloudProvider::Gcp,
        project_or_region: "my-project".to_string(),
        location: Some("europe-west1".to_string()),
        keyring: Some("my-keyring".to_string()),
        key_name: "signing-key".to_string(),
        key_version: "1".to_string(),
    };

    let provider = CloudKmsProvider::new(config).unwrap();
    assert_eq!(provider.provider_id(), "cloudkms-gcp");

    // Stub should return not-implemented error
    let result = provider.current_kid();
    assert!(result.is_err());
    assert!(matches!(result, Err(KeyError::ProviderError(_))));
}
