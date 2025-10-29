use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::io::{Supplier, Ubo};

/// Struktur für alle generierten Commitments (Merkle-Roots)
#[derive(Debug, Serialize, Deserialize)]
pub struct Commitments {
    pub supplier_root: String,
    pub ubo_root: String,
    pub company_commitment_root: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supplier_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ubo_count: Option<usize>,
}

impl Commitments {
    /// Lädt Commitments aus einer JSON-Datei
    ///
    /// # Argumente
    /// * `path` - Pfad zur JSON-Datei
    ///
    /// # Rückgabe
    /// Commitments-Objekt oder Fehler
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        load_commitments(path)
    }
}

/// Berechnet einen BLAKE3-Hash für einen einzelnen Datensatz
///
/// # Argumente
/// * `data` - Serialisierbarer Datensatz
///
/// # Rückgabe
/// Hex-String des BLAKE3-Hashes
fn hash_record<T: Serialize>(data: &T) -> Result<String, Box<dyn Error>> {
    let json = serde_json::to_string(data)?;
    let hash = blake3::hash(json.as_bytes());
    Ok(format!("0x{}", hash.to_hex()))
}

/// Berechnet einen Merkle-Root aus einer Liste von Hashes
///
/// Diese Implementierung hasht alle einzelnen Hashes konkateniert.
/// In einer Produktionsumgebung würde man einen echten Merkle-Tree verwenden.
///
/// # Argumente
/// * `hashes` - Liste von Hash-Strings
///
/// # Rückgabe
/// Root-Hash als Hex-String
fn compute_merkle_root(hashes: &[String]) -> String {
    let mut hasher = Hasher::new();
    for hash in hashes {
        hasher.update(hash.as_bytes());
    }
    let result = hasher.finalize();
    format!("0x{}", result.to_hex())
}

/// Berechnet den Merkle-Root für Supplier-Daten
///
/// # Argumente
/// * `suppliers` - Vector mit Supplier-Objekten
///
/// # Rückgabe
/// Root-Hash als Hex-String
pub fn compute_supplier_root(suppliers: &[Supplier]) -> Result<String, Box<dyn Error>> {
    let mut hashes = Vec::new();
    for supplier in suppliers {
        hashes.push(hash_record(supplier)?);
    }
    Ok(compute_merkle_root(&hashes))
}

/// Berechnet den Merkle-Root für UBO-Daten
///
/// # Argumente
/// * `ubos` - Vector mit UBO-Objekten
///
/// # Rückgabe
/// Root-Hash als Hex-String
pub fn compute_ubo_root(ubos: &[Ubo]) -> Result<String, Box<dyn Error>> {
    let mut hashes = Vec::new();
    for ubo in ubos {
        hashes.push(hash_record(ubo)?);
    }
    Ok(compute_merkle_root(&hashes))
}

/// Berechnet den Company-Commitment-Root aus Supplier- und UBO-Roots
///
/// # Argumente
/// * `supplier_root` - Hash des Supplier-Trees
/// * `ubo_root` - Hash des UBO-Trees
///
/// # Rückgabe
/// Combined Root-Hash als Hex-String
pub fn compute_company_root(supplier_root: &str, ubo_root: &str) -> String {
    let combined = format!("{}{}", supplier_root, ubo_root);
    let hash = blake3::hash(combined.as_bytes());
    format!("0x{}", hash.to_hex())
}

/// Speichert die Commitments als JSON-Datei
///
/// # Argumente
/// * `commitments` - Die zu speichernden Commitments
/// * `path` - Zielpfad für die JSON-Datei
///
/// # Rückgabe
/// Result mit () bei Erfolg oder Fehler
pub fn save_commitments<P: AsRef<Path>>(
    commitments: &Commitments,
    path: P,
) -> Result<(), Box<dyn Error>> {
    let json = serde_json::to_string_pretty(commitments)?;
    let mut file = File::create(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

/// Lädt Commitments aus einer JSON-Datei
///
/// # Argumente
/// * `path` - Pfad zur JSON-Datei
///
/// # Rückgabe
/// Commitments-Objekt oder Fehler
pub fn load_commitments<P: AsRef<Path>>(path: P) -> Result<Commitments, Box<dyn Error>> {
    let file = File::open(path)?;
    let commitments: Commitments = serde_json::from_reader(file)?;
    Ok(commitments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_record() {
        let supplier = Supplier {
            name: "Test".to_string(),
            jurisdiction: "DE".to_string(),
            tier: 1,
        };
        let hash = hash_record(&supplier).unwrap();
        assert!(hash.starts_with("0x"));
        assert!(hash.len() > 10);
    }

    #[test]
    fn test_merkle_root_deterministic() {
        let hashes = vec!["0xabc".to_string(), "0xdef".to_string()];
        let root1 = compute_merkle_root(&hashes);
        let root2 = compute_merkle_root(&hashes);
        assert_eq!(root1, root2);
    }

    #[test]
    fn test_company_root() {
        let supplier_root = "0xabc123";
        let ubo_root = "0xdef456";
        let company_root = compute_company_root(supplier_root, ubo_root);
        assert!(company_root.starts_with("0x"));
    }
}
