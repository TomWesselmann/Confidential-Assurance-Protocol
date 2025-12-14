//! File Validation - Hash validation with TOCTOU mitigation
//!
//! Provides secure file loading with:
//! - Load-Once-Pattern (TOCTOU mitigation)
//! - File size limits (DoS prevention)
//! - Hash validation

use crate::bundle::meta::BundleMeta;
use crate::crypto::{hex_lower_prefixed32, sha3_256};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;

/// 🔒 FILE SIZE LIMIT: 100 MB (DoS Prevention)
pub const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;

/// Validates file hash with Load-Once-Pattern (TOCTOU mitigation)
///
/// 1. Reads file into memory (single operation)
/// 2. Validates file size (DoS prevention)
/// 3. Computes SHA3-256 hash of memory bytes
/// 4. Returns memory bytes if hash matches
///
/// # Security
/// - TOCTOU mitigation: File read only once
/// - DoS prevention: File size limit (100 MB)
/// - Memory-safe: Hash computed on `Vec<u8>`
pub fn validate_file_hash(file_path: &Path, expected_hash: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    // 1. Load file into memory (single read)
    let bytes = fs::read(file_path)?;

    // 2. Check file size limit (DoS prevention)
    if bytes.len() as u64 > MAX_FILE_SIZE {
        return Err(format!(
            "File size exceeds limit: {} > {} MB",
            bytes.len() / 1024 / 1024,
            MAX_FILE_SIZE / 1024 / 1024
        )
        .into());
    }

    // 3. Compute hash from memory (not from disk!)
    let computed_hash = hex_lower_prefixed32(sha3_256(&bytes));

    // 4. Validate hash
    if computed_hash != expected_hash {
        return Err(format!(
            "Hash mismatch for {}: expected {}, got {}",
            file_path.display(),
            expected_hash,
            computed_hash
        )
        .into());
    }

    // 5. Return memory bytes (Load-Once-Pattern)
    Ok(bytes)
}

/// Loads and validates all files from bundle metadata
///
/// Returns HashMap: filename → validated bytes
pub fn load_and_validate_bundle(
    meta: &BundleMeta,
    bundle_dir: &Path,
) -> Result<HashMap<String, Vec<u8>>, Box<dyn Error>> {
    let mut validated_files = HashMap::new();

    for (filename, file_meta) in &meta.files {
        // Skip optional files that don't exist
        if file_meta.optional {
            let file_path = bundle_dir.join(filename);
            if !file_path.exists() {
                println!("   ⊘ {} (optional, nicht vorhanden)", filename);
                continue;
            }
        }

        // Validate and load file
        let file_path = bundle_dir.join(filename);
        println!("   🔐 Validiere Hash: {}", filename);
        let bytes = validate_file_hash(&file_path, &file_meta.hash)?;
        validated_files.insert(filename.clone(), bytes);
    }

    Ok(validated_files)
}
