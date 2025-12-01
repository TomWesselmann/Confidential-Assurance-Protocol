//! Registry Entry Signing (v0.8.0)
//!
//! Provides Ed25519 signing and verification for registry entries.

use base64::{engine::general_purpose, Engine};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::Serialize;
use std::error::Error;

use crate::keys;
use super::entry::RegistryEntry;

/// Berechnet BLAKE3-Hash des Entry-Cores (ohne Signatur-Felder)
fn compute_entry_core_hash(entry: &RegistryEntry) -> Result<Vec<u8>, Box<dyn Error>> {
    // Create core entry without signature fields for deterministic hashing
    #[derive(Serialize)]
    struct EntryCore<'a> {
        id: &'a str,
        manifest_hash: &'a str,
        proof_hash: &'a str,
        timestamp_file: &'a Option<String>,
        registered_at: &'a str,
    }

    let core = EntryCore {
        id: &entry.id,
        manifest_hash: &entry.manifest_hash,
        proof_hash: &entry.proof_hash,
        timestamp_file: &entry.timestamp_file,
        registered_at: &entry.registered_at,
    };

    let json = serde_json::to_vec(&core)?;
    let hash = blake3::hash(&json);
    Ok(hash.as_bytes().to_vec())
}

/// Signiert einen Registry-Eintrag mit Ed25519
///
/// # Argumente
/// * `entry` - Mutable Referenz auf Registry-Eintrag
/// * `signing_key` - Ed25519 Signing Key
///
/// # Rückgabe
/// Ok(()) wenn erfolgreich, Fehler sonst
pub fn sign_entry(
    entry: &mut RegistryEntry,
    signing_key: &SigningKey,
) -> Result<(), Box<dyn Error>> {
    // Compute hash of entry core (without signature fields)
    let entry_hash = compute_entry_core_hash(entry)?;

    // Sign the hash
    let signature = signing_key.sign(&entry_hash);

    // Encode signature and public key as base64
    let sig_b64 = general_purpose::STANDARD.encode(signature.to_bytes());
    let pubkey_b64 = general_purpose::STANDARD.encode(signing_key.verifying_key().to_bytes());

    // Derive KID from public key (v0.10)
    let kid = keys::derive_kid(&pubkey_b64)?;

    // Update entry with signature and key metadata
    entry.signature = Some(sig_b64);
    entry.public_key = Some(pubkey_b64);
    entry.kid = Some(kid);
    entry.signature_scheme = Some("ed25519".to_string());

    Ok(())
}

/// Verifiziert die Signatur eines Registry-Eintrags
///
/// # Argumente
/// * `entry` - Registry-Eintrag
///
/// # Rückgabe
/// Ok(true) wenn Signatur gültig, Ok(false) wenn keine Signatur, Err bei Fehler
pub fn verify_entry_signature(entry: &RegistryEntry) -> Result<bool, Box<dyn Error>> {
    // Check if signature exists
    let (sig_b64, pubkey_b64) = match (&entry.signature, &entry.public_key) {
        (Some(s), Some(p)) => (s, p),
        _ => return Ok(false), // No signature present (backward compatibility)
    };

    // Decode signature and public key
    let sig_bytes = general_purpose::STANDARD.decode(sig_b64)?;
    let pubkey_bytes = general_purpose::STANDARD.decode(pubkey_b64)?;

    // Parse Ed25519 types
    let signature = Signature::from_bytes(
        &sig_bytes
            .try_into()
            .map_err(|_| "Invalid signature length")?,
    );
    let verifying_key = VerifyingKey::from_bytes(
        &pubkey_bytes
            .try_into()
            .map_err(|_| "Invalid public key length")?,
    )?;

    // Compute entry hash
    let entry_hash = compute_entry_core_hash(entry)?;

    // Verify signature
    verifying_key.verify(&entry_hash, &signature)?;

    Ok(true)
}

/// Validiert den Status eines Schlüssels für Registry-Operationen
///
/// # Argumente
/// * `kid` - Key Identifier
/// * `key_store_path` - Pfad zum Key Store Verzeichnis
///
/// # Rückgabe
/// Ok(()) wenn Key aktiv, Err wenn nicht aktiv, retired, revoked oder nicht gefunden
pub fn validate_key_status(kid: &str, key_store_path: &str) -> Result<(), Box<dyn Error>> {
    use crate::keys::KeyStore;

    let store = KeyStore::new(key_store_path)?;

    match store.find_by_kid(kid)? {
        Some(key_meta) => match key_meta.status.as_str() {
            "active" => Ok(()),
            "retired" => {
                Err(format!("Key {} is retired and cannot be used for new entries", kid).into())
            }
            "revoked" => Err(format!("Key {} is revoked and cannot be used", kid).into()),
            other => Err(format!("Key {} has unknown status: {}", kid, other).into()),
        },
        None => Err(format!("Key not found in store: {}", kid).into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_entry() -> RegistryEntry {
        RegistryEntry::new(
            "proof_001".to_string(),
            "0xabc123".to_string(),
            "0xdef456".to_string(),
            Utc::now().to_rfc3339(),
        )
    }

    #[test]
    fn test_sign_and_verify_roundtrip() {
        let mut entry = create_test_entry();
        let signing_key = SigningKey::from_bytes(&[42u8; 32]);

        sign_entry(&mut entry, &signing_key).unwrap();

        assert!(entry.signature.is_some());
        assert!(entry.public_key.is_some());
        assert!(entry.kid.is_some());
        assert_eq!(entry.signature_scheme, Some("ed25519".to_string()));

        let valid = verify_entry_signature(&entry).unwrap();
        assert!(valid, "Signature should be valid");
    }

    #[test]
    fn test_tampered_entry_fails_verification() {
        let mut entry = create_test_entry();
        let signing_key = SigningKey::from_bytes(&[42u8; 32]);

        sign_entry(&mut entry, &signing_key).unwrap();

        // Tamper with the entry
        entry.manifest_hash = "0xTAMPERED".to_string();

        let result = verify_entry_signature(&entry);
        assert!(result.is_err(), "Tampered entry should fail verification");
    }

    #[test]
    fn test_missing_signature_returns_false() {
        let entry = create_test_entry();
        let valid = verify_entry_signature(&entry).unwrap();
        assert!(!valid, "Entry without signature should return false");
    }
}
