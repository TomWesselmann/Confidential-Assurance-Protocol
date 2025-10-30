/// Registry-Modul für lokale Proof-Verwaltung
///
/// Dieses Modul verwaltet eine lokale Registry (JSON-basiert) für ZK-Proofs,
/// Manifeste und Timestamps. Es ermöglicht das Hinzufügen, Auflisten und
/// Verifizieren von Registry-Einträgen.
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::error::Error;
use std::fs;
use std::path::Path;

/// Registry-Eintrag für einen einzelnen Proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub id: String,
    pub manifest_hash: String,
    pub proof_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp_file: Option<String>,
    pub registered_at: String, // RFC3339
}

/// Lokale Registry-Struktur
#[derive(Debug, Serialize, Deserialize)]
pub struct Registry {
    pub registry_version: String,
    pub entries: Vec<RegistryEntry>,
}

impl Registry {
    /// Erstellt eine neue, leere Registry
    pub fn new() -> Self {
        Registry {
            registry_version: "1.0".to_string(),
            entries: Vec::new(),
        }
    }

    /// Lädt eine Registry aus einer JSON-Datei
    ///
    /// # Argumente
    /// * `path` - Pfad zur Registry-Datei
    ///
    /// # Rückgabe
    /// Registry-Objekt oder Fehler
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let content = fs::read_to_string(path)?;
        let registry: Registry = serde_json::from_str(&content)?;
        Ok(registry)
    }

    /// Speichert die Registry in eine JSON-Datei
    ///
    /// # Argumente
    /// * `path` - Pfad zur Zieldatei
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Fügt einen neuen Eintrag zur Registry hinzu
    ///
    /// # Argumente
    /// * `manifest_hash` - SHA3-256 Hash des Manifests
    /// * `proof_hash` - SHA3-256 Hash des Proofs
    /// * `timestamp_file` - Optionaler Pfad zur Timestamp-Datei
    ///
    /// # Rückgabe
    /// ID des neuen Eintrags
    pub fn add_entry(
        &mut self,
        manifest_hash: String,
        proof_hash: String,
        timestamp_file: Option<String>,
    ) -> String {
        let id = format!("proof_{:03}", self.entries.len() + 1);
        let entry = RegistryEntry {
            id: id.clone(),
            manifest_hash,
            proof_hash,
            timestamp_file,
            registered_at: Utc::now().to_rfc3339(),
        };
        self.entries.push(entry);
        id
    }

    /// Sucht einen Eintrag anhand von Manifest- und Proof-Hash
    ///
    /// # Argumente
    /// * `manifest_hash` - SHA3-256 Hash des Manifests
    /// * `proof_hash` - SHA3-256 Hash des Proofs
    ///
    /// # Rückgabe
    /// Optional: Referenz auf den gefundenen Eintrag
    pub fn find_entry(
        &self,
        manifest_hash: &str,
        proof_hash: &str,
    ) -> Option<&RegistryEntry> {
        self.entries.iter().find(|e| {
            e.manifest_hash == manifest_hash && e.proof_hash == proof_hash
        })
    }

    /// Verifiziert, ob ein Manifest- und Proof-Hash in der Registry existiert
    ///
    /// # Argumente
    /// * `manifest_hash` - SHA3-256 Hash des Manifests
    /// * `proof_hash` - SHA3-256 Hash des Proofs
    ///
    /// # Rückgabe
    /// true wenn Eintrag gefunden, false sonst
    pub fn verify_entry(&self, manifest_hash: &str, proof_hash: &str) -> bool {
        self.find_entry(manifest_hash, proof_hash).is_some()
    }

    /// Gibt die Anzahl der Einträge in der Registry zurück
    pub fn count(&self) -> usize {
        self.entries.len()
    }
}

/// Berechnet SHA3-256 Hash einer Datei
///
/// # Argumente
/// * `path` - Pfad zur Datei
///
/// # Rückgabe
/// Hex-String des Hashes mit "0x"-Präfix
pub fn compute_file_hash<P: AsRef<Path>>(path: P) -> Result<String, Box<dyn Error>> {
    let content = fs::read(path)?;
    let mut hasher = Sha3_256::new();
    hasher.update(&content);
    let hash = hasher.finalize();
    Ok(format!("0x{}", hex::encode(hash)))
}

/// Timestamp-Struktur (RFC3161-Mock)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timestamp {
    pub version: String,
    pub audit_tip_hex: String,
    pub created_at: String, // RFC3339
    pub tsa: String,
    pub signature: String, // Base64-encoded mock signature
    pub status: String,
}

impl Timestamp {
    /// Erstellt einen neuen Mock-Timestamp für einen Audit-Tip
    ///
    /// # Argumente
    /// * `audit_tip_hex` - Hex-String des Audit-Chain-Heads
    ///
    /// # Rückgabe
    /// Timestamp-Objekt
    pub fn create_mock(audit_tip_hex: String) -> Self {
        // Mock-Signatur: Base64(SHA3(audit_tip + timestamp))
        let now = Utc::now().to_rfc3339();
        let mut hasher = Sha3_256::new();
        hasher.update(audit_tip_hex.as_bytes());
        hasher.update(now.as_bytes());
        let sig = hasher.finalize();
        let signature = hex::encode(&sig[..]);

        Timestamp {
            version: "tsr.v1".to_string(),
            audit_tip_hex,
            created_at: now,
            tsa: "local-mock".to_string(),
            signature,
            status: "ok".to_string(),
        }
    }

    /// Lädt einen Timestamp aus einer JSON-Datei
    ///
    /// # Argumente
    /// * `path` - Pfad zur Timestamp-Datei
    ///
    /// # Rückgabe
    /// Timestamp-Objekt oder Fehler
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let content = fs::read_to_string(path)?;
        let timestamp: Timestamp = serde_json::from_str(&content)?;
        Ok(timestamp)
    }

    /// Speichert den Timestamp in eine JSON-Datei
    ///
    /// # Argumente
    /// * `path` - Pfad zur Zieldatei
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Verifiziert einen Timestamp gegen einen Audit-Tip
    ///
    /// # Argumente
    /// * `audit_tip_hex` - Erwarteter Audit-Tip (Hex-String)
    ///
    /// # Rückgabe
    /// true wenn gültig, false sonst
    pub fn verify(&self, audit_tip_hex: &str) -> bool {
        if self.audit_tip_hex != audit_tip_hex {
            return false;
        }

        // Verifiziere Mock-Signatur
        let mut hasher = Sha3_256::new();
        hasher.update(self.audit_tip_hex.as_bytes());
        hasher.update(self.created_at.as_bytes());
        let expected_sig = hasher.finalize();
        let expected_sig_hex = hex::encode(&expected_sig[..]);

        self.signature == expected_sig_hex
    }
}

/// Verifiziert, ob ein Manifest- und Proof-Hash in einer Registry-Datei existiert
///
/// # Argumente
/// * `registry_path` - Pfad zur Registry-JSON-Datei
/// * `manifest_hash` - SHA3-256 Hash des Manifests
/// * `proof_hash` - SHA3-256 Hash des Proofs
///
/// # Rückgabe
/// true wenn Eintrag gefunden, false sonst
pub fn verify_entry_from_file(
    registry_path: &str,
    manifest_hash: &str,
    proof_hash: &str,
) -> Result<bool, Box<dyn Error>> {
    let registry = Registry::load(registry_path)?;
    Ok(registry.verify_entry(manifest_hash, proof_hash))
}

/// Verifiziert einen Mock-Timestamp aus Datei
///
/// # Argumente
/// * `ts_path` - Pfad zur Timestamp-Datei
///
/// # Rückgabe
/// true wenn Timestamp-Status "ok" ist, false sonst
pub fn verify_timestamp_from_file(ts_path: &str) -> bool {
    match Timestamp::load(ts_path) {
        Ok(ts) => ts.status == "ok",
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_registry_new() {
        let registry = Registry::new();
        assert_eq!(registry.registry_version, "1.0");
        assert_eq!(registry.entries.len(), 0);
    }

    #[test]
    fn test_registry_add_entry() {
        let mut registry = Registry::new();
        let id = registry.add_entry(
            "0xabc123".to_string(),
            "0xdef456".to_string(),
            None,
        );
        assert_eq!(id, "proof_001");
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_registry_find_entry() {
        let mut registry = Registry::new();
        registry.add_entry(
            "0xabc123".to_string(),
            "0xdef456".to_string(),
            None,
        );

        let found = registry.find_entry("0xabc123", "0xdef456");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "proof_001");

        let not_found = registry.find_entry("0xwrong", "0xhash");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_registry_verify_entry() {
        let mut registry = Registry::new();
        registry.add_entry(
            "0xabc123".to_string(),
            "0xdef456".to_string(),
            None,
        );

        assert!(registry.verify_entry("0xabc123", "0xdef456"));
        assert!(!registry.verify_entry("0xwrong", "0xhash"));
    }

    #[test]
    fn test_registry_save_load() {
        let mut registry = Registry::new();
        registry.add_entry(
            "0xabc123".to_string(),
            "0xdef456".to_string(),
            Some("timestamp.tsr".to_string()),
        );

        let temp_path = "test_registry.json";
        registry.save(temp_path).unwrap();

        let loaded = Registry::load(temp_path).unwrap();
        assert_eq!(loaded.count(), 1);
        assert_eq!(loaded.entries[0].manifest_hash, "0xabc123");

        fs::remove_file(temp_path).ok();
    }

    #[test]
    fn test_timestamp_create_mock() {
        let tip = "0x1234567890abcdef".to_string();
        let ts = Timestamp::create_mock(tip.clone());

        assert_eq!(ts.version, "tsr.v1");
        assert_eq!(ts.audit_tip_hex, tip);
        assert_eq!(ts.tsa, "local-mock");
        assert_eq!(ts.status, "ok");
        assert!(!ts.signature.is_empty());
    }

    #[test]
    fn test_timestamp_verify_ok() {
        let tip = "0x1234567890abcdef".to_string();
        let ts = Timestamp::create_mock(tip.clone());

        assert!(ts.verify(&tip));
    }

    #[test]
    fn test_timestamp_verify_fail() {
        let tip = "0x1234567890abcdef".to_string();
        let ts = Timestamp::create_mock(tip);

        assert!(!ts.verify("0xwronghash"));
    }

    #[test]
    fn test_timestamp_save_load() {
        let tip = "0x1234567890abcdef".to_string();
        let ts = Timestamp::create_mock(tip.clone());

        let temp_path = "test_timestamp.tsr";
        ts.save(temp_path).unwrap();

        let loaded = Timestamp::load(temp_path).unwrap();
        assert_eq!(loaded.audit_tip_hex, tip);
        assert!(loaded.verify(&tip));

        fs::remove_file(temp_path).ok();
    }
}
