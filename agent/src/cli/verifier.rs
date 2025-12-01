//! CLI-Handler für Verifier-Kommandos
//!
//! Extrahiert aus main.rs für bessere Wartbarkeit.
//! Enthält: run_verifier_run, run_verifier_extract, run_verifier_audit

use super::output;
use crate::audit::AuditLog;
use crate::package_verifier;
use serde_json::json;
use std::error::Error;
use std::path::Path;

/// Verifier run - Verifiziert Proof-Paket
pub fn run_verifier_run(package_path: &str) -> Result<(), Box<dyn Error>> {
    output::searching("Verifiziere Proof-Paket...");

    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
    let package_dir = Path::new(package_path);

    // Prüfe ob _meta.json existiert (cap-bundle.v1 Format)
    let meta_path = package_dir.join("_meta.json");

    if meta_path.exists() {
        // Verwende BundleVerifier für cap-bundle.v1
        output::packaging("Erkanntes Format: cap-bundle.v1");

        let bundle_verifier = package_verifier::BundleVerifier::new(package_dir);
        let result = bundle_verifier.verify_bundle()?;

        // Log Audit-Event
        audit.log_event(
            "bundle_verifier_run",
            json!({
                "bundle_id": result.bundle_id,
                "schema": result.schema,
                "status": format!("{:?}", result.status),
                "unit_count": result.unit_results.len()
            }),
        )?;

        // Zeige Ergebnisse
        output::section("");
        output::success("Bundle-Verifikation abgeschlossen!");
        output::detail("Bundle ID", &result.bundle_id);
        output::detail("Schema", &result.schema);
        output::detail_fmt("Status", format!("{:?}", result.status));
        output::detail_fmt("Proof Units", result.unit_results.len());

        // Zeige einzelne Unit-Ergebnisse
        for (unit_id, unit_result) in &result.unit_results {
            output::section(&format!("  📋 Unit '{}': {:?}", unit_id, unit_result.status));
            output::indent(&format!("  Manifest Hash: {}", unit_result.manifest_hash));
            output::indent(&format!("  Proof Hash: {}", unit_result.proof_hash));
        }
    } else {
        // Fallback zu Legacy Verifier (Backward-Compatibility)
        output::packaging("Erkanntes Format: Legacy (pre-bundle.v1)");

        let verifier = package_verifier::Verifier::new(package_path);

        // Prüfe Integrität
        let integrity = verifier.check_package_integrity()?;
        output::listing(&integrity);

        // Verifiziere
        let result = verifier.verify()?;

        audit.log_event(
            "verifier_run",
            json!({
                "package": package_path,
                "success": result.success,
                "checks_passed": result.checks_passed,
                "checks_total": result.checks_total
            }),
        )?;

        output::section("");
        output::success("Verifikation erfolgreich!");
        output::detail("Manifest Hash", &result.manifest_hash);
        output::detail("Policy Hash", &result.policy_hash);
        output::detail("Proof Status", &result.proof_status);
        output::detail_fmt("Checks", format!("{}/{}", result.checks_passed, result.checks_total));
    }

    Ok(())
}

/// Verifier extract - Extrahiert Manifest-Infos
pub fn run_verifier_extract(package_path: &str) -> Result<(), Box<dyn Error>> {
    output::searching("Extrahiere Informationen aus Proof-Paket...");

    let summary = package_verifier::show_package_summary(package_path)?;
    println!("\n{}", summary);

    Ok(())
}

/// Verifier audit - Zeigt Audit-Trail
pub fn run_verifier_audit(package_path: &str) -> Result<(), Box<dyn Error>> {
    output::searching("Zeige Audit-Trail...");

    let verifier = package_verifier::Verifier::new(package_path);
    let (tail_digest, events_count) = verifier.show_audit_trail()?;

    output::stats("Audit-Trail:");
    output::detail_fmt("Events gesamt", events_count);
    output::detail("Tail Digest", &tail_digest);

    Ok(())
}
