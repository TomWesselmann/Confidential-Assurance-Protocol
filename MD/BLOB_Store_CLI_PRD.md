# 🧱 CAP – BLOB Store CLI (v0.10 → v0.10.9)

## 🎯 Ziel
Fertigstellung einer **produktionstauglichen BLOB Store CLI** für Content-Addressable Storage (CAS) mit **Deduplication** und **Garbage Collection (GC)**. Vollständige Verknüpfung mit Registry-Einträgen, transaktional und deterministisch.


## ✅ Deliverables (Scope)
- **CLI-Kommandos:** `blob put`, `blob get`, `blob list`, `blob gc`
- **Registry-Verknüpfung:** Referenzzählung (refcount) für `manifest`, `proof`, `wasm`, `abi`
- **Deduplication:** BLAKE3-based CAS (0x-prefixed hex, 64 chars)
- **Garbage Collection:** mark-and-sweep (unreferenced blobs → prune), dry-run & force
- **Transaktionen & ACID:** SQLite WAL, atomare Updates von `blobs` + `registry_entries`
- **Dokumentation & Tests:** Unit + Integration + Property + Benchmark

---

## 🧩 Architektur (Kurz)
- **Backend:** `rusqlite` (bundled), `PRAGMA journal_mode=WAL`, `synchronous=NORMAL`
- **Schema:** Tabelle `blobs` (CAS) + `registry_entries` (bereits vorhanden)
- **Blob ID:** `blob_id = hex(blake3(content))` → `0x...` (64 hex)
- **Medientypen:** `manifest/proof/wasm/abi/other` (MIME in `media_type`)

```sql
CREATE TABLE IF NOT EXISTS blobs (
    blob_id TEXT PRIMARY KEY,
    size INTEGER NOT NULL,
    media_type TEXT NOT NULL,
    data BLOB NOT NULL,
    refcount INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_blobs_refcount ON blobs(refcount);
```

**Performance Pragmas**
```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = -40000;   -- 40 MB
PRAGMA mmap_size  = 268435456;-- 256 MB
```

---

## 🖥️ CLI-Design

### 1) `blob put`
Fügt Datei(en) in den BLOB Store ein (mit CAS & optionaler Registry-Verknüpfung).

```
cap-agent blob put   --file ./build/manifest.json   --type manifest   [--registry ./build/registry.sqlite]   [--link-entry-id <uuid>]   [--stdin]   [--out blob_id.txt]   [--no-dedup]
```

**Optionen**
- `--file <path>`: Quelldatei (mehrfach nutzbar); alternativ `--stdin`
- `--type <media>`: `manifest|proof|wasm|abi|other`
- `--registry <path>`: Registry-Datei (SQLite); Default: `./build/registry.sqlite`
- `--link-entry-id <id>`: Erhöht `refcount` für den referenzierenden Registry-Eintrag (z. B. beim Anlegen eines Eintrags)
- `--stdin`: lies Daten von stdin (Pipes)
- `--out <path>`: schreibe `blob_id` in Datei
- `--no-dedup`: erzwingt Re-Insert (nur Tests/Debug), erhöht **nicht** die `refcount`

**Verhalten**
- Hash = BLAKE3(content)
- Wenn `blob_id` existiert → nur `refcount`++ (sofern `--no-dedup` nicht gesetzt)
- Transaktion: Insert / Refcount Update atomar
- Ergebnis: `stdout` gibt `blob_id` aus, Exit 0

**Exit Codes**
- 0 OK, 10 IO-Fehler, 11 SQLite-Fehler, 12 Ungültiger Medientyp, 13 Transaktionsfehler

---

### 2) `blob get`
Extrahiert Blob-Inhalt anhand `blob_id` auf Datei oder stdout.

```
cap-agent blob get   --id 0xabc...   [--out ./out.bin]   [--stdout]
```

**Optionen**
- `--id <blob_id>` (required)
- `--out <path>`: Zielpfad
- `--stdout`: schreibt Rohdaten auf stdout (Default, wenn `--out` fehlt)

**Exit Codes**
- 0 OK, 20 NotFound, 11 SQLite-Fehler

---

### 3) `blob list`
Listet Blobs gefiltert/sortiert. Nützlich für Debug/Monitoring.

```
cap-agent blob list   [--type manifest|proof|wasm|abi|other]   [--min-size 0] [--max-size 1048576]   [--unused-only]   [--limit 100] [--order size|refcount|created_at]
```

**Spalten**
- `blob_id`, `size`, `media_type`, `refcount`

**Exit Codes**
- 0 OK, 11 SQLite-Fehler

---

### 4) `blob gc`
Garbage Collection nicht referenzierter Blobs.

```
cap-agent blob gc   [--dry-run]   [--force]   [--min-age 24h]   [--print-ids]
```

**Algorithmus (mark-and-sweep)**
1. **Mark:** Sammle alle referenzierten `blob_*` Felder aus `registry_entries` (manifest, proof, wasm, abi).
2. **Sweep:** Kandidaten = `blobs.refcount=0` **UND** nicht in Mark-Set.
3. **min-age:** Lösche nur, wenn `created_at <= now - min-age` (optional Spalte).
4. **Transaktion:** Löschen in Batches (z. B. 1000).
5. **Reporting:** Anzahl, Byte-Summe; optional IDs ausgeben.

**Exit Codes**
- 0 OK, 30 NothingToDo, 31 DryRunOnly, 11 SQLite-Fehler

---

## 🔗 Registry-Verknüpfung

### Schreib-Pfade (atomar)
- **Beim `registry add`:**
  - `blob_manifest`, `blob_proof`, `blob_wasm`, `blob_abi` setzen
  - Zu jedem gesetzten Blob: `refcount++`
- **Beim `registry delete` (zukünftig):**
  - Zu jedem Blob: `refcount--`
  - Optional: sofortige `gc` oder später via Cron

### Konsistenz-Check
- `registry verify` erweitert: Prüft, ob alle referenzierten `blob_*` existieren und `refcount>=1`
- `blob verify` (optional): Cross-Check Hash = `BLAKE3(data)`

---

## 🛡️ Sicherheit & Integrität
- **Determinismus:** BLAKE3 über Rohdaten (keine normalisierende Vorverarbeitung)
- **Authentizität:** Blobs selbst werden nicht signiert; **Signaturen leben in Manifest/Registry**
- **Rollen & Rechte:** CLI auf lokale Nutzung ausgelegt; Multi-User später (v1.0+)
- **DoS-Prävention:** `--max-size` Limit (z. B. 100 MB Default), Abort bei OOM

---

## 🧪 Tests

### Unit
- `blob_put_new_inserts_and_sets_refcount_1`
- `blob_put_existing_increments_refcount`
- `blob_get_roundtrip_binary_integrity`
- `blob_list_filters_work`
- `blob_gc_dry_run_reports_correct_bytes`
- `blob_gc_deletes_only_unreferenced`

### Integration
- `registry_add_links_blobs_and_increments_refcount`
- `registry_migration_preserves_blob_links`
- `concurrent_puts_are_idempotent` (Mutex/UPSERT)
- `gc_is_atomic_and_resilient` (Crash-Simulation)

### Property
- **Hash-Determinismus:** `blake3(content)` invariant
- **Idempotenz:** 2× `put` → gleiche `blob_id`, gleicher Zustand

---

## 📈 Performance & Ziele
- **Insert (4 MB Blob):** < 10 ms (NVMe, WAL)
- **Get (4 MB Blob):** < 8 ms
- **GC (10k unreferenced):** < 60 s
- **Speicher-Footprint:** Dedup spart ≥60% bei redundanten Proofs
- **Parallelität:** Serielle Transaktionen pro DB-Datei; Future: Sharding (v1.0+)

---

## 🧰 Rust-API (Skizze)

```rust
pub fn blob_put(conn: &mut Connection, media_type: &str, bytes: &[u8]) -> Result<String>;

pub fn blob_get(conn: &Connection, blob_id: &str) -> Result<Vec<u8>>;

pub fn blob_ref_inc(conn: &mut Connection, blob_id: &str) -> Result<()>;
pub fn blob_ref_dec(conn: &mut Connection, blob_id: &str) -> Result<()>;

pub fn blob_gc(conn: &mut Connection, min_age: Option<Duration>, dry_run: bool) -> Result<GcReport>;

pub struct GcReport {
    pub candidates: usize,
    pub deleted: usize,
    pub bytes_freed: u64,
    pub ids: Vec<String>, // optional
}
```

**Transaktionsrahmen**
```rust
let tx = conn.transaction()?;
// ... put/list/gc ...
tx.commit()?;
```

---

## 🧭 Migrationshinweise
- **Bestehende DB:** ergänze optional `created_at` in `blobs` (ISO8601); Default NOW()
- **Backfill:** setze `refcount` per Scan über `registry_entries`
- **Safe Mode:** `blob gc --dry-run` einmalig ausführen & Report prüfen

```sql
ALTER TABLE blobs ADD COLUMN created_at TEXT NOT NULL DEFAULT (datetime('now'));
```

---

## ⚠️ Fehlerbilder & Handling
- **EEXIST:** Deduplizierter Insert → `refcount++` statt Fehler (idempotent)
- **NotFound:** `blob get --id` unbekannt → Exit 20
- **DB Busy:** Wiederhole mit Exponential Backoff (bis 3×)
- **Corruption:** Prüfe Hash bei `get` mit `--verify` (optional) → Report & Exit 40

---

## 📅 Aufwand & Priorität
- **Implementierung:** 1–2 Wochen
- **Priorität:** 🟥 Hoch
- **Abhängigkeit:** SQLite-BLOB-Backend vorhanden (v0.9)

---

## ✅ Abnahmekriterien
- Alle vier Kommandos verfügbar & durch Tests gedeckt
- Deduplizierung nachweisbar (gleiche Bytes → gleiche `blob_id`)
- `registry add` erhöht `refcount`, `gc` entfernt nur unreferenzierte Blobs
- Benchmarks innerhalb Zielwerte
- Doku: `--help` + README-Abschnitt + Beispiele
