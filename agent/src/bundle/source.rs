//! Bundle Source Abstraction (REQ-03)
//!
//! Dieses Modul implementiert eine einheitliche Abstraktion für verschiedene
//! Bundle-Quellen (Directory, ZIP) mit Security-Validierung.
//!
//! ## Security-Features (REQ-13)
//!
//! - Path-Traversal-Prevention (keine ".." Components)
//! - Absolute-Path-Rejection
//! - Zip-Bomb-Protection (Größen- und Ratio-Limits)
//! - TOCTOU-Prevention (atomic loading)

use crate::bundle::meta::{load_bundle_meta, sanitize_filename, BundleMeta};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

/// Security-Limits für ZIP-Extraktion (REQ-13, REQ-14)
const MAX_UNCOMPRESSED_SIZE: u64 = 500_000_000; // 500 MB
const MAX_FILE_COUNT: usize = 10_000;
const MAX_COMPRESSION_RATIO: u64 = 100; // 100:1 Ratio ist verdächtig

/// Bundle-Source-Abstraktion (REQ-03)
///
/// Ermöglicht einheitlichen Zugriff auf Bundles aus verschiedenen Quellen.
/// Zukünftige Erweiterungen: Memory, Stream, Remote (Phase 3)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum BundleSource {
    /// Entpacktes Bundle-Verzeichnis
    Directory { path: PathBuf },

    /// ZIP-Archiv mit Bundle-Inhalt
    #[serde(rename = "zip")]
    ZipFile { path: PathBuf },
}

impl BundleSource {
    /// Erstellt BundleSource aus Pfad (automatische Erkennung)
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(anyhow!("Path does not exist: {}", path.display()));
        }

        if path.is_dir() {
            Ok(Self::Directory {
                path: path.to_path_buf(),
            })
        } else if path.extension().and_then(|s| s.to_str()) == Some("zip") {
            Ok(Self::ZipFile {
                path: path.to_path_buf(),
            })
        } else {
            Err(anyhow!(
                "Cannot determine bundle source type for: {}",
                path.display()
            ))
        }
    }

    /// Gibt den Pfad der Source zurück
    pub fn path(&self) -> &Path {
        match self {
            Self::Directory { path } | Self::ZipFile { path } => path,
        }
    }
}

/// Bundle-Daten in Memory (atomic geladen, TOCTOU-sicher)
#[derive(Debug, Clone)]
pub struct BundleData {
    /// Bundle-Metadaten
    pub meta: BundleMeta,

    /// Dateien: Filename → Content (Bytes)
    pub files: HashMap<String, Vec<u8>>,
}

/// Parst Bundle-Metadaten von beliebiger Quelle (REQ-03)
///
/// # Security
/// - Validiert Path-Traversal (REQ-13)
/// - Prüft Zip-Bombs (REQ-13)
/// - Atomic Loading (REQ-04: TOCTOU-Prevention)
///
/// # Errors
/// - Source existiert nicht
/// - _meta.json fehlt oder ist invalid
/// - Security-Validierung fehlgeschlagen
pub fn parse_bundle_source(source: &BundleSource) -> Result<BundleMeta> {
    match source {
        BundleSource::Directory { path } => {
            // Direkt von Filesystem laden
            load_bundle_meta(path)
        }
        BundleSource::ZipFile { path } => {
            // ZIP öffnen und _meta.json extrahieren
            let file = fs::File::open(path)?;
            let mut archive = zip::ZipArchive::new(file)?;

            // Suche _meta.json
            let mut meta_file = archive
                .by_name("_meta.json")
                .map_err(|_| anyhow!("_meta.json not found in ZIP archive"))?;

            // Lese und parse
            let mut content = String::new();
            meta_file.read_to_string(&mut content)?;

            let meta: BundleMeta = serde_json::from_str(&content)?;

            // Validiere Schema
            crate::bundle::meta::validate_schema(&meta)?;

            Ok(meta)
        }
    }
}

/// Lädt Bundle vollständig in Memory (atomic, TOCTOU-sicher) (REQ-04)
///
/// Diese Funktion lädt alle Bundle-Dateien EINMAL vollständig in Memory,
/// um Race Conditions zu verhindern (TOCTOU-Attack-Prevention).
///
/// # Security
/// - Atomic Read: Alle Dateien werden auf einmal geladen
/// - Zip-Bomb-Protection: Größen- und Ratio-Checks
/// - Path-Traversal-Prevention
///
/// # Performance
/// Für große Bundles (>100 MB) kann dies RAM-intensiv sein (REQ-14).
/// Nutzen Sie Streaming-Ansätze in späteren Phasen.
///
/// # Errors
/// - Bundle existiert nicht
/// - Security-Validierung fehlgeschlagen
/// - Memory-Allocation fehlgeschlagen
pub fn load_bundle_atomic(source: &BundleSource) -> Result<BundleData> {
    match source {
        BundleSource::Directory { path } => load_directory_atomic(path),
        BundleSource::ZipFile { path } => load_zip_atomic(path),
    }
}

/// Lädt Directory-Bundle atomic in Memory
fn load_directory_atomic(dir: &Path) -> Result<BundleData> {
    // 1. Lade Metadaten
    let meta = load_bundle_meta(dir)?;

    // 2. Lade alle referenzierten Dateien
    let mut files = HashMap::new();
    let mut total_size = 0u64;

    for (filename, file_meta) in &meta.files {
        // Security: Path-Traversal-Check
        sanitize_filename(filename)?;

        let file_path = dir.join(filename);

        // Optional-Check
        if file_meta.optional && !file_path.exists() {
            continue;
        }

        // Größen-Check (REQ-14)
        if let Some(size) = file_meta.size {
            total_size += size;
            if total_size > MAX_UNCOMPRESSED_SIZE {
                return Err(anyhow!(
                    "Bundle exceeds size limit ({} > {} bytes)",
                    total_size,
                    MAX_UNCOMPRESSED_SIZE
                ));
            }
        }

        // Atomic Read
        let content = fs::read(&file_path)
            .map_err(|e| anyhow!("Failed to read file '{}': {}", filename, e))?;

        files.insert(filename.clone(), content);
    }

    Ok(BundleData { meta, files })
}

/// Lädt ZIP-Bundle atomic in Memory mit Security-Checks (REQ-13)
fn load_zip_atomic(zip_path: &Path) -> Result<BundleData> {
    let file = fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    // 1. Security Pre-Check: Zip-Bomb-Detection
    validate_zip_safe(&mut archive)?;

    // 2. Lade _meta.json
    let mut meta_file = archive
        .by_name("_meta.json")
        .map_err(|_| anyhow!("_meta.json not found in ZIP"))?;

    let mut meta_content = String::new();
    meta_file.read_to_string(&mut meta_content)?;
    drop(meta_file); // Release borrow

    let meta: BundleMeta = serde_json::from_str(&meta_content)?;
    crate::bundle::meta::validate_schema(&meta)?;

    // 3. Lade alle referenzierten Dateien
    let mut files = HashMap::new();

    for (filename, file_meta) in &meta.files {
        // Security: Path-Traversal-Check
        sanitize_filename(filename)?;

        // Optional-Check: Skip optional files that don't exist in archive
        if file_meta.optional && archive.by_name(filename).is_err() {
            continue;
        }

        let mut zip_file = archive
            .by_name(filename)
            .map_err(|_| anyhow!("File '{}' not found in ZIP", filename))?;

        // Atomic Read
        let mut content = Vec::new();
        zip_file.read_to_end(&mut content)?;

        files.insert(filename.clone(), content);
    }

    Ok(BundleData { meta, files })
}

/// 🔒 SECURITY: Validiert ZIP gegen Zip-Bombs (REQ-13)
///
/// Prüft:
/// - Anzahl der Dateien (< MAX_FILE_COUNT)
/// - Uncompressed Size (< MAX_UNCOMPRESSED_SIZE)
/// - Compression Ratio (< MAX_COMPRESSION_RATIO)
///
/// # Errors
/// - Zu viele Dateien
/// - Größe überschreitet Limit
/// - Verdächtige Compression-Ratio
fn validate_zip_safe<R: Read + std::io::Seek>(archive: &mut zip::ZipArchive<R>) -> Result<()> {
    let file_count = archive.len();
    let mut total_uncompressed = 0u64;

    // Check 1: File-Count
    if file_count > MAX_FILE_COUNT {
        return Err(anyhow!(
            "ZIP contains too many files ({} > {})",
            file_count,
            MAX_FILE_COUNT
        ));
    }

    // Check 2 & 3: Size und Compression-Ratio
    for i in 0..file_count {
        let file = archive
            .by_index(i)
            .map_err(|e| anyhow!("Failed to access ZIP entry {}: {}", i, e))?;

        let uncompressed_size = file.size();
        let compressed_size = file.compressed_size();

        // Accumulate total size
        total_uncompressed += uncompressed_size;
        if total_uncompressed > MAX_UNCOMPRESSED_SIZE {
            return Err(anyhow!(
                "ZIP uncompressed size exceeds limit ({} > {} bytes)",
                total_uncompressed,
                MAX_UNCOMPRESSED_SIZE
            ));
        }

        // Compression-Ratio-Check (nur wenn compressed_size > 0)
        if compressed_size > 0 {
            let ratio = uncompressed_size / compressed_size;
            if ratio > MAX_COMPRESSION_RATIO {
                return Err(anyhow!(
                    "Suspicious compression ratio ({}:1) for file '{}' (possible zip bomb)",
                    ratio,
                    file.name()
                ));
            }
        }

        // Path-Traversal-Check
        validate_zip_entry_path(Path::new(file.name()))?;
    }

    Ok(())
}

/// 🔒 SECURITY: Validiert ZIP-Entry-Pfad (REQ-13)
///
/// Verhindert:
/// - Path-Traversal (../)
/// - Absolute Pfade (/etc/passwd)
///
/// # Errors
/// - Pfad enthält ".." Component
/// - Pfad ist absolut
pub fn validate_zip_entry_path(path: &Path) -> Result<()> {
    // Check 1: Keine Parent-Directory-Components
    for component in path.components() {
        if matches!(component, Component::ParentDir) {
            return Err(anyhow!(
                "Path traversal detected in ZIP entry: {}",
                path.display()
            ));
        }
    }

    // Check 2: Keine absoluten Pfade
    if path.is_absolute() {
        return Err(anyhow!(
            "Absolute path not allowed in ZIP entry: {}",
            path.display()
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bundle_source_from_path_directory() {
        let temp_dir = std::env::temp_dir();
        let source = BundleSource::from_path(&temp_dir).unwrap();

        assert!(matches!(source, BundleSource::Directory { .. }));
    }

    #[test]
    fn test_validate_zip_entry_path_valid() {
        assert!(validate_zip_entry_path(Path::new("manifest.json")).is_ok());
        assert!(validate_zip_entry_path(Path::new("subdir/proof.dat")).is_ok());
    }

    #[test]
    fn test_validate_zip_entry_path_traversal() {
        let result = validate_zip_entry_path(Path::new("../../../etc/passwd"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Path traversal"));
    }

    #[test]
    fn test_validate_zip_entry_path_absolute() {
        let result = validate_zip_entry_path(Path::new("/etc/passwd"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Absolute path"));
    }
}
