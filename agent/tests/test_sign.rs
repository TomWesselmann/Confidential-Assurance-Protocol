/// Integration Tests für sign.rs
///
/// Diese Tests wurden aus inline test modules extrahiert um Tarpaulin Coverage-Tracking zu ermöglichen.
/// Tarpaulin hat eine bekannte Limitation mit #[cfg(test)] inline modules.

use cap_agent::manifest::{AuditInfo, Manifest, ProofInfo};
use cap_agent::policy::PolicyInfo;
use cap_agent::sign::{
    generate_keypair, load_private_key, load_public_key, sign_manifest, verify_manifest,
};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use std::fs;

// Helper function to create test manifest
fn create_test_manifest() -> Manifest {
    Manifest {
        version: "manifest.v1.0".to_string(),
        created_at: "2025-10-25T10:00:00Z".to_string(),
        supplier_root: "0xabc123".to_string(),
        ubo_root: "0xdef456".to_string(),
        company_commitment_root: "0x789ghi".to_string(),
        policy: PolicyInfo {
            name: "Test Policy".to_string(),
            version: "lksg.v1".to_string(),
            hash: "0xpolicyhash".to_string(),
        },
        audit: AuditInfo {
            tail_digest: "0xtaildigest".to_string(),
            events_count: 10,
        },
        proof: ProofInfo {
            proof_type: "mock".to_string(),
            status: "ok".to_string(),
        },
        signatures: Vec::new(),
        time_anchor: None,
    }
}

#[test]
fn test_generate_keypair() {
    let priv_path = "/tmp/test_sign_keypair_priv.key";
    let pub_path = "/tmp/test_sign_keypair_pub.key";

    // Generate keypair
    generate_keypair(priv_path, pub_path).unwrap();

    // Verify files exist
    assert!(std::path::Path::new(priv_path).exists());
    assert!(std::path::Path::new(pub_path).exists());

    // Verify file sizes (Ed25519 keys are 32 bytes)
    let priv_metadata = fs::metadata(priv_path).unwrap();
    let pub_metadata = fs::metadata(pub_path).unwrap();
    assert_eq!(priv_metadata.len(), 32);
    assert_eq!(pub_metadata.len(), 32);

    // Cleanup
    fs::remove_file(priv_path).ok();
    fs::remove_file(pub_path).ok();
}

#[test]
fn test_generate_keypair_creates_directory() {
    let priv_path = "/tmp/test_sign_newdir/subdir/priv.key";
    let pub_path = "/tmp/test_sign_newdir/subdir/pub.key";

    // Generate keypair (should create directory)
    generate_keypair(priv_path, pub_path).unwrap();

    assert!(std::path::Path::new(priv_path).exists());
    assert!(std::path::Path::new(pub_path).exists());

    // Cleanup
    fs::remove_dir_all("/tmp/test_sign_newdir").ok();
}

#[test]
fn test_load_private_key() {
    let priv_path = "/tmp/test_load_priv.key";
    let pub_path = "/tmp/test_load_pub.key";

    // Generate keypair
    generate_keypair(priv_path, pub_path).unwrap();

    // Load private key
    let loaded_key = load_private_key(priv_path).unwrap();

    // Verify it's a valid signing key (can produce verifying key)
    let _verifying_key = loaded_key.verifying_key();

    // Cleanup
    fs::remove_file(priv_path).ok();
    fs::remove_file(pub_path).ok();
}

#[test]
fn test_load_public_key() {
    let priv_path = "/tmp/test_load_pub_priv.key";
    let pub_path = "/tmp/test_load_pub_pub.key";

    // Generate keypair
    generate_keypair(priv_path, pub_path).unwrap();

    // Load public key
    let loaded_key = load_public_key(pub_path).unwrap();

    // Verify it's 32 bytes
    assert_eq!(loaded_key.to_bytes().len(), 32);

    // Cleanup
    fs::remove_file(priv_path).ok();
    fs::remove_file(pub_path).ok();
}

#[test]
fn test_load_private_key_invalid_length() {
    let invalid_path = "/tmp/test_invalid_priv.key";

    // Write invalid key (wrong length)
    fs::write(invalid_path, &[0u8; 16]).unwrap();

    // Loading should fail
    let result = load_private_key(invalid_path);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Ungültige Schlüssellänge"));

    // Cleanup
    fs::remove_file(invalid_path).ok();
}

#[test]
fn test_load_public_key_invalid_length() {
    let invalid_path = "/tmp/test_invalid_pub.key";

    // Write invalid key (wrong length)
    fs::write(invalid_path, &[0u8; 16]).unwrap();

    // Loading should fail
    let result = load_public_key(invalid_path);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Ungültige Schlüssellänge"));

    // Cleanup
    fs::remove_file(invalid_path).ok();
}

#[test]
fn test_load_private_key_file_not_found() {
    let result = load_private_key("/nonexistent/path/priv.key");
    assert!(result.is_err());
}

#[test]
fn test_load_public_key_file_not_found() {
    let result = load_public_key("/nonexistent/path/pub.key");
    assert!(result.is_err());
}

#[test]
fn test_sign_manifest() {
    let manifest = create_test_manifest();

    // Generate signing key
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);

    // Sign manifest
    let signed = sign_manifest(&manifest, &signing_key, "TestCompany").unwrap();

    // Verify signature structure
    assert_eq!(signed.signature.alg, "Ed25519");
    assert_eq!(signed.signature.signer, "TestCompany");
    assert!(signed.signature.pubkey_hex.starts_with("0x"));
    assert!(signed.signature.sig_hex.starts_with("0x"));
    assert_eq!(signed.signature.pubkey_hex.len(), 66); // 0x + 64 hex chars
    assert_eq!(signed.signature.sig_hex.len(), 130); // 0x + 128 hex chars
}

#[test]
fn test_sign_and_verify_roundtrip() {
    let manifest = create_test_manifest();

    // Generate keys
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();

    // Sign
    let signed = sign_manifest(&manifest, &signing_key, "Company").unwrap();

    // Verify
    let result = verify_manifest(&signed, &verifying_key);
    assert!(result.is_ok());
}

#[test]
fn test_verify_manifest_with_wrong_key() {
    let manifest = create_test_manifest();

    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let wrong_key = SigningKey::generate(&mut csprng);

    // Sign with first key
    let signed = sign_manifest(&manifest, &signing_key, "Company").unwrap();

    // Verify with wrong key should fail
    let result = verify_manifest(&signed, &wrong_key.verifying_key());
    assert!(result.is_err());
}

#[test]
fn test_verify_manifest_public_key_mismatch() {
    let manifest = create_test_manifest();

    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let wrong_key = SigningKey::generate(&mut csprng);

    // Sign with first key
    let signed = sign_manifest(&manifest, &signing_key, "Company").unwrap();

    // Verify with wrong key - should fail with public key mismatch
    let result = verify_manifest(&signed, &wrong_key.verifying_key());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Public-Key-Mismatch"));
}

#[test]
fn test_verify_manifest_invalid_algorithm() {
    let manifest = create_test_manifest();

    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();

    // Sign
    let mut signed = sign_manifest(&manifest, &signing_key, "Company").unwrap();

    // Tamper with algorithm
    signed.signature.alg = "RSA".to_string();

    // Verify should fail
    let result = verify_manifest(&signed, &verifying_key);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Ungültiger Algorithmus"));
}

#[test]
fn test_verify_manifest_invalid_signature_length() {
    let manifest = create_test_manifest();

    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();

    // Sign
    let mut signed = sign_manifest(&manifest, &signing_key, "Company").unwrap();

    // Tamper with signature (wrong length)
    signed.signature.sig_hex = "0xabcd".to_string();

    // Verify should fail
    let result = verify_manifest(&signed, &verifying_key);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Ungültige Signaturlänge"));
}

#[test]
fn test_verify_manifest_tampered_manifest() {
    let manifest = create_test_manifest();

    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();

    // Sign
    let mut signed = sign_manifest(&manifest, &signing_key, "Company").unwrap();

    // Tamper with manifest content
    signed.manifest.supplier_root = "0xTAMPERED".to_string();

    // Verify should fail (signature won't match tampered content)
    let result = verify_manifest(&signed, &verifying_key);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Signaturverifikation fehlgeschlagen"));
}

#[test]
fn test_sign_manifest_deterministic_for_same_manifest() {
    let manifest = create_test_manifest();

    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);

    // Sign twice with same key
    let signed1 = sign_manifest(&manifest, &signing_key, "Company").unwrap();
    let signed2 = sign_manifest(&manifest, &signing_key, "Company").unwrap();

    // Signatures should be identical (Ed25519 is deterministic)
    assert_eq!(signed1.signature.sig_hex, signed2.signature.sig_hex);
    assert_eq!(signed1.signature.pubkey_hex, signed2.signature.pubkey_hex);
}

#[test]
fn test_keypair_roundtrip() {
    let priv_path = "/tmp/test_roundtrip_priv.key";
    let pub_path = "/tmp/test_roundtrip_pub.key";

    // Generate and save
    generate_keypair(priv_path, pub_path).unwrap();

    // Load both keys
    let signing_key = load_private_key(priv_path).unwrap();
    let verifying_key = load_public_key(pub_path).unwrap();

    // Verify keys match (public key from private key should match loaded public key)
    assert_eq!(signing_key.verifying_key().to_bytes(), verifying_key.to_bytes());

    // Cleanup
    fs::remove_file(priv_path).ok();
    fs::remove_file(pub_path).ok();
}

#[test]
fn test_signature_hex_format() {
    let manifest = create_test_manifest();

    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);

    let signed = sign_manifest(&manifest, &signing_key, "Company").unwrap();

    // Verify hex format
    assert!(signed.signature.pubkey_hex.starts_with("0x"));
    assert!(signed.signature.sig_hex.starts_with("0x"));

    // Verify only valid hex characters (0x + hex)
    let pubkey_hex = signed.signature.pubkey_hex.strip_prefix("0x").unwrap();
    let sig_hex = signed.signature.sig_hex.strip_prefix("0x").unwrap();

    assert!(pubkey_hex.chars().all(|c| c.is_ascii_hexdigit()));
    assert!(sig_hex.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_verify_manifest_with_signature_without_prefix() {
    let manifest = create_test_manifest();

    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();

    // Sign
    let mut signed = sign_manifest(&manifest, &signing_key, "Company").unwrap();

    // Remove 0x prefix from signature (should still work due to strip_prefix in verify)
    signed.signature.sig_hex = signed.signature.sig_hex.strip_prefix("0x").unwrap().to_string();

    // Verify should still work
    let result = verify_manifest(&signed, &verifying_key);
    assert!(result.is_ok());
}

#[test]
fn test_different_manifests_produce_different_signatures() {
    let manifest1 = create_test_manifest();
    let mut manifest2 = create_test_manifest();
    manifest2.supplier_root = "0xDIFFERENT".to_string();

    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);

    let signed1 = sign_manifest(&manifest1, &signing_key, "Company").unwrap();
    let signed2 = sign_manifest(&manifest2, &signing_key, "Company").unwrap();

    // Signatures should be different
    assert_ne!(signed1.signature.sig_hex, signed2.signature.sig_hex);
}

#[test]
fn test_signer_name_preserved() {
    let manifest = create_test_manifest();

    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);

    let signer_names = ["Company A", "Company B", "Test Corp", "我司"];

    for signer_name in signer_names {
        let signed = sign_manifest(&manifest, &signing_key, signer_name).unwrap();
        assert_eq!(signed.signature.signer, signer_name);
    }
}
