# CAP Proof System – Roadmap zur v1.0

## 🗺️ Ziel
Schrittweise Fertigstellung des **Confidential Assurance Protocol (CAP)** bis zur produktionsreifen Version **v1.0** mit juristisch verwertbaren Proofs, skalierbarer Architektur und Hardware-Trust-Integration.

---

## Phase 1 – Core Completion (v0.10 → v0.10.9)

| Ziel / Deliverable | Haupt-Features | Priorität | Aufwand | Ergebnis |
|--------------------|----------------|------------|----------|-----------|
| **Key Management fertigstellen** | `keygen`, `rotate`, `attest`, `archive` CLI; Rotation Chain-of-Trust; KID-basierte Registry-Signaturen | 🔴 Hoch | 2–3 Wochen | Vollständige, juristisch belastbare Schlüsselverwaltung |
| **BLOB Store CLI** | `blob put/get/list/gc`; SQLite-Integration; Garbage Collection | 🔴 Hoch | 1–2 Wochen | Speicher- & Referenzmanagement produktionsreif |
| **Self-Verification vorbereiten** | Sandbox Execution (`registry exec --id`); WASM-Isolation; Ergebnis-Rückschreibung | 🟠 Mittel | 2–3 Wochen | Proofs prüfen sich selbst |
| **CLI & Doku vereinheitlichen** | Einheitliche CLI-Optionen, Man-Pages, Rust-Doku | 🟡 Mittel | 1 Woche | Developer Experience stabil |

**Ziel:** Core stable (v0.10.9) – Alle Features technisch vorhanden, CLI-fähig, lokal reproduzierbar.

---

## Phase 2 – Skalierung & Performance (v0.11)

| Ziel / Deliverable | Haupt-Features | Priorität | Aufwand | Ergebnis |
|--------------------|----------------|------------|----------|-----------|
| **Batch Operations & Benchmarks** | `registry batch-add` / `batch-verify`; Threadpool (8–16 Threads); Benchmark `scale10k` | 🔴 Hoch | 3–4 Wochen | 10.000 Proofs parallel verarbeitbar |
| **Performance Metrics Collector** | `bench report`; JSON + Markdown Reports; Determinismus-Test | 🟠 Mittel | 1 Woche | Messbare Skalierbarkeit |
| **Self-Verification Release** | `--selfverify` Flag; Auto-WASM-Verifikation; Audit-Eintrag automatisch | 🟠 Mittel | 2 Wochen | Registry-Einträge prüfen sich autonom |

**Ziel:** Proofs at Scale (v0.11) – Hochskalierbare, stabile Nachweisarchitektur mit messbarer Performance.

---

## Phase 3 – Production & Legal (v1.0)

| Ziel / Deliverable | Haupt-Features | Priorität | Aufwand | Ergebnis |
|--------------------|----------------|------------|----------|-----------|
| **Real RFC3161 Timestamp Authority** | Externe TSA-Anbindung; Hash Chain Integration; Reproduzierbare Timestamps | 🟡 Mittel | 2 Wochen | Juristisch verwertbare Zeitbindung |
| **Hardware Trust Path (TPM/HSM)** | Signaturunterstützung via TPM/YubiKey; PKCS#11 | 🟠 Mittel-Hoch | 4 Wochen | Hardwarebasierte Vertrauenskette |
| **Real ZK Backend Integration** | zkVM / Halo2; Policy→Constraint Compiler (PoC) | 🔴 Hoch | 6–8 Wochen | Echte Zero-Knowledge-Proofs im Einsatz |
| **Juristische Standardisierung** | Manifest-Format finalisieren; Patentstrategie; Auditoren-Allianz | 🟡 Mittel | 4 Wochen | Rechtlich verwertbarer Industriestandard |

**Ziel:** v1.0 Production – Juristisch, technisch und sicherheitsseitig vollwertiges Nachweissystem.

---

## Zeitliche Planung (realistisch)

| Zeitraum | Version | Fokus | Ergebnis |
|-----------|----------|--------|-----------|
| Monat 1–2 | v0.10.9 | Core Completion | Alle Module CLI-bereit |
| Monat 3–4 | v0.11 | Skalierung & Batch | 10k Proofs + Benchmarks |
| Monat 5–7 | v1.0 | Produktion & Recht | Hardware, TSA & ZK integriert |

---

## Bonus (Go-to-Market)

| Thema | Nutzen | Zeitpunkt |
|--------|--------|------------|
| **Auditor-Alliance (TÜV / DQS / Big4)** | Glaubwürdigkeit & Marktzugang | Ab v0.11 |
| **Patentierung Manifest-Format** | Lizenz-Lock-In / EU-Standardisierung | Vor v1.0 |
| **Pilotkunde (Mittelstand ESG/LkSG)** | Proof-of-Value & Referenz | Zwischen Phase 1–2 |
| **Investor Deck + Benchmarks** | Technische & wirtschaftliche Beweislast | Ab Phase 2 |

---

**Endziel:**  
> Ein rechtssicheres, autonomes Nachweissystem – offline, mathematisch beweisbar und marktfähig als europäischer Standard für vertrauliche Compliance-Prüfungen.
