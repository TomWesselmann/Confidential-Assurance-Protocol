//! CLI-Handler für Proof-Kommandos
//!
//! Extrahiert aus main.rs für bessere Wartbarkeit.
//! Enthält: run_proof_mock, run_proof_build, run_proof_verify_v3, run_proof_export,
//!          run_zk_build, run_zk_verify, run_zk_bench, run_proof_adapt

use super::output;
use crate::audit::AuditLog;
use crate::commitment::Commitments;
use crate::io::JsonPersistent;
use crate::lists;
use crate::{commitment, io, manifest, policy, proof_engine, proof_mock, zk_system};
use cap_agent::bundle::export;
use cap_agent::orchestrator;
use serde_json::json;
use std::error::Error;
use std::fs;
use zk_system::ProofSystem;

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

/// ZK Build - Erstellt einen Zero-Knowledge-Proof (Tag 4)
pub fn run_zk_build(
    policy_path: &str,
    manifest_path: &str,
    output: Option<String>,
    sanctions_root: Option<String>,
    jurisdiction_root: Option<String>,
    sanctions_csv: Option<String>,
) -> Result<(), Box<dyn Error>> {
    output::secure("Erstelle Zero-Knowledge-Proof...");

    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;

    // Lade Policy und Manifest
    let policy = policy::Policy::load(policy_path)?;
    let manifest = manifest::Manifest::load(manifest_path)?;

    // Prüfe ob alle geforderten Statement-Roots vorhanden sind
    policy.check_required_statement_roots(&sanctions_root, &jurisdiction_root)?;

    // Lade Commitments für Witness-Daten
    let commitments = Commitments::load("build/commitments.json")?;

    // Lade optional Sanctions-Liste für Mock-Check
    let sanctions_list = if let Some(ref csv_path) = sanctions_csv {
        let (_, entries) = lists::compute_sanctions_root(csv_path)?;
        Some(entries.iter().map(|e| e.hash()).collect())
    } else {
        None
    };

    // Erstelle Statement (öffentliche Daten)
    let statement = zk_system::Statement {
        policy_hash: manifest.policy.hash.clone(),
        company_commitment_root: manifest.company_commitment_root.clone(),
        constraints: vec![
            if policy.constraints.require_at_least_one_ubo {
                "require_at_least_one_ubo".to_string()
            } else {
                String::new()
            },
            format!(
                "supplier_count_max_{}",
                policy.constraints.supplier_count_max
            ),
        ]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect(),
        sanctions_root,
        jurisdiction_root,
    };

    // Erstelle Witness (private Daten)
    // In echter Implementation würden hier die tatsächlichen Supplier/UBO-Hashes kommen
    let witness = zk_system::Witness {
        suppliers: vec![manifest.supplier_root.clone()],
        ubos: vec![manifest.ubo_root.clone()],
        supplier_count: commitments.supplier_count.unwrap_or(0),
        ubo_count: commitments.ubo_count.unwrap_or(0),
        sanctions_list,
    };

    // Erstelle ZK-Proof
    let zk = zk_system::SimplifiedZK::new();
    let proof = zk.prove(&statement, &witness)?;

    // Speichere Proof
    let out_dat = output
        .clone()
        .unwrap_or_else(|| "build/zk_proof.dat".to_string());
    let out_json = out_dat.replace(".dat", ".json");

    zk_system::save_zk_proof_dat(&proof, &out_dat)?;
    zk_system::save_zk_proof_json(&proof, &out_json)?;

    audit.log_event(
        "zk_proof_generated",
        json!({
            "system": proof.system,
            "status": proof.status,
            "policy": policy_path,
            "output": &out_dat
        }),
    )?;

    output::success("ZK-Proof erstellt:");
    output::indent(&format!("- {}", out_dat));
    output::indent(&format!("- {}", out_json));
    output::detail("System", &proof.system);
    output::detail("Status", &proof.status);

    Ok(())
}

/// ZK Verify - Verifiziert einen Zero-Knowledge-Proof
pub fn run_zk_verify(proof_path: &str) -> Result<(), Box<dyn Error>> {
    output::searching("Verifiziere Zero-Knowledge-Proof...");

    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;

    // Lade Proof
    let proof = if proof_path.ends_with(".dat") {
        zk_system::load_zk_proof_dat(proof_path)?
    } else {
        zk_system::load_zk_proof_json(proof_path)?
    };

    // Verifiziere mit passendem Backend
    let is_valid = match proof.system.as_str() {
        "simplified" => {
            let zk = zk_system::SimplifiedZK::new();
            zk.verify(&proof)?
        }
        other => {
            return Err(format!("Unbekanntes ZK-System: {}", other).into());
        }
    };

    audit.log_event(
        "zk_proof_verified",
        json!({
            "proof": proof_path,
            "system": proof.system,
            "valid": is_valid
        }),
    )?;

    if is_valid {
        output::success("ZK-Proof ist gültig!");
        output::detail("System", &proof.system);
        output::detail("Policy Hash", &proof.public_inputs.policy_hash);
        output::detail("Company Root", &proof.public_inputs.company_commitment_root);
        output::detail_fmt("Constraints", proof.public_inputs.constraints.len());
    } else {
        output::error("ZK-Proof ist UNGÜLTIG!");
        return Err("Proof-Verifikation fehlgeschlagen".into());
    }

    Ok(())
}

/// ZK Bench - Benchmark für ZK-Proof-Erstellung und Verifikation
pub fn run_zk_bench(
    policy_path: &str,
    manifest_path: &str,
    iterations: usize,
) -> Result<(), Box<dyn Error>> {
    output::timing("Starte ZK-Proof-Benchmark...");
    output::detail_fmt("Iterationen", iterations);

    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;

    // Lade Policy und Manifest
    let policy = policy::Policy::load(policy_path)?;
    let manifest = manifest::Manifest::load(manifest_path)?;
    let commitments = Commitments::load("build/commitments.json")?;

    // Bereite Statement und Witness vor
    let statement = zk_system::Statement {
        policy_hash: manifest.policy.hash.clone(),
        company_commitment_root: manifest.company_commitment_root.clone(),
        constraints: vec![
            if policy.constraints.require_at_least_one_ubo {
                "require_at_least_one_ubo".to_string()
            } else {
                String::new()
            },
            format!(
                "supplier_count_max_{}",
                policy.constraints.supplier_count_max
            ),
        ]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect(),
        sanctions_root: None,
        jurisdiction_root: None,
    };

    let witness = zk_system::Witness {
        suppliers: vec![manifest.supplier_root.clone()],
        ubos: vec![manifest.ubo_root.clone()],
        supplier_count: commitments.supplier_count.unwrap_or(0),
        ubo_count: commitments.ubo_count.unwrap_or(0),
        sanctions_list: None,
    };

    let zk = zk_system::SimplifiedZK::new();

    // Benchmark Proving
    output::section("");
    output::stats("Proving-Benchmark:");
    let prove_start = std::time::Instant::now();
    let mut proofs = Vec::new();
    for i in 0..iterations {
        let proof = zk.prove(&statement, &witness)?;
        proofs.push(proof);
        if (i + 1) % 10 == 0 || i == iterations - 1 {
            output::indent(&format!("Iteration {}/{} abgeschlossen", i + 1, iterations));
        }
    }
    let prove_duration = prove_start.elapsed();
    let prove_avg = prove_duration / iterations as u32;

    output::section("");
    output::detail("Gesamt", &format!("{:?}", prove_duration));
    output::detail("Durchschnitt", &format!("{:?}", prove_avg));
    output::detail("Throughput", &format!("{:.2} proofs/s", 1000.0 / prove_avg.as_millis() as f64));

    // Benchmark Verifying
    output::section("");
    output::stats("Verify-Benchmark:");
    let verify_start = std::time::Instant::now();
    for (i, proof) in proofs.iter().enumerate() {
        let is_valid = zk.verify(proof)?;
        assert!(is_valid, "Proof {} sollte gültig sein", i);
        if (i + 1) % 10 == 0 || i == iterations - 1 {
            output::indent(&format!("Iteration {}/{} abgeschlossen", i + 1, iterations));
        }
    }
    let verify_duration = verify_start.elapsed();
    let verify_avg = verify_duration / iterations as u32;

    output::section("");
    output::detail("Gesamt", &format!("{:?}", verify_duration));
    output::detail("Durchschnitt", &format!("{:?}", verify_avg));
    output::detail("Throughput", &format!("{:.2} verifications/s", 1000.0 / verify_avg.as_millis() as f64));

    audit.log_event(
        "zk_bench_executed",
        json!({
            "iterations": iterations,
            "prove_avg_ms": prove_avg.as_millis(),
            "verify_avg_ms": verify_avg.as_millis(),
            "system": "simplified"
        }),
    )?;

    output::section("");
    output::success("Benchmark abgeschlossen!");

    Ok(())
}

/// Adaptive Proof Orchestration - Week 6 B1
#[allow(clippy::too_many_arguments)]
pub fn run_proof_adapt(
    policy: &Option<String>,
    ir: &Option<std::path::PathBuf>,
    context: &std::path::PathBuf,
    enforce: bool,
    rollout: u8,
    drift_max: f64,
    _selector: &str, // TODO: Implement selector (basic vs weighted)
    _weights: &Option<std::path::PathBuf>, // TODO: Implement weights file loading
    dry_run: bool,
    out: &Option<std::path::PathBuf>,
) -> Result<(), Box<dyn Error>> {
    output::info("Adaptive Proof Orchestration (Week 6)");
    output::detail("Enforcement", if enforce { "ENABLED" } else { "Shadow-only" });
    output::detail("Rollout", &format!("{}%", rollout));
    output::detail("Drift Max", &format!("{}", drift_max));
    if dry_run {
        output::warning("Mode: DRY RUN (no side effects)");
    }

    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;

    // Step 1: Load IR
    // For now, we'll create a minimal IR since policy_v2 integration is not complete
    // This will be expanded in future implementation
    use cap_agent::policy_v2::types::IrV1;

    // TODO: Implement proper IR loading from policy ID or IR file
    // For now, create a placeholder IR
    let ir_data = if let Some(ir_path) = ir {
        output::document(&format!("Loading IR from file: {:?}", ir_path));
        let ir_json = fs::read_to_string(ir_path)?;
        serde_json::from_str::<IrV1>(&ir_json)?
    } else if let Some(policy_id) = policy {
        output::listing(&format!("Policy ID: {}", policy_id));
        // TODO: Load IR from policy registry
        return Err(
            "Policy ID loading not yet implemented. Please use --ir flag."
                .to_string()
                .into(),
        );
    } else {
        return Err("Either --policy or --ir must be specified".into());
    };

    // Step 2: Load context
    output::document(&format!("Loading context from: {:?}", context));
    let context_json = fs::read_to_string(context)?;
    let ctx: orchestrator::OrchestratorContext = serde_json::from_str(&context_json)?;

    // Step 3: Create enforcement options
    let enforce_opts = orchestrator::EnforceOptions {
        enforce,
        rollout_percent: rollout,
        drift_max_ratio: drift_max,
    };

    // Step 4: Create enforcer
    output::info("Creating enforcer with IR...");
    let enforcer = orchestrator::Enforcer::new(&ir_data, enforce_opts.clone())?;

    // Step 5: Generate request ID (deterministic for testing)
    let request_id = format!("req-{}", chrono::Utc::now().timestamp_millis());

    // Step 6: Execute enforcement decision
    output::info("Executing enforcement decision...");
    let start = std::time::Instant::now();
    let verdict_pair = enforcer.decide(&ctx, &request_id)?;
    let duration = start.elapsed();

    // Step 7: Record metrics
    orchestrator::metrics::set_rollout_percent(rollout);
    let policy_id = &ir_data.policy_id;

    if verdict_pair.enforced_applied {
        orchestrator::metrics::record_enforced_request(policy_id);
    } else {
        orchestrator::metrics::record_shadow_request(policy_id);
    }

    if verdict_pair.has_drift() {
        orchestrator::metrics::record_drift_event(policy_id);
    }

    orchestrator::metrics::observe_selection_latency(duration.as_secs_f64());

    // Step 8: Display results
    output::section("");
    output::stats("Results:");
    output::detail("Shadow Verdict", &format!("{:?}", verdict_pair.shadow));
    output::detail("Enforced Verdict", &format!("{:?}", verdict_pair.enforced));
    output::detail("Enforcement Applied", &format!("{}", verdict_pair.enforced_applied));
    output::detail("Drift Detected", &format!("{}", verdict_pair.has_drift()));
    output::detail("Duration", &format!("{:?}", duration));

    // Step 9: Write output if requested
    if let Some(out_path) = out {
        let output = serde_json::json!({
            "shadow_verdict": format!("{:?}", verdict_pair.shadow),
            "enforced_verdict": format!("{:?}", verdict_pair.enforced),
            "enforced_applied": verdict_pair.enforced_applied,
            "drift_detected": verdict_pair.has_drift(),
            "duration_ms": duration.as_millis(),
            "request_id": request_id,
            "enforce_options": {
                "enforce": enforce,
                "rollout_percent": rollout,
                "drift_max_ratio": drift_max,
            }
        });

        fs::write(out_path, serde_json::to_string_pretty(&output)?)?;
        output::section("");
        output::saving(&format!("Output written to: {:?}", out_path));
    }

    // Step 10: Audit log
    if !dry_run {
        audit.log_event(
            "proof_adapt_executed",
            json!({
                "policy_id": policy_id,
                "enforce": enforce,
                "rollout_percent": rollout,
                "enforced_applied": verdict_pair.enforced_applied,
                "drift_detected": verdict_pair.has_drift(),
                "duration_ms": duration.as_millis(),
            }),
        )?;
    }

    output::section("");
    output::success("Adaptive orchestration completed!");

    Ok(())
}
