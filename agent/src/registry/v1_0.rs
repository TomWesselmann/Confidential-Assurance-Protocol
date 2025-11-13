use base64::{engine::general_purpose, Engine};
/// Registry-Modul für lokale Proof-Verwaltung
///
/// Dieses Modul verwaltet eine lokale Registry (JSON-basiert) für ZK-Proofs,
/// Manifeste und Timestamps. Es ermöglicht das Hinzufügen, Auflisten und
/// Verifizieren von Registry-Einträgen.
use chrono::Utc;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::error::Error;
use std::fs;
use std::path::Path;

use crate::keys;

/// Registry-Eintrag für einen einzelnen Proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub id: String,
    pub manifest_hash: String,
    pub proof_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp_file: Option<String>,
    pub registered_at: String, // RFC3339
    /// Ed25519 signature over entry_hash (optional for backward compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    /// Base64-encoded Ed25519 public key (optional for backward compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,

    // BLOB Store Fields (v0.9)
    /// BLAKE3 hash of manifest BLOB
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob_manifest: Option<String>,
    /// BLAKE3 hash of proof BLOB
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob_proof: Option<String>,
    /// BLAKE3 hash of WASM verifier BLOB
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob_wasm: Option<String>,
    /// SHA3-256 hash of ABI JSON BLOB
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob_abi: Option<String>,

    // Self-Verification Fields (v0.9)
    /// Self-verification status: unknown, ok, fail
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selfverify_status: Option<String>,
    /// RFC3339 timestamp of last self-verification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selfverify_at: Option<String>,
    /// Verifier name (e.g., "cap-wasm-verifier")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verifier_name: Option<String>,
    /// Verifier version (e.g., "1.0.0")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verifier_version: Option<String>,

    // Key Management Fields (v0.10)
    /// Key Identifier (16 bytes = 32 hex chars, derived from public key)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kid: Option<String>,
    /// Signature scheme (e.g., "ed25519")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_scheme: Option<String>,
}

/// Lokale Registry-Struktur
#[derive(Debug, Serialize, Deserialize)]
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
            signature: None,
            public_key: None,
            // BLOB fields (v0.9) - initially None for backward compatibility
            blob_manifest: None,
            blob_proof: None,
            blob_wasm: None,
            blob_abi: None,
            // Self-verify fields (v0.9) - initially None
            selfverify_status: None,
            selfverify_at: None,
            verifier_name: None,
            verifier_version: None,
            // Key management fields (v0.10) - initially None
            kid: None,
            signature_scheme: None,
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
    pub fn find_entry(&self, manifest_hash: &str, proof_hash: &str) -> Option<&RegistryEntry> {
        self.entries
            .iter()
            .find(|e| e.manifest_hash == manifest_hash && e.proof_hash == proof_hash)
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

// ============================================================================
// Registry Entry Signing (v0.8.0)
// ============================================================================

/// Berechnet BLAKE3-Hash des Entry-Cores (ohne Signatur-Felder)
///
/// # Argumente
/// * `entry` - Registry-Eintrag
///
/// # Rückgabe
/// BLAKE3-Hash als Bytes
fn compute_entry_core_hash(entry: &RegistryEntry) -> Result<Vec<u8>, Box<dyn Error>> {
    // Create core entry without signature fields for deterministic hashing
    #[derive(Serialize)]
    struct EntryCore<'a> {
        id: &'a str,
        manifest_hash: &'a str,
        proof_hash: &'a str,
        timestamp_file: &'a Option<String>,
        registered_at: &'a str,
    }

    let core = EntryCore {
        id: &entry.id,
        manifest_hash: &entry.manifest_hash,
        proof_hash: &entry.proof_hash,
        timestamp_file: &entry.timestamp_file,
        registered_at: &entry.registered_at,
    };

    let json = serde_json::to_vec(&core)?;
    let hash = blake3::hash(&json);
    Ok(hash.as_bytes().to_vec())
}

/// Signiert einen Registry-Eintrag mit Ed25519
///
/// # Argumente
/// * `entry` - Mutable Referenz auf Registry-Eintrag
/// * `signing_key` - Ed25519 Signing Key
///
/// # Rückgabe
/// Ok(()) wenn erfolgreich, Fehler sonst
pub fn sign_entry(
    entry: &mut RegistryEntry,
    signing_key: &SigningKey,
) -> Result<(), Box<dyn Error>> {
    // Compute hash of entry core (without signature fields)
    let entry_hash = compute_entry_core_hash(entry)?;

    // Sign the hash
    let signature = signing_key.sign(&entry_hash);

    // Encode signature and public key as base64
    let sig_b64 = general_purpose::STANDARD.encode(signature.to_bytes());
    let pubkey_b64 = general_purpose::STANDARD.encode(signing_key.verifying_key().to_bytes());

    // Derive KID from public key (v0.10)
    let kid = keys::derive_kid(&pubkey_b64)?;

    // Update entry with signature and key metadata
    entry.signature = Some(sig_b64);
    entry.public_key = Some(pubkey_b64);
    entry.kid = Some(kid);
    entry.signature_scheme = Some("ed25519".to_string());

    Ok(())
}

/// Verifiziert die Signatur eines Registry-Eintrags
///
/// # Argumente
/// * `entry` - Registry-Eintrag
///
/// # Rückgabe
/// Ok(true) wenn Signatur gültig, Ok(false) wenn keine Signatur, Err bei Fehler
pub fn verify_entry_signature(entry: &RegistryEntry) -> Result<bool, Box<dyn Error>> {
    // Check if signature exists
    let (sig_b64, pubkey_b64) = match (&entry.signature, &entry.public_key) {
        (Some(s), Some(p)) => (s, p),
        _ => return Ok(false), // No signature present (backward compatibility)
    };

    // Decode signature and public key
    let sig_bytes = general_purpose::STANDARD.decode(sig_b64)?;
    let pubkey_bytes = general_purpose::STANDARD.decode(pubkey_b64)?;

    // Parse Ed25519 types
    let signature = Signature::from_bytes(
        &sig_bytes
            .try_into()
            .map_err(|_| "Invalid signature length")?,
    );
    let verifying_key = VerifyingKey::from_bytes(
        &pubkey_bytes
            .try_into()
            .map_err(|_| "Invalid public key length")?,
    )?;

    // Compute entry hash
    let entry_hash = compute_entry_core_hash(entry)?;

    // Verify signature
    verifying_key.verify(&entry_hash, &signature)?;

    Ok(true)
}

/// Validiert den Status eines Schlüssels für Registry-Operationen
///
/// # Argumente
/// * `kid` - Key Identifier
/// * `key_store_path` - Pfad zum Key Store Verzeichnis
///
/// # Rückgabe
/// Ok(()) wenn Key aktiv, Err wenn nicht aktiv, retired, revoked oder nicht gefunden
pub fn validate_key_status(kid: &str, key_store_path: &str) -> Result<(), Box<dyn Error>> {
    use crate::keys::KeyStore;

    let store = KeyStore::new(key_store_path)?;

    match store.find_by_kid(kid)? {
        Some(key_meta) => match key_meta.status.as_str() {
            "active" => Ok(()),
            "retired" => {
                Err(format!("Key {} is retired and cannot be used for new entries", kid).into())
            }
            "revoked" => Err(format!("Key {} is revoked and cannot be used", kid).into()),
            other => Err(format!("Key {} has unknown status: {}", kid, other).into()),
        },
        None => Err(format!("Key not found in store: {}", kid).into()),
    }
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

// ============================================================================
// Timestamp Provider Abstraction (Pluggable Interface)
// ============================================================================

/// Timestamp Provider Trait - allows pluggable timestamp sources
#[allow(dead_code)]
pub trait TimestampProvider {
    /// Creates a timestamp for the given audit tip
    fn create(&self, audit_tip_hex: &str) -> Result<Timestamp, Box<dyn Error>>;

    /// Verifies a timestamp against an audit tip
    fn verify(&self, audit_tip_hex: &str, ts: &Timestamp) -> Result<bool, Box<dyn Error>>;

    /// Returns the provider name
    fn name(&self) -> &'static str;
}

/// Provider kind selector
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ProviderKind {
    MockRfc3161,
    RealRfc3161 { tsa_url: String },
}

/// Factory function to create a timestamp provider
#[allow(dead_code)]
pub fn make_provider(kind: ProviderKind) -> Box<dyn TimestampProvider> {
    match kind {
        ProviderKind::MockRfc3161 => Box::new(MockRfc3161Provider),
        ProviderKind::RealRfc3161 { tsa_url } => Box::new(RealRfc3161Provider { tsa_url }),
    }
}

/// Helper to parse provider from CLI string
#[allow(dead_code)]
pub fn provider_from_cli(kind: &str, tsa_url: Option<String>) -> ProviderKind {
    match kind {
        "rfc3161" => ProviderKind::RealRfc3161 {
            tsa_url: tsa_url.unwrap_or_else(|| "https://freetsa.org/tsr".to_string()),
        },
        _ => ProviderKind::MockRfc3161,
    }
}

// ============================================================================
// Mock RFC3161 Provider (current behavior)
// ============================================================================

/// Mock RFC3161 Timestamp Provider
#[allow(dead_code)]
pub struct MockRfc3161Provider;

impl TimestampProvider for MockRfc3161Provider {
    fn create(&self, audit_tip_hex: &str) -> Result<Timestamp, Box<dyn Error>> {
        let now = Utc::now().to_rfc3339();
        let mut hasher = Sha3_256::new();
        hasher.update(audit_tip_hex.as_bytes());
        hasher.update(now.as_bytes());
        let sig = hasher.finalize();
        let signature = hex::encode(&sig[..]);

        Ok(Timestamp {
            version: "tsr.v1".to_string(),
            audit_tip_hex: audit_tip_hex.to_string(),
            created_at: now,
            tsa: "local-mock".to_string(),
            signature,
            status: "ok".to_string(),
        })
    }

    fn verify(&self, audit_tip_hex: &str, ts: &Timestamp) -> Result<bool, Box<dyn Error>> {
        if ts.audit_tip_hex != audit_tip_hex {
            return Ok(false);
        }

        // Verify mock signature
        let mut hasher = Sha3_256::new();
        hasher.update(ts.audit_tip_hex.as_bytes());
        hasher.update(ts.created_at.as_bytes());
        let expected_sig = hasher.finalize();
        let expected_sig_hex = hex::encode(&expected_sig[..]);

        Ok(ts.signature == expected_sig_hex && ts.status == "ok")
    }

    fn name(&self) -> &'static str {
        "mock_rfc3161"
    }
}

// ============================================================================
// Real RFC3161 Provider (stub for future implementation)
// ============================================================================

/// Real RFC3161 Timestamp Provider (not yet implemented)
#[allow(dead_code)]
pub struct RealRfc3161Provider {
    pub tsa_url: String,
}

impl TimestampProvider for RealRfc3161Provider {
    fn create(&self, _audit_tip_hex: &str) -> Result<Timestamp, Box<dyn Error>> {
        Err(format!(
            "Real RFC3161 provider not yet implemented (tsa_url={}). Use --provider mock for now.",
            self.tsa_url
        )
        .into())
    }

    fn verify(&self, _audit_tip_hex: &str, _ts: &Timestamp) -> Result<bool, Box<dyn Error>> {
        Err("Real RFC3161 provider not yet implemented. Use --provider mock for now.".into())
    }

    fn name(&self) -> &'static str {
        "real_rfc3161"
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
             ORDER BY registered_at DESC"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(RegistryEntry {
                id: row.get(0)?,
                manifest_hash: row.get(1)?,
                proof_hash: row.get(2)?,
                timestamp_file: row.get(3)?,
                registered_at: row.get(4)?,
                signature: row.get(5).ok(),
                public_key: row.get(6).ok(),
                // BLOB fields (v0.9)
                blob_manifest: row.get(7).ok(),
                blob_proof: row.get(8).ok(),
                blob_wasm: row.get(9).ok(),
                blob_abi: row.get(10).ok(),
                // Self-verify fields (v0.9)
                selfverify_status: row.get(11).ok(),
                selfverify_at: row.get(12).ok(),
                verifier_name: row.get(13).ok(),
                verifier_version: row.get(14).ok(),
                // Key management fields (v0.10)
                kid: row.get(15).ok(),
                signature_scheme: row.get(16).ok(),
            })
        })?;

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
            tx.execute(
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
        }

        tx.commit()?;
        Ok(())
    }

    fn add_entry(&self, entry: RegistryEntry) -> Result<(), Box<dyn Error>> {
        let conn = self.conn.borrow();
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
             LIMIT 1"
        )?;

        let mut rows = stmt.query(rusqlite::params![manifest_hash, proof_hash])?;

        if let Some(row) = rows.next()? {
            return Ok(Some(RegistryEntry {
                id: row.get(0)?,
                manifest_hash: row.get(1)?,
                proof_hash: row.get(2)?,
                timestamp_file: row.get(3)?,
                registered_at: row.get(4)?,
                signature: row.get(5).ok(),
                public_key: row.get(6).ok(),
                // BLOB fields (v0.9)
                blob_manifest: row.get(7).ok(),
                blob_proof: row.get(8).ok(),
                blob_wasm: row.get(9).ok(),
                blob_abi: row.get(10).ok(),
                // Self-verify fields (v0.9)
                selfverify_status: row.get(11).ok(),
                selfverify_at: row.get(12).ok(),
                verifier_name: row.get(13).ok(),
                verifier_version: row.get(14).ok(),
                // Key management fields (v0.10)
                kid: row.get(15).ok(),
                signature_scheme: row.get(16).ok(),
            }));
        }

        Ok(None)
    }

    fn list(&self) -> Result<Vec<RegistryEntry>, Box<dyn Error>> {
        self.load().map(|r| r.entries)
    }
}

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

    // Registry Entry Signing Tests (v0.8.0)

    #[test]
    fn test_sign_and_verify_roundtrip() {
        // Create a test entry
        let mut entry = RegistryEntry {
            id: "proof_001".to_string(),
            manifest_hash: "0xabc123".to_string(),
            proof_hash: "0xdef456".to_string(),
            timestamp_file: Some("test.tsr".to_string()),
            registered_at: chrono::Utc::now().to_rfc3339(),
            signature: None,
            public_key: None,
            blob_manifest: None,
            blob_proof: None,
            blob_wasm: None,
            blob_abi: None,
            selfverify_status: None,
            selfverify_at: None,
            verifier_name: None,
            verifier_version: None,
            kid: None,
            signature_scheme: None,
        };

        // Generate a signing key
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&[42u8; 32]);

        // Sign the entry
        sign_entry(&mut entry, &signing_key).unwrap();

        // Verify signature is present
        assert!(entry.signature.is_some());
        assert!(entry.public_key.is_some());

        // Verify signature
        let valid = verify_entry_signature(&entry).unwrap();
        assert!(valid, "Signature should be valid");
    }

    #[test]
    fn test_tampered_entry_fails_verification() {
        // Create and sign an entry
        let mut entry = RegistryEntry {
            id: "proof_001".to_string(),
            manifest_hash: "0xabc123".to_string(),
            proof_hash: "0xdef456".to_string(),
            timestamp_file: None,
            registered_at: chrono::Utc::now().to_rfc3339(),
            signature: None,
            public_key: None,
            blob_manifest: None,
            blob_proof: None,
            blob_wasm: None,
            blob_abi: None,
            selfverify_status: None,
            selfverify_at: None,
            verifier_name: None,
            verifier_version: None,
            kid: None,
            signature_scheme: None,
        };

        let signing_key = ed25519_dalek::SigningKey::from_bytes(&[42u8; 32]);
        sign_entry(&mut entry, &signing_key).unwrap();

        // Tamper with the entry
        entry.manifest_hash = "0xTAMPERED".to_string();

        // Verification should fail
        let result = verify_entry_signature(&entry);
        assert!(result.is_err(), "Tampered entry should fail verification");
    }

    #[test]
    fn test_missing_signature_returns_false() {
        // Create an entry without signature
        let entry = RegistryEntry {
            id: "proof_001".to_string(),
            manifest_hash: "0xabc123".to_string(),
            proof_hash: "0xdef456".to_string(),
            timestamp_file: None,
            registered_at: chrono::Utc::now().to_rfc3339(),
            signature: None,
            public_key: None,
            blob_manifest: None,
            blob_proof: None,
            blob_wasm: None,
            blob_abi: None,
            selfverify_status: None,
            selfverify_at: None,
            verifier_name: None,
            verifier_version: None,
            kid: None,
            signature_scheme: None,
        };

        // Verify should return Ok(false) for backward compatibility
        let valid = verify_entry_signature(&entry).unwrap();
        assert!(!valid, "Entry without signature should return false");
    }
}
