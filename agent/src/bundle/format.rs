//! Bundle Format Detection and Verification Trait
//!
//! Dieses Modul implementiert ein elegantes Trait-basiertes System zur
//! Erkennung und Verifikation verschiedener Bundle-Formate (v1 und v2).
//!
//! ## Design-Prinzipien
//!
//! - Single Responsibility: Jedes Format implementiert sein eigenes Verify
//! - Open/Closed: Neue Formate können hinzugefügt werden ohne bestehenden Code zu ändern
//! - Strategy Pattern: Format-Erkennung und -Verifikation sind entkoppelt

use anyhow::{anyhow, Result};
use serde_json::Value;
use std::path::Path;

/// Bundle-Format-Erkennung
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BundleFormatKind {
    /// cap-bundle.v1 Format (hat "schema" Feld in _meta.json)
    V1,
    /// cap-proof.v2 Format (hat "bundle_version" mit "cap-proof.v2" Präfix)
    V2,
    /// Legacy Format (pre-bundle.v1, keine _meta.json)
    Legacy,
}

impl std::fmt::Display for BundleFormatKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V1 => write!(f, "cap-bundle.v1"),
            Self::V2 => write!(f, "cap-proof.v2"),
            Self::Legacy => write!(f, "legacy"),
        }
    }
}

impl BundleFormatKind {
    /// Erkennt das Bundle-Format aus _meta.json
    ///
    /// # Arguments
    /// * `meta` - Geparster JSON-Inhalt von _meta.json
    ///
    /// # Returns
    /// Das erkannte Bundle-Format
    pub fn detect_from_meta(meta: &Value) -> Self {
        // Prüfe auf cap-bundle.v1 (hat "schema" Feld)
        if meta.get("schema").is_some() {
            return Self::V1;
        }

        // Prüfe auf cap-proof.v2 (hat "bundle_version" mit "cap-proof.v2" Präfix)
        if let Some(version) = meta.get("bundle_version").and_then(|v| v.as_str()) {
            if version.starts_with("cap-proof.v2") {
                return Self::V2;
            }
        }

        // Fallback zu Legacy
        Self::Legacy
    }

    /// Erkennt das Bundle-Format aus einem Bundle-Verzeichnis
    ///
    /// # Arguments
    /// * `bundle_dir` - Pfad zum Bundle-Verzeichnis
    ///
    /// # Returns
    /// Das erkannte Bundle-Format oder Legacy wenn keine _meta.json existiert
    pub fn detect_from_path(bundle_dir: &Path) -> Result<Self> {
        let meta_path = bundle_dir.join("_meta.json");

        if !meta_path.exists() {
            return Ok(Self::Legacy);
        }

        let content = std::fs::read_to_string(&meta_path)
            .map_err(|e| anyhow!("Failed to read _meta.json: {}", e))?;

        let meta: Value = serde_json::from_str(&content)
            .map_err(|e| anyhow!("Failed to parse _meta.json: {}", e))?;

        Ok(Self::detect_from_meta(&meta))
    }
}

/// Trait für Bundle-Format-Verifikation
///
/// Jedes Bundle-Format implementiert dieses Trait für seine spezifische
/// Verifikationslogik.
pub trait BundleVerifier {
    /// Verifiziert das Bundle und gibt einen Report zurück
    fn verify(&self, bundle_dir: &Path) -> Result<BundleVerifyReport>;

    /// Gibt das Format-Kind zurück
    fn format_kind(&self) -> BundleFormatKind;
}

/// Verifikations-Report für Bundles
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BundleVerifyReport {
    pub format: String,
    pub status: String,
    pub manifest_hash: String,
    pub proof_hash: String,
    pub signature_valid: bool,
    pub integrity_valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl Default for BundleVerifyReport {
    fn default() -> Self {
        Self {
            format: "unknown".to_string(),
            status: "pending".to_string(),
            manifest_hash: String::new(),
            proof_hash: String::new(),
            signature_valid: false,
            integrity_valid: false,
            details: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_detect_v1_format() {
        let meta = json!({
            "schema": "cap-bundle.v1",
            "bundle_id": "test-bundle"
        });
        assert_eq!(
            BundleFormatKind::detect_from_meta(&meta),
            BundleFormatKind::V1
        );
    }

    #[test]
    fn test_detect_v2_format() {
        let meta = json!({
            "bundle_version": "cap-proof.v2.0",
            "created_at": "2025-01-01T00:00:00Z"
        });
        assert_eq!(
            BundleFormatKind::detect_from_meta(&meta),
            BundleFormatKind::V2
        );
    }

    #[test]
    fn test_detect_legacy_format() {
        let meta = json!({
            "unknown_field": "value"
        });
        assert_eq!(
            BundleFormatKind::detect_from_meta(&meta),
            BundleFormatKind::Legacy
        );
    }

    #[test]
    fn test_format_display() {
        assert_eq!(format!("{}", BundleFormatKind::V1), "cap-bundle.v1");
        assert_eq!(format!("{}", BundleFormatKind::V2), "cap-proof.v2");
        assert_eq!(format!("{}", BundleFormatKind::Legacy), "legacy");
    }
}
