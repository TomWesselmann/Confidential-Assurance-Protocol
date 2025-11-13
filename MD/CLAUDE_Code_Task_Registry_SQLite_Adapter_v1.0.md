# Claude Task: Registry Adapter (JSON ⇄ SQLite) — Drop‑in Backend (v1.0)

## 🎯 Goal
Introduce a **pluggable Registry store** with two interchangeable backends:
- `json` (current behavior, default / fallback)
- `sqlite` (new; more robust queries & concurrency-safe)

No feature changes to business logic or CLI semantics — **only storage** is swapped via a flag or config.

---

## 🧱 Scope Overview

### New Abstraction
Create a trait in `registry.rs`:
```rust
pub trait RegistryStore {
    fn load(&self) -> anyhow::Result<Registry>;
    fn save(&self, reg: &Registry) -> anyhow::Result<()>;
    fn add_entry(&self, entry: RegistryEntry) -> anyhow::Result<()>;
    fn find_by_hashes(&self, manifest_hash: &str, proof_hash: &str) -> anyhow::Result<Option<RegistryEntry>>;
    fn list(&self) -> anyhow::Result<Vec<RegistryEntry>>;
}
```

Implementations:
- `JsonRegistryStore { path: PathBuf }`
- `SqliteRegistryStore { path: PathBuf }`

### New CLI
- Global flag or subcommand flag: `--registry-backend json|sqlite` (defaults to `json` for backward compatibility).
- New subcommand: `cap registry migrate --to sqlite --from json --in build/registry.json --out build/registry.sqlite`

---

## 🗄️ Data Model

```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Registry {
    pub registry_version: String,      // "1.0"
    pub entries: Vec<RegistryEntry>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RegistryEntry {
    pub id: String,                    // e.g., "proof_001"
    pub manifest_hash: String,         // 0x… (SHA3-256)
    pub proof_hash: String,            // 0x…
    pub timestamp_file: Option<String>,// path to .tsr
    pub registered_at: String,         // RFC3339
}
```

### SQLite Schema
```sql
CREATE TABLE IF NOT EXISTS registry_meta (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS registry_entries (
  id TEXT PRIMARY KEY,
  manifest_hash TEXT NOT NULL,
  proof_hash TEXT NOT NULL,
  timestamp_file TEXT,
  registered_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_registry_hashes
  ON registry_entries (manifest_hash, proof_hash);
```

Store `registry_version` in `registry_meta(key='registry_version')`.

---

## 🔧 Implementation

### 1) Trait & Selector
**File:** `agent/src/registry.rs`
```rust
pub enum RegistryBackend { Json, Sqlite }

pub fn open_store(backend: RegistryBackend, path: &std::path::Path) -> anyhow::Result<Box<dyn RegistryStore>> {
    match backend {
        RegistryBackend::Json => Ok(Box::new(JsonRegistryStore { path: path.to_path_buf() })),
        RegistryBackend::Sqlite => Ok(Box::new(SqliteRegistryStore::open(path)?)),
    }
}
```

### 2) JSON Store (existing behavior)
```rust
pub struct JsonRegistryStore { pub path: std::path::PathBuf }

impl RegistryStore for JsonRegistryStore {
    fn load(&self) -> anyhow::Result<Registry> {
        if !self.path.exists() { return Ok(Registry { registry_version: "1.0".into(), entries: vec![] }); }
        let v: Registry = serde_json::from_str(&std::fs::read_to_string(&self.path)?)?;
        Ok(v)
    }
    fn save(&self, reg: &Registry) -> anyhow::Result<()> {
        std::fs::write(&self.path, serde_json::to_string_pretty(reg)?)?; Ok(())
    }
    fn add_entry(&self, entry: RegistryEntry) -> anyhow::Result<()> {
        let mut r = self.load()?; r.entries.push(entry); self.save(&r)
    }
    fn find_by_hashes(&self, m: &str, p: &str) -> anyhow::Result<Option<RegistryEntry>> {
        let r = self.load()?;
        Ok(r.entries.into_iter().find(|e| e.manifest_hash == m && e.proof_hash == p))
    }
    fn list(&self) -> anyhow::Result<Vec<RegistryEntry>> {
        Ok(self.load()?.entries)
    }
}
```

### 3) SQLite Store
**Dependency:** `rusqlite = { version = "0.31", features = ["bundled"] }` (or without `"bundled"` if system SQLite is guaranteed).

```rust
pub struct SqliteRegistryStore { conn: rusqlite::Connection, path: std::path::PathBuf }

impl SqliteRegistryStore {
    pub fn open(path: &std::path::Path) -> anyhow::Result<Self> {
        let conn = rusqlite::Connection::open(path)?;
        conn.execute_batch(r#"
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            CREATE TABLE IF NOT EXISTS registry_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS registry_entries (
                id TEXT PRIMARY KEY,
                manifest_hash TEXT NOT NULL,
                proof_hash TEXT NOT NULL,
                timestamp_file TEXT,
                registered_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_registry_hashes ON registry_entries (manifest_hash, proof_hash);
        "#)?;
        // ensure version
        conn.execute("INSERT OR IGNORE INTO registry_meta(key,value) VALUES('registry_version','1.0')", [])?;
        Ok(Self { conn, path: path.to_path_buf() })
    }
}

impl RegistryStore for SqliteRegistryStore {
    fn load(&self) -> anyhow::Result<Registry> {
        let mut stmt = self.conn.prepare("SELECT id, manifest_hash, proof_hash, timestamp_file, registered_at FROM registry_entries ORDER BY registered_at DESC")?;
        let rows = stmt.query_map([], |row| Ok(RegistryEntry {
            id: row.get(0)?,
            manifest_hash: row.get(1)?,
            proof_hash: row.get(2)?,
            timestamp_file: row.get(3)?,
            registered_at: row.get(4)?,
        }))?;
        let mut entries = Vec::new();
        for r in rows { entries.push(r?); }
        Ok(Registry { registry_version: "1.0".into(), entries })
    }
    fn save(&self, reg: &Registry) -> anyhow::Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM registry_entries", [])?;
        for e in &reg.entries {
            tx.execute("INSERT OR REPLACE INTO registry_entries(id,manifest_hash,proof_hash,timestamp_file,registered_at) VALUES(?,?,?,?,?)",
                (&e.id, &e.manifest_hash, &e.proof_hash, &e.timestamp_file, &e.registered_at))?;
        }
        tx.commit()?; Ok(())
    }
    fn add_entry(&self, entry: RegistryEntry) -> anyhow::Result<()> {
        self.conn.execute("INSERT OR REPLACE INTO registry_entries(id,manifest_hash,proof_hash,timestamp_file,registered_at) VALUES(?,?,?,?,?)",
            (&entry.id, &entry.manifest_hash, &entry.proof_hash, &entry.timestamp_file, &entry.registered_at))?;
        Ok(())
    }
    fn find_by_hashes(&self, m: &str, p: &str) -> anyhow::Result<Option<RegistryEntry>> {
        let mut stmt = self.conn.prepare("SELECT id, manifest_hash, proof_hash, timestamp_file, registered_at FROM registry_entries WHERE manifest_hash=?1 AND proof_hash=?2 LIMIT 1")?;
        let mut rows = stmt.query((m, p))?;
        if let Some(row) = rows.next()? {
            return Ok(Some(RegistryEntry {
                id: row.get(0)?,
                manifest_hash: row.get(1)?,
                proof_hash: row.get(2)?,
                timestamp_file: row.get(3)?,
                registered_at: row.get(4)?,
            }));
        }
        Ok(None)
    }
    fn list(&self) -> anyhow::Result<Vec<RegistryEntry>> { self.load().map(|r| r.entries) }
}
```

---

## 🖥️ CLI Integration

### Flags
Add a top-level or subcommand flag to select backend:
```bash
# Examples
cap registry list --registry-backend json   --path build/registry.json
cap registry list --registry-backend sqlite --path build/registry.sqlite
```

**Main registration (`main.rs`):**
```rust
.arg(arg!(--"registry-backend" <BACKEND> "json|sqlite").required(false))
.arg(arg!(--path <FILE> "Path to registry file").required(false))
```

Resolve backend + store:
```rust
let backend = match matches.get_one::<String>("registry-backend").map(|s| s.as_str()) {
    Some("sqlite") => RegistryBackend::Sqlite,
    _ => RegistryBackend::Json,
};
let path = matches.get_one::<String>("path").map(std::path::PathBuf::from)
    .unwrap_or_else(|| std::path::PathBuf::from(match backend { RegistryBackend::Json => "build/registry.json", RegistryBackend::Sqlite => "build/registry.sqlite" }));
let store = open_store(backend, &path)?;
```

### Migration Command
```bash
cap registry migrate \
  --from json   --in build/registry.json \
  --to   sqlite --out build/registry.sqlite
```

Implementation sketch (`agent/src/cli/registry_migrate.rs`):
```rust
pub fn run(from_backend: RegistryBackend, from_path: &Path, to_backend: RegistryBackend, to_path: &Path) -> anyhow::Result<()> {
    let from = open_store(from_backend, from_path)?;
    let to   = open_store(to_backend, to_path)?;
    let data = from.load()?;
    to.save(&data)?;
    println!("✅ migrated {} entries → {}", data.entries.len(), to_path.display());
    Ok(())
}
```

---

## 🧪 Tests

**File:** `agent/tests/test_registry_sqlite.rs`
```rust
#[test]
fn sqlite_roundtrip() {
    let p = std::path::PathBuf::from("tests/out/registry.sqlite");
    let _ = std::fs::remove_file(&p);
    let store = SqliteRegistryStore::open(&p).unwrap();

    // add
    let e = RegistryEntry {
        id: "proof_001".into(),
        manifest_hash: "0xaaa".into(),
        proof_hash: "0xbbb".into(),
        timestamp_file: Some("build/timestamp.tsr".into()),
        registered_at: "2025-10-30T12:00:00Z".into(),
    };
    store.add_entry(e).unwrap();

    // find
    let found = store.find_by_hashes("0xaaa", "0xbbb").unwrap();
    assert!(found.is_some());

    // list
    let all = store.list().unwrap();
    assert_eq!(all.len(), 1);
}
```

**File:** `agent/tests/test_registry_migrate.rs`
```rust
#[test]
fn migrate_json_to_sqlite() {
    // prepare json fixture
    // run migrate
    // assert sqlite has same entries count and hashes
    assert!(true);
}
```

---

## 📦 Cargo.toml

```toml
rusqlite = { version = "0.31", features = ["bundled"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
```

> If you prefer system SQLite, drop the `"bundled"` feature.

---

## 📘 README Update

```markdown
### 🗄️ Registry Backends

Use JSON (default) or SQLite:

```bash
# JSON
cap registry list --registry-backend json --path build/registry.json

# SQLite
cap registry list --registry-backend sqlite --path build/registry.sqlite

# Migrate JSON → SQLite
cap registry migrate \
  --from json   --in build/registry.json \
  --to   sqlite --out build/registry.sqlite
```
```

---

## ✅ Acceptance Criteria

| Criterion | Description |
|-----------|-------------|
| ✔ Pluggable | `RegistryStore` trait with JSON + SQLite impls |
| ✔ CLI | `--registry-backend json|sqlite` respected across `registry` actions |
| ✔ Migrate | `cap registry migrate` copies all entries 1:1 |
| ✔ Backward‐compatible | Default remains JSON; no breaking changes |
| ✔ Tests | Roundtrip & migrate tests pass |

---

## 🔭 Future
- Add filters/pagination to `list()` via SQL queries.
- Add unique constraints on `(manifest_hash,proof_hash)` and separate surrogate key.
- Optional: sign `_meta.json` / registry rows for tamper-evidence.
