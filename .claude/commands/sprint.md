# 🚀 Engineering Sprint - CAP-konformer Workflow

**Dieser Command führt dich durch einen strukturierten, CAP-konformen Entwicklungssprint.**

Basierend auf:
- CAP_ENGINEERING_GUIDE.md
- CAP_GRUNDANFORDERUNGEN.md
- CAP_SECURITY_REQUIREMENTS.md
- CAP_PROJECT_POLICY.md

---

## ⚠️ WICHTIG: Schrittweises Vorgehen

**Führe jeden Schritt einzeln aus und warte auf User-Bestätigung!**

Dies verhindert:
- Zu große Context-Fenster
- Unkontrollierte Implementierung
- Überspringen von Tests
- Ignorieren von CAP-Policies

---

## Schritt 1: 📊 Projektstand prüfen

**Aufgabe:** Ermittle den aktuellen Stand des CAP-Projekts.

**Zu prüfen:**
1. **Roadmap:** Lies `ROADMAP_MVP_PRODUCTION.md`
   - Welche Woche im 6-Wochen-Plan?
   - Welche Tasks sind offen?
2. **Status:** Lies `docs/Projektübersicht/07-status-und-roadmap.md`
   - Was ist fertig? (v0.11.0)
   - Was kommt als Nächstes?
3. **Tests:** Führe aus:
   ```bash
   cd /Users/tomwesselmann/Desktop/LsKG-Agent/agent
   cargo test 2>&1 | tail -20
   ```
4. **Code Quality:** Prüfe Clippy:
   ```bash
   cargo clippy -- -D warnings 2>&1 | tail -20
   ```

**Ausgabe erstellen:**
```markdown
### Projektstand (Datum: [Heute])

**Roadmap-Position:**
- Woche: [X von 6]
- Phase: [Name der Phase]
- Fortschritt: [X%]

**Letzte fertige Features:**
- [Feature 1]
- [Feature 2]

**Test-Status:**
- Tests: [X/Y bestanden]
- Clippy: [X Warnings]

**Bekannte Issues:**
- [Issue 1]
- [Issue 2]

**Betroffene Module für nächstes Feature:**
- [Modul 1: z.B. proof/halo2_circuit.rs]
- [Modul 2: z.B. api/verify.rs]

**CAP-Sicherheitsimplikationen:**
- [z.B. Determinismus betroffen? Audit Chain betroffen?]
```

**⏸️ STOPP HIER**

Frage den User:
> "Projektstand ermittelt. Soll ich mit **Schritt 2 (Ziele definieren)** weitermachen?"

---

## Schritt 2: 🎯 Ziele definieren (SMART + CAP-konform)

**Aufgabe:** Definiere klare, messbare, CAP-konforme Ziele.

**CAP-Kompatibilitäts-Check:**
Das Ziel MUSS erfüllen:
- ✅ Deterministisch (gleiche Inputs → gleiche Outputs)
- ✅ Reproduzierbar (von Auditoren nachvollziehbar)
- ✅ Auditierbar (in Audit Chain dokumentiert)
- ✅ Privacy-preserving (keine Raw Data Leaks)
- ✅ Krypto-korrekt (BLAKE3/SHA3/Ed25519)
- ✅ Versioniert (Schema-Versions-Support)

**Vorgehen:**
1. Nächstes Feature laut Roadmap identifizieren
2. Prüfen: Kann es in <2h umgesetzt werden?
   - Wenn JA → Als Ziel definieren
   - Wenn NEIN → In Sub-Tasks aufteilen (max. 5)
3. SMART-Kriterien anwenden:
   - **S**pecific: Was genau wird gebaut?
   - **M**easurable: Woran erkenne ich "fertig"?
   - **A**chievable: Ist es in einer Session machbar?
   - **R**elevant: Passt es zur Roadmap?
   - **T**ime-bound: Wie lange dauert es?

**Ausgabe erstellen:**
```markdown
### Sprint-Ziele

**Hauptziel:**
[Ein Satz: z.B. "Implementiere Halo2 Circuit für 'require_at_least_one_ubo' Constraint"]

**Sub-Tasks:**
1. [ ] [Task 1: z.B. Halo2-Crate in Cargo.toml integrieren]
2. [ ] [Task 2: z.B. Basic Circuit Struct definieren]
3. [ ] [Task 3: z.B. Constraint-Logik implementieren]
4. [ ] [Task 4: z.B. Proof Generation Funktion schreiben]
5. [ ] [Task 5: z.B. Integration in proof_engine.rs]

**Success Criteria:**
- [ ] Code kompiliert ohne Warnings
- [ ] Halo2-Proof wird generiert (<1 MB Größe)
- [ ] Unit Tests grün (mind. 3 neue Tests)
- [ ] CAP-Policies erfüllt (deterministisch, auditierbar)

**Geschätzte Dauer:** [z.B. 90 Minuten]

**CAP-Kompatibilität:**
- Deterministisch: ✅ [Begründung]
- Reproduzierbar: ✅ [Begründung]
- Auditierbar: ✅ [Begründung]
- Privacy-preserving: ✅ [Begründung]
```

Nutze TodoWrite, um Sub-Tasks zu tracken!

**⏸️ STOPP HIER**

Frage den User:
> "Ziele definiert. Sind sie CAP-konform und OK? Soll ich mit **Schritt 3 (Tests definieren)** weitermachen?"

---

## Schritt 3: 🧪 Tests definieren (TDD + CAP-konform)

**Aufgabe:** Definiere Tests BEVOR du implementierst (Test-Driven Development).

**CAP Test-Kategorien:**
1. **Unit Tests** (deterministisch, isoliert)
2. **Integration Tests** (E2E Workflows)
3. **Property Tests** (Invarianten, z.B. KID-Uniqueness)
4. **Security Tests** (Threat Model basiert)

**Vorgehen:**
Für jedes Sub-Task aus Schritt 2:
1. Welche **Unit Tests** brauche ich?
   - Happy Path
   - Edge Cases (Grenzen)
   - Error Cases (Fehler)
2. Welche **Integration Tests**?
   - Workflow: CSV → Proof → Verify
3. Welche **Property Tests**?
   - Determinismus-Tests
   - Hash-Invarianz-Tests
4. Welche **Security Tests**?
   - Timeout-Tests
   - Invalid Input-Tests
   - Memory Safety-Tests

**Beispiel aus CAP_ENGINEERING_GUIDE:**
```rust
// Unit Test
#[test]
fn test_sanctions_csv_deterministic_root() {
    let csv1 = parse_sanctions("sanctions.csv");
    let csv2 = parse_sanctions("sanctions.csv");
    assert_eq!(merkle_root(&csv1), merkle_root(&csv2));
}

// Property Test
#[test]
fn test_merkle_root_order_invariance() {
    let data = vec!["A", "B", "C"];
    let shuffled = vec!["C", "A", "B"];
    assert_eq!(merkle_root(&data), merkle_root(&shuffled)); // oder nicht!
}

// Security Test
#[test]
fn test_odata_timeout() {
    let result = fetch_with_timeout(5000); // 5s
    assert!(result.is_err()); // muss nach 5s abbrechen
}
```

**Ausgabe erstellen:**
```markdown
### Test-Plan

**Neue Unit Tests:**
1. `test_halo2_circuit_basic` - Happy Path, Circuit kompiliert
2. `test_halo2_constraint_ubo_count` - Constraint prüft UBO-Anzahl
3. `test_halo2_proof_determinism` - Gleiche Inputs → gleicher Proof
4. `test_halo2_proof_size_limit` - Proof < 1 MB

**Neue Integration Tests:**
1. `test_halo2_end_to_end` - CSV → Manifest → Halo2 Proof → Verify

**Neue Property Tests:**
1. `test_halo2_proof_reproducibility` - 100 Runs → gleicher Output

**Neue Security Tests:**
1. `test_halo2_invalid_constraint` - Invalid Constraint → Error

**Zu ändernde Tests:**
- `proof_engine::tests::test_proof_build` - Erweitern für Halo2-Backend

**Benötigte Test-Daten:**
- `examples/suppliers_halo2.csv` (10 Zeilen)
- `examples/policy_halo2.yml` (mit UBO-Constraint)
```

**⏸️ STOPP HIER**

Frage den User:
> "Tests definiert. Sind sie vollständig und CAP-konform? Soll ich mit **Schritt 4 (Implementieren)** weitermachen?"

---

## Schritt 4: ⚙️ Implementieren (CAP-konform)

**Aufgabe:** Implementiere das Feature schrittweise, CAP-Policies beachtend.

**CAP Engineering-Prinzipien:**
1. **Functional Core, Imperative Shell**
   - Kernlogik → pure functions (kein I/O)
   - I/O → nur in CLI/API/SAP Adapter
2. **Hash Early, Hash Often**
   - Raw Data → sofort hashen
   - Keine Raw Data im Speicher halten
3. **Append-only Logs**
   - Audit Chain → nur append, nie delete/edit
4. **Separation of Concerns**
   - Jedes Modul hat eine klare Rolle

**Verbotene Rust-Praktiken:**
- ❌ `unwrap()` (verwende `?` oder `match`)
- ❌ `expect()` (verwende `thiserror`)
- ❌ `panic!()` (graceful error handling)
- ❌ `unsafe` (ohne Dokumentation)
- ❌ `println!()` (verwende `tracing`)
- ❌ `rand::thread_rng()` (ohne Seed)

**Vorgehen:**
1. Arbeite Sub-Tasks aus Schritt 2 ab (TodoWrite nutzen!)
2. Nach jedem Sub-Task:
   - Code kompilieren: `cargo build`
   - Tests laufen lassen: `cargo test <test_name>`
   - Git Commit erstellen
3. Dokumentiere mit `///` Kommentaren
4. Halte Clippy-Standards ein: `cargo clippy`

**Beispiel-Commit-Message:**
```
feat(proof): Add Halo2 circuit for UBO constraint

- Implement Halo2Backend struct (proof/halo2_circuit.rs)
- Add prove() and verify() functions
- Integrate with ProofSystem trait
- Tests: test_halo2_circuit_basic passing

Refs: ROADMAP_MVP_PRODUCTION.md Week 1
CAP-Compliant: Deterministic, Auditable

🤖 Generated with Claude Code
```

**⏸️ STOPP HIER**

Frage den User:
> "Implementation abgeschlossen. Code kompiliert. Soll ich mit **Schritt 5 (Testen)** weitermachen?"

---

## Schritt 5: ✅ Testen (Vollständiger Test-Durchlauf)

**Aufgabe:** Führe alle Tests aus, fixe Fehler, prüfe CAP-Policies.

**Vorgehen:**
1. **Neue Tests ausführen:**
   ```bash
   cargo test test_halo2 --lib
   ```
2. **Alle Tests ausführen:**
   ```bash
   cargo test
   ```
3. **Clippy prüfen:**
   ```bash
   cargo clippy -- -D warnings
   ```
4. **Performance-Check (falls relevant):**
   ```bash
   cargo bench --bench halo2_bench
   ```
5. **Bei Fehlern:**
   - Fehler analysieren
   - Fehler fixen
   - Erneut testen (Schritt 1-3 wiederholen)

**CAP-Policies-Check:**
- [ ] Deterministisch? (Gleiche Inputs → gleiche Outputs)
- [ ] Reproduzierbar? (Test 10x → gleicher Output)
- [ ] Auditierbar? (Audit Chain aktualisiert?)
- [ ] Privacy-preserving? (Keine Raw Data Leaks?)
- [ ] Krypto-korrekt? (BLAKE3/SHA3/Ed25519 verwendet?)

**Ausgabe erstellen:**
```markdown
### Test-Ergebnisse

**Unit Tests:**
- ✅ [X/Y] bestanden
- ❌ [Fehlgeschlagene Tests]

**Integration Tests:**
- ✅ [X/Y] bestanden

**Clippy:**
- ✅ 0 Warnings / ❌ [X] Warnings

**Performance:**
- Proof Generation: [X]ms (Ziel: <10s)
- Proof Size: [X]KB (Ziel: <1MB)

**CAP-Policies:**
- ✅ Deterministisch
- ✅ Reproduzierbar (10/10 Tests identisch)
- ✅ Auditierbar (Event "halo2_proof_generated" geloggt)
- ✅ Privacy-preserving (nur Commitments, keine Raw Data)
- ✅ Krypto-korrekt (BLAKE3 für Merkle Roots)

**Gefixte Bugs:**
- [Bug 1: Beschreibung + Fix]
```

**⏸️ STOPP HIER**

Frage den User:
> "Alle Tests grün? CAP-Policies erfüllt? Soll ich mit **Schritt 6 (Dokumentation + Roadmap)** weitermachen?"

---

## Schritt 6: 📝 Dokumentation & Roadmap aktualisieren

**Aufgabe:** Dokumentiere den Fortschritt in allen relevanten Dateien.

**Zu aktualisieren:**

### 1. ROADMAP_MVP_PRODUCTION.md
- [ ] Checkboxen setzen für fertige Tasks
- [ ] Status-Emoji aktualisieren (🔄 → ✅)
- [ ] Fortschritts-Prozent aktualisieren

**Beispiel:**
```markdown
#### Tag 1-2: Halo2 Setup & Circuit Design
- [x] Halo2-Crate integrieren (`halo2_proofs = "0.3"`) ✅
- [x] Basic Circuit für Policy-Constraints implementieren ✅
- [x] Test-Circuit kompiliert & prüft Constraints ✅
- **Deliverable:** `proof/halo2_circuit.rs` mit Basic Circuit ✅
```

### 2. docs/Projektübersicht/07-status-und-roadmap.md
- [ ] "Was ist FERTIG" Sektion aktualisieren
- [ ] Feature-Matrix aktualisieren (v0.11.0 → v0.12.0)

**Beispiel:**
```markdown
**Halo2 ZK-Proofs:**
- [x] Halo2-Crate integriert ✅
- [x] Basic Circuit implementiert ✅
- [ ] **TODO:** Proof Generation optimieren (<10s)
```

### 3. agent/CLAUDE.md (falls neue Module)
- [ ] Modul-Dokumentation hinzufügen
- [ ] CLI-Kommandos (falls neue)
- [ ] API-Endpoints (falls neue)

**Beispiel:**
```markdown
#### `proof/halo2_circuit.rs` – Halo2 ZK Circuit
- **Funktion:** Implementiert Halo2-basierte ZK-Proofs für Policy-Constraints
- **Trait:** `ProofSystem`
- **Methoden:**
  - `prove()` – Erstellt Halo2-Proof aus Policy-Statement
  - `verify()` – Verifiziert Halo2-Proof
```

### 4. Git Commit mit vollständiger Message

```bash
git add .
git commit -m "feat(proof): Implement Halo2 circuit for UBO constraint

Sub-Tasks completed:
- Halo2-Crate in Cargo.toml integriert
- Basic Circuit Struct definiert (proof/halo2_circuit.rs)
- Constraint-Logik für 'require_at_least_one_ubo' implementiert
- Proof Generation Funktion (prove()) geschrieben
- Integration in proof_engine.rs (ProofSystem trait)

Tests:
- test_halo2_circuit_basic: ✅
- test_halo2_constraint_ubo_count: ✅
- test_halo2_proof_determinism: ✅
- All tests: 149/149 ✅

Performance:
- Proof Generation: 8.5s (Target: <10s) ✅
- Proof Size: 512 KB (Target: <1MB) ✅

CAP-Compliance:
- Deterministisch: ✅
- Reproduzierbar: ✅ (10/10 identical)
- Auditierbar: ✅ (audit event logged)
- Privacy-preserving: ✅
- Krypto-korrekt: ✅

Refs: ROADMAP_MVP_PRODUCTION.md Week 1, Day 1-2

🤖 Generated with Claude Code
Co-Authored-By: Claude <noreply@anthropic.com>"
```

**Ausgabe erstellen:**
```markdown
### Sprint Abgeschlossen! 🎉

**Erreichte Ziele:**
- [Hauptziel: z.B. Halo2 Circuit für UBO Constraint]

**Aktualisierte Dokumente:**
- ✅ ROADMAP_MVP_PRODUCTION.md (Week 1, Tasks 1-2 ✅)
- ✅ docs/Projektübersicht/07-status-und-roadmap.md (Halo2 Status aktualisiert)
- ✅ agent/CLAUDE.md (Modul `proof/halo2_circuit.rs` dokumentiert)

**Git Commit:**
- Commit ID: [abc123]
- Message: "feat(proof): Implement Halo2 circuit..."

**Nächster empfohlener Schritt:**
- [z.B. "Halo2 Performance optimieren (<10s → <5s)"]
- [Oder: "Tag 3-4: Proof Generation implementieren"]
```

**✅ SPRINT FERTIG!**

Frage den User:
> "Sprint abgeschlossen! Möchtest du:
> 1. **Neuen Sprint starten?** (Führe `/sprint` erneut aus)
> 2. **Bestimmtes Feature entwickeln?** (Sag mir welches)
> 3. **Pause machen und Status reviewen?**"

---

## 📋 Sprint-Checkliste (Quick Reference)

Für schnelles Durchlaufen eines Sprints:

- [ ] **Schritt 1:** Projektstand (Tests, Roadmap, Module)
- [ ] **Schritt 2:** Ziele (SMART + CAP-konform)
- [ ] **Schritt 3:** Tests (TDD: Unit, Integration, Property, Security)
- [ ] **Schritt 4:** Implementieren (Functional Core, Hash Early)
- [ ] **Schritt 5:** Testen (cargo test, clippy, CAP-Policies)
- [ ] **Schritt 6:** Dokumentieren (Roadmap, Docs, Git Commit)

---

## ⚠️ Wichtige Erinnerungen

**CAP Engineering-Prinzipien:**
- Deterministisch über alles
- Functional Core, Imperative Shell
- Hash-first Mindset
- Auditierbarkeit als Design-Prinzip
- Security zuerst – nicht zuletzt

**Verbotene Praktiken:**
- ❌ Direkt implementieren ohne Tests
- ❌ Unklare Spezifikationen akzeptieren
- ❌ Unsichere Designentscheidungen treffen
- ❌ CAP-Policies ignorieren
- ❌ Schritte überspringen

**Definition of Done:**
- Code kompiliert ✅
- Tests grün ✅
- Performance OK ✅
- Security OK ✅
- Docs aktualisiert ✅
- Roadmap aktualisiert ✅
- CAP-Policies erfüllt ✅

---

**Ready? Los geht's mit Schritt 1! 🚀**
