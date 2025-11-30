//! Signing commands for key generation and manifest signing
//!
//! This module provides Tauri commands for:
//! - Generating Ed25519 key pairs
//! - Listing available keys in a project
//! - Signing manifests
//! - Verifying manifest signatures

use crate::audit_logger;
use crate::security::{sanitize_error_message, validate_path_exists};
use crate::types::{KeyInfo, SignResult, SignatureVerifyResult};
use cap_agent::manifest::Manifest;
use cap_agent::sign::{generate_keypair, load_private_key, load_public_key, sign_manifest, verify_manifest};
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

// ============================================================================
// Validation Helpers
// ============================================================================

/// Validates signer name for security (no path traversal)
fn validate_signer_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Signer name is required".to_string());
    }
    if name.len() > 64 {
        return Err("Signer name too long (max 64 characters)".to_string());
    }
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err("Invalid signer name: path traversal not allowed".to_string());
    }
    // Only alphanumeric, underscore, hyphen, space allowed
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == ' ') {
        return Err("Invalid signer name: only alphanumeric, underscore, hyphen, and space allowed".to_string());
    }
    Ok(())
}

/// Computes SHA-256 fingerprint of a public key
fn compute_fingerprint(public_key_bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(public_key_bytes);
    let result = hasher.finalize();
    format!("sha256:{}", hex::encode(&result[..8]))
}

/// Computes KID from public key (first 16 bytes of BLAKE3 hash)
fn compute_kid(public_key_bytes: &[u8]) -> String {
    let hash = blake3::hash(public_key_bytes);
    hex::encode(&hash.as_bytes()[..16])
}

// ============================================================================
// Commands
// ============================================================================

/// Generates a new Ed25519 key pair for signing
///
/// # Arguments
/// * `project` - Path to the project directory
/// * `signer_name` - Name of the signer (e.g., "Company Name")
///
/// # Returns
/// KeyInfo with key details
///
/// # Security
/// - Validates signer name for path traversal
/// - Keys stored in project/keys/ directory
/// - Private key never leaves the local system
#[tauri::command]
pub async fn generate_keys(project: String, signer_name: String) -> Result<KeyInfo, String> {
    // 1. Validate inputs
    let project_path = Path::new(&project);
    validate_path_exists(project_path)?;
    validate_signer_name(&signer_name)?;

    // 2. Create keys directory
    let keys_dir = project_path.join("keys");
    if !keys_dir.exists() {
        fs::create_dir_all(&keys_dir)
            .map_err(|e| sanitize_error_message(&format!("Failed to create keys directory: {}", e)))?;
    }

    // 3. Generate file paths (sanitized signer name for filename)
    let safe_name = signer_name.replace(' ', "_").to_lowercase();
    let private_key_path = keys_dir.join(format!("{}.private.key", safe_name));
    let public_key_path = keys_dir.join(format!("{}.public.key", safe_name));

    // 4. Check if key already exists
    if private_key_path.exists() || public_key_path.exists() {
        return Err(format!("Key for signer '{}' already exists. Delete existing keys first.", signer_name));
    }

    // 5. Generate key pair using CAP-Agent library
    generate_keypair(&private_key_path, &public_key_path)
        .map_err(|e| sanitize_error_message(&format!("Failed to generate key pair: {}", e)))?;

    // 6. Read public key for fingerprint/KID calculation
    let public_key_bytes = fs::read(&public_key_path)
        .map_err(|e| sanitize_error_message(&format!("Failed to read public key: {}", e)))?;

    let fingerprint = compute_fingerprint(&public_key_bytes);
    let kid = compute_kid(&public_key_bytes);
    let created_at = Utc::now().to_rfc3339();

    // 7. Log to audit
    let _ = audit_logger::events::keys_generated(
        project_path,
        &signer_name,
        &kid,
        &fingerprint,
    );

    Ok(KeyInfo {
        kid,
        signer_name,
        public_key_path: public_key_path.to_string_lossy().to_string(),
        fingerprint,
        created_at,
    })
}

/// Lists all available keys in a project
///
/// # Arguments
/// * `project` - Path to the project directory
///
/// # Returns
/// Vector of KeyInfo for each key found
#[tauri::command]
pub async fn list_keys(project: String) -> Result<Vec<KeyInfo>, String> {
    let project_path = Path::new(&project);
    validate_path_exists(project_path)?;

    let keys_dir = project_path.join("keys");
    if !keys_dir.exists() {
        return Ok(Vec::new());
    }

    let mut keys = Vec::new();

    let entries = fs::read_dir(&keys_dir)
        .map_err(|e| sanitize_error_message(&format!("Failed to read keys directory: {}", e)))?;

    for entry in entries.flatten() {
        let path = entry.path();

        // Only process .public.key files
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.ends_with(".public.key") {
                // Extract signer name from filename
                let signer_name = name.trim_end_matches(".public.key").replace('_', " ");

                // Read public key for fingerprint
                if let Ok(public_key_bytes) = fs::read(&path) {
                    let fingerprint = compute_fingerprint(&public_key_bytes);
                    let kid = compute_kid(&public_key_bytes);

                    // Get file creation time
                    let created_at = fs::metadata(&path)
                        .ok()
                        .and_then(|m| m.created().ok())
                        .map(|t| chrono::DateTime::<Utc>::from(t).to_rfc3339())
                        .unwrap_or_else(|| "unknown".to_string());

                    keys.push(KeyInfo {
                        kid,
                        signer_name,
                        public_key_path: path.to_string_lossy().to_string(),
                        fingerprint,
                        created_at,
                    });
                }
            }
        }
    }

    // Sort by creation date (newest first)
    keys.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(keys)
}

/// Signs the manifest with the specified key
///
/// # Arguments
/// * `project` - Path to the project directory
/// * `signer_name` - Name of the signer (must match existing key)
///
/// # Returns
/// SignResult with signature details
///
/// # Security
/// - Validates manifest exists
/// - Validates key exists
/// - Updates manifest in place with signature
#[tauri::command]
pub async fn sign_project_manifest(project: String, signer_name: String) -> Result<SignResult, String> {
    let project_path = Path::new(&project);
    validate_path_exists(project_path)?;
    validate_signer_name(&signer_name)?;

    // 1. Check manifest exists
    let manifest_path = project_path.join("build/manifest.json");
    if !manifest_path.exists() {
        return Err("No manifest found. Build manifest first (Step 4).".to_string());
    }

    // 2. Find key files
    let safe_name = signer_name.replace(' ', "_").to_lowercase();
    let private_key_path = project_path.join("keys").join(format!("{}.private.key", safe_name));

    if !private_key_path.exists() {
        return Err(format!("Key for signer '{}' not found. Generate keys first.", signer_name));
    }

    // 3. Load manifest
    let manifest_content = fs::read_to_string(&manifest_path)
        .map_err(|e| sanitize_error_message(&format!("Failed to read manifest: {}", e)))?;

    let manifest: Manifest = serde_json::from_str(&manifest_content)
        .map_err(|e| format!("Failed to parse manifest: {}", e))?;

    // 4. Load private key
    let signing_key = load_private_key(&private_key_path)
        .map_err(|e| sanitize_error_message(&format!("Failed to load private key: {}", e)))?;

    // 5. Sign manifest
    let signed_manifest = sign_manifest(&manifest, &signing_key, &signer_name)
        .map_err(|e| sanitize_error_message(&format!("Failed to sign manifest: {}", e)))?;

    // 6. Update manifest with signature
    let mut updated_manifest = signed_manifest.manifest.clone();
    updated_manifest.signatures.push(signed_manifest.signature.clone());

    // 7. Save updated manifest
    let updated_json = serde_json::to_string_pretty(&updated_manifest)
        .map_err(|e| format!("Failed to serialize manifest: {}", e))?;

    fs::write(&manifest_path, &updated_json)
        .map_err(|e| sanitize_error_message(&format!("Failed to write manifest: {}", e)))?;

    // 8. Extract signature hash for display
    let sig_hex = &signed_manifest.signature.sig_hex;
    let signature_hash = sig_hex.trim_start_matches("0x").chars().take(16).collect::<String>();

    // 9. Log to audit
    let _ = audit_logger::events::manifest_signed(
        project_path,
        &signer_name,
        &signature_hash,
    );

    Ok(SignResult {
        success: true,
        signer: signer_name,
        signature_hash: format!("0x{}", signature_hash),
        signature_hex: sig_hex.clone(),
        manifest_path: manifest_path.to_string_lossy().to_string(),
    })
}

/// Verifies the signature on the manifest
///
/// # Arguments
/// * `project` - Path to the project directory
///
/// # Returns
/// SignatureVerifyResult with verification status
#[tauri::command]
pub async fn verify_manifest_signature(project: String) -> Result<SignatureVerifyResult, String> {
    let project_path = Path::new(&project);
    validate_path_exists(project_path)?;

    // 1. Check manifest exists
    let manifest_path = project_path.join("build/manifest.json");
    if !manifest_path.exists() {
        return Err("No manifest found.".to_string());
    }

    // 2. Load manifest
    let manifest_content = fs::read_to_string(&manifest_path)
        .map_err(|e| sanitize_error_message(&format!("Failed to read manifest: {}", e)))?;

    let manifest: Manifest = serde_json::from_str(&manifest_content)
        .map_err(|e| format!("Failed to parse manifest: {}", e))?;

    // 3. Check if manifest has signatures
    if manifest.signatures.is_empty() {
        return Ok(SignatureVerifyResult {
            valid: false,
            signer: String::new(),
            algorithm: String::new(),
            error: Some("Manifest has no signatures".to_string()),
        });
    }

    // 4. Verify first signature (for now, single-signer workflow)
    let signature = &manifest.signatures[0];
    let signer = signature.signer.clone();
    let algorithm = signature.alg.clone();

    // 5. Find matching public key
    let safe_name = signer.replace(' ', "_").to_lowercase();
    let public_key_path = project_path.join("keys").join(format!("{}.public.key", safe_name));

    if !public_key_path.exists() {
        return Ok(SignatureVerifyResult {
            valid: false,
            signer,
            algorithm,
            error: Some("Public key not found for signer".to_string()),
        });
    }

    // 6. Load public key
    let verifying_key = load_public_key(&public_key_path)
        .map_err(|e| sanitize_error_message(&format!("Failed to load public key: {}", e)))?;

    // 7. Create SignedManifest for verification
    // We need to verify against the manifest WITHOUT the signature we're checking
    let mut manifest_for_verify = manifest.clone();
    manifest_for_verify.signatures.clear();

    let signed_manifest = cap_agent::manifest::SignedManifest {
        manifest: manifest_for_verify,
        signature: signature.clone(),
    };

    // 8. Verify
    match verify_manifest(&signed_manifest, &verifying_key) {
        Ok(()) => {
            // Log successful verification
            let _ = audit_logger::events::signature_verified(
                project_path,
                &signer,
                true,
            );

            Ok(SignatureVerifyResult {
                valid: true,
                signer,
                algorithm,
                error: None,
            })
        }
        Err(e) => {
            // Log failed verification
            let _ = audit_logger::events::signature_verified(
                project_path,
                &signer,
                false,
            );

            Ok(SignatureVerifyResult {
                valid: false,
                signer,
                algorithm,
                error: Some(e.to_string()),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_validate_signer_name_valid() {
        assert!(validate_signer_name("Company Name").is_ok());
        assert!(validate_signer_name("test-signer").is_ok());
        assert!(validate_signer_name("test_signer").is_ok());
        assert!(validate_signer_name("Test123").is_ok());
    }

    #[test]
    fn test_validate_signer_name_invalid() {
        assert!(validate_signer_name("").is_err());
        assert!(validate_signer_name("../traversal").is_err());
        assert!(validate_signer_name("path/traversal").is_err());
        assert!(validate_signer_name("path\\traversal").is_err());
        assert!(validate_signer_name("invalid<>chars").is_err());
    }

    #[test]
    fn test_compute_fingerprint() {
        let key_bytes = [0u8; 32];
        let fingerprint = compute_fingerprint(&key_bytes);
        assert!(fingerprint.starts_with("sha256:"));
        assert_eq!(fingerprint.len(), 7 + 16); // "sha256:" + 16 hex chars
    }

    #[test]
    fn test_compute_kid() {
        let key_bytes = [0u8; 32];
        let kid = compute_kid(&key_bytes);
        assert_eq!(kid.len(), 32); // 16 bytes = 32 hex chars
    }

    #[tokio::test]
    async fn test_generate_keys_success() {
        let temp = TempDir::new().unwrap();
        let project = temp.path().to_string_lossy().to_string();

        // Create audit directory (required by audit_logger)
        fs::create_dir_all(temp.path().join("audit")).unwrap();

        let result = generate_keys(project.clone(), "Test Signer".to_string()).await;

        assert!(result.is_ok());
        let key_info = result.unwrap();
        assert_eq!(key_info.signer_name, "Test Signer");
        assert!(!key_info.kid.is_empty());
        assert!(key_info.fingerprint.starts_with("sha256:"));

        // Check files exist
        assert!(temp.path().join("keys/test_signer.private.key").exists());
        assert!(temp.path().join("keys/test_signer.public.key").exists());
    }

    #[tokio::test]
    async fn test_generate_keys_already_exists() {
        let temp = TempDir::new().unwrap();
        let project = temp.path().to_string_lossy().to_string();
        fs::create_dir_all(temp.path().join("audit")).unwrap();

        // Generate first time
        generate_keys(project.clone(), "Test".to_string()).await.unwrap();

        // Try to generate again - should fail
        let result = generate_keys(project, "Test".to_string()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));
    }

    #[tokio::test]
    async fn test_list_keys_empty() {
        let temp = TempDir::new().unwrap();
        let project = temp.path().to_string_lossy().to_string();

        let result = list_keys(project).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
