# 🧩 CAP – Key Management Integration (v0.10 → v0.10.9)

## 🔑 Ziel
Vollständige Integration des **Key Management Systems** zur Sicherstellung der **juristischen Nachweisbarkeit** und **technischen Vertrauenskette** (Rotation Chain-of-Trust, KID, Signaturen).

---

## 🎯 Zweck
Das Key Management Modul stellt die kryptografische und rechtliche Identität des Unternehmens sicher.  
Jede Signatur, jeder Proof und jedes Manifest ist auf eine **Key Identity (KID)** rückführbar.  
Rotationen und Archivierungen ermöglichen Langzeit-Gültigkeit und Hardware-Integration (TPM/HSM).

---

## ⚙️ Deliverables

### 1. CLI-Kommandos
Implementierung einer vollständigen **Key Management CLI** auf Basis von `keys.rs`:

| Kommando | Funktion | Status |
|-----------|-----------|--------|
| `cap-agent keygen` | Erstellt neuen Schlüssel + Metadaten (cap-key.v1) | 🔄 Neu |
| `cap-agent keys list` | Listet aktive, archivierte und vertrauenswürdige Keys | 🔄 Neu |
| `cap-agent keys show --kid <id>` | Zeigt vollständige Metadaten eines Keys | 🔄 Neu |
| `cap-agent keys rotate` | Erstellt Nachfolgeschlüssel + Chain-of-Trust | 🔄 Neu |
| `cap-agent keys attest` | Signiert Nachfolgeschlüssel (Rotation Attestation) | 🔄 Neu |
| `cap-agent keys archive` | Verschiebt abgelaufene Keys in `archive/` | 🔄 Neu |

---

### 2. Rotation Chain-of-Trust

Implementiere eine **nachvollziehbare Schlüsselrotation** mit Attestation-Mechanismus:

```rust
struct KeyAttestation {
    schema: "cap-key-attestation.v1",
    signer_kid: String,
    subject_kid: String,
    signature: String,  // Ed25519
    issued_at: String,  // RFC3339
}
```

- Jede neue Key-Generation erzeugt eine Attestation vom alten auf den neuen Key.
- Der `signer_kid` verweist auf den vorherigen Schlüssel.
- Die Chain wird kryptographisch geprüft (`verify_chain()`).
- Audit-Protokoll dokumentiert alle Rotationen.

---

### 3. Registry-Integration

- Jeder Registry-Eintrag (`registry_entries`) enthält:
  - `kid` (32 hex chars)
  - `signature_scheme` ("ed25519")
- Registry prüft automatisch, ob der Key **aktiv**, **attestiert** oder **archiviert** ist.
- Archivierte Keys können verifiziert, aber nicht mehr verwendet werden.

---

### 4. Audit-Tests für Schlüsselverläufe

- Unit-Tests: `test_key_rotation`, `test_key_attestation`, `test_key_archive`
- Integration-Test: `test_registry_key_chain`
- Property-Test: deterministische KID-Generierung (`blake3(pubkey)[0:16]`)

---

## 📁 Dateistruktur

```
keys/
├── company.v1.json         # Aktiver Key (metadaten)
├── company.v1.ed25519      # Private Key
├── company.v1.pub          # Public Key
├── archive/
│   ├── company.v0.json
│   └── auditor.v0.json
└── trusted/
    ├── auditor.pub
    └── tsa.pub
```

---

## 📜 Spezifikation

**Key Metadata (`cap-key.v1`)**

```json
{
  "schema": "cap-key.v1",
  "kid": "b3f42c9d7e6a45a1",
  "owner": "company",
  "algorithm": "ed25519",
  "status": "active",
  "created_at": "2025-11-04T10:00:00Z",
  "valid_from": "2025-11-04T10:00:00Z",
  "valid_to": "2027-11-04T10:00:00Z",
  "usage": ["signing", "registry"],
  "fingerprint": "0x123abc...",
  "public_key": "BASE64..."
}
```

---

## 🧠 Dependencies

- Modul: `src/keys.rs`
- Bibliotheken: `ed25519-dalek`, `serde_json`, `chrono`
- Registry-Anbindung: `registry.rs`
- Optional: `ring` oder `rust-crypto` für TPM-Anbindung (v1.0)

---

## 📅 Zeit & Aufwand

| Aufgabe | Aufwand | Priorität |
|----------|----------|-----------|
| CLI-Implementierung | 1 Woche | 🟥 Hoch |
| Rotation & Attestation | 1 Woche | 🟥 Hoch |
| Registry-Verknüpfung | 3 Tage | 🟧 Mittel |
| Tests & Doku | 3 Tage | 🟧 Mittel |

**Gesamt:** 2–3 Wochen Entwicklungszeit.

---

## ✅ Erfolgskriterien

- Alle CLI-Kommandos lauffähig und getestet (`cargo test` grün).  
- Jeder Key besitzt nachweisbare Herkunft (Chain-of-Trust).  
- Registry akzeptiert nur gültige, attestierte Keys.  
- Alle Signaturen rückführbar auf KID + Audit-Log.  
- Grundlage für Hardware-Signaturen (TPM/HSM) ist vorbereitet.
