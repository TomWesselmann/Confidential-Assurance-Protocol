# 🧩 PRD – P1 Sanctions & Jurisdictions Roots + Mock Checks

**Projekt:** CAP / Proof Agent  
**Version:** v0.4.1 → v0.5.0  
**Ziel-Release:** Sprint P1 (7 Tage)  
**Owner:** Core Engineering (Compliance Proof Layer)

---

## 0️⃣ Zusammenfassung

Dieses Update erweitert den Proof Agent um reale Inputdaten und erste logische Prüfungen:

1. **Sanctions Root Generator:** CSV → Merkle Root (`sanctions.root`)  
2. **Jurisdictions Root Generator:** CSV → Merkle Root (`jurisdictions.root`)  
3. **Mock Non-Membership Checks:** einfache Verifikation, dass kein UBO auf der Sanktionsliste steht  
4. **CLI-Integration:** neue `lists`-Kommandos und erweiterte Proof-Erstellung

Ziel: Die Proofs sollen erstmals mit **realistischen, nachvollziehbaren Datenquellen** arbeiten, auch wenn die Kryptografie (ZK) weiterhin Mock-basiert bleibt.

---

## 1️⃣ Ziele & Nicht-Ziele

### 🎯 Ziele
- Datenquellen (CSV) in Merkle-Form bringen  
- Roots als Public Inputs in ZK-Proof übernehmen  
- Optionales CSV-Checking zur Mock-Verifikation  
- Eindeutige CLI-Kommandos und Audit-Trail-Einträge

### 🚫 Nicht-Ziele
- Kein echter Zero-Knowledge Non-Membership-Proof  
- Keine Blockchain-Integration  
- Kein externer API-Download von Listen

---

## 2️⃣ User Stories

| Rolle | Bedürfnis | Nutzen |
|-------|------------|--------|
| **Compliance Officer** | möchte Sanktionslisten als Root einbinden | Proofs gegen echte Daten referenzierbar |
| **Auditor** | möchte sehen, dass kein UBO in einer Liste vorkommt | Vertrauen ohne Datenoffenlegung |
| **Developer** | möchte CSV-Dateien verarbeiten & prüfen | deterministische Roots, wiederholbar |

---

## 3️⃣ CLI-Spezifikation

### 🧮 `lists sanctions-root`
Erzeugt einen Merkle-Root aus einer CSV-Sanktionsliste.

```bash
cap-agent lists sanctions-root --csv lists/eu_sanctions.csv --out build/sanctions.root
```

**CSV-Format:**
```csv
name,birthdate,nationality
Ali Hassan,1984-01-14,IR
Maria Petrova,1973-05-22,RU
```

**Hash-Algorithmus:** BLAKE3(name|birthdate|nationality)  
**Merkle-Root:** BLAKE3(leaf hashes in-order)

**Output:** `build/sanctions.root` (Hex-String, 64 chars)

---

### 🌍 `lists jurisdictions-root`
Erzeugt einen Merkle-Root aus einer Länderliste.

```bash
cap-agent lists jurisdictions-root --csv lists/highrisk.csv --out build/jurisdictions.root
```

**CSV-Format:**
```csv
iso_code,risk_level
IR,HIGH
RU,HIGH
DE,LOW
```

**Hash:** BLAKE3(iso_code|risk_level)  
**Output:** `build/jurisdictions.root`

---

### 🧾 `proof zk-build` (erweitert)
**Neue Flags:**
```bash
--sanctions-root <hex|file>
--jurisdiction-root <hex|file>
--sanctions-csv <path>
```

Wenn `--sanctions-csv` angegeben ist, prüft SimplifiedZK:
- Lade CSV → hashe alle Einträge  
- Vergleiche gegen Witness-UBO-Hashes  
- Wenn Übereinstimmung → Proof `failed`

---

## 4️⃣ Datenmodelle

### 📁 Sanctions Root File
```txt
root: "b3a8f9c4e2...d9b77a"
count: 15893
source: "lists/eu_sanctions.csv"
generated_at: "2025-11-01T10:20:00Z"
algorithm: "BLAKE3"
```

### 📁 Jurisdictions Root File
```txt
root: "6e39ff18...dcd411"
count: 198
source: "lists/highrisk.csv"
generated_at: "2025-11-01T10:22:00Z"
algorithm: "BLAKE3"
```

### 🧩 Proof JSON (Erweiterung)
```json
"public_inputs": {
  "policy_hash": "d490be94...",
  "company_commitment_root": "83a8779d...",
  "sanctions_root": "b3a8f9c4e2...",
  "jurisdiction_root": "6e39ff18...",
  "constraints": ["require_at_least_one_ubo", "sanctions_non_membership"]
},
"checks": [
  {"name": "sanctions_non_membership", "ok": true}
]
```

---

## 5️⃣ Audit-Trail

Neue Audit-Events:

| Event | Beschreibung |
|--------|---------------|
| `sanctions_root_generated` | CSV eingelesen, Root berechnet |
| `jurisdictions_root_generated` | CSV eingelesen, Root berechnet |
| `sanctions_check_executed` | Non-Membership-Mock-Check durchgeführt |

Alle Einträge erscheinen in `agent.audit.jsonl` mit SHA3-verketteter Chain.

---

## 6️⃣ Code-Änderungen

| Datei | Änderung | Beschreibung |
|--------|-----------|--------------|
| `lists/mod.rs` | neu | Modul-Entry für Listen |
| `lists/sanctions.rs` | neu | CSV → BLAKE3 Root |
| `lists/jurisdictions.rs` | neu | CSV → BLAKE3 Root |
| `zk_system.rs` | erweitert | Sanctions-Check in SimplifiedZK |
| `main.rs` | CLI `lists sanctions-root` / `lists jurisdictions-root` / neue Flags |
| `audit.rs` | neue Events (`sanctions_root_generated`, etc.) |

---

## 7️⃣ Tests

### ✅ Unit Tests
- `lists::tests::sanctions_root_deterministic`
- `lists::tests::jurisdictions_root_deterministic`
- `zk_system::tests::sanctions_non_membership_ok`
- `zk_system::tests::sanctions_non_membership_fail`

### 🧪 Integration Tests
1. **Sanctions Root Flow**  
   - CSV → `lists sanctions-root` → Root-Datei korrekt, deterministisch  
   - Event `sanctions_root_generated` erscheint im Audit

2. **Jurisdictions Root Flow**  
   - Analog; prüfe Konsistenz

3. **Proof Build**  
   - `proof zk-build --sanctions-root build/sanctions.root --sanctions-csv lists/eu_sanctions.csv`  
   - Kein Match → Proof `ok`; mit künstlichem Match → `failed`

---

## 8️⃣ Definition of Done (DoD)

- CLI-Befehle `lists sanctions-root` und `lists jurisdictions-root` funktionieren deterministisch  
- Mock-Check in SimplifiedZK aktiv  
- Audit-Events enthalten `sanctions_root_generated`  
- Alle Tests grün  
- Doku aktualisiert (`docs/lists.md`, `docs/zk_system.md`, `docs/examples/sanctions_demo.md`)

---

## 9️⃣ Risiken & Gegenmaßnahmen

| Risiko | Gegenmaßnahme |
|--------|----------------|
| Unterschiedliche CSV-Formate | Standard-Schema + Fehlermeldungen |
| Langsame Verarbeitung | Hashing parallelisieren bei großen Listen |
| Verwechslungsgefahr bei Mock-ZK | CLI-Ausgabe mit ⚠️-Warnhinweis „Simplified Check only“ |

---

## 🔟 Changelog (v0.5.0)

- `feat(lists): add sanctions & jurisdictions Merkle root generators`  
- `feat(zk): add mock sanctions non-membership check`  
- `feat(cli): integrate lists subcommands`  
- `feat(audit): add sanctions_root_generated event`  
- `docs: update lists & proof examples`  
- `tests: new unit & integration coverage`

---

© 2025 Confidential Assurance Protocol – Core Engineering
