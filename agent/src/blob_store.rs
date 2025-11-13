/// BLOB Store - Content-Addressable Storage
///
/// Speichert Proof-Package-Komponenten (Manifest, Proof, WASM, ABI) als
/// deduplizierte BLOBs mit BLAKE3/SHA3-256 Content-Addressing.
use anyhow::{anyhow, Result};
use rusqlite::{params, Connection};
use std::path::Path;

use crate::crypto;

/// BLOB Store Trait
pub trait BlobStore {
    /// Put a BLOB into the store, returns blob_id
    fn put(&mut self, data: &[u8], media_type: &str) -> Result<String>;

    /// Get a BLOB from the store by blob_id
    fn get(&self, blob_id: &str) -> Result<Vec<u8>>;

    /// Check if a BLOB exists
    fn exists(&self, blob_id: &str) -> bool;

    /// Increment reference count for a BLOB
    fn pin(&mut self, blob_id: &str) -> Result<()>;

    /// Decrement reference count for a BLOB
    #[allow(dead_code)]
    fn unpin(&mut self, blob_id: &str) -> Result<()>;

    /// Garbage collect BLOBs with refcount=0
    fn gc(&mut self, dry_run: bool) -> Result<Vec<String>>;

    /// List all BLOBs with metadata
    fn list(&self) -> Result<Vec<BlobMetadata>>;
}

/// BLOB Metadata
#[derive(Debug, Clone)]
pub struct BlobMetadata {
    pub blob_id: String,
    pub size: usize,
    pub media_type: String,
    pub refcount: i64,
}

/// SQLite-based BLOB Store
pub struct SqliteBlobStore {
    conn: Connection,
}

impl SqliteBlobStore {
    /// Create or open a BLOB store
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;

        // Enable WAL mode for better concurrency
        conn.execute_batch("PRAGMA journal_mode=WAL")?;

        // Create blobs table if not exists
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS blobs (
                blob_id TEXT PRIMARY KEY,
                size INTEGER NOT NULL,
                media_type TEXT NOT NULL,
                data BLOB NOT NULL,
                refcount INTEGER NOT NULL DEFAULT 0
            )
            "#,
            [],
        )?;

        // Create index on refcount for GC
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_blobs_refcount ON blobs(refcount)",
            [],
        )?;

        Ok(Self { conn })
    }

    /// Compute BLAKE3 hash for BLOB ID
    fn compute_blob_id(data: &[u8]) -> String {
        crypto::hex_lower_prefixed32(crypto::blake3_256(data))
    }
}

impl BlobStore for SqliteBlobStore {
    fn put(&mut self, data: &[u8], media_type: &str) -> Result<String> {
        let blob_id = Self::compute_blob_id(data);
        let size = data.len();

        // Check if BLOB already exists
        let exists: bool = self
            .conn
            .query_row(
                "SELECT 1 FROM blobs WHERE blob_id = ?1",
                params![&blob_id],
                |_| Ok(true),
            )
            .unwrap_or(false);

        if !exists {
            // Insert new BLOB
            self.conn.execute(
                "INSERT INTO blobs (blob_id, size, media_type, data, refcount) VALUES (?1, ?2, ?3, ?4, 0)",
                params![&blob_id, size as i64, media_type, data],
            )?;
        }
        // If exists, automatic deduplication (no-op)

        Ok(blob_id)
    }

    fn get(&self, blob_id: &str) -> Result<Vec<u8>> {
        let data: Vec<u8> = self
            .conn
            .query_row(
                "SELECT data FROM blobs WHERE blob_id = ?1",
                params![blob_id],
                |row| row.get(0),
            )
            .map_err(|_| anyhow!("BLOB not found: {}", blob_id))?;

        Ok(data)
    }

    fn exists(&self, blob_id: &str) -> bool {
        self.conn
            .query_row(
                "SELECT 1 FROM blobs WHERE blob_id = ?1",
                params![blob_id],
                |_| Ok(true),
            )
            .unwrap_or(false)
    }

    fn pin(&mut self, blob_id: &str) -> Result<()> {
        let rows_affected = self.conn.execute(
            "UPDATE blobs SET refcount = refcount + 1 WHERE blob_id = ?1",
            params![blob_id],
        )?;

        if rows_affected == 0 {
            return Err(anyhow!("Cannot pin non-existent BLOB: {}", blob_id));
        }

        Ok(())
    }

    fn unpin(&mut self, blob_id: &str) -> Result<()> {
        let rows_affected = self.conn.execute(
            "UPDATE blobs SET refcount = MAX(0, refcount - 1) WHERE blob_id = ?1",
            params![blob_id],
        )?;

        if rows_affected == 0 {
            return Err(anyhow!("Cannot unpin non-existent BLOB: {}", blob_id));
        }

        Ok(())
    }

    fn gc(&mut self, dry_run: bool) -> Result<Vec<String>> {
        // Find all BLOBs with refcount=0
        let mut stmt = self
            .conn
            .prepare("SELECT blob_id FROM blobs WHERE refcount = 0")?;

        let blob_ids: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        if !dry_run && !blob_ids.is_empty() {
            // Delete BLOBs with refcount=0
            self.conn
                .execute("DELETE FROM blobs WHERE refcount = 0", [])?;
        }

        Ok(blob_ids)
    }

    fn list(&self) -> Result<Vec<BlobMetadata>> {
        let mut stmt = self
            .conn
            .prepare("SELECT blob_id, size, media_type, refcount FROM blobs ORDER BY blob_id")?;

        let blobs = stmt
            .query_map([], |row| {
                Ok(BlobMetadata {
                    blob_id: row.get(0)?,
                    size: row.get::<_, i64>(1)? as usize,
                    media_type: row.get(2)?,
                    refcount: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(blobs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blob_put_get() {
        let mut store = SqliteBlobStore::new(":memory:").unwrap();

        let data = b"test blob data";
        let blob_id = store.put(data, "text/plain").unwrap();

        assert!(blob_id.starts_with("0x"));
        assert_eq!(blob_id.len(), 66); // 0x + 64 hex chars

        let retrieved = store.get(&blob_id).unwrap();
        assert_eq!(retrieved, data);
    }

    #[test]
    fn test_blob_deduplication() {
        let mut store = SqliteBlobStore::new(":memory:").unwrap();

        let data = b"duplicate data";
        let id1 = store.put(data, "text/plain").unwrap();
        let id2 = store.put(data, "text/plain").unwrap();

        // Same data = same blob_id
        assert_eq!(id1, id2);

        // Only one BLOB stored
        let blobs = store.list().unwrap();
        assert_eq!(blobs.len(), 1);
    }

    #[test]
    fn test_blob_pin_unpin() {
        let mut store = SqliteBlobStore::new(":memory:").unwrap();

        let data = b"pinned data";
        let blob_id = store.put(data, "application/octet-stream").unwrap();

        // Initial refcount = 0
        let blobs = store.list().unwrap();
        assert_eq!(blobs[0].refcount, 0);

        // Pin twice
        store.pin(&blob_id).unwrap();
        store.pin(&blob_id).unwrap();

        let blobs = store.list().unwrap();
        assert_eq!(blobs[0].refcount, 2);

        // Unpin once
        store.unpin(&blob_id).unwrap();

        let blobs = store.list().unwrap();
        assert_eq!(blobs[0].refcount, 1);
    }

    #[test]
    fn test_blob_gc() {
        let mut store = SqliteBlobStore::new(":memory:").unwrap();

        let data1 = b"gc test 1";
        let data2 = b"gc test 2";

        let id1 = store.put(data1, "text/plain").unwrap();
        let id2 = store.put(data2, "text/plain").unwrap();

        // Pin only id2
        store.pin(&id2).unwrap();

        // Dry-run GC
        let gc_list = store.gc(true).unwrap();
        assert_eq!(gc_list.len(), 1);
        assert_eq!(gc_list[0], id1);

        // BLOBs still exist
        assert_eq!(store.list().unwrap().len(), 2);

        // Real GC
        store.gc(false).unwrap();

        // id1 deleted, id2 remains
        assert_eq!(store.list().unwrap().len(), 1);
        assert!(store.exists(&id2));
        assert!(!store.exists(&id1));
    }

    #[test]
    fn test_blob_exists() {
        let mut store = SqliteBlobStore::new(":memory:").unwrap();

        let data = b"exists test";
        let blob_id = store.put(data, "text/plain").unwrap();

        assert!(store.exists(&blob_id));
        assert!(!store.exists("0x0000000000000000000000000000000000000000000000000000000000000000"));
    }

    #[test]
    fn test_blob_list() {
        let mut store = SqliteBlobStore::new(":memory:").unwrap();

        store.put(b"blob1", "text/plain").unwrap();
        store.put(b"blob2", "application/json").unwrap();
        store.put(b"blob3", "application/wasm").unwrap();

        let blobs = store.list().unwrap();
        assert_eq!(blobs.len(), 3);

        // Check metadata
        for blob in &blobs {
            assert!(blob.blob_id.starts_with("0x"));
            assert!(blob.size > 0);
            assert!(blob.refcount >= 0);
        }
    }
}
