# CAP Proof System – Punkt 4: Registry-BLOB & Sandbox

## Ziel
Registry-Einträge werden zu **autonomen Objekten**: Der komplette Proof-Paket-Inhalt (inkl. `verifier.wasm` & `verifier.abi.json`) wird als **BLOB** content-addressable gespeichert und beim Eintrag **gepinnt**. Ein eingebauter **Loader + Sandbox** führt den Verifier deterministisch offline aus.

---

## Deliverables

### 1) BLOB-Store (content-addressable)
- Speicherort: SQLite (empfohlen) + optional Filesystem Tier.
- Key: `blob_id = blake3(data)` (Hex).  
- Deduplication automatisch; GC über Refcounts der Registry-Entries.

### 2) Registry-Entry Erweiterung (v0.9)
- Felder (neu):  
  - `blob_manifest` (hex blake3)  
  - `blob_proof` (hex blake3)  
  - `blob_wasm` (hex blake3)  
  - `blob_abi` (hex sha3-256)  
  - `selfverify_status: enum{unknown, ok, fail}`  
  - `selfverify_at: rfc3339`  
  - `verifier_name: text`, `verifier_version: text`  
- Core-Hash und Signatur schließen diese Felder mit ein.

### 3) Sandboxed Loader
- Laufzeit: In-Process WASM (Host ohne FS/Netz/Time), deterministisch, Fuel/Heap-Limits.  
- Vor Ausführung: Hash-Pinning von `wasm_blake3` und `abi_sha3`.  
- Ergebnis schreibt Entry-Felder `selfverify_*` und `verifier_*`.

### 4) CLI-Erweiterungen
- `blob put/get/gc` (Low-level)  
- `registry add --selfverify` (High-level)  
- `registry exec --id <entry>` (Re-Run)

---

## Datenmodell (SQLite)

### Tabellen

#### `registry_entries`
| Feld | Typ | Beschreibung |
|------|-----|---------------|
| id | TEXT | Primary Key |
| created_at | TEXT | RFC3339 |
| manifest_hash | TEXT | sha3-256 |
| proof_hash | TEXT | sha3-256 |
| blob_manifest | TEXT | blake3 |
| blob_proof | TEXT | blake3 |
| blob_wasm | TEXT | blake3 |
| blob_abi | TEXT | sha3-256 |
| verifier_name | TEXT | |
| verifier_version | TEXT | |
| selfverify_status | TEXT | unknown / ok / fail |
| selfverify_at | TEXT | RFC3339 |
| signature | TEXT | base64 |
| public_key | TEXT | base64 |

#### `blobs`
| Feld | Typ | Beschreibung |
|------|-----|---------------|
| blob_id | TEXT | Primary Key (blake3/sha3) |
| size | INTEGER | |
| media_type | TEXT | application/json, wasm, ... |
| data | BLOB | |
| refcount | INTEGER | ≥0 |

---

## BLOB-Ablauf

1. **`registry add --selfverify`**  
   - Liest Paketdateien  
   - Berechnet Hashes → `blob_id`s  
   - Speichert Blobs (`blob put`)  
   - Sandbox-Verify mit Hash-Pinning  
   - Schreibt `selfverify_status`, signiert Entry, speichert  
2. **`registry exec --id <entry>`**  
   - Lädt Blobs, Sandbox-Verify erneut  
3. **`blob gc`**  
   - Löscht nicht referenzierte Blobs (refcount=0)

---

## CLI-Design

```bash
cap-agent blob put --file verifier.wasm --type application/wasm
cap-agent blob get --id <blake3> --out /tmp/verifier.wasm
cap-agent blob gc --dry-run
cap-agent registry add --manifest manifest.json --proof proof.dat   --with-verifier verifier.wasm --with-abi verifier.abi.json   --selfverify --signing-key keys/company.ed25519
cap-agent registry exec --id <entry-id>
```

---

## Sandbox-Regeln

- **Verboten:** FS, Netzwerk, Zeit, Zufall  
- **Limits:** fuel, heap, stack, call_ms  
- **Imports:** nur `cap_log(level, msg)`  
- **Fehlercodes:**  
  - `E-BLOB-MISSING`  
  - `E-HASH-MISMATCH`  
  - `E-SANDBOX-FORBIDDEN-IMPORT`  
  - `E-SANDBOX-OUT-OF-FUEL`  
  - `E-VERIFY-FAILED`  

---

## Migration

```bash
cap-agent registry migrate --from sqlite --input registry.sqlite   --to sqlite --output registry.sqlite   --add-columns blob_manifest,blob_proof,blob_wasm,blob_abi,selfverify_status,selfverify_at,verifier_name,verifier_version
```

---

## Implementation Steps

1. DB-Schema erweitern  
2. BlobStore implementieren (`put/get/gc`)  
3. Registry-Struct v0.9 + CoreHash + Signatur  
4. Loader (WASM Sandbox)  
5. CLI Subcommands  
6. Tests & Benchmarks  
7. Doku aktualisieren (`SYSTEMARCHITEKTUR.md`, `CLAUDE.md`)

---

## Tests & Definition of Done

- **Unit Tests:** BlobStore, Hashing, Signaturen  
- **Integration:** registry add/exec happy & fail paths  
- **Benchmarks:** 100–10k entries, repeatability <0.5%  
- **DoD:** CLI & Migration funktionsfähig, Doku ergänzt
