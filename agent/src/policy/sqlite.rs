#![allow(dead_code)]
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use rusqlite::{params, Connection, OptionalExtension};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use super::metadata::{CompiledPolicy, PolicyMetadata, PolicyStatus};
use super::store::{compute_policy_hash, now_iso8601, PolicyStore};
use crate::policy::Policy;

/// SQLite-based Policy Store (Production)
/// Note: rusqlite::Connection is not Sync, so we wrap it in Arc<Mutex<>>
pub struct SqlitePolicyStore {
    conn: Arc<Mutex<Connection>>,
}

impl SqlitePolicyStore {
    pub fn new(database_path: &str) -> Result<Self> {
        let conn = Connection::open(database_path)?;

        // Enable WAL mode for concurrent access
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;

        // Run migrations
        conn.execute_batch(include_str!(
            "../../migrations/001_create_policies_table.sql"
        ))?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn status_to_string(status: PolicyStatus) -> &'static str {
        match status {
            PolicyStatus::Active => "active",
            PolicyStatus::Deprecated => "deprecated",
            PolicyStatus::Draft => "draft",
        }
    }

    fn status_from_string(s: &str) -> PolicyStatus {
        match s {
            "active" => PolicyStatus::Active,
            "deprecated" => PolicyStatus::Deprecated,
            "draft" => PolicyStatus::Draft,
            _ => PolicyStatus::Active,
        }
    }
}

#[async_trait]
impl PolicyStore for SqlitePolicyStore {
    async fn save(&self, policy: Policy) -> Result<PolicyMetadata> {
        let hash = compute_policy_hash(&policy)?;
        let now = now_iso8601();

        let conn = self.conn.lock().expect("Failed to acquire policy store lock");

        // Check if policy with same hash exists
        let existing: Option<(String, String)> = conn
            .query_row(
                "SELECT id, created_at FROM policies WHERE hash = ?1",
                params![&hash],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;

        if let Some((existing_id, created)) = existing {
            // Update existing
            conn.execute(
                "UPDATE policies SET updated_at = ?1 WHERE id = ?2",
                params![&now, &existing_id],
            )?;

            return Ok(PolicyMetadata {
                id: Uuid::parse_str(&existing_id)?,
                name: policy.name,
                version: policy.version,
                hash,
                status: PolicyStatus::Active,
                created_at: created,
                updated_at: now,
                description: Some(policy.notes),
            });
        }

        // Insert new
        let id = Uuid::new_v4();
        let policy_json = serde_json::to_string(&policy)?;

        conn.execute(
            "INSERT INTO policies (id, name, version, hash, status, created_at, updated_at, description, policy_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                id.to_string(),
                &policy.name,
                &policy.version,
                &hash,
                "active",
                &now,
                &now,
                &policy.notes,
                &policy_json,
            ],
        )?;

        Ok(PolicyMetadata {
            id,
            name: policy.name,
            version: policy.version,
            hash,
            status: PolicyStatus::Active,
            created_at: now.clone(),
            updated_at: now,
            description: Some(policy.notes),
        })
    }

    async fn get(&self, id: &str) -> Result<Option<CompiledPolicy>> {
        let conn = self.conn.lock().expect("Failed to acquire policy store lock");

        let result: Option<(String, String, String, String, String, String, String, String, Option<Vec<u8>>)> = conn
            .query_row(
                "SELECT id, name, version, hash, status, created_at, updated_at, policy_json, compiled_bytes
                 FROM policies WHERE id = ?1",
                params![id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get(7)?,
                        row.get(8)?,
                    ))
                },
            )
            .optional()?;

        if let Some((
            id_str,
            name,
            version,
            hash,
            status,
            created_at,
            updated_at,
            policy_json,
            compiled_bytes,
        )) = result
        {
            let policy: Policy = serde_json::from_str(&policy_json)?;
            let status = Self::status_from_string(&status);

            Ok(Some(CompiledPolicy {
                metadata: PolicyMetadata {
                    id: Uuid::parse_str(&id_str)?,
                    name,
                    version,
                    hash,
                    status,
                    created_at,
                    updated_at,
                    description: Some(policy.notes.clone()),
                },
                policy,
                compiled_bytes,
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_by_hash(&self, hash: &str) -> Result<Option<CompiledPolicy>> {
        let id: Option<String> = {
            let conn = self.conn.lock().expect("Failed to acquire policy store lock");
            conn.query_row(
                "SELECT id FROM policies WHERE hash = ?1",
                params![hash],
                |row| row.get(0),
            )
            .optional()?
        };

        if let Some(id) = id {
            return self.get(&id).await;
        }
        Ok(None)
    }

    async fn list(&self, status_filter: Option<PolicyStatus>) -> Result<Vec<PolicyMetadata>> {
        let conn = self.conn.lock().expect("Failed to acquire policy store lock");

        let query = if let Some(status) = status_filter {
            let status_str = Self::status_to_string(status);
            format!(
                "SELECT id, name, version, hash, status, created_at, updated_at, description
                 FROM policies WHERE status = '{}'
                 ORDER BY created_at DESC",
                status_str
            )
        } else {
            "SELECT id, name, version, hash, status, created_at, updated_at, description
             FROM policies
             ORDER BY created_at DESC"
                .to_string()
        };

        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<String>>(7)?,
            ))
        })?;

        let mut result = Vec::new();
        for row in rows {
            let (id_str, name, version, hash, status, created_at, updated_at, description) = row?;
            let status = Self::status_from_string(&status);

            result.push(PolicyMetadata {
                id: Uuid::parse_str(&id_str)?,
                name,
                version,
                hash,
                status,
                created_at,
                updated_at,
                description,
            });
        }

        Ok(result)
    }

    async fn set_status(&self, id: &str, status: PolicyStatus) -> Result<()> {
        let conn = self.conn.lock().expect("Failed to acquire policy store lock");

        let status_str = Self::status_to_string(status);
        let now = now_iso8601();

        let rows_affected = conn.execute(
            "UPDATE policies SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status_str, &now, id],
        )?;

        if rows_affected == 0 {
            return Err(anyhow!("Policy not found: {}", id));
        }

        Ok(())
    }
}
