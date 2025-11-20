//! Upload API - Proof Package Upload Handler
//!
//! Provides REST API for uploading proof bundles as ZIP files
//! and extracting manifest.json and proof.dat

use anyhow::{anyhow, Result};
use axum::{
    extract::Multipart,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Serialize;
use std::io::Read;
use zip::ZipArchive;

// ============================================================================
// API Response Types
// ============================================================================

/// POST /proof/upload - Response Body
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    /// Manifest JSON (parsed)
    pub manifest: serde_json::Value,

    /// Proof data (base64-encoded)
    pub proof_base64: String,

    /// Company commitment root (extracted from manifest)
    pub company_commitment_root: String,

    /// Package info
    pub package_info: PackageInfo,
}

/// Package metadata
#[derive(Debug, Serialize)]
pub struct PackageInfo {
    /// ZIP file size in bytes
    pub size_bytes: usize,

    /// Number of files in ZIP
    pub file_count: usize,

    /// Files found in ZIP
    pub files: Vec<String>,
}

// ============================================================================
// Handler Logic
// ============================================================================

/// Handles POST /proof/upload request
///
/// Accepts multipart/form-data with ZIP file upload
/// Extracts manifest.json and proof.dat from the ZIP
/// Returns parsed data for WebUI to use directly
pub async fn handle_upload(mut multipart: Multipart) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Extract ZIP file from multipart form
    let zip_bytes = match extract_zip_from_multipart(&mut multipart).await {
        Ok(bytes) => bytes,
        Err(e) => return Err((StatusCode::BAD_REQUEST, format!("Failed to extract ZIP: {}", e))),
    };

    // Parse ZIP archive
    let (manifest, proof_base64, package_info) = match parse_proof_package(&zip_bytes) {
        Ok(data) => data,
        Err(e) => return Err((StatusCode::BAD_REQUEST, format!("Failed to parse ZIP: {}", e))),
    };

    // Extract company_commitment_root from manifest
    let company_commitment_root = manifest
        .get("company_commitment_root")
        .and_then(|v| v.as_str())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing company_commitment_root in manifest".to_string()))?
        .to_string();

    // Build response
    let response = UploadResponse {
        manifest,
        proof_base64,
        company_commitment_root,
        package_info,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Extracts ZIP file bytes from multipart form data
async fn extract_zip_from_multipart(multipart: &mut Multipart) -> Result<Vec<u8>> {
    while let Some(field) = multipart.next_field().await.map_err(|e| anyhow!("Multipart error: {}", e))? {
        let name = field.name().unwrap_or("");

        // Look for field named "file" or "proof_package"
        if name == "file" || name == "proof_package" || name == "zip" {
            let data = field.bytes().await.map_err(|e| anyhow!("Failed to read field: {}", e))?;
            return Ok(data.to_vec());
        }
    }

    Err(anyhow!("No file field found in multipart form data"))
}

/// Parses proof package ZIP and extracts manifest.json and proof.dat
fn parse_proof_package(zip_bytes: &[u8]) -> Result<(serde_json::Value, String, PackageInfo)> {
    // Create ZIP archive reader
    let cursor = std::io::Cursor::new(zip_bytes);
    let mut archive = ZipArchive::new(cursor)
        .map_err(|e| anyhow!("Invalid ZIP file: {}", e))?;

    let file_count = archive.len();
    let mut files = Vec::new();
    let mut manifest_json: Option<serde_json::Value> = None;
    let mut proof_base64: Option<String> = None;

    // Iterate through all files in ZIP
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| anyhow!("Failed to read ZIP entry {}: {}", i, e))?;

        let file_name = file.name().to_string();
        files.push(file_name.clone());

        // Extract manifest.json
        if file_name.ends_with("manifest.json") {
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .map_err(|e| anyhow!("Failed to read manifest.json: {}", e))?;

            manifest_json = Some(serde_json::from_str(&contents)
                .map_err(|e| anyhow!("Invalid manifest.json: {}", e))?);
        }

        // Extract proof.dat
        if file_name.ends_with("proof.dat") {
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)
                .map_err(|e| anyhow!("Failed to read proof.dat: {}", e))?;

            // Encode as base64
            proof_base64 = Some(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &contents));
        }
    }

    // Validate that we found both required files
    let manifest = manifest_json.ok_or_else(|| anyhow!("manifest.json not found in ZIP"))?;
    let proof = proof_base64.ok_or_else(|| anyhow!("proof.dat not found in ZIP"))?;

    let package_info = PackageInfo {
        size_bytes: zip_bytes.len(),
        file_count,
        files,
    };

    Ok((manifest, proof, package_info))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_proof_package_missing_files() {
        // Create empty ZIP
        let cursor = std::io::Cursor::new(Vec::new());
        let mut zip = zip::ZipWriter::new(cursor);
        let cursor = zip.finish().unwrap();
        let empty_zip = cursor.into_inner();

        // Should fail with missing files
        let result = parse_proof_package(&empty_zip);
        assert!(result.is_err());
    }
}
