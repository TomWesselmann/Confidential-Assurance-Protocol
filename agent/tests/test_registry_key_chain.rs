/// Integration Test: Registry + Key Chain Validation
///
/// Tests the complete flow of key management with registry integration:
/// 1. Generate keys
/// 2. Rotate keys with attestation
/// 3. Add registry entries with active keys (should succeed)
/// 4. Attempt to add entry with retired key (should fail)
/// 5. Verify chain of trust

use cap_agent::{keys, registry};
use ed25519_dalek::SigningKey;
use std::fs;
use std::path::Path;

#[test]
fn test_registry_with_active_key_validation() {
    let test_dir = std::env::temp_dir().join("cap_test_registry_key_chain");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();

    let keys_dir = test_dir.join("keys");
    let registry_file = test_dir.join("registry.json");

    // 1. Setup: Create a key store and generate first key
    let store = keys::KeyStore::new(&keys_dir).unwrap();

    let key1_bytes = SigningKey::generate(&mut rand::rngs::OsRng);
    let key1_pub = key1_bytes.verifying_key().to_bytes();
    let key1_meta = keys::KeyMetadata::new(&key1_pub, "TestCompany", "ed25519", 365).unwrap();

    // Save key1
    let key1_path = keys_dir.join("key1.v1.json");
    key1_meta.save(&key1_path).unwrap();
    fs::write(
        keys_dir.join("key1.v1.ed25519"),
        key1_bytes.to_bytes(),
    )
    .unwrap();

    // 2. Create registry and add entry with active key
    let reg_store = registry::open_store(
        registry::RegistryBackend::Json,
        Path::new(&registry_file),
    )
    .unwrap();

    let mut entry1 = registry::RegistryEntry {
        id: "proof_001".to_string(),
        manifest_hash: "0x1234567890abcdef".to_string(),
        proof_hash: "0xfedcba0987654321".to_string(),
        timestamp_file: None,
        registered_at: chrono::Utc::now().to_rfc3339(),
        signature: None,
        public_key: None,
        blob_manifest: None,
        blob_proof: None,
        blob_wasm: None,
        blob_abi: None,
        selfverify_status: None,
        selfverify_at: None,
        verifier_name: None,
        verifier_version: None,
        kid: None,
        signature_scheme: None,
    };

    // Sign entry with key1
    registry::sign_entry(&mut entry1, &key1_bytes).unwrap();

    // Validate key status (should succeed - key is active)
    let kid1 = entry1.kid.as_ref().unwrap().clone();
    let result = registry::validate_key_status(&kid1, keys_dir.to_str().unwrap());
    assert!(
        result.is_ok(),
        "Active key validation should succeed: {:?}",
        result
    );

    // Add entry to registry
    reg_store.add_entry(entry1).unwrap();

    // 3. Generate second key and rotate (archive first key)
    let key2_bytes = SigningKey::generate(&mut rand::rngs::OsRng);
    let key2_pub = key2_bytes.verifying_key().to_bytes();
    let key2_meta = keys::KeyMetadata::new(&key2_pub, "TestCompany", "ed25519", 365).unwrap();

    // Save key2
    let key2_path = keys_dir.join("key2.v1.json");
    key2_meta.save(&key2_path).unwrap();

    // Retire and archive key1
    store.archive(&kid1).unwrap();

    // 4. Attempt to add entry with retired key (should fail)
    let mut entry2 = registry::RegistryEntry {
        id: "proof_002".to_string(),
        manifest_hash: "0xabcdef1234567890".to_string(),
        proof_hash: "0x0987654321fedcba".to_string(),
        timestamp_file: None,
        registered_at: chrono::Utc::now().to_rfc3339(),
        signature: None,
        public_key: None,
        blob_manifest: None,
        blob_proof: None,
        blob_wasm: None,
        blob_abi: None,
        selfverify_status: None,
        selfverify_at: None,
        verifier_name: None,
        verifier_version: None,
        kid: None,
        signature_scheme: None,
    };

    // Sign with retired key (technically possible, but validation should fail)
    registry::sign_entry(&mut entry2, &key1_bytes).unwrap();

    // Validate key status (should fail - key is retired)
    let result = registry::validate_key_status(&kid1, keys_dir.to_str().unwrap());
    assert!(
        result.is_err(),
        "Retired key validation should fail, but got: {:?}",
        result
    );

    // Verify error message mentions "retired"
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("retired"),
        "Error message should mention retired status: {}",
        err_msg
    );

    // Cleanup
    fs::remove_dir_all(&test_dir).ok();
}

#[test]
fn test_chain_of_trust_verification() {
    let test_dir = std::env::temp_dir().join("cap_test_chain_verification");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();

    let keys_dir = test_dir.join("keys");
    let attestations_dir = test_dir.join("attestations");
    fs::create_dir_all(&attestations_dir).unwrap();

    // Create key store (which creates the keys directory)
    let store = keys::KeyStore::new(&keys_dir).unwrap();

    // 1. Generate key1 (root key)
    let key1_bytes = SigningKey::generate(&mut rand::rngs::OsRng);
    let key1_pub = key1_bytes.verifying_key().to_bytes();
    let key1_meta = keys::KeyMetadata::new(&key1_pub, "TestCompany", "ed25519", 365).unwrap();

    let key1_path = keys_dir.join("key1.v1.json");
    key1_meta.save(&key1_path).unwrap();
    fs::write(keys_dir.join("key1.v1.ed25519"), key1_bytes.to_bytes()).unwrap();

    // 2. Generate key2
    let key2_bytes = SigningKey::generate(&mut rand::rngs::OsRng);
    let key2_pub = key2_bytes.verifying_key().to_bytes();
    let key2_meta = keys::KeyMetadata::new(&key2_pub, "TestCompany", "ed25519", 365).unwrap();

    let key2_path = keys_dir.join("key2.v1.json");
    key2_meta.save(&key2_path).unwrap();

    // 3. Create attestation: key1 → key2
    let attestation = keys::Attestation {
        schema: "cap-attestation.v1".to_string(),
        signer_kid: key1_meta.kid.clone(),
        signer_owner: key1_meta.owner.clone(),
        subject_kid: key2_meta.kid.clone(),
        subject_owner: key2_meta.owner.clone(),
        subject_public_key: key2_meta.public_key.clone(),
        attested_at: chrono::Utc::now().to_rfc3339(),
    };

    // Sign attestation with key1
    use ed25519_dalek::Signer;
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

    let attestation_bytes = serde_json::to_vec(&attestation).unwrap();
    let signature = key1_bytes.sign(&attestation_bytes);

    let signed_attestation = keys::SignedAttestation {
        attestation,
        signature: BASE64.encode(signature.to_bytes()),
        signer_public_key: key1_meta.public_key.clone(),
    };

    let att_path = attestations_dir.join("key1_to_key2.json");
    let att_json = serde_json::to_string_pretty(&signed_attestation).unwrap();
    fs::write(&att_path, att_json).unwrap();

    // 4. Verify attestation
    let loaded_att = keys::SignedAttestation::load(&att_path).unwrap();
    let verify_result = loaded_att.verify();
    assert!(
        verify_result.is_ok(),
        "Attestation verification should succeed: {:?}",
        verify_result
    );

    // 5. Verify chain
    let chain_result = keys::verify_chain(
        &[att_path.to_str().unwrap()],
        &store,
    );
    assert!(
        chain_result.is_ok(),
        "Chain verification should succeed: {:?}",
        chain_result
    );

    // Cleanup
    fs::remove_dir_all(&test_dir).ok();
}

#[test]
fn test_revoked_key_rejection() {
    let test_dir = std::env::temp_dir().join("cap_test_revoked_key");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();

    let keys_dir = test_dir.join("keys");

    // Create key store (which creates the keys directory)
    let _store = keys::KeyStore::new(&keys_dir).unwrap();

    // 1. Generate key
    let key_bytes = SigningKey::generate(&mut rand::rngs::OsRng);
    let key_pub = key_bytes.verifying_key().to_bytes();
    let mut key_meta = keys::KeyMetadata::new(&key_pub, "TestCompany", "ed25519", 365).unwrap();

    let key_path = keys_dir.join("key.v1.json");
    key_meta.save(&key_path).unwrap();

    let kid = key_meta.kid.clone();

    // 2. Revoke key
    key_meta.revoke();
    key_meta.save(&key_path).unwrap();

    // 3. Validate key status (should fail - key is revoked)
    let result = registry::validate_key_status(&kid, keys_dir.to_str().unwrap());
    assert!(
        result.is_err(),
        "Revoked key validation should fail, but got: {:?}",
        result
    );

    // Verify error message mentions "revoked"
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("revoked"),
        "Error message should mention revoked status: {}",
        err_msg
    );

    // Cleanup
    fs::remove_dir_all(&test_dir).ok();
}
