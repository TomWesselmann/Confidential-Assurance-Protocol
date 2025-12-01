//! Registry Store - Pluggable storage backends for registry
//!
//! Provides:
//! - RegistryStore trait for abstraction
//! - JsonRegistryStore for JSON file storage
//! - SqliteRegistryStore for SQLite database storage

use std::error::Error;
use std::fs;
use std::path::Path;

use super::entry::RegistryEntry;

/// Lokale Registry-Struktur
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Registry {
    pub registry_version: String,
    pub entries: Vec<RegistryEntry>,
}

#[allow(dead_code)]
impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl Registry {
    /// Erstellt eine neue, leere Registry
    pub fn new() -> Self {
        Registry {
            registry_version: "1.0".to_string(),
            entries: Vec::new(),
        }
    }

    /// Lädt eine Registry aus einer JSON-Datei
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let content = fs::read_to_string(path)?;
        let registry: Registry = serde_json::from_str(&content)?;
        Ok(registry)
    }

    /// Speichert die Registry in eine JSON-Datei
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Fügt einen neuen Eintrag zur Registry hinzu
    pub fn add_entry(
        &mut self,
        manifest_hash: String,
        proof_hash: String,
        timestamp_file: Option<String>,
    ) -> String {
        let id = format!("proof_{:03}", self.entries.len() + 1);
        let entry = RegistryEntry::new(
            id.clone(),
            manifest_hash,
            proof_hash,
            chrono::Utc::now().to_rfc3339(),
        )
        .with_timestamp_file(timestamp_file);
        self.entries.push(entry);
        id
    }

    /// Sucht einen Eintrag anhand von Manifest- und Proof-Hash
    pub fn find_entry(&self, manifest_hash: &str, proof_hash: &str) -> Option<&RegistryEntry> {
        self.entries
            .iter()
            .find(|e| e.manifest_hash == manifest_hash && e.proof_hash == proof_hash)
    }

    /// Verifiziert, ob ein Manifest- und Proof-Hash in der Registry existiert
    pub fn verify_entry(&self, manifest_hash: &str, proof_hash: &str) -> bool {
        self.find_entry(manifest_hash, proof_hash).is_some()
    }

    /// Gibt die Anzahl der Einträge in der Registry zurück
    pub fn count(&self) -> usize {
        self.entries.len()
    }
}

// ============================================================================
// Registry Backend Types
// ============================================================================

/// Registry Backend Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistryBackend {
    Json,
    Sqlite,
}

/// Pluggable Registry Store Trait
pub trait RegistryStore {
    /// Loads the complete registry
    fn load(&self) -> Result<Registry, Box<dyn Error>>;

    /// Saves the complete registry
    fn save(&self, reg: &Registry) -> Result<(), Box<dyn Error>>;

    /// Adds a single entry
    fn add_entry(&self, entry: RegistryEntry) -> Result<(), Box<dyn Error>>;

    /// Finds entry by manifest and proof hashes
    fn find_by_hashes(
        &self,
        manifest_hash: &str,
        proof_hash: &str,
    ) -> Result<Option<RegistryEntry>, Box<dyn Error>>;

    /// Lists all entries
    fn list(&self) -> Result<Vec<RegistryEntry>, Box<dyn Error>>;
}

// ============================================================================
// JSON Registry Store
// ============================================================================

/// JSON-based Registry Store (existing behavior)
pub struct JsonRegistryStore {
    pub path: std::path::PathBuf,
}

impl RegistryStore for JsonRegistryStore {
    fn load(&self) -> Result<Registry, Box<dyn Error>> {
        if !self.path.exists() {
            return Ok(Registry {
                registry_version: "1.0".to_string(),
                entries: Vec::new(),
            });
        }
        let content = fs::read_to_string(&self.path)?;
        let registry: Registry = serde_json::from_str(&content)?;
        Ok(registry)
    }

    fn save(&self, reg: &Registry) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(reg)?;
        fs::write(&self.path, json)?;
        Ok(())
    }

    fn add_entry(&self, entry: RegistryEntry) -> Result<(), Box<dyn Error>> {
        let mut reg = self.load()?;
        reg.entries.push(entry);
        self.save(&reg)
    }

    fn find_by_hashes(
        &self,
        manifest_hash: &str,
        proof_hash: &str,
    ) -> Result<Option<RegistryEntry>, Box<dyn Error>> {
        let reg = self.load()?;
        Ok(reg
            .entries
            .into_iter()
            .find(|e| e.manifest_hash == manifest_hash && e.proof_hash == proof_hash))
    }

    fn list(&self) -> Result<Vec<RegistryEntry>, Box<dyn Error>> {
        Ok(self.load()?.entries)
    }
}

// ============================================================================
// SQLite Registry Store
// ============================================================================

/// SQLite-based Registry Store
pub struct SqliteRegistryStore {
    conn: std::cell::RefCell<rusqlite::Connection>,
    #[allow(dead_code)]
    path: std::path::PathBuf,
}

impl SqliteRegistryStore {
    /// Opens or creates a SQLite registry database
    pub fn open(path: &Path) -> Result<Self, Box<dyn Error>> {
        let conn = rusqlite::Connection::open(path)?;

        // Initialize schema
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;

            CREATE TABLE IF NOT EXISTS registry_meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS registry_entries (
                id TEXT PRIMARY KEY,
                manifest_hash TEXT NOT NULL,
                proof_hash TEXT NOT NULL,
                timestamp_file TEXT,
                registered_at TEXT NOT NULL,
                signature TEXT,
                public_key TEXT,
                -- BLOB fields (v0.9)
                blob_manifest TEXT,
                blob_proof TEXT,
                blob_wasm TEXT,
                blob_abi TEXT,
                -- Self-verification fields (v0.9)
                selfverify_status TEXT,
                selfverify_at TEXT,
                verifier_name TEXT,
                verifier_version TEXT,
                -- Key management fields (v0.10)
                kid TEXT,
                signature_scheme TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_registry_hashes
                ON registry_entries (manifest_hash, proof_hash);
        "#,
        )?;

        // Ensure version
        conn.execute(
            "INSERT OR IGNORE INTO registry_meta(key, value) VALUES('registry_version', '1.0')",
            [],
        )?;

        Ok(Self {
            conn: std::cell::RefCell::new(conn),
            path: path.to_path_buf(),
        })
    }

    fn row_to_entry(row: &rusqlite::Row) -> rusqlite::Result<RegistryEntry> {
        Ok(RegistryEntry {
            id: row.get(0)?,
            manifest_hash: row.get(1)?,
            proof_hash: row.get(2)?,
            timestamp_file: row.get(3)?,
            registered_at: row.get(4)?,
            signature: row.get(5).ok(),
            public_key: row.get(6).ok(),
            blob_manifest: row.get(7).ok(),
            blob_proof: row.get(8).ok(),
            blob_wasm: row.get(9).ok(),
            blob_abi: row.get(10).ok(),
            selfverify_status: row.get(11).ok(),
            selfverify_at: row.get(12).ok(),
            verifier_name: row.get(13).ok(),
            verifier_version: row.get(14).ok(),
            kid: row.get(15).ok(),
            signature_scheme: row.get(16).ok(),
        })
    }

    fn insert_entry(
        conn: &rusqlite::Connection,
        entry: &RegistryEntry,
    ) -> Result<(), Box<dyn Error>> {
        conn.execute(
            "INSERT OR REPLACE INTO registry_entries(
                id, manifest_hash, proof_hash, timestamp_file, registered_at, signature, public_key,
                blob_manifest, blob_proof, blob_wasm, blob_abi,
                selfverify_status, selfverify_at, verifier_name, verifier_version,
                kid, signature_scheme
             ) VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                &entry.id,
                &entry.manifest_hash,
                &entry.proof_hash,
                &entry.timestamp_file,
                &entry.registered_at,
                &entry.signature,
                &entry.public_key,
                &entry.blob_manifest,
                &entry.blob_proof,
                &entry.blob_wasm,
                &entry.blob_abi,
                &entry.selfverify_status,
                &entry.selfverify_at,
                &entry.verifier_name,
                &entry.verifier_version,
                &entry.kid,
                &entry.signature_scheme
            ],
        )?;
        Ok(())
    }
}

impl RegistryStore for SqliteRegistryStore {
    fn load(&self) -> Result<Registry, Box<dyn Error>> {
        let conn = self.conn.borrow();
        let mut stmt = conn.prepare(
            "SELECT id, manifest_hash, proof_hash, timestamp_file, registered_at, signature, public_key,
                    blob_manifest, blob_proof, blob_wasm, blob_abi,
                    selfverify_status, selfverify_at, verifier_name, verifier_version,
                    kid, signature_scheme
             FROM registry_entries
             ORDER BY registered_at DESC",
        )?;

        let rows = stmt.query_map([], Self::row_to_entry)?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }

        Ok(Registry {
            registry_version: "1.0".to_string(),
            entries,
        })
    }

    fn save(&self, reg: &Registry) -> Result<(), Box<dyn Error>> {
        let mut conn = self.conn.borrow_mut();
        let tx = conn.transaction()?;

        // Clear existing entries
        tx.execute("DELETE FROM registry_entries", [])?;

        // Insert all entries
        for entry in &reg.entries {
            Self::insert_entry(&tx, entry)?;
        }

        tx.commit()?;
        Ok(())
    }

    fn add_entry(&self, entry: RegistryEntry) -> Result<(), Box<dyn Error>> {
        let conn = self.conn.borrow();
        Self::insert_entry(&conn, &entry)
    }

    fn find_by_hashes(
        &self,
        manifest_hash: &str,
        proof_hash: &str,
    ) -> Result<Option<RegistryEntry>, Box<dyn Error>> {
        let conn = self.conn.borrow();
        let mut stmt = conn.prepare(
            "SELECT id, manifest_hash, proof_hash, timestamp_file, registered_at, signature, public_key,
                    blob_manifest, blob_proof, blob_wasm, blob_abi,
                    selfverify_status, selfverify_at, verifier_name, verifier_version,
                    kid, signature_scheme
             FROM registry_entries
             WHERE manifest_hash = ?1 AND proof_hash = ?2
             LIMIT 1",
        )?;

        let mut rows = stmt.query(rusqlite::params![manifest_hash, proof_hash])?;

        if let Some(row) = rows.next()? {
            return Ok(Some(Self::row_to_entry(row)?));
        }

        Ok(None)
    }

    fn list(&self) -> Result<Vec<RegistryEntry>, Box<dyn Error>> {
        self.load().map(|r| r.entries)
    }
}

// ============================================================================
// Factory Function
// ============================================================================

/// Opens a registry store based on backend type
pub fn open_store(
    backend: RegistryBackend,
    path: &Path,
) -> Result<Box<dyn RegistryStore>, Box<dyn Error>> {
    match backend {
        RegistryBackend::Json => Ok(Box::new(JsonRegistryStore {
            path: path.to_path_buf(),
        })),
        RegistryBackend::Sqlite => Ok(Box::new(SqliteRegistryStore::open(path)?)),
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Verifiziert, ob ein Manifest- und Proof-Hash in einer Registry-Datei existiert
pub fn verify_entry_from_file(
    registry_path: &str,
    manifest_hash: &str,
    proof_hash: &str,
) -> Result<bool, Box<dyn Error>> {
    let registry = Registry::load(registry_path)?;
    Ok(registry.verify_entry(manifest_hash, proof_hash))
}

/// Berechnet SHA3-256 Hash einer Datei
pub fn compute_file_hash<P: AsRef<Path>>(path: P) -> Result<String, Box<dyn Error>> {
    use sha3::{Digest, Sha3_256};
    let content = fs::read(path)?;
    let mut hasher = Sha3_256::new();
    hasher.update(&content);
    let hash = hasher.finalize();
    Ok(format!("0x{}", hex::encode(hash)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new() {
        let registry = Registry::new();
        assert_eq!(registry.registry_version, "1.0");
        assert_eq!(registry.entries.len(), 0);
    }

    #[test]
    fn test_registry_add_entry() {
        let mut registry = Registry::new();
        let id = registry.add_entry("0xabc123".to_string(), "0xdef456".to_string(), None);
        assert_eq!(id, "proof_001");
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_registry_find_entry() {
        let mut registry = Registry::new();
        registry.add_entry("0xabc123".to_string(), "0xdef456".to_string(), None);

        let found = registry.find_entry("0xabc123", "0xdef456");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "proof_001");

        let not_found = registry.find_entry("0xwrong", "0xhash");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_registry_verify_entry() {
        let mut registry = Registry::new();
        registry.add_entry("0xabc123".to_string(), "0xdef456".to_string(), None);

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

        let temp_path = std::env::temp_dir().join("test_registry_store.json");
        registry.save(&temp_path).unwrap();

        let loaded = Registry::load(&temp_path).unwrap();
        assert_eq!(loaded.count(), 1);
        assert_eq!(loaded.entries[0].manifest_hash, "0xabc123");

        std::fs::remove_file(&temp_path).ok();
    }
}
