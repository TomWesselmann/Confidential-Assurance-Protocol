# CAP Proof System – Punkt 5: Key-Management (KID/Rotation)

## Ziel
Das Key-Management-Modul stellt sicher, dass kryptografische Signaturen in CAP **nachvollziehbar, rotierbar und langfristig gültig** bleiben – unabhängig von Hardware oder Schlüsselalter.  
Es ist Grundlage für den späteren **Hardware-Sicherheits-Pfad (HSM/TPM/Smartcard)** und für die **Nachprüfbarkeit über viele Jahre**.

---

## Deliverables

### 1) Key Identifier (KID) System
- Jeder Signaturschlüssel (privat/public) erhält eine **eindeutige Kennung (KID)**.  
- KID wird aus dem Public Key abgeleitet:
  ```bash
  kid = blake3(base64(public_key))[0:16]
  ```
- Jede Signatur enthält ihren `kid` im Header oder Begleitobjekt.
- Registry-Einträge speichern `public_key`, `kid`, und `signature`.

**Beispiel (JSON Entry-Signatur):**
```json
{
  "signature_scheme": "ed25519",
  "kid": "b3f42c9d7e6a45a1",
  "public_key": "MCowBQYDK2VwAyEA…",
  "signature": "MEUCIQDL1B…"
}
```

Damit können Signaturen auch dann verifiziert werden, wenn der Schlüssel später rotiert oder archiviert wurde.

---

### 2) Key Store Struktur

#### `keys/`
```
keys/
  ├── company.ed25519
  ├── company.pub
  ├── company.v1.json
  ├── archive/
  │   ├── company.v0.json
  │   ├── auditor.v0.json
  └── trusted/
      ├── auditor.pub
      └── tsa.pub
```

- **`.vX.json`** Dateien enthalten Metadaten zu Schlüsselgeneration, Ablaufdatum, Rotation.  
- **`archive/`** speichert alte Schlüssel (readonly, signiert).  
- **`trusted/`** enthält öffentliche Schlüssel Dritter (Auditoren, TSA).

**Schlüssel-Metadaten (Schema):**
```json
{
  "schema": "cap-key.v1",
  "kid": "b3f42c9d7e6a45a1",
  "owner": "company",
  "created_at": "2025-10-30T12:00:00Z",
  "valid_from": "2025-10-30T12:00:00Z",
  "valid_to": "2027-10-30T12:00:00Z",
  "algorithm": "ed25519",
  "status": "active",
  "usage": ["signing", "registry"],
  "public_key": "MCowBQYDK2VwAyEA…",
  "fingerprint": "sha256:…",
  "comment": "Main corporate key for CAP registry signing"
}
```

---

### 3) Key Rotation

#### Ablauf
1. **Neuen Schlüssel generieren:**
   ```bash
   cap-agent keygen --owner company --algo ed25519 --out keys/company.v2.json
   ```
2. **Neuen Key aktivieren:**
   ```bash
   cap-agent keys rotate --current company.v1.json --new company.v2.json
   ```
   → aktualisiert `status` der alten Datei (`retired`), signiert neuen Schlüssel mit altem.
3. **Registry-Metadaten aktualisieren:**
   - neue `kid` im nächsten Signaturlauf verwenden
   - alte Einträge bleiben über archivierte Schlüssel verifizierbar
4. **Optional:** Signaturkette erzeugen
   ```bash
   cap-agent keys attest --signer company.v1.json --subject company.v2.json
   ```

#### Sicherheitsprinzipien
- Rotation darf **nie** alten Keys löschen → nur archivieren.
- Jeder Keywechsel ist selbst signiert (Chain of Trust).
- Alte Signaturen bleiben **gültig**, solange ihr `kid` im Archiv gefunden wird.

---

### 4) Integration mit Registry

| Feld | Bedeutung |
|------|------------|
| `kid` | Key Identifier (z. B. `b3f42c9d7e6a45a1`) |
| `signature_scheme` | z. B. `ed25519` |
| `public_key` | base64-codiert |
| `signature` | base64-codiert |
| `created_at` | RFC3339 |
| `verified_by` | Verifier Info (CLI/Agent Name) |

**CLI-Beispiel:**
```bash
cap-agent registry add --manifest manifest.json --proof proof.dat   --signing-key keys/company.v2.json   --backend sqlite --registry registry.sqlite
```

---

### 5) Hardware-Sicherheits-Pfad (Zukunft)
Dieses Key-System ist so gebaut, dass es später direkt erweitert werden kann auf:

- **HSM/TPM Backend** (`cap-agent keys hsm`)
- **Smartcard Signing** (PKCS#11 Interface)
- **Remote Signer Service** (via gRPC)
- Alle Varianten teilen die gleiche `kid`-Mechanik und Metadatenstruktur.

---

## CLI-Erweiterungen

```bash
# Schlüsselverwaltung
cap-agent keygen --owner company --algo ed25519 --out keys/company.v1.json
cap-agent keys list
cap-agent keys show --kid b3f42c9d7e6a45a1
cap-agent keys rotate --current company.v1.json --new company.v2.json
cap-agent keys attest --signer company.v1.json --subject company.v2.json
cap-agent keys archive --kid b3f42c9d7e6a45a1

# Registry mit neuem Key signieren
cap-agent registry add --signing-key keys/company.v2.json --manifest manifest.json --proof proof.dat
```

---

## Tests & Definition of Done

### Unit Tests
- Keygen: erzeugt valide ed25519 Keys, inkl. KID.
- Rotation: neue Datei korrekt erstellt, alte archiviert.
- Verification: Signaturen mit archiviertem Key gültig.

### Integration
- `registry add` mit aktivem Key funktioniert.
- `registry verify` erkennt archivierte Keys (Legacy Verification).

### Benchmarks
- Key-Parsing und Sign-Latenz (ms) dokumentieren.

### DoD
- Key-Metadaten-Schema dokumentiert (`cap-key.v1`)
- CLI-Kommandos vollständig implementiert
- Alte Signaturen weiterhin verifizierbar
- Docs: `SYSTEMARCHITEKTUR.md` & `CLAUDE.md` aktualisiert

---

## Warum dieser Punkt wichtig ist

| Zweck | Nutzen |
|-------|--------|
| **Langfristige Nachprüfbarkeit** | Signaturen bleiben gültig, auch wenn Schlüssel gewechselt wurden |
| **Sicherheit** | Schlüsselrotation minimiert Risiko bei Kompromittierung |
| **Hardware-Kompatibilität** | Grundgerüst für spätere HSM-/Smartcard-Integration |
| **Compliance** | Nachweisbare Signaturhistorie über mehrere Jahre |

**Kurz:** Dieser Punkt schafft die Grundlage dafür, dass CAP-Signaturen **dauerhaft beweisbar und vertrauenswürdig** bleiben – unabhängig von Gerät, Zeit oder Schlüsselalter.
