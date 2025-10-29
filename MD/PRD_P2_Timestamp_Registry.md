# 🧩 PRD – P2 Timestamping & Proof Registry

**Projekt:** CAP / Proof Agent  
**Version:** v0.5.0 → v0.6.0  
**Ziel-Release:** Sprint P2 (7–10 Tage)  
**Owner:** Core Engineering (Audit & Registry Layer)

---

## 0️⃣ Zusammenfassung

Dieses Release erweitert den Proof Agent um **nachvollziehbare Zeitverankerung** und eine **lokale Proof-Registry**.

Ziele:
1. **RFC3161-kompatibles Timestamping** für Audit-Chain-Heads  
2. **Lokale Proof-Registry**, die Manifeste und ZK-Proofs referenziert  
3. **Audit-Einträge** für Timestamp- und Registry-Operationen  

Damit entsteht eine **prüfbare Nachweis-Kette über Zeit und Versionen**, die ohne Blockchain auskommt, aber dafür vorbereitet ist.

---

## 1️⃣ Ziele & Nicht-Ziele

### 🎯 Ziele
- Export und Import von **Audit-Head-Timestamps** (`timestamp.tsr`)  
- Lokale Registry (`registry.json` oder SQLite) für Proof-Metadaten  
- CLI-Integration mit Validierungen  
- Audit-Trail für alle Vorgänge  

### 🚫 Nicht-Ziele
- Keine externe TSA oder Blockchain-Anbindung (Mock/optional)  
- Keine Web-API oder Multi-User-Registry  
- Kein Signaturwechsel (weiterhin Ed25519)

---

## 2️⃣ User Stories

| Rolle | Bedürfnis | Nutzen |
|-------|------------|--------|
| **Auditor** | möchte sehen, wann ein Proof erzeugt und verankert wurde | Zeitlicher Nachweis |
| **Compliance Officer** | möchte Proofs in einer lokalen Registry verwalten | einfache Nachverfolgung |
| **Developer** | möchte Timestamp-Dateien generieren und prüfen können | deterministische Nachweise |

---

## 3️⃣ CLI-Spezifikation

### 🕒 `audit timestamp`
Erstellt eine Timestamp-Datei für den aktuellen Audit-Head.

```bash
cap-agent audit timestamp   --head build/audit.head   --out build/timestamp.tsr   [--mock | --tsa-url <url>]
```

**Parameter:**
- `--mock`: erzeugt lokalen RFC3161-Mock-Timestamp (ohne TSA)
- `--tsa-url`: optionaler echter TSA-Endpunkt (z. B. DigiCert)
- `--head`: Datei mit Hash des Audit-Chain-Heads (`audit.head`)

**Output:**
- `build/timestamp.tsr` (RFC3161-kompatibles ASN.1- oder JSON-Format)
- Audit-Event: `timestamp_generated`

---

### 🔍 `audit verify-timestamp`
Überprüft einen Timestamp gegen Audit-Head.

```bash
cap-agent audit verify-timestamp   --head build/audit.head   --timestamp build/timestamp.tsr
```

**Ergebnis:**  
✅ `"Timestamp valid"`  
❌ `"Timestamp invalid or mismatched head"`

---

### 🗃️ `registry add`
Registriert einen Proof in der lokalen Registry.

```bash
cap-agent registry add   --manifest build/manifest.json   --proof build/zk_proof.dat   [--timestamp build/timestamp.tsr]
```

**Aktion:**
- Liest Hashes von Manifest, Proof und Timestamp
- Fügt Eintrag in lokale `registry.json` ein
- Audit-Event: `registry_entry_added`

---

### 📋 `registry list`
Listet alle gespeicherten Proofs mit Metadaten.

```bash
cap-agent registry list
```

**Beispielausgabe:**
```
──────────────────────────────────────────────
Proofs in local registry (./build/registry.json)
──────────────────────────────────────────────
#1  Manifest: d490be94…  Proof: 83a8779d…  Date: 2025-11-05T10:22Z
#2  Manifest: a239faae…  Proof: 6c39efaa…  Date: 2025-11-05T11:18Z
──────────────────────────────────────────────
```

---

### 🧾 `registry verify`
Verifiziert, ob Proof- und Manifest-Hashes mit Registry übereinstimmen.

```bash
cap-agent registry verify --manifest build/manifest.json --proof build/zk_proof.dat
```

Ergebnis:  
✅ `"Entry verified in registry"`  
❌ `"Hash mismatch or not registered"`

---

## 4️⃣ Datenmodelle

### 📁 Timestamp File (`timestamp.tsr`)
```json
{
  "version": "tsr.v1",
  "audit_tip_hex": "83a8779dc1f6a3b0...",
  "created_at": "2025-11-05T10:15:00Z",
  "tsa": "local-mock",
  "signature": "base64-encoded-mock-sig",
  "status": "ok"
}
```

---

### 📘 Registry (`registry.json`)
```json
{
  "registry_version": "1.0",
  "entries": [
    {
      "id": "proof_001",
      "manifest_hash": "d490be94...",
      "proof_hash": "83a8779d...",
      "timestamp_file": "build/timestamp.tsr",
      "registered_at": "2025-11-05T10:22:00Z"
    }
  ]
}
```

---

## 5️⃣ Audit-Trail

Neue Events:

| Event | Beschreibung |
|--------|---------------|
| `timestamp_generated` | Audit-Head wurde mit Zeitanker versehen |
| `timestamp_verified` | Timestamp erfolgreich geprüft |
| `registry_entry_added` | Proof wurde in Registry eingetragen |
| `registry_verified` | Registry-Check erfolgreich |

Alle Ereignisse laufen über SHA3-verkettete `audit.rs`-Chain.

---

## 6️⃣ Code-Änderungen

| Datei | Änderung | Beschreibung |
|--------|-----------|--------------|
| `audit.rs` | neue Funktionen `timestamp_mock()`, `verify_timestamp()` |
| `registry.rs` | **neu** | JSON- oder SQLite-Registry-Modul |
| `main.rs` | CLI-Subcommands `audit timestamp`, `audit verify-timestamp`, `registry *` |
| `manifest.rs` | optional: Feld `registry_ref` für Proof-ID |
| `audit/tests.rs` | neue Unit-Tests für Timestamp |

---

## 7️⃣ Tests

### ✅ Unit Tests
- `audit::tests::timestamp_mock_roundtrip`
- `audit::tests::verify_timestamp_ok_fail`
- `registry::tests::add_entry_ok`
- `registry::tests::list_entries`
- `registry::tests::verify_entry_ok`

### 🧪 Integration Tests
1. **Timestamp Flow**
   - `audit tip` → `audit timestamp`  
   - Timestamp-Datei existiert, gültig verifiziert

2. **Registry Flow**
   - `registry add --manifest --proof` → Eintrag erzeugt  
   - `registry list` zeigt neuen Eintrag  
   - `registry verify` → „verified“

3. **Audit Chain**
   - Events `timestamp_generated` und `registry_entry_added` korrekt protokolliert

---

## 8️⃣ Definition of Done (DoD)

- Timestamp-Dateien generierbar & verifizierbar  
- Registry persistiert Proof-Metadaten  
- Audit-Log vollständig aktualisiert  
- Alle Tests grün  
- Doku: `docs/audit-timestamp.md`, `docs/registry.md`, `docs/examples/timestamp_flow.md`

---

## 9️⃣ Risiken & Gegenmaßnahmen

| Risiko | Gegenmaßnahme |
|--------|----------------|
| Unterschiedliche Hashformate | Audit-Head-Validator erzwingt 64-char Hex |
| Timestamp-Spoofing (Mock) | deutlicher CLI-Hinweis „MOCK TIMESTAMP“ |
| Registry-Korruption | SHA3-Prüfsummen + Signatur optional |

---

## 🔟 Changelog (v0.6.0)

- `feat(audit): add timestamp export and verification (RFC3161 mock)`  
- `feat(registry): local proof registry with add/list/verify commands`  
- `feat(cli): new audit & registry subcommands`  
- `feat(audit): log timestamp_generated and registry_entry_added events`  
- `docs: update timestamping and registry examples`  
- `tests: new audit and registry test coverage`

---

© 2025 Confidential Assurance Protocol – Core Engineering
