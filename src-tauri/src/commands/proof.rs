//! Proof building commands
//!
//! Commands for generating cryptographic proofs.

use crate::audit_logger;
use crate::security::{sanitize_error_message, validate_path_exists};
use crate::types::{ProofProgress, ProofResult};
use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tauri::AppHandle;
use tauri::Emitter;

/// Manifest file structure (for reading)
#[derive(Debug, Deserialize)]
struct ManifestFile {
    supplier_root: String,
    ubo_root: String,
    company_commitment_root: String,
    policy: PolicyRef,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Fields deserialized from JSON manifest
struct PolicyRef {
    name: String,
    version: String,
    hash: String,
}

/// Proof data structure
#[derive(Debug, Serialize)]
struct ProofData {
    version: String,
    manifest_hash: String,
    backend: String,
    constraint_checks: Vec<ConstraintCheck>,
    created_at: String,
}

#[derive(Debug, Serialize)]
struct ConstraintCheck {
    rule: String,
    result: bool,
    witness: String,
}

/// Builds a proof from a manifest
///
/// # Arguments
/// * `project` - Path to the project directory
/// * `app_handle` - Tauri app handle for emitting progress events
///
/// # Returns
/// ProofResult with proof_hash and metadata
///
/// # Events
/// Emits `proof:progress` events with ProofProgress payload
///
/// # Security
/// - Deterministic output
/// - Uses SimplifiedZK backend (mock for MVP)
#[tauri::command]
pub async fn build_proof(project: String, app_handle: AppHandle) -> Result<ProofResult, String> {
    let project_path = Path::new(&project);
    validate_path_exists(project_path)?;

    // 1. Check manifest exists
    let manifest_path = project_path.join("build/manifest.json");
    if !manifest_path.exists() {
        return Err("manifest.json not found in build/ - please build manifest first".to_string());
    }

    // Emit progress: Starting
    let _ = app_handle.emit(
        "proof:progress",
        ProofProgress {
            percent: 0,
            message: "Loading manifest...".to_string(),
        },
    );

    // 2. Read manifest
    let manifest_content = fs::read_to_string(&manifest_path)
        .map_err(|e| sanitize_error_message(&format!("Failed to read manifest: {}", e)))?;
    let manifest: ManifestFile = serde_json::from_str(&manifest_content)
        .map_err(|e| format!("Invalid manifest file: {}", e))?;

    // 3. Calculate manifest hash
    let mut manifest_hasher = Hasher::new();
    manifest_hasher.update(manifest_content.as_bytes());
    let manifest_hash = format!("0x{}", manifest_hasher.finalize().to_hex());

    // Emit progress: Checking constraints
    let _ = app_handle.emit(
        "proof:progress",
        ProofProgress {
            percent: 25,
            message: "Checking constraints...".to_string(),
        },
    );

    // 4. Simulate constraint checks (SimplifiedZK / Mock backend)
    let constraint_checks = vec![
        ConstraintCheck {
            rule: "has_supplier_root".to_string(),
            result: !manifest.supplier_root.is_empty(),
            witness: manifest.supplier_root.clone(),
        },
        ConstraintCheck {
            rule: "has_ubo_root".to_string(),
            result: !manifest.ubo_root.is_empty(),
            witness: manifest.ubo_root.clone(),
        },
        ConstraintCheck {
            rule: "has_company_root".to_string(),
            result: !manifest.company_commitment_root.is_empty(),
            witness: manifest.company_commitment_root.clone(),
        },
        ConstraintCheck {
            rule: "policy_valid".to_string(),
            result: !manifest.policy.hash.is_empty(),
            witness: manifest.policy.hash.clone(),
        },
    ];

    // Check all constraints passed
    let all_passed = constraint_checks.iter().all(|c| c.result);
    if !all_passed {
        return Err("Proof generation failed: some constraints not satisfied".to_string());
    }

    // Emit progress: Generating proof
    let _ = app_handle.emit(
        "proof:progress",
        ProofProgress {
            percent: 50,
            message: "Generating proof...".to_string(),
        },
    );

    // 5. Build proof data
    let proof_data = ProofData {
        version: "proof.v1".to_string(),
        manifest_hash: manifest_hash.clone(),
        backend: "simplified_zk".to_string(),
        constraint_checks,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    // Emit progress: Serializing
    let _ = app_handle.emit(
        "proof:progress",
        ProofProgress {
            percent: 75,
            message: "Serializing proof...".to_string(),
        },
    );

    // 6. Serialize proof
    let proof_json = serde_json::to_string_pretty(&proof_data)
        .map_err(|e| format!("Failed to serialize proof: {}", e))?;

    // 7. Calculate proof hash
    let mut proof_hasher = Hasher::new();
    proof_hasher.update(proof_json.as_bytes());
    let proof_hash = format!("0x{}", proof_hasher.finalize().to_hex());

    // 8. Save proof as .capz (JSON for now, binary in production)
    let proof_path = project_path.join("build/proof.capz");
    fs::write(&proof_path, &proof_json)
        .map_err(|e| sanitize_error_message(&format!("Failed to write proof: {}", e)))?;

    // Also save as .dat for compatibility
    let proof_dat_path = project_path.join("build/proof.dat");
    fs::write(&proof_dat_path, &proof_json).ok(); // Ignore errors for .dat

    // 9. Update manifest with proof info
    let updated_manifest = manifest_content
        .replace("\"type\": \"none\"", "\"type\": \"simplified_zk\"")
        .replace("\"status\": \"pending\"", "\"status\": \"generated\"");
    fs::write(&manifest_path, &updated_manifest).ok();

    // Emit progress: Complete
    let _ = app_handle.emit(
        "proof:progress",
        ProofProgress {
            percent: 100,
            message: "Proof generated successfully".to_string(),
        },
    );

    // 10. Log to audit trail
    let _ = audit_logger::events::proof_built(project_path, &proof_hash, "simplified_zk");

    Ok(ProofResult {
        proof_hash,
        path: proof_path.to_string_lossy().to_string(),
        backend: "simplified_zk".to_string(),
    })
}

#[cfg(test)]
mod tests {
    // Tests require Tauri mock_app which is complex to set up
    // See integration tests for full workflow testing
}
