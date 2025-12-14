//! Bundle verification commands
//!
//! Commands for verifying proof bundles (existing functionality, refactored).

use crate::security::sanitize_error_message;
use crate::types::{BundleInfo, ProofUnitInfo, VerifyBundleRequest, VerifyBundleResponse};
use cap_agent::bundle::{parse_bundle_source, BundleSource};
use cap_agent::verifier::{verify_from_source, VerifyOptions};
use std::path::Path;

/// Verifies a proof bundle from local file system
///
/// # Security (REQ-13)
/// - Path traversal validation
/// - Zip bomb protection (via bundle::load_bundle_atomic)
/// - TOCTOU prevention (atomic loading)
/// - Error message sanitization
///
/// # Arguments
/// * `request` - Bundle path and verification options
///
/// # Returns
/// Verification report with detailed results
#[tauri::command]
pub async fn verify_bundle(request: VerifyBundleRequest) -> Result<VerifyBundleResponse, String> {
    // 1. Security: Path validation
    let bundle_path = Path::new(&request.bundle_path);
    if !bundle_path.exists() {
        return Err("Bundle not found".to_string());
    }

    // 2. Parse bundle source (auto-detect ZIP vs Directory)
    let source = BundleSource::from_path(bundle_path)
        .map_err(|e| sanitize_error_message(&e.to_string()))?;

    // 3. Parse bundle metadata first (for bundle_id)
    let meta = parse_bundle_source(&source)
        .map_err(|e| sanitize_error_message(&e.to_string()))?;

    // 4. Build verify options (offline defaults: REQ-07)
    let verify_opts = if let Some(opts) = request.options {
        VerifyOptions {
            check_timestamp: opts.check_timestamp,
            check_registry: opts.check_registry,
        }
    } else {
        VerifyOptions::default() // Offline defaults (false, false)
    };

    // 5. Verify bundle (atomic, deterministic: REQ-04)
    let report = verify_from_source(&source, Some(&verify_opts))
        .map_err(|e| sanitize_error_message(&e.to_string()))?;

    // 6. Build response
    Ok(VerifyBundleResponse {
        status: report.status,
        bundle_id: meta.bundle_id,
        manifest_hash: report.manifest_hash,
        proof_hash: report.proof_hash,
        signature_valid: report.signature_valid,
        timestamp_valid: report.timestamp_valid,
        registry_match: report.registry_match,
        details: report.details,
    })
}

/// Gets bundle metadata without verification (for preview)
///
/// # Arguments
/// * `bundle_path` - Path to bundle (ZIP or directory)
///
/// # Returns
/// Bundle info with metadata
#[tauri::command]
pub async fn get_bundle_info(bundle_path: String) -> Result<BundleInfo, String> {
    let source = BundleSource::from_path(&bundle_path)
        .map_err(|e| sanitize_error_message(&e.to_string()))?;

    let meta = parse_bundle_source(&source)
        .map_err(|e| sanitize_error_message(&e.to_string()))?;

    let proof_units = meta
        .proof_units
        .iter()
        .map(|pu| ProofUnitInfo {
            id: pu.id.clone(),
            policy_id: pu.policy_id.clone(),
            backend: pu.backend.clone(),
        })
        .collect();

    Ok(BundleInfo {
        bundle_id: meta.bundle_id,
        schema: meta.schema,
        created_at: meta.created_at,
        proof_units,
        file_count: meta.files.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_verify_bundle_not_found() {
        let request = VerifyBundleRequest {
            bundle_path: "/nonexistent/bundle.zip".to_string(),
            options: None,
        };

        let result = verify_bundle(request).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Bundle not found"));
    }

    #[tokio::test]
    async fn test_get_bundle_info_not_found() {
        let result = get_bundle_info("/nonexistent/bundle.zip".to_string()).await;

        assert!(result.is_err());
    }
}
