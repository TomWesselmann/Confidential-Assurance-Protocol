# CAP-Agent Refactoring Guide

_Maßgeschneiderte Anleitung für strukturelle Verbesserungen des CAP-Agent Projekts_

---

## 1. Projekt-Profil

| Metrik | Wert | Bewertung |
|--------|------|-----------|
| **Sprache** | Rust 2021 | |
| **Projektart** | CLI-Tool + Library + REST-API | Multi-Binary |
| **Quellcode** | 76 Dateien, ~28.000 LOC | Mittelgroß |
| **Tests** | 400 Unit-Tests, 718 Test-Funktionen | Gut abgedeckt |
| **Externe Deps** | ~40 Crates | Moderat |

### Architektur-Übersicht

```
cap-agent/
├── src/
│   ├── main.rs           # 4.467 LOC - CLI Entry Point (KRITISCH)
│   ├── lib.rs            # Public Library API
│   ├── api/              # REST API Layer (Axum)
│   ├── audit/            # Audit Trail System
│   ├── bundle/           # Bundle Format Handling
│   ├── crypto/           # Kryptographie-Wrapper
│   ├── keys.rs           # Key Management (924 LOC)
│   ├── manifest.rs       # Manifest Handling (871 LOC)
│   ├── orchestrator/     # Policy Orchestration
│   ├── package_verifier.rs  # Package Verification (1.070 LOC)
│   ├── policy/           # Policy Engine v1
│   ├── policy_v2/        # Policy Engine v2 (IR)
│   ├── proof_engine.rs   # Proof Generation (666 LOC)
│   ├── providers/        # Key Providers (HSM, Software)
│   ├── registry/         # Key Registry
│   ├── verifier/         # Core Verification Logic
│   └── zk_system.rs      # ZK Abstraction (622 LOC)
└── tests/                # 39 Integration Test Files
```

---

## 2. Selbstreflexion: Bisheriges Vorgehen & Fehlerpotential

### Was gut funktioniert hat
- **Workflow-Disziplin**: 7-Schritte-Prozess mit Security Review
- **Inkrementelle Änderungen**: Kleine, verifizierbare Schritte
- **Test-First-Validierung**: Immer `cargo test` vor und nach Änderungen
- **CAP-Kompatibilität**: Keine Änderung an Krypto-Verhalten

### Potentielle Fehlerquellen bei Refactoring
| Risiko | Beschreibung | Mitigation |
|--------|--------------|------------|
| **API-Bruch** | Öffentliche Funktionen/Structs ändern | Nur interne Refactorings |
| **Verhaltensänderung** | Subtile Logik-Unterschiede | Extensive Tests vor/nach |
| **Krypto-Inkonsistenz** | Hash-Reihenfolge, Serialisierung | BLAKE3/SHA3 nie anfassen |
| **CLI-Regression** | Argument-Parsing, Output-Format | E2E-Tests behalten |
| **Parallelität** | Änderungen an async Code | Sequentiell testen |

### Projekt-spezifische Regeln (NICHT VERLETZEN)

1. **Krypto-Invarianten**:
   - BLAKE3 für Commitments
   - SHA3-256 für Audit-Trail
   - Ed25519 für Signaturen
   - Keine Änderung an Hash-Berechnungen

2. **Öffentliche API** (lib.rs exports):
   - `pub mod` Deklarationen beibehalten
   - Struct-Namen nicht ändern
   - Funktion-Signaturen stabil halten

3. **CLI-Kompatibilität**:
   - Subcommand-Namen beibehalten
   - Argument-Namen beibehalten
   - Output-Format für Scripts stabil

---

## 3. Identifizierte Code-Smells

### 3.1 KRITISCH: `main.rs` ist zu groß (4.467 LOC)

**Problem:**
- 52 `run_*` Funktionen in einer Datei
- Schwer navigierbar und wartbar
- Keine klare Trennung zwischen Command-Gruppen

**Aktuelle Struktur:**
```rust
// main.rs enthält ALLES:
fn run_prepare(...)        // Data Preparation
fn run_manifest_build(...) // Manifest Commands
fn run_proof_build(...)    // Proof Commands
fn run_sign_keygen(...)    // Signing Commands
fn run_registry_add(...)   // Registry Commands
fn run_audit_tip(...)      // Audit Commands
fn run_keys_keygen(...)    // Key Management
fn run_verifier_run(...)   // Verification
// ... 44 weitere Funktionen
```

**Empfohlene Struktur:**
```
src/
├── main.rs           # Nur CLI-Parsing + Dispatch (~200 LOC)
├── cli/
│   ├── mod.rs        # CLI Enum Definitions
│   ├── prepare.rs    # run_prepare, run_inspect
│   ├── manifest.rs   # run_manifest_*
│   ├── proof.rs      # run_proof_*, run_zk_*
│   ├── sign.rs       # run_sign_*
│   ├── registry.rs   # run_registry_*
│   ├── audit.rs      # run_audit_*
│   ├── keys.rs       # run_keys_*
│   └── verifier.rs   # run_verifier_*
```

### 3.2 Duplizierter Code

**Beispiel 1: Fehlerbehandlung mit println!**
```rust
// Wiederholt sich ~50x in main.rs:
match some_operation() {
    Ok(result) => println!("✓ Success: {}", result),
    Err(e) => {
        eprintln!("✗ Error: {}", e);
        std::process::exit(1);
    }
}
```

**Lösung:**
```rust
fn handle_result<T: std::fmt::Display>(result: Result<T, Box<dyn Error>>, success_msg: &str) {
    match result {
        Ok(val) => println!("✓ {}: {}", success_msg, val),
        Err(e) => {
            eprintln!("✗ Error: {}", e);
            std::process::exit(1);
        }
    }
}
```

**Beispiel 2: JSON-Output-Pattern**
```rust
// Wiederholt sich ~20x:
if let Some(output) = output {
    fs::write(&output, serde_json::to_string_pretty(&data)?)?;
    println!("Written to {}", output);
} else {
    println!("{}", serde_json::to_string_pretty(&data)?);
}
```

### 3.3 Inkonsistente Output-Patterns

| Pattern | Vorkommen | Problem |
|---------|-----------|---------|
| `println!("✓ ...")` | ~80x | Emoji-Inkonsistenz |
| `eprintln!("Error: ...")` | ~40x | Format variiert |
| `eprintln!("DEBUG: ...")` | ~15x | Sollte `tracing` nutzen |

### 3.4 Große Funktionen (>100 LOC)

| Funktion | LOC | Problem |
|----------|-----|---------|
| `run_registry_add` | ~130 | Zu viele Verantwortlichkeiten |
| `run_proof_export` | ~280 | Mehrere Export-Formate vermischt |
| `run_zk_build` | ~150 | Build-Logik + CLI-Output gemischt |

### 3.5 Unused Imports & Dead Code

- 55 `#[allow(dead_code)]` Annotationen
- Legitim für ZK-Stubs, aber unübersichtlich
- Sollte dokumentiert werden warum

---

## 4. Refactoring-Plan (Priorisiert)

### Phase 1: main.rs aufteilen (HÖCHSTE PRIORITÄT)

**Ziel:** 4.467 LOC → ~200 LOC main.rs + 8 CLI-Module

**Schritte:**
1. `src/cli/mod.rs` erstellen mit Subcommand-Enums
2. `run_prepare`, `run_inspect` → `src/cli/prepare.rs`
3. `run_manifest_*` → `src/cli/manifest.rs`
4. `run_proof_*`, `run_zk_*` → `src/cli/proof.rs`
5. `run_sign_*` → `src/cli/sign.rs`
6. `run_registry_*` → `src/cli/registry.rs`
7. `run_audit_*` → `src/cli/audit.rs`
8. `run_keys_*` → `src/cli/keys.rs`
9. `run_verifier_*` → `src/cli/verifier.rs`
10. main.rs nur noch: Parsing + match-Dispatch

**Verhaltensrisiko:** NIEDRIG (nur Code-Organisation)

### Phase 2: Output-Helpers konsolidieren

**Ziel:** Einheitliche CLI-Output-Funktionen

**Schritte:**
1. `src/cli/output.rs` erstellen
2. `print_success()`, `print_error()`, `print_warning()` definieren
3. `write_json_output()` für JSON-File-oder-Stdout
4. Alle `println!`/`eprintln!` durch Helper ersetzen

**Verhaltensrisiko:** NIEDRIG (nur Output-Format)

### Phase 3: Große Funktionen aufteilen

**Ziel:** Keine Funktion >80 LOC

**Kandidaten:**
- `run_proof_export` → `export_capz()`, `export_json()`, `export_raw()`
- `run_registry_add` → `validate_key()`, `store_key()`, `print_result()`

**Verhaltensrisiko:** MITTEL (Logik-Extraktion)

### Phase 4: Test-Abdeckung für CLI

**Ziel:** CLI-Regression verhindern

**Schritte:**
1. `tests/cli_*.rs` für jedes CLI-Modul
2. Snapshot-Tests für Output-Format
3. E2E-Tests für kritische Workflows

**Verhaltensrisiko:** KEINES (nur Tests hinzufügen)

---

## 5. Refactoring-Regeln für dieses Projekt

### DO (Erlaubt)
- [ ] Code in neue Dateien verschieben
- [ ] Private Hilfsfunktionen erstellen
- [ ] Imports reorganisieren
- [ ] Kommentare verbessern
- [ ] Formatting mit `cargo fmt`
- [ ] Interne Struct-Namen in neuen Modulen

### DON'T (Verboten)
- [ ] Öffentliche API ändern (lib.rs exports)
- [ ] CLI-Argument-Namen ändern
- [ ] Hash-Berechnungen modifizieren
- [ ] Krypto-Konstanten ändern
- [ ] Bestehende Tests löschen
- [ ] Neue externe Dependencies

### WARN (Mit Vorsicht)
- [ ] Funktions-Signaturen ändern (nur intern)
- [ ] Error-Messages ändern (können in Scripts geparst werden)
- [ ] Output-Format ändern (Rücksprache halten)

---

## 6. Validierungs-Checkliste (nach jeder Änderung)

```bash
# 1. Kompiliert alles?
cargo build --all-targets

# 2. Keine Warnungen?
cargo clippy --all-targets

# 3. Alle Tests grün?
cargo test --lib
cargo test --tests

# 4. Docs generierbar?
cargo doc --lib --no-deps

# 5. CLI funktioniert?
cargo run -- --help
cargo run -- prepare --help
cargo run -- verifier run --help
```

---

## 7. Vorgeschlagene Reihenfolge für Refactoring-Sessions

| Session | Fokus | Geschätzte Änderungen | Risiko |
|---------|-------|----------------------|--------|
| 1 | `cli/mod.rs` Grundstruktur | Neue Dateien | Niedrig |
| 2 | `run_prepare`, `run_inspect` extrahieren | ~150 LOC verschieben | Niedrig |
| 3 | `run_manifest_*` extrahieren | ~200 LOC verschieben | Niedrig |
| 4 | `run_proof_*` extrahieren | ~600 LOC verschieben | Mittel |
| 5 | `run_audit_*` extrahieren | ~400 LOC verschieben | Mittel |
| 6 | `run_registry_*` extrahieren | ~500 LOC verschieben | Mittel |
| 7 | `run_keys_*` extrahieren | ~300 LOC verschieben | Niedrig |
| 8 | `run_sign_*`, `run_verifier_*` | ~300 LOC verschieben | Niedrig |
| 9 | Output-Helpers konsolidieren | ~100 LOC neu | Niedrig |
| 10 | Große Funktionen aufteilen | Intern | Mittel |

---

## 8. Metriken für Erfolg

**Vorher:**
- `main.rs`: 4.467 LOC
- Funktionen in main.rs: 53
- Max Funktion LOC: ~280

**Nachher (Ziel):**
- `main.rs`: <300 LOC
- Funktionen pro CLI-Modul: 5-8
- Max Funktion LOC: <80
- Keine neuen Clippy-Warnings
- Alle 400 Tests grün

---

## 9. Entscheidungsbaum: Wann NICHT refactoren?

```
Ist die Änderung rein strukturell?
├── NEIN → STOP (Verhaltensänderung vermeiden)
└── JA → Weiter

Sind alle Tests vorhanden für betroffenen Code?
├── NEIN → Erst Tests schreiben
└── JA → Weiter

Ist die öffentliche API betroffen?
├── JA → STOP (API-Bruch)
└── NEIN → Weiter

Ist Krypto-Code betroffen?
├── JA → STOP (CAP-Invarianten)
└── NEIN → Refactoring erlaubt
```

---

_Erstellt: 2025-11-30_
_Projekt: CAP-Agent v0.1.0 MVP_
_Autor: Claude (Refactoring-Analyse)_
