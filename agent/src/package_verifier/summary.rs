//! Package Summary - Formatted package information display

use std::error::Error;
use std::path::Path;

use super::verifier::Verifier;

/// Zeigt eine formatierte Zusammenfassung eines Proof-Pakets
///
/// # Argumente
/// * `package_dir` - Pfad zum Proof-Paket
///
/// # Rückgabe
/// Formatierter String oder Fehler
pub fn show_package_summary<P: AsRef<Path>>(package_dir: P) -> Result<String, Box<dyn Error>> {
    let verifier = Verifier::new(package_dir);

    let manifest = verifier.extract_manifest()?;
    let proof = verifier.extract_proof()?;

    let mut summary = String::new();
    summary.push_str("=== PROOF-PAKET ZUSAMMENFASSUNG ===\n\n");

    summary.push_str("Manifest:\n");
    summary.push_str(&format!("  Version: {}\n", manifest.version));
    summary.push_str(&format!("  Erstellt: {}\n", manifest.created_at));
    summary.push_str(&format!(
        "  Company Root: {}\n",
        manifest.company_commitment_root
    ));
    summary.push_str(&format!(
        "  Policy: {} ({})\n",
        manifest.policy.name, manifest.policy.version
    ));
    summary.push_str(&format!("  Policy Hash: {}\n", manifest.policy.hash));
    summary.push_str(&format!(
        "  Audit Events: {}\n",
        manifest.audit.events_count
    ));
    summary.push_str(&format!("  Audit Tail: {}\n\n", manifest.audit.tail_digest));

    summary.push_str("Proof:\n");
    summary.push_str(&format!("  Version: {}\n", proof.version));
    summary.push_str(&format!("  Typ: {}\n", proof.proof_type));
    summary.push_str(&format!("  Statement: {}\n", proof.statement));
    summary.push_str(&format!("  Status: {}\n", proof.status));
    summary.push_str(&format!(
        "  Checks: {}/{}\n",
        proof
            .proof_data
            .checked_constraints
            .iter()
            .filter(|c| c.ok)
            .count(),
        proof.proof_data.checked_constraints.len()
    ));

    summary.push_str("\nConstraint-Checks:\n");
    for check in &proof.proof_data.checked_constraints {
        summary.push_str(&format!(
            "  {} {}\n",
            if check.ok { "✅" } else { "❌" },
            check.name
        ));
    }

    Ok(summary)
}
