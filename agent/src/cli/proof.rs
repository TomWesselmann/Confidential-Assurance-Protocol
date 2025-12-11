//! CLI-Handler für Proof-Kommandos (Minimal Local Agent)
//!
//! Enthält: run_proof_mock, run_proof_build, run_proof_verify_v3, run_proof_export

use super::output;
use crate::audit::AuditLog;
use crate::commitment::Commitments;
use crate::io::JsonPersistent;
use crate::{commitment, io, manifest, policy, proof_engine, proof_mock};
use crate::bundle::export;
use serde_json::json;
use std::error::Error;
use std::fs;

// ============================================================================
// Öffentliche CLI-Funktionen
// ============================================================================

/// Proof mock - Generiert einen Mock-Proof für Tests
pub fn run_proof_mock(policy_path: &str, manifest_path: &str) -> Result<(), Box<dyn Error>> {
    output::secure("Generiere Mock-Proof...");

    fs::create_dir_all("build")?;
    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;

    // Lade Policy und Manifest
    let policy = policy::Policy::load(policy_path)?;
    let manifest = manifest::Manifest::load(manifest_path)?;

    // Lade Original-Daten für Count
    let suppliers = io::read_suppliers_csv("../examples/suppliers.csv").unwrap_or_default();
    let ubos = io::read_ubos_csv("../examples/ubos.csv").unwrap_or_default();

    // Generiere Mock-Proof
    let proof = proof_mock::MockProof::generate(&policy, &manifest, suppliers.len(), ubos.len())?;

    // Speichere
    let output_path = "build/proof.mock.json";
    proof.save(output_path)?;

    audit.log_event(
        "mock_proof_generated",
        json!({
            "output": output_path,
            "status": &proof.status
        }),
    )?;

    output::success_with("Mock-Proof generiert", output_path);
    output::detail("Status", &proof.status);

    Ok(())
}

/// Proof build - Erstellt strukturierten Proof
pub fn run_proof_build(policy_path: &str, manifest_path: &str) -> Result<(), Box<dyn Error>> {
    output::secure("Erstelle Proof...");

    fs::create_dir_all("build")?;
    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;

    // Lade Policy und Manifest
    let policy = policy::Policy::load(policy_path)?;
    let manifest = manifest::Manifest::load(manifest_path)?;

    // Lade Commitments für Count-Daten
    let commitments = commitment::load_commitments("build/commitments.json")?;
    let supplier_count = commitments.supplier_count.unwrap_or(0);
    let ubo_count = commitments.ubo_count.unwrap_or(0);

    // Generiere Proof
    let proof = proof_engine::Proof::build(&policy, &manifest, supplier_count, ubo_count)?;

    // Speichere als .dat und .json
    let output_path_dat = "build/proof.dat";
    let output_path_json = "build/proof.json";
    proof.save_as_dat(output_path_dat)?;
    proof.save(output_path_json)?;

    audit.log_event(
        "proof_built",
        json!({
            "output_dat": output_path_dat,
            "output_json": output_path_json,
            "status": &proof.status
        }),
    )?;

    output::success("Proof erstellt:");
    output::indent(&format!("- {}", output_path_dat));
    output::indent(&format!("- {}", output_path_json));
    output::detail("Status", &proof.status);

    Ok(())
}

/// Proof verify - Verifiziert Proof gegen Manifest
pub fn run_proof_verify_v3(proof_path: &str, manifest_path: &str) -> Result<(), Box<dyn Error>> {
    output::searching("Verifiziere Proof...");

    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;

    // Lade Proof (automatische Erkennung .dat oder .json)
    let proof = if proof_path.ends_with(".dat") {
        proof_engine::Proof::load_from_dat(proof_path)?
    } else {
        proof_engine::Proof::load(proof_path)?
    };

    // Lade Manifest
    let manifest = manifest::Manifest::load(manifest_path)?;

    // Verifiziere
    proof.verify(&manifest)?;

    audit.log_event("proof_verified", json!({ "proof": proof_path }))?;

    output::success("Proof ist gültig!");
    output::detail("Manifest Hash", &proof.manifest_hash);
    output::detail("Policy Hash", &proof.policy_hash);
    output::detail("Status", &proof.status);

    Ok(())
}

/// Proof export - Exportiert standardisiertes CAP Proof-Paket (v1.0)
pub fn run_proof_export(
    manifest_path: &str,
    proof_path: &str,
    timestamp_path: Option<String>,
    registry_path: Option<String>,
    report_path: Option<String>,
    output: Option<String>,
    force: bool,
) -> Result<(), Box<dyn Error>> {
    output::packaging("Exportiere CAP Bundle (cap-bundle.v1)...");

    // Delegiere an bundle::export
    let result = export::export_bundle(
        manifest_path,
        proof_path,
        timestamp_path.clone(),
        registry_path.clone(),
        report_path.clone(),
        output,
        force,
    )?;

    // Audit-Log-Eintrag
    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
    audit.log_event(
        "bundle_exported",
        json!({
            "output": &result.output_dir,
            "schema": "cap-bundle.v1",
            "bundle_id": &result.bundle_id,
            "has_timestamp": timestamp_path.is_some(),
            "has_registry": registry_path.is_some(),
            "has_report": report_path.is_some()
        }),
    )?;

    // Erfolg-Output
    output::section("");
    output::success("CAP Bundle erfolgreich exportiert (cap-bundle.v1)!");
    output::detail("Verzeichnis", &result.output_dir);
    output::detail("Bundle ID", &result.bundle_id);
    output::detail_fmt("Dateien", result.file_count);
    output::detail("Package Version", "cap-proof.v1.0");

    Ok(())
}

// Note: ZK functions (run_zk_build, run_zk_verify, run_zk_bench) and
// adaptive orchestration (run_proof_adapt) removed in minimal local agent.
// These features require:
// - zk_system module (real ZK backends)
// - lists module (sanctions lists)
// - orchestrator module (adaptive proof selection)
