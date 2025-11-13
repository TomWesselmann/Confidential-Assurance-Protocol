# Follow-Up Tasks — Registry SQLite Adapter (v0.7.1 → v0.8.0)

## 🎯 Goal
Implement the post-review improvements for the **Registry SQLite Adapter**,  
focusing on test coverage, signing, schema versioning, and performance benchmarking.

These tasks follow the approved review report (⭐ 95/100 — Approved for Production).

---

## 🧩 1. SQLite-Specific Tests (v0.7.1)

### Objective
Add 2–3 targeted tests that explicitly verify SQLite backend behavior under edge conditions.

### Implementation Plan
**File:** `agent/tests/test_registry_sqlite.rs`

Add test cases:
1. **Error Handling:**
   ```rust
   #[test]
   fn sqlite_error_on_corrupt_db() {
       std::fs::write("tests/out/bad_registry.sqlite", b"garbage").unwrap();
       let result = SqliteRegistryStore::open("tests/out/bad_registry.sqlite");
       assert!(result.is_err(), "Expected error on corrupted SQLite file");
   }
   ```

2. **Migration Edge Case:**
   ```rust
   #[test]
   fn migrate_empty_json_to_sqlite() {
       let _ = std::fs::write("tests/out/empty.json", r#"{ \"entries\": [] }"#);
       let res = run(
           RegistryBackend::Json, Path::new("tests/out/empty.json"),
           RegistryBackend::Sqlite, Path::new("tests/out/empty.sqlite")
       );
       assert!(res.is_ok());
   }
   ```

3. **Duplicate Entry Handling (optional):**
   Insert the same `manifest_hash + proof_hash` twice → ensure `REPLACE` or unique constraint works.

### Acceptance Criteria
| Criterion | Description |
|-----------|--------------|
| ✔ Corruption Handling | Opening an invalid SQLite file produces a clean error |
| ✔ Empty Migration | JSON→SQLite migration with empty entries passes |
| ✔ Duplicate Handling | Duplicate insertions handled deterministically |

---

## 🔏 2. Registry Entry Signatures (v0.8.0)

### Objective
Enhance integrity by **digitally signing each registry entry** using the existing Ed25519 system (`sign.rs`).

### Implementation Plan
**File:** `agent/src/registry.rs`
- Extend `RegistryEntry`:
  ```rust
  pub struct RegistryEntry {
      pub id: String,
      pub manifest_hash: String,
      pub proof_hash: String,
      pub timestamp_file: Option<String>,
      pub registered_at: String,
      pub signature: Option<String>, // base64(Ed25519 signature)
  }
  ```
- Use existing company private key (`sign keygen` / `sign manifest`) to sign each new entry:
  ```rust
  let signature = sign::sign_message(private_key, &format!(\"{}{}\", manifest_hash, proof_hash))?;
  entry.signature = Some(base64::encode(signature));
  ```

### CLI Addition
Add optional flag:
```bash
cap registry add --sign-key keys/company_ed25519.key
```

### Acceptance Criteria
| Criterion | Description |
|-----------|-------------|
| ✔ Signing Optional | Registry works without signature if flag omitted |
| ✔ Verified Signatures | `cap registry verify` checks signature if present |
| ✔ Compatibility | JSON and SQLite backends store identical signature field |

---

## 📜 3. Schema Versioning (v0.8.0)

### Objective
Track schema versions directly inside SQLite meta table for future migrations.

### Implementation Plan
**File:** `agent/src/registry.rs`
- During `SqliteRegistryStore::open()`:
  ```rust
  conn.execute(
      \"INSERT OR IGNORE INTO registry_meta(key,value) VALUES('schema_version','1.0')\",
      [],
  )?;
  ```
- Add helper:
  ```rust
  pub fn schema_version(&self) -> anyhow::Result<String> {
      self.conn.query_row(\"SELECT value FROM registry_meta WHERE key='schema_version'\", [], |r| r.get(0))
  }
  ```

### Tests
- Validate schema version persisted.
- Increment to `1.1` in later migrations.

### Acceptance Criteria
| Criterion | Description |
|-----------|-------------|
| ✔ Version in Meta | `registry_meta` contains `schema_version` |
| ✔ Getter Works | `schema_version()` returns current version |
| ✔ Forward Compatible | No breaking change for JSON backend |

---

## ⚙️ 4. Performance Benchmarks (v0.8.0)

### Objective
Compare **JSON vs SQLite** performance for large registries (≥ 1 000 entries).

### Implementation Plan
**File:** `agent/benches/registry_bench.rs`
(use `cargo bench`)

```rust
use criterion::{criterion_group, criterion_main, Criterion};
use agent::registry::{JsonRegistryStore, SqliteRegistryStore};

fn bench_json_vs_sqlite(c: &mut Criterion) {
    let json_store = JsonRegistryStore { path: \"tests/out/registry.json\".into() };
    let sqlite_store = SqliteRegistryStore::open(Path::new(\"tests/out/registry.sqlite\")).unwrap();

    c.bench_function(\"json_load_1000\", |b| b.iter(|| json_store.load()));
    c.bench_function(\"sqlite_load_1000\", |b| b.iter(|| sqlite_store.load()));
}

criterion_group!(benches, bench_json_vs_sqlite);
criterion_main!(benches);
```

### Metrics to Record
- Load/save time (ms)
- Memory footprint (optional)
- File size comparison

### Acceptance Criteria
| Criterion | Description |
|-----------|-------------|
| ✔ Bench Runs | `cargo bench` runs without failure |
| ✔ Results Logged | JSON vs SQLite load times displayed |
| ✔ Performance Gain | SQLite ≥ 2× faster on ≥ 1 000 entries |

---

## 📘 Summary Table

| Area | Target Version | Type | Status | Responsible |
|------|----------------|------|---------|--------------|
| SQLite Edge Tests | v0.7.1 | QA | ⏳ Pending | QA / Maintainer |
| Entry Signing | v0.8.0 | Feature | ⏳ Planned | Core Dev |
| Schema Versioning | v0.8.0 | Refactor | ⏳ Planned | DB Engineer |
| Performance Benchmarks | v0.8.0 | Benchmark | ⏳ Planned | Perf Team |

---

## ✅ Acceptance Criteria (for Full Completion)

| Criterion | Description |
|-----------|-------------|
| ✔ Tests | SQLite backend handles edge cases and corruption gracefully |
| ✔ Signing | Optional Ed25519 signing implemented and verifiable |
| ✔ Schema | Version stored and retrievable |
| ✔ Bench | Benchmark results documented (≥ 2× improvement target) |

---

## 🔭 Optional Future Work
- Introduce registry synchronization between multiple SQLite nodes.
- Add compression support for large JSON registry files.
- Include registry schema migration tooling (`cap registry upgrade`).
- Automate signing verification in CI pipeline.

---

**Status:** Planned (v0.7.1 → v0.8.0)  
**Author:** Core Engineering  
**Reviewer:** Claude / CAP Maintainer  
**Last Updated:** 2025-10-30
