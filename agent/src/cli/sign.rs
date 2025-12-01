//! CLI-Handler für Sign-Kommandos
//!
//! Extrahiert aus main.rs für bessere Wartbarkeit.
//! Enthält: run_sign_keygen, run_sign_manifest, run_verify_manifest

use super::output;
use crate::audit::AuditLog;
use crate::{manifest, sign};
use serde_json::json;
use std::error::Error;
use std::fs;

/// Sign keygen
pub fn run_sign_keygen(dir: Option<String>) -> Result<(), Box<dyn Error>> {
    let key_dir = dir.unwrap_or_else(|| "keys".to_string());
    fs::create_dir_all(&key_dir)?;

    let priv_path = format!("{}/company.ed25519", key_dir);
    let pub_path = format!("{}/company.pub", key_dir);

    output::key("Generiere Ed25519-Schlüsselpaar...");

    sign::generate_keypair(&priv_path, &pub_path)?;

    output::success("Schlüsselpaar generiert:");
    output::detail("Private", &priv_path);
    output::detail("Public", &pub_path);

    Ok(())
}

/// Sign manifest
pub fn run_sign_manifest(
    key_path: &str,
    manifest_path: &str,
    out: &str,
    signer: Option<String>,
) -> Result<(), Box<dyn Error>> {
    output::writing("Signiere Manifest...");

    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;

    // Lade Schlüssel und Manifest
    let signing_key = sign::load_private_key(key_path)?;
    let manifest = manifest::Manifest::load(manifest_path)?;

    // Signiere
    let signer_name = signer.unwrap_or_else(|| "Company".to_string());
    let signed = sign::sign_manifest(&manifest, &signing_key, &signer_name)?;

    // Speichere
    signed.save(out)?;

    audit.log_event(
        "manifest_signed",
        json!({
            "output": out,
            "signer": &signer_name
        }),
    )?;

    output::success_with("Manifest signiert", out);

    Ok(())
}

/// Verify signed manifest
pub fn run_verify_manifest(pub_key_path: &str, signed_path: &str) -> Result<(), Box<dyn Error>> {
    output::searching("Verifiziere signiertes Manifest...");

    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;

    // Lade Public Key und signiertes Manifest
    let verifying_key = sign::load_public_key(pub_key_path)?;
    let signed = manifest::SignedManifest::load(signed_path)?;

    // Verifiziere
    sign::verify_manifest(&signed, &verifying_key)?;

    audit.log_event("manifest_verified", json!({ "file": signed_path }))?;

    output::success("Signatur ist gültig!");
    output::detail("Signer", &signed.signature.signer);

    Ok(())
}
