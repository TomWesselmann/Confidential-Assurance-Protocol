use crate::manifest::{Manifest, SignatureInfo, SignedManifest};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use std::error::Error;
use std::fs::{File, create_dir_all};
use std::io::{Read, Write};
use std::path::Path;

/// Generiert ein Ed25519-Schlüsselpaar
///
/// # Argumente
/// * `private_key_path` - Pfad für den privaten Schlüssel
/// * `public_key_path` - Pfad für den öffentlichen Schlüssel
///
/// # Rückgabe
/// Result mit () bei Erfolg
pub fn generate_keypair<P: AsRef<Path>>(
    private_key_path: P,
    public_key_path: P,
) -> Result<(), Box<dyn Error>> {
    // Erstelle Verzeichnis falls nicht vorhanden
    if let Some(parent) = private_key_path.as_ref().parent() {
        create_dir_all(parent)?;
    }

    // Generiere Schlüsselpaar
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();

    // Speichere privaten Schlüssel (32 Bytes)
    let mut priv_file = File::create(private_key_path)?;
    priv_file.write_all(&signing_key.to_bytes())?;

    // Speichere öffentlichen Schlüssel (32 Bytes)
    let mut pub_file = File::create(public_key_path)?;
    pub_file.write_all(&verifying_key.to_bytes())?;

    Ok(())
}

/// Lädt einen privaten Schlüssel aus einer Datei
///
/// # Argumente
/// * `path` - Pfad zum privaten Schlüssel
///
/// # Rückgabe
/// SigningKey-Objekt
pub fn load_private_key<P: AsRef<Path>>(path: P) -> Result<SigningKey, Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;

    if bytes.len() != 32 {
        return Err(format!(
            "Ungültige Schlüssellänge: {} (erwartet 32)",
            bytes.len()
        )
        .into());
    }

    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&bytes[0..32]);

    Ok(SigningKey::from_bytes(&key_bytes))
}

/// Lädt einen öffentlichen Schlüssel aus einer Datei
///
/// # Argumente
/// * `path` - Pfad zum öffentlichen Schlüssel
///
/// # Rückgabe
/// VerifyingKey-Objekt
pub fn load_public_key<P: AsRef<Path>>(path: P) -> Result<VerifyingKey, Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;

    if bytes.len() != 32 {
        return Err(format!(
            "Ungültige Schlüssellänge: {} (erwartet 32)",
            bytes.len()
        )
        .into());
    }

    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&bytes[0..32]);

    VerifyingKey::from_bytes(&key_bytes)
        .map_err(|e| format!("Fehler beim Laden des öffentlichen Schlüssels: {}", e).into())
}

/// Signiert ein Manifest mit einem privaten Schlüssel
///
/// # Argumente
/// * `manifest` - Das zu signierende Manifest
/// * `signing_key` - Der private Signaturschlüssel
/// * `signer_name` - Name des Signierers (z.B. "Company")
///
/// # Rückgabe
/// SignedManifest mit Signatur
pub fn sign_manifest(
    manifest: &Manifest,
    signing_key: &SigningKey,
    signer_name: &str,
) -> Result<SignedManifest, Box<dyn Error>> {
    // Kanonisches JSON des Manifests
    let canonical_json = manifest.to_canonical_json()?;

    // Signiere
    let signature: Signature = signing_key.sign(canonical_json.as_bytes());

    // Öffentlicher Schlüssel für Verifikation
    let verifying_key = signing_key.verifying_key();

    let signature_info = SignatureInfo {
        alg: "Ed25519".to_string(),
        signer: signer_name.to_string(),
        pubkey_hex: format!("0x{}", hex::encode(verifying_key.to_bytes())),
        sig_hex: format!("0x{}", hex::encode(signature.to_bytes())),
    };

    Ok(SignedManifest {
        manifest: manifest.clone(),
        signature: signature_info,
    })
}

/// Verifiziert die Signatur eines signierten Manifests
///
/// # Argumente
/// * `signed_manifest` - Das signierte Manifest
/// * `verifying_key` - Der öffentliche Schlüssel zur Verifikation
///
/// # Rückgabe
/// Result mit () bei erfolgreicher Verifikation
pub fn verify_manifest(
    signed_manifest: &SignedManifest,
    verifying_key: &VerifyingKey,
) -> Result<(), Box<dyn Error>> {
    // Prüfe, ob public key übereinstimmt
    let expected_pubkey = format!("0x{}", hex::encode(verifying_key.to_bytes()));
    if signed_manifest.signature.pubkey_hex != expected_pubkey {
        return Err("Public-Key-Mismatch".into());
    }

    // Prüfe Algorithmus
    if signed_manifest.signature.alg != "Ed25519" {
        return Err(format!("Ungültiger Algorithmus: {}", signed_manifest.signature.alg).into());
    }

    // Extrahiere Signatur
    let sig_hex = signed_manifest
        .signature
        .sig_hex
        .strip_prefix("0x")
        .unwrap_or(&signed_manifest.signature.sig_hex);
    let sig_bytes = hex::decode(sig_hex)?;

    if sig_bytes.len() != 64 {
        return Err(format!(
            "Ungültige Signaturlänge: {} (erwartet 64)",
            sig_bytes.len()
        )
        .into());
    }

    let mut sig_array = [0u8; 64];
    sig_array.copy_from_slice(&sig_bytes);
    let signature = Signature::from_bytes(&sig_array);

    // Kanonisches JSON
    let canonical_json = signed_manifest.manifest.to_canonical_json()?;

    // Verifiziere
    verifying_key
        .verify(canonical_json.as_bytes(), &signature)
        .map_err(|e| format!("Signaturverifikation fehlgeschlagen: {}", e).into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{AuditInfo, Manifest, ProofInfo};
    use crate::policy::PolicyInfo;

    #[test]
    fn test_keypair_generation() {
        let priv_path = "/tmp/test_sign_priv.key";
        let pub_path = "/tmp/test_sign_pub.key";

        generate_keypair(priv_path, pub_path).unwrap();

        assert!(std::path::Path::new(priv_path).exists());
        assert!(std::path::Path::new(pub_path).exists());

        // Cleanup
        std::fs::remove_file(priv_path).ok();
        std::fs::remove_file(pub_path).ok();
    }

    #[test]
    fn test_sign_and_verify() {
        let manifest = Manifest {
            version: "manifest.v0".to_string(),
            created_at: "2025-10-25T10:00:00Z".to_string(),
            supplier_root: "0xabc".to_string(),
            ubo_root: "0xdef".to_string(),
            company_commitment_root: "0x123".to_string(),
            policy: PolicyInfo {
                name: "Test".to_string(),
                version: "lksg.v1".to_string(),
                hash: "0xpolicy".to_string(),
            },
            audit: AuditInfo {
                tail_digest: "0xtail".to_string(),
                events_count: 5,
            },
            proof: ProofInfo {
                proof_type: "mock".to_string(),
                status: "ok".to_string(),
            },
            signatures: Vec::new(),
            time_anchor: None,
        };

        // Generiere Schlüssel
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();

        // Signiere
        let signed = sign_manifest(&manifest, &signing_key, "TestCompany").unwrap();

        assert_eq!(signed.signature.alg, "Ed25519");
        assert_eq!(signed.signature.signer, "TestCompany");

        // Verifiziere
        assert!(verify_manifest(&signed, &verifying_key).is_ok());
    }

    #[test]
    fn test_verify_fails_with_wrong_key() {
        let manifest = Manifest {
            version: "manifest.v0".to_string(),
            created_at: "2025-10-25T10:00:00Z".to_string(),
            supplier_root: "0xabc".to_string(),
            ubo_root: "0xdef".to_string(),
            company_commitment_root: "0x123".to_string(),
            policy: PolicyInfo {
                name: "Test".to_string(),
                version: "lksg.v1".to_string(),
                hash: "0xpolicy".to_string(),
            },
            audit: AuditInfo {
                tail_digest: "0xtail".to_string(),
                events_count: 5,
            },
            proof: ProofInfo {
                proof_type: "mock".to_string(),
                status: "ok".to_string(),
            },
            signatures: Vec::new(),
            time_anchor: None,
        };

        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let wrong_key = SigningKey::generate(&mut csprng);

        let signed = sign_manifest(&manifest, &signing_key, "TestCompany").unwrap();

        // Verifikation mit falschem Schlüssel sollte fehlschlagen
        assert!(verify_manifest(&signed, &wrong_key.verifying_key()).is_err());
    }
}
