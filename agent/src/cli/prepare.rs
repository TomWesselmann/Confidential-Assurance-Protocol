//! CLI-Handler für prepare und inspect Kommandos
//!
//! Extrahiert aus main.rs für bessere Wartbarkeit.

use super::output;
use crate::audit::AuditLog;
use crate::commitment::{
    self, compute_company_root, compute_supplier_root, compute_ubo_root, Commitments,
};
use crate::io::{self, Supplier, Ubo};
use serde_json::json;
use std::error::Error;
use std::fs;

// ============================================================================
// Helper-Funktionen
// ============================================================================

/// Lädt CSV-Daten und loggt Audit-Event
fn load_csv_data(
    suppliers_path: &str,
    ubos_path: &str,
    audit: &mut AuditLog,
) -> Result<(Vec<Supplier>, Vec<Ubo>), Box<dyn Error>> {
    output::input(&format!("Lese Suppliers aus: {}", suppliers_path));
    let suppliers = io::read_suppliers_csv(suppliers_path)?;
    audit.log_event("data_loaded", json!({"type": "suppliers", "count": suppliers.len()}))?;

    output::input(&format!("Lese UBOs aus: {}", ubos_path));
    let ubos = io::read_ubos_csv(ubos_path)?;
    audit.log_event("data_loaded", json!({"type": "ubos", "count": ubos.len()}))?;

    Ok((suppliers, ubos))
}

/// Berechnet alle Merkle-Roots und loggt Audit-Events
fn compute_merkle_roots(
    suppliers: &[Supplier],
    ubos: &[Ubo],
    audit: &mut AuditLog,
) -> Result<(String, String, String), Box<dyn Error>> {
    output::stats("Berechne Supplier-Root...");
    let supplier_root = compute_supplier_root(suppliers)?;
    audit.log_event("merkle_root_computed", json!({"target": "suppliers", "root": &supplier_root}))?;

    output::stats("Berechne UBO-Root...");
    let ubo_root = compute_ubo_root(ubos)?;
    audit.log_event("merkle_root_computed", json!({"target": "ubos", "root": &ubo_root}))?;

    output::stats("Berechne Company-Commitment-Root...");
    let company_root = compute_company_root(&supplier_root, &ubo_root);
    audit.log_event("merkle_root_computed", json!({"target": "company", "root": &company_root}))?;

    Ok((supplier_root, ubo_root, company_root))
}

/// Zeigt Ergebnis-Zusammenfassung an
fn print_results(supplier_root: &str, ubo_root: &str, company_root: &str, output_path: &str) {
    output::success("Erfolgreich abgeschlossen!");
    output::section("Ergebnisse:");
    output::indent(&format!("Supplier Root:  {}", supplier_root));
    output::indent(&format!("UBO Root:       {}", ubo_root));
    output::indent(&format!("Company Root:   {}", company_root));
    output::section("Ausgabedateien:");
    output::indent(&format!("- {}", output_path));
    output::indent("- build/agent.audit.jsonl");
}

// ============================================================================
// Öffentliche CLI-Funktionen
// ============================================================================

/// Hauptfunktion: Führt das prepare-Kommando aus
pub fn run_prepare(suppliers_path: &str, ubos_path: &str) -> Result<(), Box<dyn Error>> {
    output::info("Starte Commitment-Berechnung...");
    fs::create_dir_all("build")?;

    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
    audit.log_event("prepare_started", json!({"suppliers_file": suppliers_path, "ubos_file": ubos_path}))?;

    // Lade Daten
    let (suppliers, ubos) = load_csv_data(suppliers_path, ubos_path, &mut audit)?;

    // Berechne Roots
    let (supplier_root, ubo_root, company_root) = compute_merkle_roots(&suppliers, &ubos, &mut audit)?;

    // Erstelle und speichere Commitments
    let commitments = Commitments {
        supplier_root: supplier_root.clone(),
        ubo_root: ubo_root.clone(),
        company_commitment_root: company_root.clone(),
        supplier_count: Some(suppliers.len()),
        ubo_count: Some(ubos.len()),
    };

    let output_path = "build/commitments.json";
    output::saving(&format!("Speichere Commitments nach: {}", output_path));
    commitment::save_commitments(&commitments, output_path)?;
    audit.log_event("commitments_saved", json!({"path": output_path}))?;

    print_results(&supplier_root, &ubo_root, &company_root, output_path);
    Ok(())
}

/// Führt das inspect-Kommando aus
pub fn run_inspect(path: &str) -> Result<(), Box<dyn Error>> {
    output::searching(&format!("Lese Commitments von: {}", path));

    let commitments = commitment::load_commitments(path)?;
    let json = serde_json::to_string_pretty(&commitments)?;

    println!("\n{}", json);

    Ok(())
}
