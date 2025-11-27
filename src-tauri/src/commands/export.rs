//! Bundle export commands
//!
//! Commands for exporting proof bundles as ZIP files with cap-bundle.v1 format.

use crate::audit_logger;
use crate::security::{sanitize_error_message, validate_path_exists};
use crate::types::ExportResult;
use blake3::Hasher;
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
#[allow(unused_imports)]
use std::io::Read;
use std::path::Path;
use uuid::Uuid;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

/// Calculate SHA3-256 hash of content (0x-prefixed hex)
fn sha3_hash(content: &[u8]) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(content);
    let result = hasher.finalize();
    format!("0x{}", hex::encode(result))
}

/// File metadata for _meta.json
#[derive(serde::Serialize)]
struct BundleFileMeta {
    role: String,
    hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<u64>,
}

/// Proof unit metadata for _meta.json
#[derive(serde::Serialize)]
struct ProofUnitMeta {
    id: String,
    manifest_file: String,
    proof_file: String,
    policy_id: String,
    policy_hash: String,
    backend: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    depends_on: Vec<String>,
}

/// Bundle metadata structure (cap-bundle.v1)
#[derive(serde::Serialize)]
struct BundleMeta {
    schema: String,
    bundle_id: String,
    created_at: String,
    files: HashMap<String, BundleFileMeta>,
    proof_units: Vec<ProofUnitMeta>,
}

/// Exports a proof bundle as a ZIP file with cap-bundle.v1 format
///
/// # Arguments
/// * `project` - Path to the project directory
/// * `output` - Path for the output ZIP file
///
/// # Returns
/// ExportResult with bundle_path, size, hash, and file list
///
/// # Security
/// - Uses fixed filenames (no user input in ZIP)
/// - Validates all source files exist
#[tauri::command]
pub async fn export_bundle(project: String, output: String) -> Result<ExportResult, String> {
    let project_path = Path::new(&project);
    let output_path = Path::new(&output);

    validate_path_exists(project_path)?;

    // 1. Check required files exist
    let manifest_path = project_path.join("build/manifest.json");
    let proof_path = project_path.join("build/proof.capz");
    let proof_dat_path = project_path.join("build/proof.dat");

    if !manifest_path.exists() {
        return Err("manifest.json not found - please build manifest first".to_string());
    }

    let actual_proof_path = if proof_path.exists() {
        proof_path
    } else if proof_dat_path.exists() {
        proof_dat_path
    } else {
        return Err("proof.capz/proof.dat not found - please build proof first".to_string());
    };

    // 2. Read and hash all files
    let manifest_content = fs::read(&manifest_path)
        .map_err(|e| sanitize_error_message(&format!("Failed to read manifest: {}", e)))?;
    let manifest_hash = sha3_hash(&manifest_content);

    let proof_content = fs::read(&actual_proof_path)
        .map_err(|e| sanitize_error_message(&format!("Failed to read proof: {}", e)))?;
    let proof_hash = sha3_hash(&proof_content);

    let proof_filename = if actual_proof_path.extension().map(|e| e == "capz").unwrap_or(false) {
        "proof.capz"
    } else {
        "proof.dat"
    };

    // 3. Parse manifest.json to extract policy_hash and backend
    let manifest_json: serde_json::Value = serde_json::from_slice(&manifest_content)
        .map_err(|e| format!("Failed to parse manifest.json: {}", e))?;

    let policy_hash = manifest_json.get("policy_hash")
        .and_then(|v| v.as_str())
        .unwrap_or("0x0")
        .to_string();

    let backend = manifest_json.get("backend")
        .and_then(|v| v.as_str())
        .unwrap_or("mock")
        .to_string();

    // 4. Build files map for _meta.json
    let mut files_map: HashMap<String, BundleFileMeta> = HashMap::new();

    files_map.insert("manifest.json".to_string(), BundleFileMeta {
        role: "manifest".to_string(),
        hash: manifest_hash.clone(),
        size: Some(manifest_content.len() as u64),
    });

    files_map.insert(proof_filename.to_string(), BundleFileMeta {
        role: "proof".to_string(),
        hash: proof_hash.clone(),
        size: Some(proof_content.len() as u64),
    });

    // 5. Create bundle metadata
    let bundle_id = Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339();

    let bundle_meta = BundleMeta {
        schema: "cap-bundle.v1".to_string(),
        bundle_id: bundle_id.clone(),
        created_at,
        files: files_map,
        proof_units: vec![ProofUnitMeta {
            id: "main".to_string(),
            manifest_file: "manifest.json".to_string(),
            proof_file: proof_filename.to_string(),
            policy_id: "lksg.demo.v1".to_string(),
            policy_hash,
            backend,
            depends_on: vec![],
        }],
    };

    let meta_json = serde_json::to_string_pretty(&bundle_meta)
        .map_err(|e| format!("Failed to serialize _meta.json: {}", e))?;

    // 6. Ensure output directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| sanitize_error_message(&format!("Failed to create output directory: {}", e)))?;
    }

    // 7. Create ZIP file
    let zip_file = File::create(output_path)
        .map_err(|e| sanitize_error_message(&format!("Failed to create ZIP file: {}", e)))?;

    let mut zip = ZipWriter::new(zip_file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    let mut files_added = Vec::new();
    let mut total_hasher = Hasher::new();

    // 8. Add _meta.json FIRST (required by verifier)
    total_hasher.update(meta_json.as_bytes());
    zip.start_file("_meta.json", options)
        .map_err(|e| format!("Failed to add _meta.json to ZIP: {}", e))?;
    zip.write_all(meta_json.as_bytes())
        .map_err(|e| format!("Failed to write _meta.json to ZIP: {}", e))?;
    files_added.push("_meta.json".to_string());

    // 9. Add manifest.json
    total_hasher.update(&manifest_content);
    zip.start_file("manifest.json", options)
        .map_err(|e| format!("Failed to add manifest to ZIP: {}", e))?;
    zip.write_all(&manifest_content)
        .map_err(|e| format!("Failed to write manifest to ZIP: {}", e))?;
    files_added.push("manifest.json".to_string());

    // 10. Add proof file
    total_hasher.update(&proof_content);
    zip.start_file(proof_filename, options)
        .map_err(|e| format!("Failed to add proof to ZIP: {}", e))?;
    zip.write_all(&proof_content)
        .map_err(|e| format!("Failed to write proof to ZIP: {}", e))?;
    files_added.push(proof_filename.to_string());

    // 11. Optionally add commitments.json
    let commitments_path = project_path.join("build/commitments.json");
    if commitments_path.exists() {
        if let Ok(content) = fs::read(&commitments_path) {
            total_hasher.update(&content);
            if zip.start_file("commitments.json", options).is_ok() {
                let _ = zip.write_all(&content);
                files_added.push("commitments.json".to_string());
            }
        }
    }

    // 12. Optionally add policy.yml
    let policy_path = project_path.join("input/policy.yml");
    if policy_path.exists() {
        if let Ok(content) = fs::read(&policy_path) {
            total_hasher.update(&content);
            if zip.start_file("policy.yml", options).is_ok() {
                let _ = zip.write_all(&content);
                files_added.push("policy.yml".to_string());
            }
        }
    }

    // 13. Finalize ZIP
    zip.finish()
        .map_err(|e| format!("Failed to finalize ZIP: {}", e))?;

    // 14. Calculate bundle hash
    let bundle_hash = format!("0x{}", total_hasher.finalize().to_hex());

    // 15. Get file size
    let metadata = fs::metadata(output_path)
        .map_err(|e| sanitize_error_message(&format!("Failed to get ZIP metadata: {}", e)))?;

    // 16. Copy to project export/ folder
    let export_dir = project_path.join("export");
    fs::create_dir_all(&export_dir).ok();
    let timestamp = chrono::Utc::now().format("%Y-%m-%d_%H%M%S");
    let export_copy = export_dir.join(format!("cap-bundle-{}.zip", timestamp));
    fs::copy(output_path, &export_copy).ok();

    // 17. Log to audit trail
    let _ = audit_logger::events::bundle_exported(
        project_path,
        &output_path.to_string_lossy(),
        &bundle_hash,
        metadata.len(),
    );

    Ok(ExportResult {
        bundle_path: output_path.to_string_lossy().to_string(),
        size_bytes: metadata.len(),
        hash: bundle_hash,
        files: files_added,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_export_bundle_missing_manifest() {
        let temp = TempDir::new().unwrap();
        let project = temp.path().to_string_lossy().to_string();
        let output = temp.path().join("bundle.zip").to_string_lossy().to_string();

        // Create empty project structure
        fs::create_dir_all(temp.path().join("build")).unwrap();

        let result = export_bundle(project, output).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("manifest.json not found"));
    }
}
