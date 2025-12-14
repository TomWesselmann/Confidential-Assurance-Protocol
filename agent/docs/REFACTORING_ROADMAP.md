# Refactoring Roadmap - LsKG-Agent

**Datum:** 2025-12-01
**Geschätzter Gesamtaufwand:** ~53 Stunden
**Autor:** Claude Code Review

---

## Übersicht

| Sprint | Focus | Tasks | Aufwand | Priorität |
|--------|-------|-------|---------|-----------|
| Sprint 0 | Sicherheit | 4 | ~4h | KRITISCH |
| Sprint 1 | Kritische Stabilität | 6 | ~8h | HOCH |
| Sprint 2 | Error Handling | 5 | ~10h | HOCH |
| Sprint 3 | Wartbarkeit | 9 | ~12h | MITTEL |
| Sprint 4 | Polish | 9 | ~10h | NIEDRIG |
| Bonus | Nice-to-Have | 4 | ~9h | OPTIONAL |

---

## SPRINT 0: Sicherheit (~4h) - KRITISCH ✅ ABGESCHLOSSEN

> **Ziel:** Alle kritischen Sicherheitslücken schließen

| # | Task | Datei | Aufwand | Status |
|---|------|-------|---------|--------|
| 1 | SQL Injection fixen (format! → params!) | `policy/sqlite.rs:202` | 10min | [x] |
| 2 | Hardcoded Token 'admin-tom' entfernen | `api/auth.rs:95` | 1h | [x] |
| 3 | Path Traversal absichern | `api/upload.rs` | 2h | [x] |
| 4 | Upload Size Limits einbauen | `api/upload.rs` | 1h | [x] |

### Details

#### 1. SQL Injection (KRITISCH)

**Aktuell:**
```rust
// policy/sqlite.rs:202 - VULNERABLE
format!("WHERE status = '{}'", status_str)
```

**Fix:**
```rust
// Parameterized query
conn.prepare("SELECT ... FROM policies WHERE status = ?1")?
    .query(params![status_str])?
```

#### 2. Hardcoded Token (KRITISCH)

**Problem:** Dev-Token "admin-tom" im Production-Code
**Fix:** Environment Variable oder Config-File

#### 3. Path Traversal (KRITISCH)

**Problem:** File-Upload ohne Pfad-Validierung
**Fix:**
```rust
// Pfad normalisieren und validieren
let safe_path = path.canonicalize()?;
if !safe_path.starts_with(&upload_dir) {
    return Err(anyhow!("Path traversal detected"));
}
```

#### 4. Upload Size Limits (HOCH)

**Problem:** Keine Größenbeschränkung für Uploads
**Fix:** Content-Length Header prüfen, Streaming mit Limit

---

## SPRINT 1: Kritische Stabilität (~8h)

> **Ziel:** Crash-Risiken eliminieren, Code-Basis stabilisieren

| # | Task | Datei(en) | Aufwand | Status |
|---|------|-----------|---------|--------|
| 5 | paths.rs mit Build-Pfad-Konstanten erstellen | `src/cli/paths.rs` (NEU) | 1h | [ ] |
| 6 | 34x hardcoded audit path ersetzen | `cli/*.rs` | 1h | [ ] |
| 7 | audit_helper.rs Wrapper erstellen | `src/cli/audit_helper.rs` (NEU) | 2h | [ ] |
| 8 | Kritisches unwrap() fixen | `keys/store.rs:86` | 15min | [ ] |
| 9 | Kritisches unwrap() fixen | `audit/v1_0.rs:36` | 15min | [ ] |
| 10 | Mutex-Handling verbessern (30+ Stellen) | `api/`, `policy/` | 3h | [ ] |

### Details

#### 5. paths.rs erstellen

```rust
// src/cli/paths.rs (NEU)
pub const AUDIT_LOG_PATH: &str = "build/agent.audit.jsonl";
pub const COMMITMENTS_PATH: &str = "build/commitments.json";
pub const REGISTRY_JSON_PATH: &str = "build/registry.json";
pub const REGISTRY_SQLITE_PATH: &str = "build/registry.sqlite";
pub const DEFAULT_KEYS_DIR: &str = "keys";
pub const DEFAULT_BUILD_DIR: &str = "build";
```

#### 7. audit_helper.rs erstellen

```rust
// src/cli/audit_helper.rs (NEU)
use crate::audit::v1_0::AuditLog;
use crate::cli::paths;

pub fn log_audit_event(event: &str, data: serde_json::Value) -> Result<(), Box<dyn Error>> {
    let mut audit = AuditLog::new(paths::AUDIT_LOG_PATH)?;
    audit.log_event(event, data)
}

// Macro für häufige Events
#[macro_export]
macro_rules! audit_event {
    ($event:expr, $($key:ident: $value:expr),*) => {
        crate::cli::audit_helper::log_audit_event($event, serde_json::json!({ $($key: $value),* }))
    };
}
```

#### 8-9. Kritische unwraps fixen

```rust
// keys/store.rs:86 - VORHER
let filename = path.file_name().unwrap();

// keys/store.rs:86 - NACHHER
let filename = path.file_name()
    .ok_or_else(|| anyhow!("Path has no filename: {:?}", path))?;
```

```rust
// audit/v1_0.rs:36 - VORHER
let path_str = path.as_ref().to_str().unwrap().to_string();

// audit/v1_0.rs:36 - NACHHER
let path_str = path.as_ref()
    .to_str()
    .ok_or_else(|| anyhow!("Invalid UTF-8 in path"))?
    .to_string();
```

#### 10. Mutex-Handling

```rust
// VORHER (30+ Stellen)
let guard = mutex.lock().unwrap();

// NACHHER - Option A: expect mit Kontext
let guard = mutex.lock().expect("Mutex poisoned in policy store");

// NACHHER - Option B: Error propagation
let guard = mutex.lock().map_err(|e| anyhow!("Mutex poisoned: {}", e))?;
```

---

## SPRINT 2: Error Handling (~10h)

> **Ziel:** Robuste Fehlerbehandlung in allen Modulen

| # | Task | Datei(en) | Aufwand | Status |
|---|------|-----------|---------|--------|
| 11 | 82 API unwraps → ? operator | `api/verify/*.rs` | 4h | [ ] |
| 12 | SQLite-Backend Tests schreiben | `policy/sqlite.rs` | 2h | [ ] |
| 13 | Custom Error Types (AppError) | alle Module | 2h | [ ] |
| 14 | Verifier APIs vereinheitlichen | `verifier/verify.rs`, `core_verify.rs` | 2h | [ ] |
| 15 | Registry Error-Typ-Inkonsistenz | `registry/*.rs` | 1h | [ ] |

### Details

#### 13. Custom Error Types

```rust
// src/error.rs (NEU)
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CapAgentError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Verification failed: {0}")]
    Verification(String),

    #[error("Policy error: {0}")]
    Policy(String),

    #[error("Registry error: {0}")]
    Registry(String),
}
```

#### 14. Verifier API Vereinheitlichung

```rust
// Option: Facade Pattern
pub fn verify_simple(...) -> VerifyReport {
    let result = verify_core(CoreVerifyInput { ... });
    VerifyReport::from(result)
}

// Oder: Eine API mit Options
pub fn verify(input: VerifyInput, opts: VerifyOptions) -> VerifyResult {
    // Einheitliche Implementierung
}
```

---

## SPRINT 3: Wartbarkeit (~12h)

> **Ziel:** Code-Duplizierung reduzieren, Strukturen verbessern

| # | Task | Datei(en) | Aufwand | Status |
|---|------|-----------|---------|--------|
| 16 | BlobPutOptions Struct (7 params) | `cli/blob.rs` | 30min | [ ] |
| 17 | RegistryAddOptions Struct (8 params) | `cli/registry.rs` | 30min | [ ] |
| 18 | ProofAdaptOptions Struct (10 params) | `cli/proof.rs` | 30min | [ ] |
| 19 | Audit v1/v2 Duplizierung eliminieren | `audit/*.rs` | 3h | [ ] |
| 20 | E2003 Linter-Check implementieren | `policy_v2/linter.rs` | 1.5h | [ ] |
| 21 | E3002 Linter-Check implementieren | `policy_v2/linter.rs` | 1.5h | [ ] |
| 22 | Domain-Logik extrahieren (keys) | `cli/keys.rs` → `keys/` | 1h | [ ] |
| 23 | Domain-Logik extrahieren (registry) | `cli/registry.rs` → `registry/` | 1h | [ ] |
| 24 | Domain-Logik extrahieren (proof) | `cli/proof.rs` → `zk_system/` | 1h | [ ] |

### Details

#### 16-18. Options-Structs

```rust
// cli/blob.rs
pub struct BlobPutOptions {
    pub file: Option<String>,
    pub media_type: BlobType,
    pub registry: String,
    pub link_entry_id: Option<String>,
    pub stdin: bool,
    pub out: Option<String>,
    pub no_dedup: bool,
}

pub fn run_blob_put(opts: BlobPutOptions) -> Result<(), Box<dyn Error>> {
    // ...
}
```

#### 19. Audit Konsolidierung

```rust
// audit/common.rs (NEU) - Shared abstractions
pub trait AuditStore {
    fn append(&mut self, event: &str, data: Value) -> Result<String>;
    fn read_last(&self) -> Result<Option<AuditEntry>>;
    fn verify_chain(&self) -> Result<VerifyReport>;
}

// Beide Versionen implementieren das gleiche Trait
impl AuditStore for AuditLogV1 { ... }
impl AuditStore for AuditChain { ... }
```

#### 20-21. Fehlende Linter-Checks

```rust
// policy_v2/linter.rs

// E2003: Input-Referenz-Validierung
fn check_input_references(policy: &PolicyV2, diagnostics: &mut Vec<Diagnostic>) {
    let defined_inputs: HashSet<_> = policy.inputs.iter()
        .map(|i| i.name.as_str())
        .collect();

    for constraint in &policy.constraints {
        if let Some(ref lhs) = constraint.lhs {
            if !defined_inputs.contains(lhs.as_str()) {
                diagnostics.push(Diagnostic::error(
                    "E2003",
                    format!("Undefined input reference: {}", lhs),
                ));
            }
        }
    }
}

// E3002: Range-Expression-Validierung
fn check_range_expressions(policy: &PolicyV2, diagnostics: &mut Vec<Diagnostic>) {
    for constraint in &policy.constraints {
        if constraint.op == "range_min" {
            if constraint.rhs.is_none() {
                diagnostics.push(Diagnostic::error(
                    "E3002",
                    "range_min requires rhs value",
                ));
            }
        }
    }
}
```

---

## SPRINT 4: Polish (~10h)

> **Ziel:** Code-Qualität und Konsistenz verbessern

| # | Task | Datei(en) | Aufwand | Status |
|---|------|-----------|---------|--------|
| 25 | KeyStatus String → Enum | `keys/types.rs` | 1h | [ ] |
| 26 | Key-Verzeichnis-Konstanten | `keys/store.rs` | 30min | [ ] |
| 27 | Signatur-Längen Konstanten | `verifier/core_verify.rs` | 30min | [ ] |
| 28 | Bundle CLI Subcommand-Struktur | `cli/mod.rs`, `main.rs` | 1h | [ ] |
| 29 | Tests reorganisieren | `verifier/core.rs` → `tests/` | 1h | [ ] |
| 30 | Kommentare vereinheitlichen (EN) | diverse | 2h | [ ] |
| 31 | Policy Status → Enum | `policy/sqlite.rs` | 1h | [ ] |
| 32 | Genesis-Hash zentral definieren | `audit/`, `crypto/` | 30min | [ ] |
| 33 | Hash-Funktionen konsolidieren | `crypto/`, `policy/`, `policy_v2/` | 2h | [ ] |

### Details

#### 25. KeyStatus Enum

```rust
// keys/types.rs - VORHER
pub status: String,  // "active", "retired", "revoked"

// keys/types.rs - NACHHER
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyStatus {
    Active,
    Retired,
    Revoked,
}

pub struct KeyMetadata {
    pub status: KeyStatus,
    // ...
}
```

#### 27. Signatur-Konstanten

```rust
// verifier/constants.rs (NEU)
pub const ED25519_SIGNATURE_LEN: usize = 64;
pub const ED25519_PUBKEY_LEN: usize = 32;
pub const SHA3_256_HASH_LEN: usize = 32;

// Verwendung in core_verify.rs
if sig.len() != ED25519_SIGNATURE_LEN {
    return Err(anyhow!(
        "Invalid signature length: {}, expected {}",
        sig.len(),
        ED25519_SIGNATURE_LEN
    ));
}
```

#### 28. Bundle Subcommand-Struktur

```rust
// cli/mod.rs - VORHER
pub enum Commands {
    BundleV2 { ... },
    VerifyBundle { ... },
    // ...
}

// cli/mod.rs - NACHHER
pub enum Commands {
    Bundle(BundleCommands),
    // ...
}

#[derive(Subcommand)]
pub enum BundleCommands {
    Create { ... },   // war BundleV2
    Verify { ... },   // war VerifyBundle
}
```

#### 32. Genesis-Hash zentral

```rust
// crypto/constants.rs (NEU)
pub const GENESIS_HASH: &str =
    "0x0000000000000000000000000000000000000000000000000000000000000000";

// Verwendung in audit/v1_0.rs und audit/hash_chain.rs
use crate::crypto::constants::GENESIS_HASH;
```

#### 33. Hash-Funktionen konsolidieren

```rust
// crypto/hash.rs - Einheitliche API
pub fn sha3_256_hex(data: &[u8]) -> String {
    let hash = sha3_256(data);
    hex_lower_prefixed32(hash)
}

pub fn compute_policy_hash(policy_json: &str) -> String {
    sha3_256_hex(policy_json.as_bytes())
}

// Entfernen aus:
// - policy/types.rs:121 (compute_hash)
// - policy/store.rs:28 (compute_policy_hash)
// - policy_v2/hasher.rs:4 (sha3_256_hex)
```

---

## BONUS: Nice-to-Have (~9h)

| # | Task | Aufwand | Status |
|---|------|---------|--------|
| 34 | CI: Clippy als Fehler | 1h | [ ] |
| 35 | Benchmark Suite | 3h | [ ] |
| 36 | Property-Based Tests | 2h | [ ] |
| 37 | Module-Dokumentation | 3h | [ ] |

### Details

#### 34. CI Clippy-Integration

```yaml
# .github/workflows/ci.yml
- name: Clippy
  run: cargo clippy -- -D warnings
```

#### 35. Benchmark Suite

```rust
// benches/verify_benchmark.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn verify_benchmark(c: &mut Criterion) {
    c.bench_function("verify_manifest", |b| {
        b.iter(|| {
            // Benchmark code
        })
    });
}

criterion_group!(benches, verify_benchmark);
criterion_main!(benches);
```

#### 36. Property-Based Tests

```rust
// tests/property_tests.rs
use proptest::prelude::*;

proptest! {
    #[test]
    fn hash_is_deterministic(data: Vec<u8>) {
        let hash1 = crypto::sha3_256(&data);
        let hash2 = crypto::sha3_256(&data);
        prop_assert_eq!(hash1, hash2);
    }

    #[test]
    fn hash_length_is_constant(data: Vec<u8>) {
        let hash = crypto::sha3_256(&data);
        prop_assert_eq!(hash.len(), 32);
    }
}
```

---

## Fortschritts-Tracking

### Sprint-Status

| Sprint | Status | Fortschritt |
|--------|--------|-------------|
| Sprint 0 | ⏳ Ausstehend | 0/4 |
| Sprint 1 | ⏳ Ausstehend | 0/6 |
| Sprint 2 | ⏳ Ausstehend | 0/5 |
| Sprint 3 | ⏳ Ausstehend | 0/9 |
| Sprint 4 | ⏳ Ausstehend | 0/9 |
| Bonus | ⏳ Ausstehend | 0/4 |

### Metriken-Ziele

| Metrik | Aktuell | Nach Sprint 0 | Nach Sprint 2 | Ziel |
|--------|---------|---------------|---------------|------|
| Sicherheitslücken | 4 | 0 | 0 | 0 |
| Unwraps (Production) | 350+ | 350+ | <50 | <30 |
| Hardcoded Paths | 34 | 34 | 0 | 0 |
| Code-Duplizierung | 70% | 70% | 70% | <20% |
| Test Coverage | 60% | 60% | 75% | 85% |

---

## Abhängigkeiten

```
Sprint 0 (Sicherheit)
    ↓
Sprint 1 (Stabilität)
    ↓
Sprint 2 (Error Handling)
    ↓
Sprint 3 (Wartbarkeit) ←── kann teilweise parallel
    ↓
Sprint 4 (Polish)
    ↓
Bonus (Nice-to-Have)
```

---

## Siehe auch

- [CODE_QUALITY_MASTER_REPORT.md](CODE_QUALITY_MASTER_REPORT.md) - Detaillierte Analyse
- [CLI_REFACTORING_REPORT.md](CLI_REFACTORING_REPORT.md) - CLI-spezifische Issues
- [VERIFIER_REFACTORING_REPORT.md](VERIFIER_REFACTORING_REPORT.md) - Verifier-Analyse

---

*Generiert von Claude Code Review - 2025-12-01*
