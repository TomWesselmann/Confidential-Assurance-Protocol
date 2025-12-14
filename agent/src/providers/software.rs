//! Software Provider - Ed25519 File-based Key Management
//!
//! Dieser Provider nutzt den bestehenden KeyStore für Ed25519-Schlüssel.
//! Implementiert das KeyProvider-Interface für konsistente API.

use super::key_provider::{derive_kid, KeyError, KeyProvider};
use crate::crypto;
use crate::keys::{KeyMetadata, KeyStore};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// Software-basierter Key Provider (Ed25519)
///
/// Nutzt file-based KeyStore für Schlüsselverwaltung.
/// Thread-safe durch RwLock.
pub struct SoftwareProvider {
    key_store: Arc<RwLock<KeyStore>>,
    keys_dir: PathBuf,
    default_key_name: Option<String>,
}

impl SoftwareProvider {
    /// Erstellt neuen Software Provider
    ///
    /// # Arguments
    /// * `keys_dir` - Pfad zum Schlüsselverzeichnis
    /// * `default_key_name` - Optional: Name des Default-Schlüssels (ohne Pfad/Extension)
    pub fn new<P: AsRef<Path>>(
        keys_dir: P,
        default_key_name: Option<String>,
    ) -> Result<Self, KeyError> {
        let keys_path = keys_dir.as_ref().to_path_buf();

        let key_store = KeyStore::new(&keys_path)
            .map_err(|e| KeyError::IoError(format!("Failed to open key store: {}", e)))?;

        Ok(Self {
            key_store: Arc::new(RwLock::new(key_store)),
            keys_dir: keys_path,
            default_key_name,
        })
    }

    /// Lädt Ed25519 Private Key aus Datei
    fn load_private_key(&self, key_name: &str) -> Result<crypto::Ed25519SecretKey, KeyError> {
        let key_path = self.keys_dir.join(format!("{}.ed25519", key_name));

        if !key_path.exists() {
            return Err(KeyError::NotFound(format!(
                "Private key file not found: {}",
                key_path.display()
            )));
        }

        let key_bytes = fs::read(&key_path)
            .map_err(|e| KeyError::IoError(format!("Failed to read private key: {}", e)))?;

        if key_bytes.len() != 32 {
            return Err(KeyError::IoError(format!(
                "Invalid private key size: {} bytes (expected 32)",
                key_bytes.len()
            )));
        }

        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&key_bytes);

        Ok(crypto::Ed25519SecretKey::from_bytes(&bytes))
    }

    /// Lädt KeyMetadata für einen Schlüssel
    fn load_metadata(&self, key_name: &str) -> Result<KeyMetadata, KeyError> {
        let metadata_path = self.keys_dir.join(format!("{}.v1.json", key_name));

        KeyMetadata::load(&metadata_path)
            .map_err(|e| KeyError::NotFound(format!("Key metadata not found: {}", e)))
    }

    /// Findet Default-Schlüssel
    fn find_default_key(&self) -> Result<String, KeyError> {
        if let Some(ref name) = self.default_key_name {
            return Ok(name.clone());
        }

        // Fallback: Finde ersten aktiven Schlüssel
        let store = self
            .key_store
            .read()
            .map_err(|e| KeyError::ProviderError(format!("Lock error: {}", e)))?;

        let keys = store
            .list()
            .map_err(|e| KeyError::IoError(format!("Failed to list keys: {}", e)))?;

        for key in keys {
            if key.status == crate::keys::KeyStatus::Active {
                // Extrahiere Schlüsselnamen aus metadata kid
                // Format: <keyname>.v1.json
                let metadata_path = self.keys_dir.join(format!("{}.v1.json", key.kid));
                if metadata_path.exists() {
                    return Ok(key.kid);
                }

                // Try to extract from first active key
                // Scan all files for matching kid
                for entry in fs::read_dir(&self.keys_dir)
                    .map_err(|e| KeyError::IoError(format!("Failed to read directory: {}", e)))?
                {
                    let entry = entry.map_err(|e| KeyError::IoError(e.to_string()))?;
                    let path = entry.path();

                    if path.extension().and_then(|s| s.to_str()) == Some("json") {
                        if let Ok(meta) = KeyMetadata::load(&path) {
                            if meta.kid == key.kid {
                                // Extract key name from path
                                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                                    // Remove .v1 suffix if present
                                    let key_name = stem.trim_end_matches(".v1");
                                    return Ok(key_name.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        Err(KeyError::NotFound("No active keys found".to_string()))
    }
}

impl KeyProvider for SoftwareProvider {
    fn provider_id(&self) -> &'static str {
        "software"
    }

    fn current_kid(&self) -> Result<String, KeyError> {
        let key_name = self.find_default_key()?;
        let metadata = self.load_metadata(&key_name)?;

        // Use Week 7 KID derivation formula
        let pubkey = metadata
            .public_key_bytes()
            .map_err(|e| KeyError::ProviderError(format!("Invalid public key: {}", e)))?;

        let kid = derive_kid(&pubkey, self.provider_id(), &key_name);
        Ok(kid)
    }

    fn sign(&self, kid: Option<&str>, msg: &[u8]) -> Result<Vec<u8>, KeyError> {
        let key_name = if let Some(kid_str) = kid {
            // Find key by KID
            let store = self
                .key_store
                .read()
                .map_err(|e| KeyError::ProviderError(format!("Lock error: {}", e)))?;

            let _keys = store
                .list()
                .map_err(|e| KeyError::IoError(format!("Failed to list keys: {}", e)))?;

            // Search for key with matching KID (using Week 7 derivation)
            let mut found_key_name = None;
            for entry in fs::read_dir(&self.keys_dir)
                .map_err(|e| KeyError::IoError(format!("Failed to read directory: {}", e)))?
            {
                let entry = entry.map_err(|e| KeyError::IoError(e.to_string()))?;
                let path = entry.path();

                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(meta) = KeyMetadata::load(&path) {
                        let pubkey = meta.public_key_bytes().map_err(|e| {
                            KeyError::ProviderError(format!("Invalid public key: {}", e))
                        })?;

                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            let kname = stem.trim_end_matches(".v1");
                            let computed_kid = derive_kid(&pubkey, self.provider_id(), kname);

                            if computed_kid == kid_str {
                                found_key_name = Some(kname.to_string());
                                break;
                            }
                        }
                    }
                }
            }

            found_key_name
                .ok_or_else(|| KeyError::NotFound(format!("Key with KID {} not found", kid_str)))?
        } else {
            self.find_default_key()?
        };

        // Load metadata and check status
        let metadata = self.load_metadata(&key_name)?;
        if metadata.status == crate::keys::KeyStatus::Revoked {
            return Err(KeyError::ProviderError(format!(
                "Key {} is revoked and cannot be used",
                key_name
            )));
        }

        // Load private key and sign
        let secret_key = self.load_private_key(&key_name)?;
        let signature = crypto::ed25519_sign(&secret_key, msg)
            .map_err(|e| KeyError::SignatureError(format!("Signing failed: {}", e)))?;

        Ok(signature.to_bytes().to_vec())
    }

    fn public_key(&self, kid: &str) -> Result<Vec<u8>, KeyError> {
        // Find key by KID (using Week 7 derivation)
        for entry in fs::read_dir(&self.keys_dir)
            .map_err(|e| KeyError::IoError(format!("Failed to read directory: {}", e)))?
        {
            let entry = entry.map_err(|e| KeyError::IoError(e.to_string()))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(meta) = KeyMetadata::load(&path) {
                    let pubkey = meta.public_key_bytes().map_err(|e| {
                        KeyError::ProviderError(format!("Invalid public key: {}", e))
                    })?;

                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        let kname = stem.trim_end_matches(".v1");
                        let computed_kid = derive_kid(&pubkey, self.provider_id(), kname);

                        if computed_kid == kid {
                            return Ok(pubkey);
                        }
                    }
                }
            }
        }

        Err(KeyError::NotFound(format!(
            "Key with KID {} not found",
            kid
        )))
    }

    fn list_kids(&self) -> Result<Vec<String>, KeyError> {
        let mut kids = Vec::new();

        for entry in fs::read_dir(&self.keys_dir)
            .map_err(|e| KeyError::IoError(format!("Failed to read directory: {}", e)))?
        {
            let entry = entry.map_err(|e| KeyError::IoError(e.to_string()))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(meta) = KeyMetadata::load(&path) {
                    if let Ok(pubkey) = meta.public_key_bytes() {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            let kname = stem.trim_end_matches(".v1");
                            let kid = derive_kid(&pubkey, self.provider_id(), kname);
                            kids.push(kid);
                        }
                    }
                }
            }
        }

        Ok(kids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_software_provider_creation() {
        let temp_dir = tempdir().unwrap();
        let provider = SoftwareProvider::new(temp_dir.path(), None);
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.provider_id(), "software");
    }

    #[test]
    fn test_software_provider_no_keys() {
        let temp_dir = tempdir().unwrap();
        let provider = SoftwareProvider::new(temp_dir.path(), None).unwrap();

        // No keys should exist
        let result = provider.current_kid();
        assert!(result.is_err());
        assert!(matches!(result, Err(KeyError::NotFound(_))));
    }

    #[test]
    fn test_list_kids_empty() {
        let temp_dir = tempdir().unwrap();
        let provider = SoftwareProvider::new(temp_dir.path(), None).unwrap();

        let kids = provider.list_kids().unwrap();
        assert_eq!(kids.len(), 0);
    }
}
