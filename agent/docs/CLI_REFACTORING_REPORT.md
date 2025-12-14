# CLI Modul Refactoring Report

**Datum:** 2025-12-01
**Analysiert:** 12 Dateien, ~3.500 LoC
**Autor:** Claude Code Review

---

## Executive Summary

Das CLI-Modul zeigt typische Wachstumsprobleme eines organisch gewachsenen Projekts:
- **34x** wiederholte Hardcoded Paths
- **7x** duplizierte Audit-Event-Patterns
- **Domain-Logik** in CLI-Layer eingestreut
- **Inkonsistente** Command-Struktur

**Empfehlung:** Strukturierte Refactoring-Phasen nach Priorität

---

## Kritische Findings (Prio 1)

### 1. Hardcoded Path-Konstanten

**Problem:** `"build/agent.audit.jsonl"` erscheint 34x in 10 Dateien

**Betroffene Dateien:**
- audit.rs (5x)
- blob.rs (4x)
- bundle.rs (2x)
- keys.rs (5x)
- manifest.rs (3x)
- policy.rs (2x)
- prepare.rs (2x)
- proof.rs (4x)
- registry.rs (5x)
- sign.rs (2x)

**Lösung:**
```rust
// src/cli/paths.rs (NEU)
pub const AUDIT_LOG_PATH: &str = "build/agent.audit.jsonl";
pub const COMMITMENTS_PATH: &str = "build/commitments.json";
pub const REGISTRY_JSON_PATH: &str = "build/registry.json";
pub const REGISTRY_SQLITE_PATH: &str = "build/registry.sqlite";
pub const DEFAULT_KEYS_DIR: &str = "keys";
pub const DEFAULT_BUILD_DIR: &str = "build";
```

**Aufwand:** ~2h

---

### 2. Dupliziertes Audit-Event-Pattern

**Problem:** Jede Funktion erstellt eigenen AuditLog

```rust
// Aktuell (34x wiederholt):
let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
audit.log_event("event_name", json!({ ... }))?;
```

**Lösung:**
```rust
// src/cli/audit_helper.rs (NEU)
pub fn log_audit_event(event: &str, data: serde_json::Value) -> Result<(), Box<dyn Error>> {
    let mut audit = AuditLog::new(paths::AUDIT_LOG_PATH)?;
    audit.log_event(event, data)
}

// Macro für häufige Events
macro_rules! audit_event {
    ($event:expr, $($key:ident: $value:expr),*) => {
        log_audit_event($event, serde_json::json!({ $($key: $value),* }))
    };
}
```

**Aufwand:** ~3h

---

### 3. Domain-Logik in CLI-Layer

**Problem:** Geschäftslogik sollte nicht in CLI-Dateien sein

| Datei | Zeilen | Domain-Code |
|-------|--------|-------------|
| keys.rs | 238-244 | Ed25519 Key-Loading |
| registry.rs | 108-123 | Entry Signing |
| proof.rs | 276-289 | Constraint Building |
| manifest.rs | 22-40 | Timestamp Verification |

**Lösung:** Domain-Funktionen in entsprechende Module verschieben:
- `keys.rs:238-244` → `src/keys/loader.rs`
- `registry.rs:108-123` → `src/registry/signing.rs`
- `proof.rs:276-289` → `src/zk_system.rs`

**Aufwand:** ~4h

---

## Hohe Priorität (Prio 2)

### 4. Inkonsistente Command-Struktur

**Problem:** `BundleV2` und `VerifyBundle` sind Top-Level Commands

```rust
// Aktuell:
Commands::BundleV2 { ... }
Commands::VerifyBundle { ... }

// Besser:
Commands::Bundle(BundleCommands)
  - BundleCommands::Create { ... }    // war BundleV2
  - BundleCommands::Verify { ... }    // war VerifyBundle
```

**Aufwand:** ~2h (inkl. Breaking Change Dokumentation)

---

### 5. Duplizierte Registry-Args

**Problem:** 6 Commands haben identische `--registry` und `--backend` Args

```rust
// 6x kopiert:
#[arg(long)]
registry: Option<String>,
#[arg(long, default_value = "auto")]
backend: String,
```

**Lösung:**
```rust
// Shared Args Struct
#[derive(Args, Clone)]
pub struct RegistryArgs {
    #[arg(long)]
    pub registry: Option<String>,
    #[arg(long, default_value = "auto")]
    pub backend: String,
}

// Usage:
RegistryCommands::Add {
    #[command(flatten)]
    registry_args: RegistryArgs,
    // ... andere args
}
```

**Aufwand:** ~2h

---

### 6. Magic Strings als Enum

**Problem:** Hardcoded Strings für Typen

```rust
// blob.rs:27
let valid_types = ["manifest", "proof", "wasm", "abi", "other"];

// audit.rs:116-119
"ethereum" | "hedera" | "btc"
```

**Lösung:**
```rust
// src/cli/types.rs (NEU)
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum BlobType {
    Manifest,
    Proof,
    Wasm,
    Abi,
    Other,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum BlockchainType {
    Ethereum,
    Hedera,
    Btc,
}
```

**Aufwand:** ~2h

---

## Mittlere Priorität (Prio 3)

### 7. Funktions-Parameter-Explosion

**Problem:** Funktionen mit 7-8 Parametern

| Funktion | Parameter |
|----------|-----------|
| `run_blob_put` | 7 |
| `run_registry_add` | 8 |
| `run_proof_adapt` | 10 |

**Lösung:** Options-Structs

```rust
pub struct BlobPutOptions {
    pub file: Option<String>,
    pub media_type: BlobType,
    pub registry: String,
    pub link_entry_id: Option<String>,
    pub stdin: bool,
    pub out: Option<String>,
    pub no_dedup: bool,
}

pub fn run_blob_put(opts: BlobPutOptions) -> Result<(), Box<dyn Error>>
```

**Aufwand:** ~3h

---

### 8. Hash-Verifikation Duplizierung

**Problem:** Gleiche Hash-Check-Logik in bundle.rs und verifier.rs

```rust
// bundle.rs:288-302 UND verifier.rs:142-169
let manifest_bytes = std::fs::read(path.join("manifest.json"))?;
let actual_hash = crypto::hex_lower_prefixed32(crypto::sha3_256(&manifest_bytes));
if actual_hash != expected_hash {
    return Err(...);
}
```

**Lösung:**
```rust
// src/bundle/integrity.rs (NEU)
pub fn verify_bundle_integrity(
    bundle_dir: &Path,
    meta: &serde_json::Value,
) -> Result<IntegrityReport, BundleError> {
    // Zentrale Hash-Verifikation
}
```

**Aufwand:** ~2h

---

### 9. Step-Counter Pattern

**Problem:** Hardcoded Step-Nummern

```rust
output::step(1, 7, "Copying manifest...");
output::step(2, 7, "Copying proof...");
// ...
output::step(7, 7, "Creating ZIP...");
```

**Lösung:**
```rust
struct ProgressTracker {
    current: usize,
    total: usize,
}

impl ProgressTracker {
    fn step(&mut self, msg: &str) {
        self.current += 1;
        output::step(self.current, self.total, msg);
    }
}
```

**Aufwand:** ~1h

---

## Niedrige Priorität (Prio 4)

### 10. Inkonsistente Arg-Patterns

- Manche Args haben `#[arg(long)]`, andere sind positional
- Inkonsistente Default-Werte
- Gemischte Sprachen (DE/EN) in Hilfe-Texten

### 11. Unvollständige Implementations

```rust
// proof.rs:504
_selector: &str, // TODO: Implement selector

// manifest.rs:190
// TODO: actual digest validation if needed
```

---

## Refactoring-Roadmap

```
Phase 1 (Woche 1): Kritisch
├── [ ] paths.rs erstellen
├── [ ] audit_helper.rs erstellen
└── [ ] Domain-Logik extrahieren

Phase 2 (Woche 2): Hoch
├── [ ] Bundle Command umstrukturieren
├── [ ] RegistryArgs extrahieren
└── [ ] Enum-Typen erstellen

Phase 3 (Woche 3): Mittel
├── [ ] Options-Structs für große Funktionen
├── [ ] Hash-Verifikation zentralisieren
└── [ ] ProgressTracker implementieren

Phase 4 (Fortlaufend): Niedrig
├── [ ] Arg-Patterns vereinheitlichen
├── [ ] TODOs implementieren/dokumentieren
└── [ ] Sprache vereinheitlichen
```

---

## Metriken

| Metrik | Vor Refactoring | Nach Refactoring (Ziel) |
|--------|-----------------|-------------------------|
| Hardcoded Paths | 34 | 0 |
| Duplizierte Audit-Calls | 34 | 1 (Helper) |
| Domain-Code in CLI | ~150 LoC | 0 |
| Functions >5 Params | 8 | 0 |
| Magic Strings | 25+ | 0 (Enums) |

---

## Nächste Schritte

1. **Review dieses Reports** mit dem Team
2. **Priorisierung bestätigen**
3. **Phase 1 starten** mit `paths.rs`

---

*Generiert von Claude Code Review*
