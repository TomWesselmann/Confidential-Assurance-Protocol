# 🧩 LkSG Proof Agent (Local Core MVP)

**Version:** 0.1.0  
**Status:** Tag 1 – Core MVP  
**Ziel:** Lokales CLI-Tool zur Erzeugung kryptografisch prüfbarer **Commitments** (Merkle-Roots) für Lieferketten- und Sanktionsprüfungen.  
**Teil des Projekts:** *Confidential Assurance Protocol (CAP)*

---

## 🚀 Zweck & Kontext

Der **LkSG Proof Agent** bildet den technischen Kern des *Confidential Assurance Protocol (CAP)*.  
Er verarbeitet **lokale Unternehmensdaten** (CSV/JSON) vollständig offline und erzeugt daraus kryptografische Fingerabdrücke (*Merkle-Roots*).  
Diese dienen als Grundlage für spätere Zero-Knowledge-Proofs, Signaturen und prüfbare Manifeste.

### Warum?
- ✅ Compliance-Nachweise ohne Datenoffenlegung  
- ✅ Vollständige Datenhoheit  
- ✅ Deterministisch & reproduzierbar  
- ✅ Juristisch auditierbare Nachweisstruktur  

---

## ⚙️ Funktionsumfang (Tag 1)

| Komponente | Beschreibung |
|-------------|--------------|
| 🧮 **Commitment-Engine** | Berechnet BLAKE3-Merkle-Roots aus CSV/JSON |
| 🧾 **Audit-Log-System** | JSONL-Datei mit SHA3-256-Hash-Chain |
| 🖥️ **CLI-Tool** | `cap-agent` mit Subcommands: `prepare`, `inspect`, `version` |
| 📦 **Output** | `commitments.json` mit supplier_root, ubo_root, company_root |
| 🛡️ **Offline-Modus** | Kein Netzwerk, keine externen Abhängigkeiten |

---

## 🧱 Projektstruktur

```
/agent/
  Cargo.toml
  src/
    main.rs
    audit.rs
    commitment.rs
    io.rs
/examples/
  suppliers.csv
  ubos.csv
/build/              # Output-Verzeichnis (Logs & Commitments)
```

---

## 🖥️ CLI-Befehle

### 🔹 prepare
Liest CSV/JSON, berechnet Merkle-Roots und schreibt `commitments.json` + `agent.audit.jsonl`.

```bash
cargo run -- prepare --suppliers examples/suppliers.csv --ubos examples/ubos.csv
```

### 🔹 inspect
Zeigt das JSON aus `commitments.json` formatiert an.

```bash
cargo run -- inspect build/commitments.json
```

### 🔹 version
Zeigt die aktuelle Tool-Version.

```bash
cargo run -- version
```

---

## 🧩 Beispiel-Daten

**examples/suppliers.csv**
```
name,jurisdiction,tier
Acme GmbH,DE,1
Globex AG,PL,2
```

**examples/ubos.csv**
```
name,birthdate,citizenship
Alice Example,1980-01-01,DE
Bob Muster,1975-02-02,AT
```

---

## 📄 Beispiel-Ausgaben

**commitments.json**
```json
{
  "supplier_root": "0xabc123...",
  "ubo_root": "0xdef456...",
  "company_commitment_root": "0x987abc..."
}
```

**agent.audit.jsonl**
```json
{
  "seq": 3,
  "ts": "2025-10-25T09:00:00Z",
  "event": "merkle_root_computed",
  "details": {"target": "suppliers","root": "0xabc123..."},
  "prev_digest": "0x1234...",
  "digest": "0x5678..."
}
```

---

## ✅ Akzeptanzkriterien

- `prepare` erzeugt 3 Roots (supplier, ubo, company)
- `inspect` gibt valides JSON aus
- Audit-Log enthält verkettete Hash-Chain
- Gleicher Input → gleiche Roots (deterministisch)
- Keine Warnings: `cargo clippy -- -D warnings`
- Unit-Tests bestehen (Merkle, Audit-Digest)

---

## 🧪 Test-Kommandos

```bash
cargo test
cargo clippy -- -D warnings
```

---

## 🧰 Technische Vorgaben

| Bereich | Entscheidung |
|----------|---------------|
| Sprache | Rust (Edition 2021) |
| CLI | clap v4 (derive) |
| Hashing | blake3 |
| Audit-Hash | sha3-256 |
| JSON | serde + serde_json |
| CSV | csv crate |
| Zeitformat | RFC3339 (UTC) |
| Plattform | Linux / macOS / Windows |
| Netzwerk | **verboten** |

---

## 🔐 Architekturprinzipien

1. **Lokalität:** Alle Daten bleiben im Unternehmensnetzwerk.  
2. **Reproduzierbarkeit:** Jeder Schritt ist deterministisch nachvollziehbar.  
3. **Integrität:** Audit-Log sichert jedes Event kryptografisch ab.  
4. **Erweiterbarkeit:** Spätere Module (Policy, Proof, Signatur) bauen direkt auf dem Commitment-Kern auf.

---

## 🧭 Nächste Schritte (Tag 2)

- Policy-Loader & Validator  
- Mock-Proof-Engine  
- Manifest-Builder  
- Signierung (Ed25519)

---

## 🤖 Für KI-Code-Agenten (Claude / GPT)

1. Lies die README vollständig.  
2. Erstelle ein Rust-Projekt `cap-agent` exakt nach dieser Struktur.  
3. Implementiere **nur** die Funktionen aus „Tag 1“.  
4. Erzeuge Unit-Tests gemäß Akzeptanzkriterien.  
5. Ergebnis muss direkt baubar sein mit:  
   ```bash
   cargo build && cargo test
   ```

---

## 📄 Lizenz

© 2025 Confidential Assurance Protocol – Core Engineering  
Alle Rechte vorbehalten.

## Erkläre bitte immer kurz im Code welche funktion es hat