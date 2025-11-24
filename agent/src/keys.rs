/// Key Management Module - KID/Rotation System (v0.10)
///
/// Stellt sicher, dass kryptografische Signaturen nachvollziehbar, rotierbar
/// und langfristig gültig bleiben.
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::crypto;

/// Key Identifier (KID) - 16 bytes = 32 hex characters
pub type Kid = String;

/// Key Metadata Schema (cap-key.v1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    /// Schema version
    pub schema: String,

    /// Key Identifier (derived from public key)
    pub kid: Kid,

    /// Owner/Organization name
    pub owner: String,

    /// Creation timestamp (RFC3339)
    pub created_at: String,

    /// Valid from timestamp (RFC3339)
    pub valid_from: String,

    /// Valid until timestamp (RFC3339)
    pub valid_to: String,

    /// Algorithm (e.g., "ed25519")
    pub algorithm: String,

    /// Key status: "active", "retired", "revoked"
    pub status: String,

    /// Usage types: ["signing", "registry", "attestation"]
    pub usage: Vec<String>,

    /// Public key (base64-encoded)
    pub public_key: String,

    /// SHA-256 fingerprint of public key
    pub fingerprint: String,

    /// Optional comment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

impl KeyMetadata {
    /// Creates new key metadata from public key bytes
    pub fn new(
        public_key_bytes: &[u8],
        owner: &str,
        algorithm: &str,
        valid_for_days: u64,
    ) -> Result<Self> {
        let public_key_b64 = BASE64.encode(public_key_bytes);
        let kid = derive_kid(&public_key_b64)?;
        let fingerprint = compute_fingerprint(public_key_bytes);

        let now = Utc::now();
        let valid_to = now + chrono::Duration::days(valid_for_days as i64);

        Ok(Self {
            schema: "cap-key.v1".to_string(),
            kid,
            owner: owner.to_string(),
            created_at: now.to_rfc3339(),
            valid_from: now.to_rfc3339(),
            valid_to: valid_to.to_rfc3339(),
            algorithm: algorithm.to_string(),
            status: "active".to_string(),
            usage: vec!["signing".to_string(), "registry".to_string()],
            public_key: public_key_b64,
            fingerprint,
            comment: None,
        })
    }

    /// Loads key metadata from JSON file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let metadata: Self = serde_json::from_str(&content)?;
        Ok(metadata)
    }

    /// Saves key metadata to JSON file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Marks this key as retired (for rotation)
    pub fn retire(&mut self) {
        self.status = "retired".to_string();
    }

    /// Marks this key as revoked (for security incidents)
    #[allow(dead_code)]
    pub fn revoke(&mut self) {
        self.status = "revoked".to_string();
    }

    /// Gets the public key bytes
    #[allow(dead_code)]
    pub fn public_key_bytes(&self) -> Result<Vec<u8>> {
        BASE64
            .decode(&self.public_key)
            .map_err(|e| anyhow!("Failed to decode public key: {}", e))
    }
}

/// Derives KID from public key
///
/// Formula: kid = blake3(base64(public_key))[0:16]
/// Returns: 32 hex characters (16 bytes)
pub fn derive_kid(public_key_b64: &str) -> Result<Kid> {
    let hash = crypto::blake3_256(public_key_b64.as_bytes());

    // Take first 16 bytes (128 bits)
    let kid_bytes = &hash[0..16];

    // Encode as hex (32 characters)
    Ok(hex::encode(kid_bytes))
}

/// Computes SHA-256 fingerprint of public key
fn compute_fingerprint(public_key_bytes: &[u8]) -> String {
    let hash = crypto::sha3_256(public_key_bytes);
    format!("sha256:{}", hex::encode(&hash[0..16]))
}

/// Key Store - manages keys directory structure
pub struct KeyStore {
    base_path: PathBuf,
}

impl KeyStore {
    /// Opens or creates a key store at the given path
    pub fn new<P: AsRef<Path>>(base_path: P) -> Result<Self> {
        let base = base_path.as_ref().to_path_buf();

        // Create directory structure
        fs::create_dir_all(&base)?;
        fs::create_dir_all(base.join("archive"))?;
        fs::create_dir_all(base.join("trusted"))?;

        Ok(Self { base_path: base })
    }

    /// Lists all key metadata files
    pub fn list(&self) -> Result<Vec<KeyMetadata>> {
        let mut keys = Vec::new();

        // Scan base directory for .json files
        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(metadata) = KeyMetadata::load(&path) {
                    keys.push(metadata);
                }
            }
        }

        // Scan archive directory
        let archive_path = self.base_path.join("archive");
        if archive_path.exists() {
            for entry in fs::read_dir(&archive_path)? {
                let entry = entry?;
                let path = entry.path();

                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(metadata) = KeyMetadata::load(&path) {
                        keys.push(metadata);
                    }
                }
            }
        }

        Ok(keys)
    }

    /// Finds a key by KID
    pub fn find_by_kid(&self, kid: &str) -> Result<Option<KeyMetadata>> {
        for key in self.list()? {
            if key.kid == kid {
                return Ok(Some(key));
            }
        }
        Ok(None)
    }

    /// Archives a key (moves to archive/)
    pub fn archive(&self, kid: &str) -> Result<()> {
        // Find the key file
        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(mut metadata) = KeyMetadata::load(&path) {
                    if metadata.kid == kid {
                        // Mark as retired
                        metadata.retire();
                        metadata.save(&path)?;

                        // Move to archive
                        let filename = path.file_name().unwrap();
                        let archive_path = self.base_path.join("archive").join(filename);
                        fs::rename(&path, &archive_path)?;

                        return Ok(());
                    }
                }
            }
        }

        Err(anyhow!("Key not found: {}", kid))
    }

    /// Gets the active key for an owner
    #[allow(dead_code)]
    pub fn get_active(&self, owner: &str) -> Result<Option<KeyMetadata>> {
        for key in self.list()? {
            if key.owner == owner && key.status == "active" {
                return Ok(Some(key));
            }
        }
        Ok(None)
    }
}

/// Attestation Schema (cap-attestation.v1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attestation {
    /// Schema version
    pub schema: String,

    /// Signer KID (old/current key)
    pub signer_kid: String,

    /// Signer owner
    pub signer_owner: String,

    /// Subject KID (new key being attested)
    pub subject_kid: String,

    /// Subject owner
    pub subject_owner: String,

    /// Subject public key (base64)
    pub subject_public_key: String,

    /// Attestation timestamp (RFC3339)
    pub attested_at: String,
}

/// Signed Attestation Document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedAttestation {
    /// The attestation content
    pub attestation: Attestation,

    /// Ed25519 signature (base64)
    pub signature: String,

    /// Signer's public key (base64)
    pub signer_public_key: String,
}

impl SignedAttestation {
    /// Loads signed attestation from JSON file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let attestation: Self = serde_json::from_str(&content)?;
        Ok(attestation)
    }

    /// Verifies the attestation signature
    pub fn verify(&self) -> Result<()> {
        use ed25519_dalek::{Signature, Verifier, VerifyingKey};

        // Decode public key
        let pubkey_bytes = BASE64
            .decode(&self.signer_public_key)
            .map_err(|e| anyhow!("Failed to decode public key: {}", e))?;
        let verifying_key = VerifyingKey::from_bytes(
            &pubkey_bytes
                .try_into()
                .map_err(|_| anyhow!("Invalid public key length"))?,
        )
        .map_err(|e| anyhow!("Invalid public key: {}", e))?;

        // Decode signature
        let sig_bytes = BASE64
            .decode(&self.signature)
            .map_err(|e| anyhow!("Failed to decode signature: {}", e))?;
        let signature = Signature::from_bytes(
            &sig_bytes
                .try_into()
                .map_err(|_| anyhow!("Invalid signature length"))?,
        );

        // Verify signature over attestation
        let attestation_bytes = serde_json::to_vec(&self.attestation)?;
        verifying_key
            .verify(&attestation_bytes, &signature)
            .map_err(|e| anyhow!("Signature verification failed: {}", e))?;

        // Verify that signer_public_key matches attestation.signer_kid
        let derived_kid = derive_kid(&self.signer_public_key)?;
        if derived_kid != self.attestation.signer_kid {
            return Err(anyhow!(
                "Signer KID mismatch: expected {}, got {}",
                self.attestation.signer_kid,
                derived_kid
            ));
        }

        // Verify that subject_public_key matches attestation.subject_kid
        let subject_kid = derive_kid(&self.attestation.subject_public_key)?;
        if subject_kid != self.attestation.subject_kid {
            return Err(anyhow!(
                "Subject KID mismatch: expected {}, got {}",
                self.attestation.subject_kid,
                subject_kid
            ));
        }

        Ok(())
    }
}

/// Chain-of-Trust Verification
///
/// Verifies a complete attestation chain from root key to current key.
/// Each attestation must be signed by the previous key in the chain.
pub fn verify_chain(attestation_paths: &[&str], key_store: &KeyStore) -> Result<()> {
    if attestation_paths.is_empty() {
        return Err(anyhow!("No attestations provided"));
    }

    let mut previous_subject_kid: Option<String> = None;

    for (i, path) in attestation_paths.iter().enumerate() {
        // Load and verify attestation
        let signed_att = SignedAttestation::load(path)?;
        signed_att.verify()?;

        // Check chain continuity: subject of previous = signer of current
        if let Some(ref prev_kid) = previous_subject_kid {
            if signed_att.attestation.signer_kid != *prev_kid {
                return Err(anyhow!(
                    "Chain broken at attestation {}: expected signer_kid {}, got {}",
                    i,
                    prev_kid,
                    signed_att.attestation.signer_kid
                ));
            }
        }

        // Verify signer key exists in key store
        let signer_key = key_store
            .find_by_kid(&signed_att.attestation.signer_kid)?
            .ok_or_else(|| {
                anyhow!(
                    "Signer key not found in store: {}",
                    signed_att.attestation.signer_kid
                )
            })?;

        // Check signer key status (must be active or retired, not revoked)
        if signer_key.status == "revoked" {
            return Err(anyhow!(
                "Signer key is revoked: {}",
                signed_att.attestation.signer_kid
            ));
        }

        previous_subject_kid = Some(signed_att.attestation.subject_kid.clone());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Signer;

    #[test]
    fn test_derive_kid() {
        let pubkey_b64 = "MCowBQYDK2VwAyEAGb9ECWmEzf6FQbrBZ9w7lshQhqowtrbLDFw4rXAxZuE=";
        let kid = derive_kid(pubkey_b64).unwrap();

        // KID should be 32 hex characters (16 bytes)
        assert_eq!(kid.len(), 32);
        assert!(kid.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_kid_deterministic() {
        let pubkey_b64 = "MCowBQYDK2VwAyEAGb9ECWmEzf6FQbrBZ9w7lshQhqowtrbLDFw4rXAxZuE=";
        let kid1 = derive_kid(pubkey_b64).unwrap();
        let kid2 = derive_kid(pubkey_b64).unwrap();

        assert_eq!(kid1, kid2);
    }

    #[test]
    fn test_key_metadata_new() {
        let pubkey_bytes = vec![1, 2, 3, 4, 5];
        let metadata = KeyMetadata::new(&pubkey_bytes, "company", "ed25519", 730).unwrap();

        assert_eq!(metadata.schema, "cap-key.v1");
        assert_eq!(metadata.owner, "company");
        assert_eq!(metadata.algorithm, "ed25519");
        assert_eq!(metadata.status, "active");
        assert_eq!(metadata.kid.len(), 32);
    }

    #[test]
    fn test_key_metadata_retire() {
        let pubkey_bytes = vec![1, 2, 3, 4, 5];
        let mut metadata = KeyMetadata::new(&pubkey_bytes, "company", "ed25519", 730).unwrap();

        assert_eq!(metadata.status, "active");
        metadata.retire();
        assert_eq!(metadata.status, "retired");
    }

    #[test]
    fn test_key_store_new() {
        let temp_dir = std::env::temp_dir().join("cap_test_keystore");
        let _ = fs::remove_dir_all(&temp_dir);

        let _store = KeyStore::new(&temp_dir).unwrap();

        assert!(temp_dir.exists());
        assert!(temp_dir.join("archive").exists());
        assert!(temp_dir.join("trusted").exists());

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_key_metadata_roundtrip() {
        let pubkey_bytes = vec![1, 2, 3, 4, 5];
        let metadata = KeyMetadata::new(&pubkey_bytes, "company", "ed25519", 730).unwrap();

        let temp_file = std::env::temp_dir().join("test_key.json");
        metadata.save(&temp_file).unwrap();

        let loaded = KeyMetadata::load(&temp_file).unwrap();
        assert_eq!(loaded.kid, metadata.kid);
        assert_eq!(loaded.owner, metadata.owner);

        fs::remove_file(&temp_file).ok();
    }

    /// Property Test: KID Determinism
    /// Same public key should always produce the same KID
    #[test]
    fn test_kid_determinism_property() {
        use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
        use ed25519_dalek::{SigningKey, VerifyingKey};

        // Generate 10 random keys and verify determinism
        for _ in 0..10 {
            let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
            let verifying_key: VerifyingKey = signing_key.verifying_key();
            let pubkey_bytes = verifying_key.to_bytes();
            let pubkey_b64 = BASE64.encode(pubkey_bytes);

            // Derive KID multiple times
            let kid1 = derive_kid(&pubkey_b64).unwrap();
            let kid2 = derive_kid(&pubkey_b64).unwrap();
            let kid3 = derive_kid(&pubkey_b64).unwrap();

            // All should be identical
            assert_eq!(kid1, kid2, "KID derivation is not deterministic");
            assert_eq!(kid2, kid3, "KID derivation is not deterministic");

            // KID should be 32 hex characters
            assert_eq!(kid1.len(), 32, "KID should be 32 characters");
            assert!(
                kid1.chars().all(|c| c.is_ascii_hexdigit()),
                "KID should only contain hex characters"
            );
        }
    }

    /// Property Test: KID Uniqueness
    /// Different public keys should produce different KIDs (with high probability)
    #[test]
    fn test_kid_uniqueness_property() {
        use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
        use ed25519_dalek::SigningKey;
        use std::collections::HashSet;

        let mut kids = HashSet::new();

        // Generate 100 keys and verify they all have unique KIDs
        for _ in 0..100 {
            let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
            let verifying_key = signing_key.verifying_key();
            let pubkey_bytes = verifying_key.to_bytes();
            let pubkey_b64 = BASE64.encode(pubkey_bytes);

            let kid = derive_kid(&pubkey_b64).unwrap();

            // Check for collision
            assert!(
                !kids.contains(&kid),
                "KID collision detected! This should be extremely rare"
            );

            kids.insert(kid);
        }

        assert_eq!(kids.len(), 100, "Should have 100 unique KIDs");
    }

    /// Property Test: KID matches KeyMetadata
    /// KID in KeyMetadata should match derived KID from public key
    #[test]
    fn test_kid_metadata_consistency() {
        use ed25519_dalek::SigningKey;

        // Generate 10 keys and verify metadata KID matches derived KID
        for _ in 0..10 {
            let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
            let verifying_key = signing_key.verifying_key();
            let pubkey_bytes = verifying_key.to_bytes();

            // Create metadata (which derives KID internally)
            let metadata = KeyMetadata::new(&pubkey_bytes, "TestOwner", "ed25519", 365).unwrap();

            // Derive KID manually from the same public key
            let expected_kid = derive_kid(&metadata.public_key).unwrap();

            // They should match
            assert_eq!(
                metadata.kid, expected_kid,
                "Metadata KID should match derived KID"
            );
        }
    }

    #[test]
    fn test_key_metadata_revoke() {
        let pubkey_bytes = vec![1, 2, 3, 4, 5];
        let mut metadata = KeyMetadata::new(&pubkey_bytes, "company", "ed25519", 730).unwrap();

        assert_eq!(metadata.status, "active");
        metadata.revoke();
        assert_eq!(metadata.status, "revoked");
    }

    #[test]
    fn test_key_metadata_public_key_bytes() {
        let pubkey_bytes = vec![1, 2, 3, 4, 5];
        let metadata = KeyMetadata::new(&pubkey_bytes, "company", "ed25519", 730).unwrap();

        let decoded = metadata.public_key_bytes().unwrap();
        assert_eq!(decoded, pubkey_bytes);
    }

    #[test]
    fn test_key_metadata_with_comment() {
        let pubkey_bytes = vec![1, 2, 3, 4, 5];
        let mut metadata = KeyMetadata::new(&pubkey_bytes, "company", "ed25519", 730).unwrap();

        assert!(metadata.comment.is_none());
        metadata.comment = Some("Test key for development".to_string());
        assert_eq!(metadata.comment.unwrap(), "Test key for development");
    }

    #[test]
    fn test_compute_fingerprint_deterministic() {
        let pubkey_bytes = vec![1, 2, 3, 4, 5];
        let fp1 = compute_fingerprint(&pubkey_bytes);
        let fp2 = compute_fingerprint(&pubkey_bytes);

        assert_eq!(fp1, fp2);
        assert!(fp1.starts_with("sha256:"));
        assert_eq!(fp1.len(), 7 + 32); // "sha256:" + 32 hex chars (16 bytes)
    }

    #[test]
    fn test_key_store_find_by_kid_not_found() {
        let temp_dir = std::env::temp_dir().join("cap_test_find_kid");
        let _ = fs::remove_dir_all(&temp_dir);

        let store = KeyStore::new(&temp_dir).unwrap();

        let result = store.find_by_kid("nonexistent_kid").unwrap();
        assert!(result.is_none());

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_key_store_find_by_kid_success() {
        let temp_dir = std::env::temp_dir().join("cap_test_find_kid_success");
        let _ = fs::remove_dir_all(&temp_dir);

        let store = KeyStore::new(&temp_dir).unwrap();

        // Create and save a key
        let pubkey_bytes = vec![1, 2, 3, 4, 5];
        let metadata = KeyMetadata::new(&pubkey_bytes, "company", "ed25519", 730).unwrap();
        let kid = metadata.kid.clone();
        let key_file = temp_dir.join("test_key.json");
        metadata.save(&key_file).unwrap();

        // Find by KID
        let found = store.find_by_kid(&kid).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().kid, kid);

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_key_store_archive() {
        let temp_dir = std::env::temp_dir().join("cap_test_archive");
        let _ = fs::remove_dir_all(&temp_dir);

        let store = KeyStore::new(&temp_dir).unwrap();

        // Create and save a key
        let pubkey_bytes = vec![1, 2, 3, 4, 5];
        let metadata = KeyMetadata::new(&pubkey_bytes, "company", "ed25519", 730).unwrap();
        let kid = metadata.kid.clone();
        let key_file = temp_dir.join("test_key.json");
        metadata.save(&key_file).unwrap();

        // Archive the key
        store.archive(&kid).unwrap();

        // Key should no longer be in base directory
        assert!(!key_file.exists());

        // Key should be in archive with "retired" status
        let archived_key = temp_dir.join("archive").join("test_key.json");
        assert!(archived_key.exists());

        let loaded = KeyMetadata::load(&archived_key).unwrap();
        assert_eq!(loaded.status, "retired");

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_key_store_archive_not_found() {
        let temp_dir = std::env::temp_dir().join("cap_test_archive_not_found");
        let _ = fs::remove_dir_all(&temp_dir);

        let store = KeyStore::new(&temp_dir).unwrap();

        let result = store.archive("nonexistent_kid");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Key not found"));

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_key_store_get_active() {
        let temp_dir = std::env::temp_dir().join("cap_test_get_active");
        let _ = fs::remove_dir_all(&temp_dir);

        let store = KeyStore::new(&temp_dir).unwrap();

        // Create and save two keys for same owner
        let pubkey1 = vec![1, 2, 3];
        let metadata1 = KeyMetadata::new(&pubkey1, "company", "ed25519", 730).unwrap();
        metadata1.save(temp_dir.join("key1.json")).unwrap();

        let pubkey2 = vec![4, 5, 6];
        let mut metadata2 = KeyMetadata::new(&pubkey2, "company", "ed25519", 730).unwrap();
        metadata2.retire(); // Retire this one
        metadata2.save(temp_dir.join("key2.json")).unwrap();

        // Get active should return the active one
        let active = store.get_active("company").unwrap();
        assert!(active.is_some());
        assert_eq!(active.unwrap().status, "active");

        // Get active for different owner should return None
        let other = store.get_active("other_company").unwrap();
        assert!(other.is_none());

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_key_store_list_includes_archive() {
        let temp_dir = std::env::temp_dir().join("cap_test_list_archive");
        let _ = fs::remove_dir_all(&temp_dir);

        let store = KeyStore::new(&temp_dir).unwrap();

        // Create keys in base and archive
        let pubkey1 = vec![1, 2, 3];
        let metadata1 = KeyMetadata::new(&pubkey1, "company1", "ed25519", 730).unwrap();
        metadata1.save(temp_dir.join("key1.json")).unwrap();

        let pubkey2 = vec![4, 5, 6];
        let metadata2 = KeyMetadata::new(&pubkey2, "company2", "ed25519", 730).unwrap();
        metadata2.save(temp_dir.join("archive").join("key2.json")).unwrap();

        // List should include both
        let keys = store.list().unwrap();
        assert_eq!(keys.len(), 2);

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_signed_attestation_verify_success() {
        use ed25519_dalek::SigningKey;

        // Generate signer key
        let signer_sk = SigningKey::generate(&mut rand::rngs::OsRng);
        let signer_vk = signer_sk.verifying_key();
        let signer_pubkey_b64 = BASE64.encode(signer_vk.to_bytes());

        // Generate subject key
        let subject_sk = SigningKey::generate(&mut rand::rngs::OsRng);
        let subject_vk = subject_sk.verifying_key();
        let subject_pubkey_b64 = BASE64.encode(subject_vk.to_bytes());

        // Create attestation
        let attestation = Attestation {
            schema: "cap-attestation.v1".to_string(),
            signer_kid: derive_kid(&signer_pubkey_b64).unwrap(),
            signer_owner: "Company".to_string(),
            subject_kid: derive_kid(&subject_pubkey_b64).unwrap(),
            subject_owner: "Company".to_string(),
            subject_public_key: subject_pubkey_b64.clone(),
            attested_at: Utc::now().to_rfc3339(),
        };

        // Sign attestation
        let attestation_bytes = serde_json::to_vec(&attestation).unwrap();
        let signature = signer_sk.sign(&attestation_bytes);

        let signed_attestation = SignedAttestation {
            attestation,
            signature: BASE64.encode(signature.to_bytes()),
            signer_public_key: signer_pubkey_b64,
        };

        // Verify should succeed
        let result = signed_attestation.verify();
        assert!(result.is_ok(), "Verification failed: {:?}", result);
    }

    #[test]
    fn test_signed_attestation_verify_invalid_signature() {
        use ed25519_dalek::SigningKey;

        // Generate keys
        let signer_sk = SigningKey::generate(&mut rand::rngs::OsRng);
        let signer_vk = signer_sk.verifying_key();
        let signer_pubkey_b64 = BASE64.encode(signer_vk.to_bytes());

        let subject_sk = SigningKey::generate(&mut rand::rngs::OsRng);
        let subject_vk = subject_sk.verifying_key();
        let subject_pubkey_b64 = BASE64.encode(subject_vk.to_bytes());

        // Create attestation
        let attestation = Attestation {
            schema: "cap-attestation.v1".to_string(),
            signer_kid: derive_kid(&signer_pubkey_b64).unwrap(),
            signer_owner: "Company".to_string(),
            subject_kid: derive_kid(&subject_pubkey_b64).unwrap(),
            subject_owner: "Company".to_string(),
            subject_public_key: subject_pubkey_b64.clone(),
            attested_at: Utc::now().to_rfc3339(),
        };

        // Create INVALID signature (all zeros)
        let invalid_signature = BASE64.encode([0u8; 64]);

        let signed_attestation = SignedAttestation {
            attestation,
            signature: invalid_signature,
            signer_public_key: signer_pubkey_b64,
        };

        // Verify should fail
        let result = signed_attestation.verify();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("verification failed"));
    }

    #[test]
    fn test_signed_attestation_verify_kid_mismatch() {
        use ed25519_dalek::SigningKey;

        // Generate keys
        let signer_sk = SigningKey::generate(&mut rand::rngs::OsRng);
        let signer_vk = signer_sk.verifying_key();
        let signer_pubkey_b64 = BASE64.encode(signer_vk.to_bytes());

        let subject_sk = SigningKey::generate(&mut rand::rngs::OsRng);
        let subject_vk = subject_sk.verifying_key();
        let subject_pubkey_b64 = BASE64.encode(subject_vk.to_bytes());

        // Create attestation with WRONG signer_kid
        let attestation = Attestation {
            schema: "cap-attestation.v1".to_string(),
            signer_kid: "wrong_kid_0123456789abcdef0123".to_string(),
            signer_owner: "Company".to_string(),
            subject_kid: derive_kid(&subject_pubkey_b64).unwrap(),
            subject_owner: "Company".to_string(),
            subject_public_key: subject_pubkey_b64.clone(),
            attested_at: Utc::now().to_rfc3339(),
        };

        // Sign attestation (signature will be valid, but KID won't match)
        let attestation_bytes = serde_json::to_vec(&attestation).unwrap();
        let signature = signer_sk.sign(&attestation_bytes);

        let signed_attestation = SignedAttestation {
            attestation,
            signature: BASE64.encode(signature.to_bytes()),
            signer_public_key: signer_pubkey_b64,
        };

        // Verify should fail due to KID mismatch
        let result = signed_attestation.verify();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Signer KID mismatch"));
    }

    #[test]
    fn test_verify_chain_empty() {
        let temp_dir = std::env::temp_dir().join("cap_test_chain_empty");
        let _ = fs::remove_dir_all(&temp_dir);
        let store = KeyStore::new(&temp_dir).unwrap();

        let result = verify_chain(&[], &store);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No attestations"));

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_verify_chain_signer_key_not_found() {
        use ed25519_dalek::SigningKey;

        let temp_dir = std::env::temp_dir().join("cap_test_chain_not_found");
        let _ = fs::remove_dir_all(&temp_dir);
        let store = KeyStore::new(&temp_dir).unwrap();

        // Create attestation
        let signer_sk = SigningKey::generate(&mut rand::rngs::OsRng);
        let signer_vk = signer_sk.verifying_key();
        let signer_pubkey_b64 = BASE64.encode(signer_vk.to_bytes());

        let subject_sk = SigningKey::generate(&mut rand::rngs::OsRng);
        let subject_vk = subject_sk.verifying_key();
        let subject_pubkey_b64 = BASE64.encode(subject_vk.to_bytes());

        let attestation = Attestation {
            schema: "cap-attestation.v1".to_string(),
            signer_kid: derive_kid(&signer_pubkey_b64).unwrap(),
            signer_owner: "Company".to_string(),
            subject_kid: derive_kid(&subject_pubkey_b64).unwrap(),
            subject_owner: "Company".to_string(),
            subject_public_key: subject_pubkey_b64.clone(),
            attested_at: Utc::now().to_rfc3339(),
        };

        let attestation_bytes = serde_json::to_vec(&attestation).unwrap();
        let signature = signer_sk.sign(&attestation_bytes);

        let signed_att = SignedAttestation {
            attestation,
            signature: BASE64.encode(signature.to_bytes()),
            signer_public_key: signer_pubkey_b64,
        };

        // Save attestation
        let att_file = temp_dir.join("att1.json");
        let json = serde_json::to_string_pretty(&signed_att).unwrap();
        fs::write(&att_file, json).unwrap();

        // Verify chain should fail because signer key is not in store
        let result = verify_chain(&[att_file.to_str().unwrap()], &store);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Signer key not found"));

        fs::remove_dir_all(&temp_dir).ok();
    }
}
