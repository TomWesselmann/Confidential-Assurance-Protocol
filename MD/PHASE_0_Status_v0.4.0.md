# 🧱 Phase 0 – Status sichern (Tag v0.4.0)

## 🎯 Ziel
Einen **sauberen, dokumentierten Projektstand** herstellen, der als stabile Basis für die Erweiterungen (ZK-Integration, Sanctions, Registry) dient.  
Diese Phase schließt die Proof-Agent-Grundarchitektur (Tag 1–4) formal ab.

---

## ✅ To-Do-Liste

### 1️⃣ Git & Versionierung
- [ ] Alle Änderungen committen (`git add . && git commit -m "Phase 0 baseline"`)
- [ ] Versionstag setzen:  
  ```bash
  git tag v0.4.0
  ```
- [ ] Sicherstellen, dass alle PRDs (`Tag 1–4`) im Repository liegen

---

### 2️⃣ Dokumentation
- [ ] `README.md` aktualisieren  
  - kurzer Abschnitt: *„Stand v0.4.0 – Proof Core abgeschlossen, ZK-Integration vorbereitet“*
  - Liste der CLI-Befehle (prepare, inspect, version)
- [ ] `docs/system-architecture.md` erweitern  
  - Proof Engine + Verifier aufnehmen  
  - Abschnitt **„Proof Chain“** mit mermaid-Diagramm hinzufügen:
    ```mermaid
    graph TD
      A[Commitment Engine] --> B[Manifest Builder]
      B --> C[Proof Engine]
      C --> D[Registry & Timestamp]
      D --> E[Verifier CLI]
    ```
- [ ] Architekturdiagramm in README oder `/docs/images/` speichern

---

### 3️⃣ Tests & Qualität
- [ ] Alle Tests laufen lassen:
  ```bash
  cargo test
  ```
  ✅ Alle 8 Unit-Tests grün
- [ ] Clippy prüfen:
  ```bash
  cargo clippy -- -D warnings
  ```
  ✅ Keine Warnungen
- [ ] `build/` prüfen: `commitments.json` + `agent.audit.jsonl` vorhanden

---

### 4️⃣ Artefakte & Ordnerstruktur
Überprüfen, dass folgende Struktur im Projekt enthalten ist:
```
/agent/
  ├── Cargo.toml
  ├── src/
  │   ├── main.rs
  │   ├── audit.rs
  │   ├── commitment.rs
  │   ├── io.rs
  │   ├── manifest.rs
  │   ├── sign.rs
  │   └── verifier.rs
  ├── examples/
  │   ├── suppliers.csv
  │   ├── ubos.csv
  │   └── policy.lksg.v1.yml
  └── build/
      ├── commitments.json
      ├── agent.audit.jsonl
      └── (zukünftig: manifest.json, zk_proof.dat, registry.json)
```

---

### 5️⃣ Abschluss der Phase 0
Wenn alle Punkte erledigt sind:
1. **Commit & Tag** bestätigt (`v0.4.0`)
2. **Doku** aktualisiert (`README`, `system-architecture.md`)
3. **Tests grün**, keine Clippy-Warnings
4. **Proof Chain-Diagramm** vorhanden
5. **PRDs (Tag 1–4)** vollständig im Repo

---

## 📈 Nächster Schritt: Phase 1 (v0.5.0)
→ Sanctions- & Jurisdictions-Modul  
→ Erweiterung um `lists/sanctions.rs`, `lists/jurisdictions.rs`  
→ Integration in ZK-Engine („non-membership constraint“)

---

© 2025 Confidential Assurance Protocol – Core Engineering  
**Version:** v0.4.0 **Status:** Baseline / Documentation Complete
