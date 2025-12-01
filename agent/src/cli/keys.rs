//! CLI-Handler für Keys-Kommandos
//!
//! Extrahiert aus main.rs für bessere Wartbarkeit.
//! Enthält: run_keys_keygen, run_keys_list, run_keys_show, run_keys_rotate,
//!          run_keys_attest, run_keys_archive, run_keys_verify_chain

use super::output;
use crate::audit::AuditLog;
use crate::keys;
use serde_json::json;
use std::error::Error;
use std::fs;

/// Keys keygen - Generiert neuen Ed25519-Schlüssel mit Metadata
pub fn run_keys_keygen(
    owner: &str,
    algo: &str,
    out_path: &str,
    valid_days: u64,
    comment: Option<String>,
) -> Result<(), Box<dyn Error>> {
    use ed25519_dalek::SigningKey;

    output::key("Generiere neuen Schlüssel...");
    output::detail("Owner", owner);
    output::detail("Algorithm", algo);
    output::detail_fmt("Valid for", format!("{} days", valid_days));

    if algo != "ed25519" {
        return Err(format!("Unsupported algorithm: {}", algo).into());
    }

    // Generate Ed25519 keypair
    let mut rng = rand::rngs::OsRng;
    let signing_key = SigningKey::generate(&mut rng);
    let verifying_key = signing_key.verifying_key();

    // Create key metadata
    let mut metadata = keys::KeyMetadata::new(&verifying_key.to_bytes(), owner, algo, valid_days)?;

    if let Some(ref c) = comment {
        metadata.comment = Some(c.clone());
    }

    // Save metadata
    metadata.save(out_path)?;

    // Save private key (raw bytes)
    let priv_key_path = out_path.replace(".json", ".ed25519");
    fs::write(&priv_key_path, signing_key.to_bytes())?;

    // Save public key (raw bytes)
    let pub_key_path = out_path.replace(".json", ".pub");
    fs::write(&pub_key_path, verifying_key.to_bytes())?;

    output::success("Schlüssel generiert:");
    output::detail("KID", &metadata.kid);
    output::detail("Metadata", out_path);
    output::detail("Private", &priv_key_path);
    output::detail("Public", &pub_key_path);
    output::detail("Fingerprint", &metadata.fingerprint);

    // Log audit event
    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
    audit.log_event(
        "key_generated",
        json!({
            "kid": metadata.kid,
            "owner": owner,
            "algorithm": algo,
            "metadata_file": out_path,
        }),
    )?;

    Ok(())
}

/// Keys list - Listet alle Schlüssel auf
pub fn run_keys_list(
    dir: &str,
    status_filter: Option<String>,
    owner_filter: Option<String>,
) -> Result<(), Box<dyn Error>> {
    output::listing("Schlüsselliste:");
    output::detail("Verzeichnis", dir);
    println!();

    let store = keys::KeyStore::new(dir)?;
    let all_keys = store.list()?;

    // Apply filters
    let filtered_keys: Vec<_> = all_keys
        .iter()
        .filter(|k| {
            if let Some(ref status) = status_filter {
                if k.status != *status {
                    return false;
                }
            }
            if let Some(ref owner) = owner_filter {
                if k.owner != *owner {
                    return false;
                }
            }
            true
        })
        .collect();

    if filtered_keys.is_empty() {
        output::indent("(Keine Schlüssel gefunden)");
        return Ok(());
    }

    output::table_header(&[("KID", 32), ("Owner", 15), ("Status", 10), ("Valid Until", 20)]);

    for key in &filtered_keys {
        let valid_to_short = &key.valid_to[0..10]; // YYYY-MM-DD
        output::table_row(&[
            (&key.kid, 32),
            (&key.owner, 15),
            (&key.status, 10),
            (valid_to_short, 20),
        ]);
    }

    output::section("");
    output::indent(&format!("Total: {} Schlüssel", filtered_keys.len()));

    Ok(())
}

/// Keys show - Zeigt Details eines Schlüssels
pub fn run_keys_show(dir: &str, kid: &str) -> Result<(), Box<dyn Error>> {
    output::searching("Schlüssel-Details:");

    let store = keys::KeyStore::new(dir)?;
    let key_opt = store.find_by_kid(kid)?;

    match key_opt {
        Some(key) => {
            output::detail("KID", &key.kid);
            output::detail("Owner", &key.owner);
            output::detail("Algorithm", &key.algorithm);
            output::detail("Status", &key.status);
            output::detail("Created", &key.created_at);
            output::detail("Valid From", &key.valid_from);
            output::detail("Valid To", &key.valid_to);
            output::detail("Fingerprint", &key.fingerprint);
            output::detail_fmt("Usage", format!("{:?}", key.usage));
            if let Some(ref comment) = key.comment {
                output::detail("Comment", comment);
            }
            output::detail("Public Key", &format!("{}...", &key.public_key[0..20]));

            Ok(())
        }
        None => Err(format!("Schlüssel nicht gefunden: {}", kid).into()),
    }
}

/// Keys rotate - Rotiert Schlüssel
pub fn run_keys_rotate(dir: &str, current_path: &str, new_path: &str) -> Result<(), Box<dyn Error>> {
    output::info("Rotiere Schlüssel...");

    let store = keys::KeyStore::new(dir)?;

    // Load current key metadata
    let mut current_key = keys::KeyMetadata::load(current_path)?;
    output::detail("Aktuell", &format!("{} ({})", current_key.kid, current_key.owner));

    // Load new key metadata
    let new_key = keys::KeyMetadata::load(new_path)?;
    output::detail("Neu", &format!("{} ({})", new_key.kid, new_key.owner));

    // Mark current key as retired
    current_key.retire();
    current_key.save(current_path)?;

    // Archive current key
    store.archive(&current_key.kid)?;

    output::success("Rotation erfolgreich:");
    output::indent("Alter Schlüssel -> retired + archiviert");
    output::indent("Neuer Schlüssel -> aktiv");

    // Log audit event
    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
    audit.log_event(
        "key_rotated",
        json!({
            "old_kid": current_key.kid,
            "new_kid": new_key.kid,
            "owner": new_key.owner,
        }),
    )?;

    Ok(())
}

/// Keys attest - Attestiert neuen Schlüssel mit altem
pub fn run_keys_attest(
    signer_path: &str,
    subject_path: &str,
    out_path: &str,
) -> Result<(), Box<dyn Error>> {
    use base64::Engine;
    use ed25519_dalek::{Signer, SigningKey};

    output::document("Attestiere Schlüssel...");

    // Load signer key metadata
    let signer_meta = keys::KeyMetadata::load(signer_path)?;
    output::detail("Signer", &format!("{} ({})", signer_meta.kid, signer_meta.owner));

    // Load subject key metadata
    let subject_meta = keys::KeyMetadata::load(subject_path)?;
    output::detail("Subject", &format!("{} ({})", subject_meta.kid, subject_meta.owner));

    // Load signer private key
    let signer_priv_path = signer_path.replace(".json", ".ed25519");
    let signer_key_bytes = fs::read(&signer_priv_path)?;
    let signing_key = SigningKey::from_bytes(
        &signer_key_bytes
            .try_into()
            .map_err(|_| "Invalid key length")?,
    );

    // Create attestation document
    let attestation = json!({
        "schema": "cap-attestation.v1",
        "signer_kid": signer_meta.kid,
        "signer_owner": signer_meta.owner,
        "subject_kid": subject_meta.kid,
        "subject_owner": subject_meta.owner,
        "subject_public_key": subject_meta.public_key,
        "attested_at": chrono::Utc::now().to_rfc3339(),
    });

    // Sign the attestation
    let attestation_bytes = serde_json::to_vec(&attestation)?;
    let signature = signing_key.sign(&attestation_bytes);

    // Create final document with signature
    let signed_attestation = json!({
        "attestation": attestation,
        "signature": base64::engine::general_purpose::STANDARD.encode(signature.to_bytes()),
        "signer_public_key": signer_meta.public_key,
    });

    // Save attestation
    let json_output = serde_json::to_string_pretty(&signed_attestation)?;
    fs::write(out_path, json_output)?;

    output::success("Attestation erstellt:");
    output::detail("Output", out_path);

    // Log audit event
    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
    audit.log_event(
        "key_attested",
        json!({
            "signer_kid": signer_meta.kid,
            "subject_kid": subject_meta.kid,
            "attestation_file": out_path,
        }),
    )?;

    Ok(())
}

/// Keys archive - Archiviert Schlüssel
pub fn run_keys_archive(dir: &str, kid: &str) -> Result<(), Box<dyn Error>> {
    output::packaging("Archiviere Schlüssel...");
    output::detail("KID", kid);

    let store = keys::KeyStore::new(dir)?;
    store.archive(kid)?;

    output::success("Schlüssel archiviert");
    output::detail("Verschoben nach", &format!("{}/archive/", dir));

    // Log audit event
    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
    audit.log_event(
        "key_archived",
        json!({
            "kid": kid,
        }),
    )?;

    Ok(())
}

/// Keys verify-chain - Verifiziert eine Chain-of-Trust
pub fn run_keys_verify_chain(dir: &str, attestation_paths: &[String]) -> Result<(), Box<dyn Error>> {
    output::secure("Verifiziere Chain-of-Trust...");
    output::detail("Keys Directory", dir);
    output::detail_fmt("Attestationen", attestation_paths.len());

    // Convert Vec<String> to Vec<&str> for verify_chain
    let paths: Vec<&str> = attestation_paths.iter().map(|s| s.as_str()).collect();

    // Open key store
    let store = keys::KeyStore::new(dir)?;

    // Verify chain
    keys::verify_chain(&paths, &store)?;

    output::success("Chain-of-Trust verifiziert");
    output::indent("Alle Attestationen gültig");
    output::indent("Chain ist konsistent");

    // Log audit event
    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
    audit.log_event(
        "chain_verified",
        json!({
            "attestation_count": attestation_paths.len(),
            "keys_dir": dir,
        }),
    )?;

    Ok(())
}
