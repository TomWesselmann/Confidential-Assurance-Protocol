//! Bundle Export - Erstellt CAP Proof-Pakete (cap-bundle.v1)
//!
//! Enthält die Kernlogik zum Exportieren von Proof-Bundles:
//! - Verzeichnisstruktur vorbereiten
//! - Dateien kopieren und hashen
//! - Metadaten (_meta.json) erstellen

use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use uuid::Uuid;

use super::meta::{BundleFileMeta, BundleMeta, ProofUnitMeta, BUNDLE_SCHEMA_V1};
use crate::manifest::Manifest;

/// Struktur für exportierte Dateipfade
pub struct ExportedFiles {
    pub manifest_dst: PathBuf,
    pub proof_dst: PathBuf,
    pub timestamp_dst: Option<PathBuf>,
    pub registry_dst: Option<PathBuf>,
    pub report_dst: PathBuf,
}

/// Ergebnis eines Bundle-Exports
pub struct ExportResult {
    pub output_dir: String,
    pub bundle_id: String,
    pub file_count: usize,
}

/// Bereitet das Output-Verzeichnis vor (erstellt oder löscht mit force)
pub fn prepare_export_dir(output_dir: &str, force: bool) -> Result<(), Box<dyn Error>> {
    let out_path = Path::new(output_dir);

    if out_path.exists() {
        if force {
            fs::remove_dir_all(out_path)?;
        } else {
            return Err(format!(
                "Output-Verzeichnis '{}' existiert bereits. Verwenden Sie --force zum Überschreiben.",
                output_dir
            )
            .into());
        }
    }
    fs::create_dir_all(out_path)?;
    Ok(())
}

/// Kopiert die Bundle-Dateien ins Output-Verzeichnis
pub fn copy_bundle_files(
    out_path: &Path,
    manifest_path: &str,
    proof_path: &str,
    timestamp_path: &Option<String>,
    registry_path: &Option<String>,
    report_path: &Option<String>,
) -> Result<ExportedFiles, Box<dyn Error>> {
    // Manifest
    let manifest_dst = out_path.join("manifest.json");
    fs::copy(manifest_path, &manifest_dst)?;

    // Proof
    let proof_dst = out_path.join("proof.dat");
    fs::copy(proof_path, &proof_dst)?;

    // Timestamp (optional)
    let timestamp_dst = if let Some(ts) = timestamp_path.as_ref() {
        let dst = out_path.join("timestamp.tsr");
        fs::copy(ts, &dst)?;
        Some(dst)
    } else {
        None
    };

    // Registry (optional)
    let registry_dst = if let Some(reg) = registry_path.as_ref() {
        let dst = out_path.join("registry.json");
        fs::copy(reg, &dst)?;
        Some(dst)
    } else {
        None
    };

    // Report (optional oder minimal)
    let report_dst = out_path.join("verification.report.json");
    if let Some(rep) = report_path.as_ref() {
        fs::copy(rep, &report_dst)?;
    } else {
        fs::write(
            &report_dst,
            r#"{"status":"unknown","note":"No verification performed before export"}"#,
        )?;
    }

    Ok(ExportedFiles {
        manifest_dst,
        proof_dst,
        timestamp_dst,
        registry_dst,
        report_dst,
    })
}

/// Erstellt die README.txt für das Bundle
pub fn create_bundle_readme(out_path: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let readme_dst = out_path.join("README.txt");
    let readme = format!(
        r#"CAP Bundle Package (cap-bundle.v1)
===================================

This package contains a complete, offline-verifiable proof bundle
following the Confidential Assurance Protocol (CAP) standard.

Files:
------
- manifest.json              : Manifest with commitments and policy info
- proof.dat                  : Zero-knowledge proof (Base64-encoded)
- timestamp.tsr              : Timestamp signature (optional)
- registry.json              : Local proof registry (optional)
- verification.report.json   : Pre-verification report
- README.txt                 : This file
- _meta.json                 : Bundle metadata (cap-bundle.v1 format)

Verification:
-------------
To verify this bundle offline, use:

  cap-agent verifier run --package <this-directory>

Or manually:

  cap-agent manifest verify \
    --manifest manifest.json \
    --proof proof.dat \
    --registry registry.json \
    --timestamp timestamp.tsr \
    --out verification.report.json

Bundle Format:
--------------
This bundle uses the cap-bundle.v1 format with structured metadata.
The _meta.json file contains:
- schema: "cap-bundle.v1"
- bundle_id: Unique bundle identifier
- files: Map of filename -> BundleFileMeta (role, hash, size, content_type, optional)
- proof_units: Array of proof unit metadata

Package created: {}
Package schema: cap-bundle.v1

For more information, see: https://cap.protocol/
"#,
        Utc::now().to_rfc3339()
    );
    fs::write(&readme_dst, readme)?;
    Ok(readme_dst)
}

/// Berechnet SHA3-256 Hash einer Datei
pub fn compute_file_sha3(path: &Path) -> Result<String, Box<dyn Error>> {
    let bytes = fs::read(path)?;
    let hash = crate::crypto::sha3_256(&bytes);
    Ok(crate::crypto::hex_lower_prefixed32(hash))
}

/// Erstellt die BundleFileMeta-Map für _meta.json
pub fn create_files_map(
    files: &ExportedFiles,
    readme_dst: &Path,
) -> Result<HashMap<String, BundleFileMeta>, Box<dyn Error>> {
    let mut map = HashMap::new();

    // Manifest
    map.insert(
        "manifest.json".to_string(),
        BundleFileMeta {
            role: "manifest".to_string(),
            hash: compute_file_sha3(&files.manifest_dst)?,
            size: Some(fs::metadata(&files.manifest_dst)?.len()),
            content_type: Some("application/json".to_string()),
            optional: false,
        },
    );

    // Proof
    map.insert(
        "proof.dat".to_string(),
        BundleFileMeta {
            role: "proof".to_string(),
            hash: compute_file_sha3(&files.proof_dst)?,
            size: Some(fs::metadata(&files.proof_dst)?.len()),
            content_type: Some("application/octet-stream".to_string()),
            optional: false,
        },
    );

    // Timestamp (optional)
    if let Some(ts) = files.timestamp_dst.as_ref() {
        map.insert(
            "timestamp.tsr".to_string(),
            BundleFileMeta {
                role: "timestamp".to_string(),
                hash: compute_file_sha3(ts)?,
                size: Some(fs::metadata(ts)?.len()),
                content_type: None,
                optional: true,
            },
        );
    }

    // Registry (optional)
    if let Some(reg) = files.registry_dst.as_ref() {
        map.insert(
            "registry.json".to_string(),
            BundleFileMeta {
                role: "registry".to_string(),
                hash: compute_file_sha3(reg)?,
                size: Some(fs::metadata(reg)?.len()),
                content_type: Some("application/json".to_string()),
                optional: true,
            },
        );
    }

    // Report
    map.insert(
        "verification.report.json".to_string(),
        BundleFileMeta {
            role: "report".to_string(),
            hash: compute_file_sha3(&files.report_dst)?,
            size: Some(fs::metadata(&files.report_dst)?.len()),
            content_type: Some("application/json".to_string()),
            optional: false,
        },
    );

    // README
    map.insert(
        "README.txt".to_string(),
        BundleFileMeta {
            role: "documentation".to_string(),
            hash: compute_file_sha3(readme_dst)?,
            size: Some(fs::metadata(readme_dst)?.len()),
            content_type: Some("text/plain".to_string()),
            optional: false,
        },
    );

    Ok(map)
}

/// Erstellt die _meta.json Datei
pub fn create_bundle_meta(
    out_path: &Path,
    files_map: HashMap<String, BundleFileMeta>,
    manifest: &Manifest,
) -> Result<BundleMeta, Box<dyn Error>> {
    let proof_units = vec![ProofUnitMeta {
        id: "main".to_string(),
        manifest_file: "manifest.json".to_string(),
        proof_file: "proof.dat".to_string(),
        policy_id: manifest.policy.hash.clone(),
        policy_hash: manifest.policy.hash.clone(),
        backend: "mock".to_string(),
        depends_on: vec![],
    }];

    let meta = BundleMeta {
        schema: BUNDLE_SCHEMA_V1.to_string(),
        bundle_id: Uuid::new_v4().to_string(),
        created_at: Utc::now().to_rfc3339(),
        files: files_map,
        proof_units,
    };

    let meta_dst = out_path.join("_meta.json");
    fs::write(&meta_dst, serde_json::to_string_pretty(&meta)?)?;

    Ok(meta)
}

/// Exportiert ein komplettes CAP Bundle
///
/// Hauptfunktion für den Bundle-Export. Koordiniert alle Schritte:
/// 1. Verzeichnis vorbereiten
/// 2. Dateien kopieren
/// 3. README erstellen
/// 4. Metadaten berechnen und speichern
pub fn export_bundle(
    manifest_path: &str,
    proof_path: &str,
    timestamp_path: Option<String>,
    registry_path: Option<String>,
    report_path: Option<String>,
    output_dir: Option<String>,
    force: bool,
) -> Result<ExportResult, Box<dyn Error>> {
    let output_dir = output_dir.unwrap_or_else(|| "build/cap-proof".to_string());
    prepare_export_dir(&output_dir, force)?;
    let out_path = Path::new(&output_dir);

    // Dateien kopieren
    let exported = copy_bundle_files(
        out_path,
        manifest_path,
        proof_path,
        &timestamp_path,
        &registry_path,
        &report_path,
    )?;

    // Manifest laden für Policy-Info
    let manifest = Manifest::load(&exported.manifest_dst)?;

    // README erstellen
    let readme_dst = create_bundle_readme(out_path)?;

    // Metadaten erstellen
    let files_map = create_files_map(&exported, &readme_dst)?;
    let meta = create_bundle_meta(out_path, files_map, &manifest)?;

    // Dateianzahl berechnen
    let file_count = 5
        + exported.timestamp_dst.is_some() as usize
        + exported.registry_dst.is_some() as usize;

    Ok(ExportResult {
        output_dir,
        bundle_id: meta.bundle_id,
        file_count,
    })
}
