# TODO: Echtes Refactoring

> Erstellt: 2025-11-30
> Aktualisiert: Nach Session 23 - Fokus auf echte Code-Reduktion

---

## Abgeschlossen (Session 17-23): Dead-Code Analyse

**15 falsche `#[allow(dead_code)]` Annotationen entfernt:**
- `keys.rs`: 1 (`public_key_bytes()`)
- `hash_chain.rs`: 11 (komplettes Modul produktiv genutzt)
- `metadata.rs`: 3 (`PolicyStatus`, `PolicyMetadata`, `CompiledPolicy`)

---

## Phase 2: Echtes Refactoring (LOC-Reduktion)

### Priorisierung

| Priorität | Kriterium |
|-----------|-----------|
| 🔴 P1 | >1000 LOC, komplexe Logik, aufspaltbar |
| 🟠 P2 | 800-1000 LOC, Mixed Concerns |
| 🟡 P3 | Code-Duplikation, Trait-Migration |
| 🟢 P4 | Nice-to-have, Optimierungen |

---

## 🔴🔴 P0: Systemweite Probleme

### 0. Debug-Prints entfernen ✅ (Session 24)
- [x] **Problem**: 143x `eprintln!`/`println!`/`dbg!` in Produktionscode
- [x] **Erledigt**:
  - `src/package_verifier.rs`: 11x DEBUG-Prints entfernt (17 LOC)
  - `src/verifier/core_verify.rs`: 2x DEBUG-Prints entfernt (5 LOC)
- [x] **Verbleibend** (legitimer CLI-Output):
  - `src/cli/registry.rs`: 59x (CLI-Output für User)
  - `src/cli/output.rs`: 30x (Output-Modul)
  - `src/policy_v2/cli.rs`: 16x (CLI-Output)
- [x] **Ersparnis**: ~22 LOC + sauberere Logs

---

## 🔴 P1: Große Dateien verbessern

### 1. `src/package_verifier.rs` (1,037 LOC) ✅ Teilweise (Session 24-25)

#### Erledigte Probleme:
- [x] **Debug-Spam**: 11x `eprintln!("DEBUG:` entfernt (Session 24)
- [x] **Duplikation**: `BundleVerifier` → Type-Alias für `Verifier` (Session 25)

#### Verbleibende Probleme:
- [ ] **Redundante Structs**: `VerificationResult` vs `BundleVerifyResult` - fast gleiche Felder
- [ ] **Copy-Paste Code**: Hash-Validierung in `validate_file_hash()` und `verify_proof_unit_internal()` ähnlich
- [ ] **Tests**: 500+ LOC Tests - viele repetitiv mit `create_test_manifest()`

#### Erledigte Ersparnis: ~38 LOC (17 + 16 + 5)

### 2. `src/registry/v1_0.rs` (1,068 LOC) ✅ KOMPLETT (Session 29 + 31)

#### Erledigte Probleme:
- [x] **Session 29**: Timestamp-Duplikation entfernt (~40 LOC)
- [x] **Session 31**: ECHTES REFACTORING - Datei aufgeteilt in 4 Module:
  - `entry.rs` (93 LOC) - RegistryEntry Struct
  - `signing.rs` (187 LOC) - Ed25519 Signing/Verification
  - `timestamp.rs` (252 LOC) - Timestamp + Provider Abstraktion
  - `store.rs` (473 LOC) - Registry, RegistryStore Trait, JSON/SQLite Backends
  - `v1_0.rs` (80 LOC) - Re-Export Layer für Backward-Kompatibilität
- [x] **Ergebnis**: 1,068 LOC → 4 fokussierte Module mit je <300 LOC (außer store.rs)
- [x] **Tests**: 36 Registry-Tests grün

#### Erledigte Ersparnis:
- LOC-Reduktion: ~0 (Code wurde aufgeteilt, nicht entfernt)
- Code-Qualität: STARK VERBESSERT - Single-Responsibility-Principle umgesetzt

---

## 🟠 P2: Mixed Concerns trennen

### 3. `src/api/verify.rs` (897 LOC) ✅ Teilweise (Session 27)

#### Erledigte Probleme:
- [x] **Dead Code entfernt**: `build_manifest_from_context()` gelöscht (33 LOC)
- [x] **Dead Code entfernt**: `compute_policy_hash()` gelöscht (5 LOC) - nur von gelöschter Funktion genutzt
- [x] **Dead Test entfernt**: `test_compute_policy_hash_deterministic()` (16 LOC)
- [x] **Cleanup**: Unused imports in `proof_mock.rs` entfernt (4 LOC)

#### Verbleibende Probleme:
- [ ] **Redundante Hash-Berechnung**: `compute_company_root()` prüft `company_commitment_root` zweimal
- [ ] **Tests ~520 LOC**: Viel Boilerplate mit `create_test_ir()`, `create_test_context()`

#### Erledigte Ersparnis: ~63 LOC (verify.rs) + ~5 LOC (proof_mock.rs) = **~68 LOC**

### 4. `src/api/policy_compiler.rs` (851 LOC) - Analyse abgeschlossen (Session 30)

#### Analyse-Ergebnis:
- [x] **Globaler State**: OnceLock+Arc+Mutex ist Rust-idiomatisch für threadsafe singletons
- [x] **Test-Helper `pub`**: Sind `pub` für Integration Tests außerhalb des Moduls - legitim
- [x] **Error-Mapping**: 6x gleicher Code, aber nur ~6 LOC Ersparnis durch Helper - minimal

#### Entscheidung:
Keine Änderungen - Aufwand übersteigt Nutzen. Die Datei ist gut strukturiert.

#### Geschätzte Ersparnis: ~0 LOC (keine Änderung)

### 5. `src/keys.rs` (919 LOC) ✅ Teilweise (Session 28)

#### Erledigte Probleme:
- [x] **Scan-Helper extrahiert**: `scan_json_files()` ersetzt duplizierten Code in `list()` und `archive()`

#### Verbleibende Probleme:
- [ ] **Duplikation**: `KeyMetadata::load/save` vs `SignedAttestation::load` - nicht migrierbar (nutzt `anyhow::Result`)
- [ ] **Legitime dead_code**: `revoke()` und `get_active()` - für zukünftige Features
- [ ] **Tests ~520 LOC**: Sehr ausführlich, einige repetitiv

#### Erledigte Ersparnis: ~5 LOC (Code-Qualität verbessert, Duplikation entfernt)

---

## 🟡 P3: Code-Duplikation & Trait-Migration

### 6. JsonPersistent Trait anwenden ✅ Teilweise (Session 26)
- [x] **Status**: Trait existiert in `src/io.rs` (Session 17)
- [x] **Migriert (Session 26)**:
  - `Proof` in `src/proof_engine.rs` (~25 LOC entfernt)
  - `MockProof` in `src/proof_mock.rs` (~25 LOC entfernt)
- [ ] **Nicht migrierbar** (andere Error-Signatur):
  - `KeyMetadata` in `src/keys.rs` (nutzt `anyhow::Result`)
  - `Manifest` in `src/manifest.rs` (hat spezielle Signatur-Logik)
- [ ] **Ersparnis**: ~50 LOC

### 7. Error-Handling vereinheitlichen
- [ ] **Problem**: Mix aus `Box<dyn Error>`, `anyhow::Result`, custom Errors
- [ ] **Action**:
  - Definiere `cap_agent::Error` enum
  - Migriere schrittweise
- [ ] **Aufwand**: Hoch, über mehrere Sessions

---

## 🟢 P4: Optimierungen

### 8. Test-Utilities konsolidieren
- [ ] **Problem**: Ähnliche Setup-Funktionen in Test-Dateien
- [ ] **Action**: `tests/common/mod.rs` erstellen
- [ ] **Aufwand**: Niedrig

### 9. `src/manifest.rs` (871 LOC) aufräumen
- [ ] **Analyse**: Groß aber gut strukturiert
- [ ] **Action**: Nach JsonPersistent-Migration evaluieren

---

## Gesamte geschätzte Ersparnis

| Bereich | LOC |
|---------|-----|
| P0: Debug-Prints | ~100 |
| P1: package_verifier.rs | ~200-300 |
| P1: registry/v1_0.rs | ~50-80 |
| P2: api/verify.rs | ~80 |
| P2: policy_compiler.rs | ~50 |
| P2: keys.rs | ~60 |
| P3: JsonPersistent | ~80 |
| **Total** | **~620-750 LOC** |

---

## Empfohlene Reihenfolge

```
Session 24: P0 - Debug-Prints entfernen (schneller Gewinn, 100+ LOC)
Session 25: P1 - package_verifier.rs (größte Ersparnis, 200-300 LOC)
Session 26: P3 - JsonPersistent Migration (wirkt auf viele Dateien)
Session 27: P2 - api/verify.rs Dead Code + Duplikation
Session 28: P2 - keys.rs Trait + Scan-Helper
Session 29: P1 - registry/v1_0.rs Timestamp-Duplikation
Session 30: P2 - policy_compiler.rs Cache-Struct
```

---

## Metriken (aktualisiert nach Session 33)

| Metrik | Session 30 | Session 33 | Ziel |
|--------|------------|------------|------|
| Dateien >800 LOC | 6 | 2 | 2 ✅ |
| Dateien >500 LOC | 10 | 4 | 4 ✅ |
| Größte Datei in registry/ | 1,068 LOC | 473 LOC | <500 LOC ✅ |
| Größte Datei in verifier/ | 1,042 LOC | 468 LOC | <500 LOC ✅ |
| Größte Datei in package_verifier/ | 1,037 LOC | 286 LOC | <500 LOC ✅ |

**Top 5 größte Dateien nach Session 37:**
1. `keys/mod.rs` - 503 LOC (aufgeteilt)
2. `api/verify/mod.rs` - 533 LOC (aufgeteilt)
3. `manifest/mod.rs` - 462 LOC (aufgeteilt)
4. `api/policy_compiler/mod.rs` - 396 LOC (aufgeteilt)
5. `verifier/core.rs` - 468 LOC (aufgeteilt)

**Alle Dateien >1000 LOC eliminiert!** 🎉

**Registry-Module (alle <500 LOC):**
- `store.rs` - 473 LOC
- `timestamp.rs` - 252 LOC
- `signing.rs` - 187 LOC
- `entry.rs` - 93 LOC
- `v1_0.rs` - 80 LOC

**Verifier-Module (alle <500 LOC):**
- `core.rs` - 468 LOC
- `verify.rs` - 319 LOC
- `statement.rs` - 172 LOC
- `types.rs` - 115 LOC

**Package-Verifier-Module (alle <300 LOC):**
- `verifier.rs` - 286 LOC
- `mod.rs` - 270 LOC (Tests)
- `validation.rs` - 88 LOC
- `types.rs` - 68 LOC
- `summary.rs` - 68 LOC

---

## Regeln für echtes Refactoring

1. **Keine Funktionalitätsänderung** - nur Struktur
2. **Alle Tests müssen grün bleiben** nach jeder Session
3. **Schrittweise** - eine Datei pro Session
4. **Re-Exports** - öffentliche API bleibt kompatibel via `mod.rs`
5. **Commit nach jeder Session** mit klarer Beschreibung

---

## Session Log (Phase 2)

| Session | Datum | Datei | LOC vorher | LOC nachher | Ersparnis |
|---------|-------|-------|------------|-------------|-----------|
| 24 | 2025-11-30 | P0: DEBUG-Prints | 1,053 | 1,053 | 22 LOC |
| 25 | 2025-11-30 | P1: BundleVerifier→Alias | 1,053 | 1,037 | 16 LOC |
| 26 | 2025-11-30 | P3: JsonPersistent (Proof, MockProof) | - | - | ~50 LOC |
| 27 | 2025-11-30 | P2: api/verify.rs Dead Code | 960 | 897 | 68 LOC |
| 28 | 2025-11-30 | P2: keys.rs scan_json_files | 924 | 919 | 5 LOC |
| 29 | 2025-11-30 | P1: registry/v1_0.rs Timestamp Delegation | 1,108 | 1,068 | 40 LOC |
| 30 | 2025-11-30 | P2: policy_compiler.rs Analyse | 851 | 851 | 0 LOC |
| 31 | 2025-11-30 | **P1: registry/v1_0.rs → 4 Module** | 1,068 | 80+93+187+252+473=1,085 | Struktur! |
| 32 | 2025-11-30 | **P1: verifier/core.rs → 3 Module** | 1,042 | 468+115+172+319=1,074 | Struktur! |
| 33 | 2025-11-30 | **P1: package_verifier.rs → 4 Module** | 1,037 | 270+68+68+88+286=780 | -257 LOC! |
| 34 | 2025-11-30 | **P2: keys.rs → 4 Module** | 919 | 503+140+107+165=915 | -4 LOC |
| 35 | 2025-11-30 | **P2: api/verify.rs → 5 Module** | 897 | 533+152+146+71+24=926 | Struktur |
| 36 | 2025-11-30 | **P2: manifest.rs → 5 Module** | 871 | 462+179+61+51+35=788 | -83 LOC |
| 37 | 2025-11-30 | **P2: api/policy_compiler.rs → 5 Module** | 851 | 396+304+77+61+45=883 | Struktur |
| 38 | 2025-12-01 | **P2: cli/proof.rs → bundle/export.rs** | 866 | 561 + 337 = 898 | Struktur! |

**Kumulierte Ersparnis: ~545 LOC + 8x ECHTES STRUKTURELLES REFACTORING**

### Session 31: Echtes Refactoring (Modulaufteilung)

**Ziel erreicht**: `registry/v1_0.rs` (1,068 LOC) wurde in 4 fokussierte Module aufgeteilt:

| Modul | LOC | Verantwortung |
|-------|-----|---------------|
| `entry.rs` | 93 | RegistryEntry Datenstruktur |
| `signing.rs` | 187 | Ed25519 Signing & Verification |
| `timestamp.rs` | 252 | RFC3161 Timestamp Provider |
| `store.rs` | 473 | Registry, JSON/SQLite Backends |
| `v1_0.rs` | 80 | Re-Exports (Backward-Kompatibilität) |

**Ergebnis**: Keine einzelne Datei >500 LOC in `registry/`, Single-Responsibility-Principle

### Session 32: Echtes Refactoring (verifier/core.rs)

**Ziel erreicht**: `verifier/core.rs` (1,042 LOC) wurde in 3 fokussierte Module aufgeteilt:

| Modul | LOC | Verantwortung |
|-------|-----|---------------|
| `types.rs` | 115 | ProofStatement, VerifyOptions, VerifyReport |
| `statement.rs` | 172 | Statement-Extraktion aus Manifests |
| `verify.rs` | 319 | Pure Verification Logic |
| `core.rs` | 468 | Re-Exports + Tests (Integration) |

**Ergebnis**: Klare Trennung von Types, Statement-Parsing, und Verification-Logik

### Session 33: Echtes Refactoring (package_verifier.rs)

**Ziel erreicht**: `package_verifier.rs` (1,037 LOC) wurde in 4 fokussierte Module aufgeteilt:

| Modul | LOC | Verantwortung |
|-------|-----|---------------|
| `types.rs` | 68 | BundleType, VerificationResult, BundleVerifyResult |
| `validation.rs` | 88 | File hash validation (TOCTOU mitigation) |
| `verifier.rs` | 286 | Verifier struct + impl |
| `summary.rs` | 68 | Package summary display |
| `mod.rs` | 270 | Re-Exports + Tests |

**Ergebnis**: 1,037 LOC → 780 LOC = **257 LOC echte Reduktion** durch Entfernung von Test-Duplikation

### Session 34: Echtes Refactoring (keys.rs)

**Ziel erreicht**: `keys.rs` (919 LOC) wurde in 4 fokussierte Module aufgeteilt:

| Modul | LOC | Verantwortung |
|-------|-----|---------------|
| `types.rs` | 140 | Kid, KeyMetadata, derive_kid, compute_fingerprint |
| `store.rs` | 107 | KeyStore (list, find, archive) |
| `attestation.rs` | 165 | Attestation, SignedAttestation, verify_chain |
| `mod.rs` | 503 | Re-Exports + Tests |

**Ergebnis**: Klare Trennung von Types, Store-Logik, und Attestation-Handling

### Session 35: Echtes Refactoring (api/verify.rs)

**Ziel erreicht**: `api/verify.rs` (897 LOC) wurde in 5 fokussierte Module aufgeteilt:

| Modul | LOC | Verantwortung |
|-------|-----|---------------|
| `types.rs` | 146 | VerifyRequest, VerifyContext, VerifyResponse |
| `handler.rs` | 152 | handle_verify, mode resolution |
| `manifest.rs` | 71 | build_manifest_from_ir, compute_company_root |
| `proof.rs` | 24 | create_mock_proof |
| `mod.rs` | 533 | Re-Exports + Tests |

**Ergebnis**: Klare Trennung von Types, Handler-Logik, Manifest-Building, und Proof-Erstellung

### Session 36: Echtes Refactoring (manifest.rs)

**Ziel erreicht**: `manifest.rs` (871 LOC) wurde in 5 fokussierte Module aufgeteilt:

| Modul | LOC | Verantwortung |
|-------|-----|---------------|
| `types.rs` | 51 | Manifest, AuditInfo, ProofInfo, SignatureInfo |
| `anchor.rs` | 61 | TimeAnchor, TimeAnchorPrivate/Public, PublicChain |
| `io.rs` | 179 | build, save/load, anchor methods |
| `signed.rs` | 35 | SignedManifest |
| `mod.rs` | 462 | Re-Exports + Tests |

**Ergebnis**: 871 LOC → 788 LOC = **83 LOC echte Reduktion**

### Session 37: Echtes Refactoring (api/policy_compiler.rs)

**Ziel erreicht**: `api/policy_compiler.rs` (851 LOC) wurde in 5 fokussierte Module aufgeteilt:

| Modul | LOC | Verantwortung |
|-------|-----|---------------|
| `cache.rs` | 45 | LRU Cache, PolicyEntry Struct |
| `types.rs` | 77 | Request/Response Structures |
| `handlers.rs` | 304 | HTTP Handler für compile/get |
| `test_helpers.rs` | 61 | Test-Utilities für Cache-Manipulation |
| `mod.rs` | 396 | Re-Exports + Tests |

**Ergebnis**: Klare Trennung von Cache, Types, Handlers, und Test-Utilities

### Session 38: Echtes Refactoring (cli/proof.rs)

**Ziel erreicht**: `cli/proof.rs` (866 LOC) Export-Logik nach `bundle/export.rs` extrahiert:

| Modul | LOC | Verantwortung |
|-------|-----|---------------|
| `bundle/export.rs` | 337 | Export-Logik (prepare_dir, copy_files, create_meta) |
| `cli/proof.rs` | 561 | CLI-Handler (proof mock/build/verify/export/zk) |

**Änderungen**:
- Neues Modul `bundle/export.rs` mit 8 Funktionen erstellt
- `cli/proof.rs` um ~305 LOC reduziert (866 → 561)
- `run_proof_export` delegiert jetzt an `export::export_bundle()`
- Alle Tests grün (10 bundle-tests, 34 proof-tests)

**Ergebnis**: CLI-Layer ist jetzt dünn (nur parse → call core → render)

---

## Phase 2 abgeschlossen

**Zusammenfassung der Refactoring-Sessions 24-30:**
- 7 Sessions durchgeführt
- ~201 LOC echte Ersparnis
- Code-Qualität verbessert durch Deduplizierung und Trait-Migration
- Keine Funktionalitätsänderungen

---

## Archiv: Phase 1 (Session 17-23)

<details>
<summary>Dead-Code Analyse abgeschlossen</summary>

| Session | Datum | Ergebnis |
|---------|-------|----------|
| 17 | 2025-11-30 | `JsonPersistent` Trait erstellt |
| 18 | 2025-11-30 | TimestampProvider + CSV Reader - keine Duplikation |
| 19 | 2025-11-30 | registry/v1_0.rs - dead_code legitim |
| 20 | 2025-11-30 | keys.rs - 1x dead_code entfernt |
| 21 | 2025-11-30 | zk_system.rs - dead_code legitim |
| 22 | 2025-11-30 | hash_chain.rs - 11x dead_code entfernt |
| 23 | 2025-11-30 | metadata.rs - 3x dead_code entfernt |

</details>
