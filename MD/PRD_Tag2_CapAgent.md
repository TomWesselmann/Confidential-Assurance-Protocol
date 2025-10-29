# PRD – LkSG Proof Agent (Tag 2 – Policy / Manifest / Signierung)

## 1. Ziel & Umfang
Der LkSG Proof Agent (Tag 2) erweitert den lokalen Commitment-Kern (Tag 1) um:
- **Policy-Loader & Validator**
- **Manifest-Builder**
- **Mock-Proof-Engine** (strukturierter Platzhalter)
- **Signierung & Verifikation (Ed25519)**

Ziel ist die Erzeugung eines **offline-prüfbaren Manifests** aus vorhandenen Commitments und Policies.  
Es werden **keine echten Zero-Knowledge-Beweise** implementiert – nur Struktur, Hashes und Signaturen.

---

## 2. Lieferziel (Tag 2 = Policy Layer MVP)
1. CLI-Binary `cap-agent` mit neuen Subcommands:  
   - `policy validate`  
   - `manifest build`  
   - `proof mock` / `proof verify`  
   - `sign manifest` / `verify manifest`
2. Eingaben:  
   - `build/commitments.json` (Tag 1-Output)  
   - `examples/policy.lksg.v1.yml`
3. Ausgaben:  
   - `build/manifest.json`  
   - `build/proof.mock.json`  
   - `build/manifest.signed.json`
4. Lokale Keyfiles (`keys/company.ed25519`, `keys/company.pub`)

---

## 3. Funktionale Anforderungen

### 3.1 Neue CLI-Befehle

| Command | Beschreibung | Input | Output |
|----------|---------------|--------|--------|
| `policy validate` | Liest Policy-Datei (YAML/JSON), prüft Schema & Regeln | `--file` | Konsole OK/Fehler |
| `manifest build` | Baut Manifest aus Commitments + Policy + Audit-Tail | `--policy`, optional `--out` | `manifest.json` |
| `proof mock` | Erzeugt strukturierten Mock-Proof (kein ZKP) | `--policy`, `--manifest` | `proof.mock.json` |
| `proof verify` | Prüft Mock-Proof auf Konsistenz | `--proof` | Konsole OK/Fehler |
| `sign manifest` | Signiert Manifest mit Ed25519-Key | `--key`, `--in`, `--out` | `manifest.signed.json` |
| `verify manifest` | Verifiziert Signatur | `--pub`, `--in` | Konsole OK/Fehler |

---

### 3.2 Policy-Loader & Validator
- Unterstützte Formate: YAML (`.yml`) oder JSON  
- Beispiel:
```yaml
version: "lksg.v1"
name: "LkSG-Minimal"
created_at: "2025-10-25T09:00:00Z"
constraints:
  require_at_least_one_ubo: true
  supplier_count_max: 100000
notes: "Demo-Policy ohne ZKP"
```
- Funktionen:
  - Schema-Validierung (Pflichtfelder, Datentypen)  
  - Einfache semantische Checks (Mock): z. B. mind. 1 UBO, Lieferanten ≤ 100 000 
  - Policy-Hash = SHA3-256 über kanonische JSON-Repräsentation

---

### 3.3 Manifest-Builder
- Input: `commitments.json`, `policy.yml`, `agent.audit.jsonl`
- Output: `manifest.json`
- Inhalt:
```json
{
  "version": "manifest.v0",
  "created_at": "2025-10-25T10:15:00Z",
  "supplier_root": "0x...",
  "ubo_root": "0x...",
  "company_commitment_root": "0x...",
  "policy": {
    "name": "LkSG-Minimal",
    "version": "lksg.v1",
    "hash": "0x<sha3>"
  },
  "audit": {
    "tail_digest": "0x...",
    "events_count": 12
  },
  "proof": {
    "type": "none",
    "status": "none"
  },
  "signatures": []
}
```
- Audit-Tail: letzter Digest + Anzahl Events aus Audit-Log  
- Proof: initial `none`

---

### 3.4 Mock-Proof-Engine
- Erzeugt ein JSON-Objekt:
```json
{
  "type": "mock",
  "policy_hash": "0x...",
  "company_commitment_root": "0x...",
  "status": "ok",
  "details": {"checks":[{"name":"require_at_least_one_ubo","ok":true}]}
}
```
- `proof verify` prüft Hash-Konsistenz + Status.

---

### 3.5 Signierung (Ed25519)
- Funktionen:
  - `keygen()` → `company.ed25519`, `company.pub`
  - `sign_manifest(priv, manifest) -> sig_hex`
  - `verify_manifest(pub, manifest, sig)`
- Output:
```json
{
  "manifest": { ... },
  "signature": {
    "alg": "Ed25519",
    "signer": "Company",
    "pubkey_hex": "0x...",
    "sig_hex": "0x..."
  }
}
```

---

## 4. Audit-Log-Integration
Neue Events (append in `agent.audit.jsonl`):
- `policy_loaded`  
- `policy_validated`  
- `manifest_built`  
- `mock_proof_generated`  
- `mock_proof_verified`  
- `manifest_signed`

Digest-Verkettung bleibt identisch (SHA3-256 über Felder).

---

## 5. Technische Vorgaben

| Bereich | Entscheidung |
|----------|---------------|
| Sprache | Rust (Edition 2021) |
| CLI | clap v4 (derive) |
| Hashing | sha3-256 + blake3 (Tag 1) |
| Signatur | ed25519-dalek |
| JSON/YAML | serde + serde_yaml |
| Zeitformat | RFC3339 (UTC) |
| Plattform | Offline, Linux/macOS/Win |
| Netzwerk | **verboten** |

---

## 6. Akzeptanzkriterien

- CLI läuft offline (kein Netzwerk).  
- `policy validate` meldet OK bei gültiger Policy.  
- `manifest build` erzeugt gültiges `manifest.json`.  
- `proof mock` + `proof verify` funktionieren deterministisch.  
- `sign manifest` + `verify manifest` funktionieren korrekt.  
- Audit-Log enthält alle neuen Events mit verketteten Hashes.  
- Keine Compiler-Warnings (`cargo clippy -- -D warnings`).  
- Unit-Tests für Policy, Manifest, Sign, Proof.

---

## 7. Beispieldaten

**examples/policy.lksg.v1.yml**
```yaml
version: "lksg.v1"
name: "LkSG-Demo"
created_at: "2025-10-25T09:00:00Z"
constraints:
  require_at_least_one_ubo: true
  supplier_count_max: 10
notes: "Testpolicy für Tag 2"
```

---

## 8. Beispiel-Outputs

**manifest.json**
```json
{
  "version":"manifest.v0",
  "supplier_root":"0xabc123...",
  "ubo_root":"0xdef456...",
  "company_commitment_root":"0x987abc...",
  "policy":{"name":"LkSG-Demo","version":"lksg.v1","hash":"0x..."},
  "audit":{"tail_digest":"0x1234...","events_count":5},
  "proof":{"type":"mock","status":"ok"},
  "signatures":[]
}
```

---

## 9. Definition of Done (Tag 2)

- `cargo run -- policy validate` liefert OK.  
- `cargo run -- manifest build` baut Manifest.  
- `cargo run -- proof mock` und `verify` laufen grün.  
- `cargo run -- sign manifest` + `verify manifest` OK.  
- Audit-Log enthält Events.  
- Tests bestehen, keine Warnings.  
- Deterministische Hashes und Signaturen.  
- Dokumentation aktualisiert (`docs/system-architecture.md`, `docs/manifest.schema.v0.md`).

---

## 10. Claude-Hinweise (Code-Erstellung)

1. Lies dieses PRD vollständig.  
2. Erstelle ein Rust-Projekt `cap-agent` nach bestehender Struktur (Tag 1 Basis beibehalten).  
3. Füge neue Module hinzu: `policy.rs`, `manifest.rs`, `proof_mock.rs`, `sign.rs`.  
4. Erweitere `main.rs` um die neuen CLI-Commands.  
5. Implementiere nur lokale Funktionen – keine Netzwerkzugriffe.  
6. Nutze bestehende Hilfsfunktionen (audit, io, commitment).  
7. Erstelle Tests nach den Akzeptanzkriterien.  
8. Ergebnis muss baubar sein:
   ```bash
   cargo build && cargo test
   ```
9. Ausgabe nur lokale Dateien, keine externen Verbindungen.
