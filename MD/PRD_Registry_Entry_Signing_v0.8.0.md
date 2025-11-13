# 🧾 PRD / Design – Registry Entry Signing (v0.8.0)

**Datum:** 2025-10-30  
**Status:** In Planung (P1-Scope)  
**Zielversion:** v0.8.0 (Security-Upgrade)

---

## 🎯 Ziel
Jeder Eintrag in der Proof-Registry wird künftig **kryptografisch signiert** (Ed25519), um Integrität und Authentizität nachweisbar zu machen. Damit kann `registry verify` sowohl die Signatur als auch die Proof-Hashes prüfen.

---

## 💡 Motivation
- **Integrität:** Schutz vor Manipulation der Registry-Datei (JSON oder SQLite)
- **Authentizität:** Belegt, dass die Einträge vom legitimen Agent-Schlüssel stammen
- **Audit-Nachvollziehbarkeit:** Jede Signatur ist eindeutig und reproduzierbar

Bisher: Registry-Einträge werden nur gehasht → keine Absenderauthentizität.  
Neu: `entry.signature` = Ed25519-Signatur über den Eintrags-Hash.

---

## 🧭 Scope (v0.8.0)
**In-Scope**
- Signaturerzeugung bei `registry add`
- Verifikationslogik bei `registry verify`
- Schlüsselverwaltung (Reuse des Agent-Schlüsselpaares aus `keys/agent_key.pem`)
- CLI-Flag `--signing-key <path>` (optional)

**Out-of-Scope**
- Multi-Signature-Support (kommt ggf. v0.9.0)
- PKI-Verknüpfungen / DID-Verankerung

---

## 🏗️ Architektur / Design

### 1) Schema-Erweiterung (JSON / SQLite)
```json
{
  "id": "proof_001",
  "manifest_hash": "0xd490be94...",
  "proof_hash": "0x83a8779d...",
  "timestamp_file": "build/timestamp.tsr",
  "registered_at": "2025-10-29T...",
  "signature": "base64(Ed25519(sig(entry_hash)))",
  "public_key": "base64(agent_pubkey)"
}
```

Für SQLite-Backend:
```sql
ALTER TABLE registry ADD COLUMN signature TEXT;
ALTER TABLE registry ADD COLUMN public_key TEXT;
```

### 2) Hash-Basis der Signatur
```rust
let entry_hash = blake3::hash(serde_json::to_vec(&entry_core)?);
let signature = keypair.sign(entry_hash.as_bytes());
```

`entry_core` enthält alle Felder außer `signature` & `public_key`.

### 3) Verifikation
```rust
let pubkey = PublicKey::from_bytes(entry.public_key)?;
let sig = Signature::from_bytes(entry.signature)?;
let entry_hash = blake3::hash(serde_json::to_vec(&entry_core)?);
assert!(pubkey.verify(entry_hash.as_bytes(), &sig).is_ok());
```

---

## ⚙️ CLI-Integration

### registry add
```
cap-agent registry add   --manifest build/manifest.json   --proof build/proof.dat   --timestamp build/timestamp.tsr   [--signing-key keys/agent_key.pem]
```

→ erzeugt Signatur über Eintrag und speichert `signature` + `public_key`.

### registry verify
```
cap-agent registry verify [--entry <id>]
```

→ prüft:
1. Datei-Hashes (wie bisher)
2. Ed25519-Signatur korrekt
3. Optional: Public-Key gehört zur lokalen Keychain

---

## ✅ Akzeptanzkriterien
1. `registry add` erzeugt `signature` + `public_key` Felder korrekt
2. `registry verify` erkennt manipulierte Einträge als ungültig
3. CLI ohne `--signing-key` nutzt Default `keys/agent_key.pem`
4. Alte Registry-Dateien ohne Signatur → Warnung, kein Abbruch
5. Kompatibel mit JSON- und SQLite-Backend

---

## 🧪 Testplan
- **Unit:**
  - `sign_and_verify_roundtrip_ok()`
  - `tampered_entry_fails_verification()`
  - `missing_signature_warns()`
- **CLI Smoke:**
  - `registry add` mit Key → Signatur vorhanden
  - `registry verify` → ✅
  - `registry verify` nach Manipulation → ❌

---

## 🔁 Migrationsschritte (Dev)
1. `registry.rs`: Struct `RegistryEntry` um Felder `signature`, `public_key` erweitern
2. `sign_entry()` Funktion hinzufügen
3. `verify_entry_signature()` in Verifikationspfad integrieren
4. CLI-Flag `--signing-key` registrieren
5. Tests + Migration-Script (ALTER TABLE)

---

## 🔐 Schlüsselverwaltung
- Default-Pfad: `keys/agent_key.pem` (Ed25519, wie in Proof-Engine)
- Struktur:
```bash
keys/
  ├── agent_key.pem   # private
  ├── agent_pub.pem   # public
```
- Nutzung via `ed25519_dalek`:
```rust
let keypair = Keypair::from_file(path)?;
```

---

## 📈 Erweiterungen (v0.9+)
- Multi-Sig / Chain-of-Trust (Signaturen mehrerer Instanzen)
- Zeitstempel-Verknüpfung (Signatur + TSA)
- Remote-Verification über CAP-Registry-API

---

## 📚 Doku-Updates
- **README.md:** Sektion „Registry-Sicherheit“ ergänzen
- **SYSTEMARCHITEKTUR.md:** Registry-Layer aktualisieren (Signaturpfad)
- **CLI.md:** neue Flag-Beispiele `--signing-key`

---

## 📝 Changelog (geplant)
- **Added:** Ed25519-Signatur + Verifikation pro Registry-Eintrag
- **Changed:** CLI `registry add` / `registry verify`
- **Docs:** Registry-Sicherheitsabschnitt
- **Tests:** Roundtrip + Manipulations-Tests
