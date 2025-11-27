//! Manifest building commands
//!
//! Commands for creating manifest files from commitments and policy.

use crate::audit_logger;
use crate::security::{sanitize_error_message, validate_path_exists};
use crate::types::ManifestResult;
use blake3::Hasher;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Commitments file structure
#[derive(Debug, Deserialize)]
struct CommitmentsFile {
    supplier_root: String,
    ubo_root: String,
    company_root: String,
}

/// Policy file structure (minimal)
#[derive(Debug, Deserialize)]
struct PolicyFile {
    name: String,
    version: String,
}

/// Manifest structure
#[derive(Debug, Serialize)]
struct Manifest {
    version: String,
    created_at: String,
    supplier_root: String,
    ubo_root: String,
    company_commitment_root: String,
    policy: PolicyManifest,
    audit: AuditManifest,
    proof: ProofManifest,
    signatures: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct PolicyManifest {
    name: String,
    version: String,
    hash: String,
}

#[derive(Debug, Serialize)]
struct AuditManifest {
    tail_digest: String,
    events_count: usize,
}

#[derive(Debug, Serialize)]
struct ProofManifest {
    #[serde(rename = "type")]
    proof_type: String,
    status: String,
}

/// Builds a manifest from commitments and policy
///
/// # Arguments
/// * `project` - Path to the project directory
///
/// # Returns
/// ManifestResult with manifest_hash and metadata
///
/// # Security
/// - Deterministic output (same inputs → same hash)
/// - No PII in output
#[tauri::command]
pub async fn build_manifest(project: String) -> Result<ManifestResult, String> {
    let project_path = Path::new(&project);
    validate_path_exists(project_path)?;

    // 1. Check required files exist
    let commitments_path = project_path.join("build/commitments.json");
    let policy_path = project_path.join("input/policy.yml");

    if !commitments_path.exists() {
        return Err(
            "commitments.json not found in build/ - please create commitments first".to_string(),
        );
    }
    if !policy_path.exists() {
        return Err("policy.yml not found in input/ - please load policy first".to_string());
    }

    // 2. Read commitments
    let commitments_content = fs::read_to_string(&commitments_path)
        .map_err(|e| sanitize_error_message(&format!("Failed to read commitments: {}", e)))?;
    let commitments: CommitmentsFile = serde_json::from_str(&commitments_content)
        .map_err(|e| format!("Invalid commitments file: {}", e))?;

    // 3. Read policy
    let policy_content = fs::read_to_string(&policy_path)
        .map_err(|e| sanitize_error_message(&format!("Failed to read policy: {}", e)))?;
    let policy: PolicyFile =
        serde_yaml::from_str(&policy_content).map_err(|e| format!("Invalid policy file: {}", e))?;

    // 4. Calculate policy hash
    let mut policy_hasher = Hasher::new();
    policy_hasher.update(policy_content.as_bytes());
    let policy_hash = format!("0x{}", policy_hasher.finalize().to_hex());

    // 5. Read audit log for tail digest
    let audit_path = project_path.join("audit/agent.audit.jsonl");
    let (tail_digest, events_count) = if audit_path.exists() {
        let audit_content = fs::read_to_string(&audit_path).unwrap_or_default();
        let lines: Vec<&str> = audit_content.lines().filter(|l| !l.is_empty()).collect();
        let count = lines.len();
        if let Some(last_line) = lines.last() {
            let mut hasher = Hasher::new();
            hasher.update(last_line.as_bytes());
            (format!("0x{}", hasher.finalize().to_hex()), count)
        } else {
            ("0x0".to_string(), 0)
        }
    } else {
        ("0x0".to_string(), 0)
    };

    // 6. Build manifest
    let manifest = Manifest {
        version: "manifest.v0".to_string(),
        created_at: Utc::now().to_rfc3339(),
        supplier_root: commitments.supplier_root.clone(),
        ubo_root: commitments.ubo_root.clone(),
        company_commitment_root: commitments.company_root.clone(),
        policy: PolicyManifest {
            name: policy.name,
            version: policy.version,
            hash: policy_hash.clone(),
        },
        audit: AuditManifest {
            tail_digest,
            events_count,
        },
        proof: ProofManifest {
            proof_type: "none".to_string(),
            status: "pending".to_string(),
        },
        signatures: vec![],
    };

    // 7. Serialize to canonical JSON (sorted keys for determinism)
    let manifest_json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("Failed to serialize manifest: {}", e))?;

    // 8. Calculate manifest hash
    let mut manifest_hasher = Hasher::new();
    manifest_hasher.update(manifest_json.as_bytes());
    let manifest_hash = format!("0x{}", manifest_hasher.finalize().to_hex());

    // 9. Save manifest
    let manifest_path = project_path.join("build/manifest.json");
    fs::write(&manifest_path, &manifest_json)
        .map_err(|e| sanitize_error_message(&format!("Failed to write manifest: {}", e)))?;

    // 10. Log to audit trail
    let _ = audit_logger::events::manifest_built(project_path, &manifest_hash);

    Ok(ManifestResult {
        manifest_hash,
        path: manifest_path.to_string_lossy().to_string(),
        supplier_root: commitments.supplier_root,
        ubo_root: commitments.ubo_root,
        policy_hash,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::commitments::create_commitments;
    use crate::commands::import::import_csv;
    use crate::commands::policy::load_policy;
    use crate::commands::project::create_project;
    use crate::types::CsvType;
    use tempfile::TempDir;

    async fn setup_project_for_manifest(temp: &TempDir) -> String {
        let workspace = temp.path().to_string_lossy().to_string();
        let project = create_project(workspace.clone(), "test".to_string())
            .await
            .unwrap();

        // Create CSVs
        let suppliers = temp.path().join("suppliers.csv");
        fs::write(&suppliers, "name,jurisdiction,tier\nAcme,DE,1\n").unwrap();
        let ubos = temp.path().join("ubos.csv");
        fs::write(&ubos, "name,birthdate,citizenship\nMax,1980-01-01,DE\n").unwrap();

        import_csv(
            project.path.clone(),
            CsvType::Suppliers,
            suppliers.to_string_lossy().to_string(),
        )
        .await
        .unwrap();
        import_csv(
            project.path.clone(),
            CsvType::Ubos,
            ubos.to_string_lossy().to_string(),
        )
        .await
        .unwrap();

        create_commitments(project.path.clone()).await.unwrap();

        // Create policy
        let policy = temp.path().join("policy.yml");
        fs::write(&policy, "name: Test Policy\nversion: v1\nrules: []").unwrap();
        load_policy(project.path.clone(), policy.to_string_lossy().to_string())
            .await
            .unwrap();

        project.path
    }

    #[tokio::test]
    async fn test_build_manifest_success() {
        let temp = TempDir::new().unwrap();
        let project_path = setup_project_for_manifest(&temp).await;

        let result = build_manifest(project_path.clone()).await;

        assert!(result.is_ok());
        let manifest = result.unwrap();
        assert!(manifest.manifest_hash.starts_with("0x"));
        assert!(Path::new(&project_path).join("build/manifest.json").exists());
    }

    #[tokio::test]
    async fn test_build_manifest_missing_commitments() {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path().to_string_lossy().to_string();
        let project = create_project(workspace, "test".to_string()).await.unwrap();

        let result = build_manifest(project.path).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("commitments.json not found"));
    }
}
