# WEEK7_R — Registry Schema v1.1

**Ziel:** Versionierte Registry mit v1.1, kompatible Reader/Writer, Migration & Backfill, Benchmarks.

## Änderungen
- `registry_meta`: `schema_version: "1.1"`, `migrated_at`, `tool_version`
- Einträge: Pflicht `entry_id, created_at, policy_id, ir_hash, manifest_hash`; Signatur via `kid` (Fallback `public_key`)
- Optional: `prev_hash` (für Hash‑Chain‑Integration)

## CLI
- `cap registry inspect` → zeigt Version/Felder
- `cap registry migrate --to 1.1 --dry-run`
- `cap registry backfill --field kid --from public_key`

## Akzeptanzkriterien (DoD)
1. **R/W‑Kompat:** v1.0 lesbar, v1.1 schreibbar/lesbar
2. **Idempotenz:** erneute Migration = no‑op
3. **Performance:** 1000 Entries Migration ≤ 5s
4. **Signatur‑Felder:** Einträge mit `kid` verifizierbar; Legacy `public_key` weiterhin verifizierbar

## Tests & Befehle
```bash
cargo test --test registry_migration -- --nocapture
cargo test --test registry_compat -- --nocapture
cargo bench --bench registry_bench
```

## Dateien (neu/ändern)
```
src/registry/schema.rs
src/registry/migrate.rs
tests/registry_migration.rs
tests/registry_compat.rs
benches/registry_bench.rs
```