//! KeyProvider Trait und Factory für HSM/TPM/KMS Integration
//!
//! Dieser Trait abstrahiert verschiedene Key-Management-Backends:
//! - Software: Ed25519 in-memory oder file-based
//! - PKCS11: HSM, TPM, SoftHSM2
//! - CloudKMS: GCP, AWS, Azure KMS
//!
//! KID-Ableitung: blake3(pubkey || provider_id || key_name)

use std::fmt;

/// Fehlertypen für KeyProvider-Operationen
#[derive(Debug, Clone)]
pub enum KeyError {
    /// Schlüssel nicht gefunden
    NotFound(String),
    /// Ungültige KID
    InvalidKid(String),
    /// Signatur-Fehler
    SignatureError(String),
    /// Provider-spezifischer Fehler
    ProviderError(String),
    /// I/O-Fehler
    IoError(String),
    /// Authentifizierung fehlgeschlagen (z.B. falscher PIN)
    AuthenticationFailed(String),
    /// Token/HSM gesperrt oder nicht verfügbar
    TokenLocked(String),
    /// Timeout (z.B. bei KMS-Anfragen)
    Timeout(String),
    /// Konfigurationsfehler
    ConfigError(String),
}

impl fmt::Display for KeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyError::NotFound(msg) => write!(f, "Key not found: {}", msg),
            KeyError::InvalidKid(msg) => write!(f, "Invalid KID: {}", msg),
            KeyError::SignatureError(msg) => write!(f, "Signature error: {}", msg),
            KeyError::ProviderError(msg) => write!(f, "Provider error: {}", msg),
            KeyError::IoError(msg) => write!(f, "I/O error: {}", msg),
            KeyError::AuthenticationFailed(msg) => write!(f, "Authentication failed: {}", msg),
            KeyError::TokenLocked(msg) => write!(f, "Token locked: {}", msg),
            KeyError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            KeyError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for KeyError {}

/// Abstraktes Key-Management-Interface für verschiedene Provider
///
/// Alle Implementierungen müssen thread-safe sein (Send + Sync).
pub trait KeyProvider: Send + Sync {
    /// Provider-Identifier (z.B. "software", "pkcs11", "cloudkms")
    fn provider_id(&self) -> &'static str;

    /// Gibt den aktuellen (default) KID zurück
    fn current_kid(&self) -> Result<String, KeyError>;

    /// Signiert eine Nachricht mit dem angegebenen Schlüssel
    ///
    /// Falls `kid` None ist, wird der aktuelle Default-Schlüssel verwendet.
    fn sign(&self, kid: Option<&str>, msg: &[u8]) -> Result<Vec<u8>, KeyError>;

    /// Gibt den Public Key für den angegebenen KID zurück
    fn public_key(&self, kid: &str) -> Result<Vec<u8>, KeyError>;

    /// Listet alle verfügbaren KIDs auf
    fn list_kids(&self) -> Result<Vec<String>, KeyError>;
}

/// Provider-Typ für Factory-Auswahl
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderType {
    /// Software Ed25519 (file-based oder in-memory)
    Software,
    /// PKCS#11 (HSM, TPM, SoftHSM2)
    Pkcs11,
    /// Cloud KMS (GCP, AWS, Azure)
    CloudKms,
}

impl ProviderType {
    /// Parst Provider-Typ aus String
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, KeyError> {
        match s.to_lowercase().as_str() {
            "software" => Ok(ProviderType::Software),
            "pkcs11" => Ok(ProviderType::Pkcs11),
            "cloudkms" | "cloud-kms" | "kms" => Ok(ProviderType::CloudKms),
            _ => Err(KeyError::ConfigError(format!(
                "Unknown provider type: {}. Valid: software, pkcs11, cloudkms",
                s
            ))),
        }
    }
}

/// KID-Ableitung gemäß Week 7 Spec
///
/// Formula: blake3(pubkey || provider_id || key_name)
pub fn derive_kid(pubkey: &[u8], provider_id: &str, key_name: &str) -> String {
    use blake3::Hasher;

    let mut hasher = Hasher::new();
    hasher.update(pubkey);
    hasher.update(provider_id.as_bytes());
    hasher.update(key_name.as_bytes());

    let hash = hasher.finalize();
    format!("0x{}", hex::encode(hash.as_bytes()))
}

/// Provider Configuration (loaded from YAML)
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ProviderConfig {
    /// Provider type: software, pkcs11, cloudkms
    pub provider: String,

    /// Software provider config (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software: Option<SoftwareConfig>,

    /// PKCS#11 provider config (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pkcs11: Option<Pkcs11Config>,

    /// CloudKMS provider config (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloudkms: Option<CloudKmsConfig>,
}

/// Software Provider Configuration
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SoftwareConfig {
    /// Path to keys directory
    pub keys_dir: String,

    /// Default key name (without path/extension)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_key: Option<String>,
}

/// PKCS#11 Provider Configuration
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Pkcs11Config {
    /// Path to PKCS#11 module
    pub module: String,

    /// Slot ID
    pub slot: u64,

    /// Environment variable name for PIN
    pub pin_env: String,

    /// Key label
    pub key_label: String,
}

/// CloudKMS Provider Configuration
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct CloudKmsConfig {
    /// Cloud provider: gcp, aws, azure
    pub cloud: String,

    /// GCP: project, AWS: region, Azure: vault
    pub project: String,

    /// GCP: location (e.g., "europe-west1")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,

    /// GCP: keyring name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyring: Option<String>,

    /// Key name/ID
    pub key: String,

    /// Key version
    #[serde(default = "default_version")]
    pub version: String,
}

fn default_version() -> String {
    "latest".to_string()
}

/// Creates a KeyProvider instance from configuration
///
/// # Arguments
/// * `config` - Provider configuration (loaded from YAML)
///
/// # Returns
/// Box<dyn KeyProvider> or KeyError
pub fn create_provider(config: ProviderConfig) -> Result<Box<dyn KeyProvider>, KeyError> {
    let provider_type = ProviderType::from_str(&config.provider)?;

    match provider_type {
        ProviderType::Software => {
            let sw_config = config.software.ok_or_else(|| {
                KeyError::ConfigError("Missing 'software' configuration".to_string())
            })?;

            let provider = crate::providers::SoftwareProvider::new(
                &sw_config.keys_dir,
                sw_config.default_key,
            )?;

            Ok(Box::new(provider))
        }

        ProviderType::Pkcs11 => {
            #[cfg(feature = "pkcs11")]
            {
                let p11_config = config.pkcs11.ok_or_else(|| {
                    KeyError::ConfigError("Missing 'pkcs11' configuration".to_string())
                })?;

                // Load PIN from environment variable
                let pin = std::env::var(&p11_config.pin_env).map_err(|_| {
                    KeyError::ConfigError(format!(
                        "PIN environment variable '{}' not set",
                        p11_config.pin_env
                    ))
                })?;

                let internal_config = crate::providers::pkcs11::Pkcs11InternalConfig {
                    module_path: p11_config.module,
                    slot: p11_config.slot,
                    pin,
                    key_label: p11_config.key_label,
                };

                let provider = crate::providers::Pkcs11Provider::new(internal_config)?;
                Ok(Box::new(provider))
            }

            #[cfg(not(feature = "pkcs11"))]
            {
                Err(KeyError::ProviderError(
                    "PKCS#11 provider not compiled. Enable with --features pkcs11".to_string(),
                ))
            }
        }

        ProviderType::CloudKms => {
            #[cfg(feature = "cloudkms")]
            {
                let kms_config = config.cloudkms.ok_or_else(|| {
                    KeyError::ConfigError("Missing 'cloudkms' configuration".to_string())
                })?;

                let cloud_provider = match kms_config.cloud.as_str() {
                    "gcp" => crate::providers::cloudkms::CloudProvider::Gcp,
                    "aws" => crate::providers::cloudkms::CloudProvider::Aws,
                    "azure" => crate::providers::cloudkms::CloudProvider::Azure,
                    other => {
                        return Err(KeyError::ConfigError(format!(
                            "Unknown cloud provider: {}",
                            other
                        )))
                    }
                };

                let internal_config = crate::providers::cloudkms::CloudKmsInternalConfig {
                    provider: cloud_provider,
                    project_or_region: kms_config.project,
                    location: kms_config.location,
                    keyring: kms_config.keyring,
                    key_name: kms_config.key,
                    key_version: kms_config.version,
                };

                let provider = crate::providers::CloudKmsProvider::new(internal_config)?;
                Ok(Box::new(provider))
            }

            #[cfg(not(feature = "cloudkms"))]
            {
                Err(KeyError::ProviderError(
                    "CloudKMS provider not compiled. Enable with --features cloudkms".to_string(),
                ))
            }
        }
    }
}

/// Loads provider configuration from YAML file
///
/// # Arguments
/// * `path` - Path to YAML configuration file
///
/// # Returns
/// ProviderConfig or KeyError
pub fn load_config<P: AsRef<std::path::Path>>(path: P) -> Result<ProviderConfig, KeyError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| KeyError::IoError(format!("Failed to read config file: {}", e)))?;

    let config: ProviderConfig = serde_yaml::from_str(&content)
        .map_err(|e| KeyError::ConfigError(format!("Failed to parse YAML: {}", e)))?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_type_parsing() {
        assert_eq!(
            ProviderType::from_str("software").unwrap(),
            ProviderType::Software
        );
        assert_eq!(
            ProviderType::from_str("pkcs11").unwrap(),
            ProviderType::Pkcs11
        );
        assert_eq!(
            ProviderType::from_str("cloudkms").unwrap(),
            ProviderType::CloudKms
        );
        assert_eq!(
            ProviderType::from_str("kms").unwrap(),
            ProviderType::CloudKms
        );
        assert!(ProviderType::from_str("invalid").is_err());
    }

    #[test]
    fn test_kid_derivation_deterministic() {
        let pubkey = b"test_public_key_32_bytes_long!!";
        let provider = "software";
        let key_name = "test-key";

        let kid1 = derive_kid(pubkey, provider, key_name);
        let kid2 = derive_kid(pubkey, provider, key_name);

        assert_eq!(kid1, kid2, "KID derivation must be deterministic");
        assert!(kid1.starts_with("0x"), "KID must be 0x-prefixed");
        assert_eq!(kid1.len(), 66, "KID must be 64 hex chars + 0x prefix");
    }

    #[test]
    fn test_kid_derivation_uniqueness() {
        let pubkey = b"test_public_key_32_bytes_long!!";
        let provider = "software";

        let kid1 = derive_kid(pubkey, provider, "key1");
        let kid2 = derive_kid(pubkey, provider, "key2");

        assert_ne!(
            kid1, kid2,
            "Different key names must produce different KIDs"
        );

        let kid3 = derive_kid(pubkey, "pkcs11", "key1");
        assert_ne!(
            kid1, kid3,
            "Different providers must produce different KIDs"
        );
    }

    #[test]
    fn test_key_error_display() {
        let err = KeyError::NotFound("test-key".to_string());
        assert!(err.to_string().contains("test-key"));

        let err = KeyError::AuthenticationFailed("Invalid PIN".to_string());
        assert!(err.to_string().contains("Authentication failed"));
    }

    #[test]
    fn test_provider_config_deserialization() {
        let yaml = r#"
provider: software
software:
  keys_dir: /path/to/keys
  default_key: my-key
"#;

        let config: ProviderConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.provider, "software");
        assert!(config.software.is_some());

        let sw_config = config.software.unwrap();
        assert_eq!(sw_config.keys_dir, "/path/to/keys");
        assert_eq!(sw_config.default_key.unwrap(), "my-key");
    }
}
