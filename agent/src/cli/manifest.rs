//! CLI-Handler für Manifest-Kommandos
//!
//! Extrahiert aus main.rs für bessere Wartbarkeit.

use super::output;
use crate::audit::AuditLog;
use crate::commitment;
use crate::manifest;
use crate::policy;
use crate::registry;
use crate::verifier::core as verifier_core;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::Error;
use std::fs;

/// Verification Report für manifest verify Kommando
#[allow(dead_code)] // Used for JSON output in CLI commands
#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationReport {
    pub manifest_hash: String,
    pub proof_hash: String,
    pub timestamp_valid: bool,
    pub registry_match: bool,
    pub signature_valid: bool,
    pub status: String,
}

// ============================================================================
// Helper-Funktionen für run_manifest_verify
// ============================================================================

/// Verifiziert Timestamp aus Datei (optional)
fn verify_timestamp_if_provided(timestamp_path: Option<&str>) -> bool {
    match timestamp_path {
        Some(ts_path) => {
            let valid = registry::verify_timestamp_from_file(ts_path);
            println!(
                "      Timestamp: {}",
                if valid {
                    "✅ gültig"
                } else {
                    "❌ ungültig"
                }
            );
            valid
        }
        None => {
            println!("      Timestamp: ⚠️  nicht angegeben (optional)");
            true
        }
    }
}

/// Verifiziert Registry-Eintrag
fn verify_registry_entry(
    registry_path: &str,
    manifest_hash: &str,
    proof_hash: &str,
) -> bool {
    let registry_match = registry::verify_entry_from_file(
        registry_path,
        manifest_hash,
        proof_hash,
    )
    .unwrap_or(false);

    println!(
        "      Registry: {}",
        if registry_match {
            "✅ Eintrag gefunden"
        } else {
            "❌ Kein Eintrag"
        }
    );

    registry_match
}

/// Erstellt VerificationReport und speichert ihn
fn create_and_save_report(
    core_report: &verifier_core::VerifyReport,
    timestamp_valid: bool,
    registry_match: bool,
    out_path: Option<String>,
) -> Result<(VerificationReport, String), Box<dyn Error>> {
    let all_ok = core_report.signature_valid
        && timestamp_valid
        && registry_match
        && core_report.status == "ok";
    let status = if all_ok { "ok" } else { "fail" }.to_string();

    let report = VerificationReport {
        manifest_hash: core_report.manifest_hash.clone(),
        proof_hash: core_report.proof_hash.clone(),
        timestamp_valid,
        registry_match,
        signature_valid: core_report.signature_valid,
        status,
    };

    let report_path = out_path.unwrap_or_else(|| "build/verification.report.json".to_string());
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&report_path, json)?;

    Ok((report, report_path))
}

/// Loggt Audit-Event für Verifikation
fn log_verification_audit(
    manifest_path: &str,
    proof_path: &str,
    registry_path: &str,
    timestamp_path: &Option<String>,
    status: &str,
    report_path: &str,
    core_details: &serde_json::Value,
) -> Result<(), Box<dyn Error>> {
    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
    audit.log_event(
        "manifest_verified",
        json!({
            "manifest_file": manifest_path,
            "proof_file": proof_path,
            "registry_file": registry_path,
            "timestamp_file": timestamp_path,
            "status": status,
            "report_file": report_path,
            "core_details": core_details
        }),
    )?;
    Ok(())
}

// ============================================================================
// Öffentliche CLI-Funktionen
// ============================================================================

/// Manifest build - Erstellt ein Manifest aus Commitments und Policy
pub fn run_manifest_build(
    policy_path: &str,
    out: Option<String>,
) -> Result<(), Box<dyn Error>> {
    output::writing("Erstelle Manifest...");

    fs::create_dir_all("build")?;
    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;

    // Lade Policy
    let policy = policy::Policy::load(policy_path)?;
    policy.validate()?;
    let policy_info = policy.to_info()?;

    // Lade Commitments
    let commitments = commitment::load_commitments("build/commitments.json")?;

    // Erstelle Manifest
    let manifest =
        manifest::Manifest::build(&commitments, policy_info, "build/agent.audit.jsonl")?;

    // Speichere Manifest
    let output_path = out.unwrap_or_else(|| "build/manifest.json".to_string());
    manifest.save(&output_path)?;

    audit.log_event(
        "manifest_built",
        json!({
            "output": &output_path,
            "policy": &policy.name
        }),
    )?;

    output::success_with("Manifest erstellt", &output_path);

    Ok(())
}

/// Manifest validate - Validiert ein Manifest gegen das JSON Schema
pub fn run_manifest_validate(
    manifest_path: &str,
    schema_path: Option<String>,
) -> Result<(), Box<dyn Error>> {
    output::searching("Validiere Manifest gegen JSON Schema...");

    // Default schema path
    let schema_file = schema_path.unwrap_or_else(|| "docs/manifest.schema.json".to_string());

    // Load manifest JSON
    let manifest_content = std::fs::read_to_string(manifest_path)?;
    let manifest_json: serde_json::Value = serde_json::from_str(&manifest_content)?;

    // Load schema JSON
    let schema_content = std::fs::read_to_string(&schema_file)?;
    let schema_json: serde_json::Value = serde_json::from_str(&schema_content)?;

    // Compile schema (Draft 2020-12)
    use jsonschema::JSONSchema;
    let compiled = JSONSchema::options()
        .compile(&schema_json)
        .map_err(|e| format!("Schema compilation failed: {}", e))?;

    // Validate
    if compiled.is_valid(&manifest_json) {
        output::success_with("Manifest ist gültig gemäß Schema", &schema_file);
        output::detail("Manifest", manifest_path);
        output::detail("Schema", &schema_file);

        // Log audit event
        let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
        audit.log_event(
            "manifest_validated",
            json!({
                "manifest_file": manifest_path,
                "schema_file": schema_file,
                "status": "valid"
            }),
        )?;

        Ok(())
    } else {
        output::error("Manifest-Validierung fehlgeschlagen:");

        // Collect errors
        if let Err(errors) = compiled.validate(&manifest_json) {
            for (i, error) in errors.enumerate() {
                output::indent(&format!("Fehler #{}: {}", i + 1, error));
            }
        }

        Err("Manifest validation failed".into())
    }
}

/// Manifest verify - Führt vollständige Offline-Verifikation eines Proof-Pakets durch
pub fn run_manifest_verify(
    manifest_path: &str,
    proof_path: &str,
    registry_path: &str,
    timestamp_path: Option<String>,
    out_path: Option<String>,
) -> Result<(), Box<dyn Error>> {
    output::searching("Starte vollständige Offline-Verifikation (mit portable core)...");

    // 1️⃣ Load files
    output::step(1, 5, "Lade Dateien");
    let manifest_bytes = fs::read(manifest_path)?;
    let proof_bytes = fs::read(proof_path)?;
    let manifest_json: serde_json::Value = serde_json::from_slice(&manifest_bytes)?;
    output::indent("   ✅ Manifest geladen");

    // 2️⃣ Extract statement from manifest
    output::step(2, 5, "Extrahiere Statement");
    let stmt = verifier_core::extract_statement_from_manifest(&manifest_json)?;
    output::indent("   ✅ Statement extrahiert");
    output::indent(&format!("      Policy Hash: {}", stmt.policy_hash));
    output::indent(&format!("      Company Root: {}", stmt.company_commitment_root));

    // 3️⃣ Core verification
    output::step(3, 5, "Führe Verifikation durch");
    let opts = verifier_core::VerifyOptions {
        check_timestamp: timestamp_path.is_some(),
        check_registry: true,
    };
    let core_report = verifier_core::verify(&manifest_json, &proof_bytes, &stmt, &opts)?;

    output::indent("   ✅ Core Verifikation abgeschlossen");
    output::indent(&format!("      Manifest Hash: {}", core_report.manifest_hash));
    output::indent(&format!("      Proof Hash: {}", core_report.proof_hash));
    output::indent(&format!(
        "      Signatur: {}",
        if core_report.signature_valid { "✅" } else { "⚠️" }
    ));

    // 4️⃣ Additional checks (timestamp and registry)
    output::step(4, 5, "Zusätzliche Prüfungen");
    let timestamp_valid = verify_timestamp_if_provided(timestamp_path.as_deref());
    let registry_match = verify_registry_entry(
        registry_path,
        &core_report.manifest_hash,
        &core_report.proof_hash,
    );

    // 5️⃣ Create and save report
    output::step(5, 5, "Speichere Report");
    let (report, report_path) = create_and_save_report(
        &core_report,
        timestamp_valid,
        registry_match,
        out_path,
    )?;

    // 6️⃣ Log audit event
    log_verification_audit(
        manifest_path,
        proof_path,
        registry_path,
        &timestamp_path,
        &report.status,
        &report_path,
        &core_report.details,
    )?;

    // 7️⃣ Print result
    output::section("");
    if report.status == "ok" {
        output::success("Verifikation erfolgreich!");
        output::detail("Report", &report_path);
        Ok(())
    } else {
        output::error("Verifikation fehlgeschlagen!");
        output::detail("Report", &report_path);
        output::indent(&format!(
            "Details: {}",
            serde_json::to_string_pretty(&core_report.details)?
        ));
        Err("Verification failed".into())
    }
}
