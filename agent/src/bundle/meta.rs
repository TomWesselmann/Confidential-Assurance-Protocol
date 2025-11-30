//! Bundle Metadata Layer (cap-bundle.v1)
//!
//! Dieses Modul definiert das cap-bundle.v1 Format für Proof-Packages.
//! Es bietet Strukturen für _meta.json Parsing, File-Metadaten, Proof-Units
//! und Security-Validierungen (Path Traversal Prevention, Cycle Detection).

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};

/// Schema-Version für cap-bundle
pub const BUNDLE_SCHEMA_V1: &str = "cap-bundle.v1";

/// Einzelne Datei im Bundle mit Role und Hash
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BundleFileMeta {
    /// Role der Datei: "manifest" | "proof" | "timestamp" | "registry"
    pub role: String,

    /// SHA3-256 Hash (0x-präfixiert, 64 hex chars)
    pub hash: String,

    /// Dateigröße in Bytes (optional)
    #[serde(default)]
    pub size: Option<u64>,

    /// Content-Type (optional)
    #[serde(default)]
    pub content_type: Option<String>,

    /// Ob die Datei optional ist (default: false)
    #[serde(default)]
    pub optional: bool,
}

/// Proof Unit - Einzelner verifizierbarer Proof im Bundle
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProofUnitMeta {
    /// Eindeutige ID dieser Proof Unit
    pub id: String,

    /// Dateiname des Manifests (relativ zum Bundle-Root)
    pub manifest_file: String,

    /// Dateiname des Proofs (relativ zum Bundle-Root)
    pub proof_file: String,

    /// Policy-ID (z.B. "lksg.demo.v1")
    pub policy_id: String,

    /// SHA3-256 Hash der Policy
    pub policy_hash: String,

    /// Backend-Typ: "mock" | "zkvm" | "halo2"
    pub backend: String,

    /// IDs anderer Proof Units, von denen diese abhängt
    #[serde(default)]
    pub depends_on: Vec<String>,
}

/// Vollständige Bundle-Metadaten (cap-bundle.v1)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BundleMeta {
    /// Schema-Version (sollte "cap-bundle.v1" sein)
    pub schema: String,

    /// UUID v4 des Bundles
    pub bundle_id: String,

    /// RFC3339 Timestamp der Erstellung
    pub created_at: String,

    /// Map: Filename → File Metadata
    pub files: HashMap<String, BundleFileMeta>,

    /// Liste aller Proof Units im Bundle
    pub proof_units: Vec<ProofUnitMeta>,
}

/// Lädt und parsed _meta.json aus einem Bundle-Verzeichnis
///
/// # Errors
/// - Datei nicht gefunden
/// - JSON-Parsing-Fehler
/// - Schema-Validierung fehlgeschlagen
pub fn load_bundle_meta(bundle_dir: &Path) -> Result<BundleMeta> {
    let meta_path = bundle_dir.join("_meta.json");

    if !meta_path.exists() {
        return Err(anyhow!(
            "_meta.json not found in bundle directory: {}",
            bundle_dir.display()
        ));
    }

    let content = std::fs::read_to_string(&meta_path)?;
    let meta: BundleMeta = serde_json::from_str(&content)?;

    // Validiere Schema
    validate_schema(&meta)?;

    Ok(meta)
}

/// Validiert das Bundle-Schema
///
/// # Errors
/// - Schema-Feld fehlt oder ist nicht "cap-bundle.v1"
pub fn validate_schema(meta: &BundleMeta) -> Result<()> {
    if meta.schema.is_empty() {
        return Err(anyhow!("Missing schema field in _meta.json"));
    }

    if meta.schema != BUNDLE_SCHEMA_V1 {
        // Warnung, aber kein harter Fehler (Backward-Compatibility)
        eprintln!(
            "⚠️  Warning: Unknown bundle schema '{}', expected '{}'",
            meta.schema, BUNDLE_SCHEMA_V1
        );
    }

    Ok(())
}

/// 🔒 SECURITY: Sanitiert Dateinamen gegen Path Traversal
///
/// Prüft dass der Pfad:
/// - Nicht absolut ist
/// - Keine ".." Komponenten enthält (Parent Directory)
///
/// # Errors
/// - Absolute Pfade werden rejected
/// - Pfade mit ".." werden rejected
///
/// # Examples
/// ```
/// use std::path::PathBuf;
/// use cap_agent::bundle::meta::sanitize_filename;
///
/// // Erlaubt: Relative Pfade
/// assert!(sanitize_filename("manifest.json").is_ok());
/// assert!(sanitize_filename("subdir/proof.capz").is_ok());
///
/// // Verboten: Absolute Pfade
/// assert!(sanitize_filename("/etc/passwd").is_err());
///
/// // Verboten: Path Traversal
/// assert!(sanitize_filename("../../../etc/passwd").is_err());
/// assert!(sanitize_filename("subdir/../../etc/passwd").is_err());
/// ```
pub fn sanitize_filename(name: &str) -> Result<PathBuf> {
    let path = Path::new(name);

    // Keine absoluten Pfade
    if path.is_absolute() {
        return Err(anyhow!("Absolute paths not allowed: {}", name));
    }

    // Keine ".." Komponenten (Path Traversal Prevention)
    for component in path.components() {
        if matches!(component, Component::ParentDir) {
            return Err(anyhow!("Path traversal not allowed: {}", name));
        }
    }

    Ok(path.to_path_buf())
}

/// 🔒 SECURITY: Cycle Detection in Proof Unit Dependencies
///
/// Verwendet DFS (Depth-First Search) mit Recursion Stack um Zyklen
/// in den depends_on-Beziehungen zu erkennen.
///
/// # Errors
/// - Zirkuläre Abhängigkeiten werden erkannt und rejected
///
/// # Algorithm
/// - Komplexität: O(V + E) für V Proof Units, E Kanten
/// - DFS mit Visited-Set und Recursion-Stack
///
/// # Examples
/// ```
/// use cap_agent::bundle::meta::{ProofUnitMeta, check_dependency_cycles};
///
/// // Kein Zyklus: A → B → C
/// let units = vec![
///     ProofUnitMeta {
///         id: "A".to_string(),
///         depends_on: vec!["B".to_string()],
///         manifest_file: "m.json".to_string(),
///         proof_file: "p.capz".to_string(),
///         policy_id: "test".to_string(),
///         policy_hash: "0x00".to_string(),
///         backend: "mock".to_string(),
///     },
///     ProofUnitMeta {
///         id: "B".to_string(),
///         depends_on: vec!["C".to_string()],
///         manifest_file: "m.json".to_string(),
///         proof_file: "p.capz".to_string(),
///         policy_id: "test".to_string(),
///         policy_hash: "0x00".to_string(),
///         backend: "mock".to_string(),
///     },
///     ProofUnitMeta {
///         id: "C".to_string(),
///         depends_on: vec![],
///         manifest_file: "m.json".to_string(),
///         proof_file: "p.capz".to_string(),
///         policy_id: "test".to_string(),
///         policy_hash: "0x00".to_string(),
///         backend: "mock".to_string(),
///     },
/// ];
///
/// assert!(check_dependency_cycles(&units).is_ok());
///
/// // Zyklus: A → B → C → A
/// let units_cyclic = vec![
///     ProofUnitMeta {
///         id: "A".to_string(),
///         depends_on: vec!["B".to_string()],
///         manifest_file: "m.json".to_string(),
///         proof_file: "p.capz".to_string(),
///         policy_id: "test".to_string(),
///         policy_hash: "0x00".to_string(),
///         backend: "mock".to_string(),
///     },
///     ProofUnitMeta {
///         id: "B".to_string(),
///         depends_on: vec!["C".to_string()],
///         manifest_file: "m.json".to_string(),
///         proof_file: "p.capz".to_string(),
///         policy_id: "test".to_string(),
///         policy_hash: "0x00".to_string(),
///         backend: "mock".to_string(),
///     },
///     ProofUnitMeta {
///         id: "C".to_string(),
///         depends_on: vec!["A".to_string()],  // Cycle!
///         manifest_file: "m.json".to_string(),
///         proof_file: "p.capz".to_string(),
///         policy_id: "test".to_string(),
///         policy_hash: "0x00".to_string(),
///         backend: "mock".to_string(),
///     },
/// ];
///
/// assert!(check_dependency_cycles(&units_cyclic).is_err());
/// ```
pub fn check_dependency_cycles(units: &[ProofUnitMeta]) -> Result<()> {
    // Build dependency graph
    let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
    for unit in units {
        graph.insert(
            &unit.id,
            unit.depends_on.iter().map(|s| s.as_str()).collect(),
        );
    }

    // DFS-based cycle detection
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();

    for unit in units {
        if !visited.contains(unit.id.as_str())
            && has_cycle(&graph, &unit.id, &mut visited, &mut rec_stack)?
        {
            return Err(anyhow!(
                "Circular dependency detected in proof units starting from '{}'",
                unit.id
            ));
        }
    }

    Ok(())
}

/// DFS-Helper für Cycle Detection
fn has_cycle(
    graph: &HashMap<&str, Vec<&str>>,
    node: &str,
    visited: &mut HashSet<String>,
    rec_stack: &mut HashSet<String>,
) -> Result<bool> {
    visited.insert(node.to_string());
    rec_stack.insert(node.to_string());

    if let Some(neighbors) = graph.get(node) {
        for &neighbor in neighbors {
            if !visited.contains(neighbor) {
                if has_cycle(graph, neighbor, visited, rec_stack)? {
                    return Ok(true);
                }
            } else if rec_stack.contains(neighbor) {
                return Ok(true); // Cycle found
            }
        }
    }

    rec_stack.remove(node);
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename_valid_relative() {
        let filename = "subdir/manifest.json";
        let result = sanitize_filename(filename);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("subdir/manifest.json"));
    }

    #[test]
    fn test_sanitize_filename_rejects_absolute() {
        let filename = "/etc/passwd";
        let result = sanitize_filename(filename);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Absolute paths not allowed"));
    }

    #[test]
    fn test_sanitize_filename_rejects_parent_dir() {
        let filename = "../../../etc/passwd";
        let result = sanitize_filename(filename);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Path traversal not allowed"));
    }

    #[test]
    fn test_check_dependency_cycles_no_cycle() {
        let units = vec![
            ProofUnitMeta {
                id: "A".to_string(),
                depends_on: vec!["B".to_string()],
                manifest_file: "m.json".to_string(),
                proof_file: "p.capz".to_string(),
                policy_id: "test".to_string(),
                policy_hash: "0x00".to_string(),
                backend: "mock".to_string(),
            },
            ProofUnitMeta {
                id: "B".to_string(),
                depends_on: vec![],
                manifest_file: "m.json".to_string(),
                proof_file: "p.capz".to_string(),
                policy_id: "test".to_string(),
                policy_hash: "0x00".to_string(),
                backend: "mock".to_string(),
            },
        ];

        assert!(check_dependency_cycles(&units).is_ok());
    }

    #[test]
    fn test_check_dependency_cycles_detects_cycle() {
        let units = vec![
            ProofUnitMeta {
                id: "A".to_string(),
                depends_on: vec!["B".to_string()],
                manifest_file: "m.json".to_string(),
                proof_file: "p.capz".to_string(),
                policy_id: "test".to_string(),
                policy_hash: "0x00".to_string(),
                backend: "mock".to_string(),
            },
            ProofUnitMeta {
                id: "B".to_string(),
                depends_on: vec!["C".to_string()],
                manifest_file: "m.json".to_string(),
                proof_file: "p.capz".to_string(),
                policy_id: "test".to_string(),
                policy_hash: "0x00".to_string(),
                backend: "mock".to_string(),
            },
            ProofUnitMeta {
                id: "C".to_string(),
                depends_on: vec!["A".to_string()], // Cycle: A → B → C → A
                manifest_file: "m.json".to_string(),
                proof_file: "p.capz".to_string(),
                policy_id: "test".to_string(),
                policy_hash: "0x00".to_string(),
                backend: "mock".to_string(),
            },
        ];

        let result = check_dependency_cycles(&units);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Circular dependency"));
    }

    #[test]
    fn test_bundle_meta_parse_roundtrip() {
        let meta = BundleMeta {
            schema: BUNDLE_SCHEMA_V1.to_string(),
            bundle_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            created_at: "2025-11-24T16:00:00Z".to_string(),
            files: {
                let mut map = HashMap::new();
                map.insert(
                    "manifest.json".to_string(),
                    BundleFileMeta {
                        role: "manifest".to_string(),
                        hash: "0x1234567890123456789012345678901234567890123456789012345678901234"
                            .to_string(),
                        size: Some(1234),
                        content_type: Some("application/json".to_string()),
                        optional: false,
                    },
                );
                map
            },
            proof_units: vec![ProofUnitMeta {
                id: "main".to_string(),
                manifest_file: "manifest.json".to_string(),
                proof_file: "proof.capz".to_string(),
                policy_id: "lksg.demo.v1".to_string(),
                policy_hash: "0xabcd1234".to_string(),
                backend: "mock".to_string(),
                depends_on: vec![],
            }],
        };

        // Serialize → Deserialize
        let json = serde_json::to_string(&meta).unwrap();
        let parsed: BundleMeta = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.schema, BUNDLE_SCHEMA_V1);
        assert_eq!(parsed.bundle_id, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(parsed.proof_units.len(), 1);
        assert_eq!(parsed.proof_units[0].id, "main");
    }
}
