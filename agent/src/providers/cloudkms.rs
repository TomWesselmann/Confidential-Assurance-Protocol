//! CloudKMS Provider - GCP/AWS/Azure KMS Integration
//!
//! Dieser Provider nutzt Cloud KMS für managed Key Management.
//! Unterstützt: GCP Cloud KMS, AWS KMS, Azure Key Vault.
//!
//! Requires: cloud-kms specific crates (optional feature)

use super::key_provider::{derive_kid, KeyError, KeyProvider};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Cloud Provider Type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloudProvider {
    /// Google Cloud Platform KMS
    Gcp,
    /// AWS Key Management Service
    Aws,
    /// Azure Key Vault
    Azure,
}

/// CloudKMS Configuration
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct CloudKmsInternalConfig {
    /// Cloud provider type
    pub provider: CloudProvider,

    /// GCP: Project ID
    /// AWS: Region
    /// Azure: Vault Name
    pub project_or_region: String,

    /// GCP: Location (e.g., "europe-west1")
    /// AWS: Not used
    /// Azure: Not used
    pub location: Option<String>,

    /// GCP: Keyring name
    /// AWS: Not used
    /// Azure: Not used
    pub keyring: Option<String>,

    /// Key name/ID
    pub key_name: String,

    /// Key version (e.g., "latest", "1", "2")
    pub key_version: String,
}

/// CloudKMS Key Provider
///
/// Wrapper around Cloud KMS APIs für managed key operations.
/// Thread-safe durch Mutex.
pub struct CloudKmsProvider {
    config: CloudKmsInternalConfig,
    /// Cache for KIDs and public keys
    key_cache: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl CloudKmsProvider {
    /// Erstellt neuen CloudKMS Provider
    ///
    /// # Arguments
    /// * `config` - CloudKMS Configuration
    ///
    /// # Returns
    /// CloudKmsProvider or KeyError if initialization fails
    pub fn new(config: CloudKmsInternalConfig) -> Result<Self, KeyError> {
        // Validate config
        if config.project_or_region.is_empty() {
            return Err(KeyError::ConfigError(
                "Project/Region is empty".to_string()
            ));
        }

        if config.key_name.is_empty() {
            return Err(KeyError::ConfigError("Key name is empty".to_string()));
        }

        // For GCP, keyring is required
        if config.provider == CloudProvider::Gcp && config.keyring.is_none() {
            return Err(KeyError::ConfigError(
                "GCP requires keyring parameter".to_string()
            ));
        }

        Ok(Self {
            config,
            key_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Builds KMS resource path
    ///
    /// GCP: projects/{project}/locations/{location}/keyRings/{keyring}/cryptoKeys/{key}/cryptoKeyVersions/{version}
    /// AWS: arn:aws:kms:{region}:account:key/{key-id}
    /// Azure: https://{vault}.vault.azure.net/keys/{key-name}/{version}
    fn build_resource_path(&self) -> String {
        match self.config.provider {
            CloudProvider::Gcp => {
                let location = self.config.location.as_ref().unwrap();
                let keyring = self.config.keyring.as_ref().unwrap();
                format!(
                    "projects/{}/locations/{}/keyRings/{}/cryptoKeys/{}/cryptoKeyVersions/{}",
                    self.config.project_or_region,
                    location,
                    keyring,
                    self.config.key_name,
                    self.config.key_version
                )
            }
            CloudProvider::Aws => {
                format!(
                    "arn:aws:kms:{}:account:key/{}",
                    self.config.project_or_region, self.config.key_name
                )
            }
            CloudProvider::Azure => {
                format!(
                    "https://{}.vault.azure.net/keys/{}/{}",
                    self.config.project_or_region,
                    self.config.key_name,
                    self.config.key_version
                )
            }
        }
    }

    /// Gets public key from KMS
    ///
    /// This is a placeholder. Real implementation would:
    /// - GCP: kms.GetPublicKey()
    /// - AWS: kms.GetPublicKey()
    /// - Azure: keyVault.GetKey()
    fn get_public_key_from_kms(&self) -> Result<Vec<u8>, KeyError> {
        Err(KeyError::ProviderError(
            "CloudKMS provider not yet implemented. Compile with --features cloudkms".to_string()
        ))
    }

    /// Signs message with KMS
    ///
    /// This is a placeholder. Real implementation would:
    /// - GCP: kms.AsymmetricSign()
    /// - AWS: kms.Sign()
    /// - Azure: keyVault.Sign()
    fn sign_with_kms(&self, _msg: &[u8]) -> Result<Vec<u8>, KeyError> {
        Err(KeyError::ProviderError(
            "CloudKMS provider not yet implemented".to_string()
        ))
    }
}

impl KeyProvider for CloudKmsProvider {
    fn provider_id(&self) -> &'static str {
        match self.config.provider {
            CloudProvider::Gcp => "cloudkms-gcp",
            CloudProvider::Aws => "cloudkms-aws",
            CloudProvider::Azure => "cloudkms-azure",
        }
    }

    fn current_kid(&self) -> Result<String, KeyError> {
        // Get public key from KMS
        let pubkey = self.get_public_key_from_kms()?;

        // Derive KID using Week 7 formula
        let kid = derive_kid(&pubkey, self.provider_id(), &self.config.key_name);

        // Cache it
        let mut cache = self.key_cache.lock()
            .map_err(|e| KeyError::ProviderError(format!("Lock error: {}", e)))?;
        cache.insert(kid.clone(), pubkey);

        Ok(kid)
    }

    fn sign(&self, kid: Option<&str>, msg: &[u8]) -> Result<Vec<u8>, KeyError> {
        // Validate KID if provided
        if let Some(kid_str) = kid {
            let current_kid = self.current_kid()?;
            if current_kid != kid_str {
                return Err(KeyError::NotFound(format!(
                    "Key with KID {} not found (current: {})",
                    kid_str, current_kid
                )));
            }
        }

        // Sign with KMS (with timeout handling)
        self.sign_with_kms(msg)
    }

    fn public_key(&self, kid: &str) -> Result<Vec<u8>, KeyError> {
        // Check cache first
        let cache = self.key_cache.lock()
            .map_err(|e| KeyError::ProviderError(format!("Lock error: {}", e)))?;

        if let Some(pubkey) = cache.get(kid) {
            return Ok(pubkey.clone());
        }

        drop(cache); // Release lock before expensive operation

        // Get current key and check if KID matches
        let current_kid = self.current_kid()?;
        if current_kid == kid {
            let pubkey = self.get_public_key_from_kms()?;
            return Ok(pubkey);
        }

        Err(KeyError::NotFound(format!("Key with KID {} not found", kid)))
    }

    fn list_kids(&self) -> Result<Vec<String>, KeyError> {
        // CloudKMS typically has single key version active
        // In real implementation, we would list all versions

        let current_kid = self.current_kid()?;
        Ok(vec![current_kid])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloudkms_provider_gcp_creation() {
        let config = CloudKmsInternalConfig {
            provider: CloudProvider::Gcp,
            project_or_region: "my-project".to_string(),
            location: Some("europe-west1".to_string()),
            keyring: Some("my-keyring".to_string()),
            key_name: "signing-key".to_string(),
            key_version: "1".to_string(),
        };

        let provider = CloudKmsProvider::new(config);
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.provider_id(), "cloudkms-gcp");
    }

    #[test]
    fn test_cloudkms_provider_gcp_missing_keyring() {
        let config = CloudKmsInternalConfig {
            provider: CloudProvider::Gcp,
            project_or_region: "my-project".to_string(),
            location: Some("europe-west1".to_string()),
            keyring: None, // Missing required keyring
            key_name: "signing-key".to_string(),
            key_version: "1".to_string(),
        };

        let result = CloudKmsProvider::new(config);
        assert!(result.is_err());
        assert!(matches!(result, Err(KeyError::ConfigError(_))));
    }

    #[test]
    fn test_cloudkms_resource_path_gcp() {
        let config = CloudKmsInternalConfig {
            provider: CloudProvider::Gcp,
            project_or_region: "my-project".to_string(),
            location: Some("europe-west1".to_string()),
            keyring: Some("my-keyring".to_string()),
            key_name: "signing-key".to_string(),
            key_version: "1".to_string(),
        };

        let provider = CloudKmsProvider::new(config).unwrap();
        let path = provider.build_resource_path();

        assert_eq!(
            path,
            "projects/my-project/locations/europe-west1/keyRings/my-keyring/cryptoKeys/signing-key/cryptoKeyVersions/1"
        );
    }

    #[test]
    fn test_cloudkms_resource_path_aws() {
        let config = CloudKmsInternalConfig {
            provider: CloudProvider::Aws,
            project_or_region: "us-east-1".to_string(),
            location: None,
            keyring: None,
            key_name: "my-key-id".to_string(),
            key_version: "latest".to_string(),
        };

        let provider = CloudKmsProvider::new(config).unwrap();
        let path = provider.build_resource_path();

        assert_eq!(path, "arn:aws:kms:us-east-1:account:key/my-key-id");
    }

    #[test]
    fn test_cloudkms_provider_not_implemented() {
        let config = CloudKmsInternalConfig {
            provider: CloudProvider::Gcp,
            project_or_region: "my-project".to_string(),
            location: Some("europe-west1".to_string()),
            keyring: Some("my-keyring".to_string()),
            key_name: "signing-key".to_string(),
            key_version: "1".to_string(),
        };

        let provider = CloudKmsProvider::new(config).unwrap();

        // Should return "not implemented" error
        let result = provider.current_kid();
        assert!(result.is_err());
        assert!(matches!(result, Err(KeyError::ProviderError(_))));
    }
}
