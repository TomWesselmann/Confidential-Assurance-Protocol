# 🔒 PRD – LkSG Proof Agent (Tag 4 – ZK Integration & Real Proofs)

## 1. Ziel & Umfang
Tag 4 erweitert den LkSG Proof Agent (Tag 3) um die erste **echte Zero-Knowledge-Proof (⁠ZKP⁠) Implementierung**.

**Ziel:**
- Mock-Proofs aus Tag 3 durch echte ZK-Beweise ersetzen.  
- Proof-Engine über standardisierte Schnittstelle (`ProofSystem`-Trait).  
- Proofs kryptografisch verifizierbar – ohne Offenlegung der Rohdaten.  
- Proof-Pakete vollständig kompatibel zum bestehenden Manifest / Verifier.

---

## 2. Lieferziel (Tag 4 = ZK Core MVP)
1. Implementierung einer echten ZK-Library (z. B. Halo2, Spartan, Risc0 oder Nova).  
2. Proof-Engine (`proof_engine.rs`) mit Trait-basierter Struktur:
   ```rust
   pub trait ProofSystem {
       fn prove(statement: &Statement, witness: &Witness) -> Result<Proof>;
       fn verify(statement: &Statement, proof: &Proof) -> Result<bool>;
   }
   ```
3. CLI-Erweiterungen:
   - `proof zk-build` → erzeugt echten ZK-Beweis  
   - `proof zk-verify` → prüft Beweis  
   - optional: `proof bench` → misst Proof-Zeit & Verifikationszeit
4. Integration in bestehende Proof-Pipeline (`manifest.json`, `proof_package/`).
5. Dokumentiertes ZK-Schema (`docs/zk-schema.v1.md`).

---

## 3. Funktionale Anforderungen

### 3.1 Proof Engine (ZKP)
| Funktion | Beschreibung |
|-----------|---------------|
| `zk-build` | Liest Manifest + Policy + lokalen Witness (aggregierte Unternehmensdaten), erzeugt Zero-Knowledge-Proof |
| `zk-verify` | Prüft Proof gegen Statement (Policy + Commitments) |
| `zk-bench` | Zeit- & Ressourcenmessung (Proving / Verifying) |

**Zentrale Struktur:**
```rust
struct Statement {
    policy_hash: String,
    company_commitment_root: String,
}
struct Witness {
    suppliers: Vec<Hash>,   // gehashte Lieferantendaten
    ubos: Vec<Hash>,        // gehashte UBOs
}
struct Proof {
    system: String,         // "halo2" | "spartan" | "risc0"
    data: Vec<u8>,          // serialisierter Beweis
    public_inputs: Vec<String>,
}
```

---

### 3.2 Proof-Paket (v1)
```
build/proof_package/
├── manifest.json
├── proof.dat          # echter ZK-Beweis
├── public_inputs.json # optional
├── signature.json
└── timestamp.tsa
```
`proof.dat` enthält den echten, Base64-codierten Beweis (z. B. Halo2).  
`public_inputs.json` enthält öffentliche Eingaben (z. B. Merkle-Roots, Policy-Hash).

---

### 3.3 Verifier
- Erkennt automatisch den Proof-Typ (`mock` vs. `zk`) und ruft das passende Backend auf.  
- `verifier run` verifiziert Proof + Manifest + Signatur offline.  
- Prüft: Policy-Hash, Commitment-Root, ZK-Proof gültig.

---

## 4. Audit-Integration
Neue Events:
- `zk_proof_generated`
- `zk_proof_verified`
- `zk_bench_executed`

Digest-Logik bleibt unverändert (SHA3-256-Kette).

---

## 5. Technische Vorgaben

| Bereich | Entscheidung |
|----------|--------------|
| Sprache | Rust 2021 |
| ZK-Lib | Halo2 (default) – alternativ Spartan/Nova/Risc0 |
| Hashing | blake3 + sha3-256 |
| Signatur | ed25519-dalek |
| CLI | clap v4 (derive) |
| Serialisierung | bincode + serde_json |
| Zeitformat | RFC3339 (UTC) |
| Plattform | Offline (Linux / macOS / Windows) |
| Netzwerk | **verboten** |

---

## 6. ZK-Architektur

```mermaid
graph TD
  A[Company Data CSV/JSON]
  --> B[Commitment Engine]
  --> C[Manifest Builder]
  --> D[Proof Engine (ZK)]
  --> E[Proof Package]
  --> F[Verifier CLI]
```

---

## 7. Unit / Integration Tests

| Modul | Testfall | Erwartung |
|--------|-----------|-----------|
| proof_engine | `prove` erzeugt gültigen Proof → verify = true | OK |
| proof_engine | manipulierte Inputs → verify = false | FAIL |
| verifier | erkennt Proof-Typ korrekt | OK |
| verifier | prüft Policy / Commitment-Konsistenz | OK |
| bench | misst Laufzeit > 0 ms, ohne Fehler | OK |
| audit | Log-Events korrekt verkettet | OK |

---

## 8. Akzeptanzkriterien
- `proof zk-build` erzeugt echten ZK-Proof (`proof.dat`)  
- `proof zk-verify` verifiziert Proof offline  
- `verifier run` bestätigt Proof-Paket  
- Proof-Pakete deterministisch bei gleichen Inputs  
- Audit-Log enthält neue Events  
- Clippy clean, Tests grün  
- Dokumentation aktualisiert (`docs/zk-schema.v1.md`, `docs/system-architecture.md`)

---

## 9. Definition of Done (Tag 4)
✅ End-to-End-Proof-Pipeline:
```
prepare → policy validate → manifest build → proof zk-build → proof zk-verify → sign manifest → verifier run
```
✅ Proof = echter ZK-Beweis (keine Mockdaten)  
✅ Verifier CLI prüft Beweis offline  
✅ Architektur- & Schema-Dokumentation aktualisiert  
✅ CI (Pipeline: build / test / clippy) grün  
✅ Release v0.4.0 mit ZK-Proof-Support

---

## 10. Claude-Hinweise (Code-Erstellung)
1. Lies dieses PRD vollständig.  
2. Baue auf dem bestehenden Projekt (Tag 1 + 2 + 3) auf.  
3. Implementiere Proof-System-Trait (`proof_engine.rs`).  
4. Füge Backends ein (erst Mock, dann echte Halo2/Spartan).  
5. Passe `verifier.rs` an (Proof-Typ-Erkennung + ZK-Verify).  
6. Erstelle Tests und Benchmarks (Proof Zeit < 60 s).  
7. Keine Netzwerk-/Cloud-Zugriffe.  
8. Ergebnis muss baubar sein:
   ```bash
   cargo build && cargo test
   ```

---

## 11. Nächste Schritte (Tag 5 – Auditor & Integration)
- Open-Source Verifier (CLI + Web-WASM)  
- Registrierung von öffentlichen Proofs (Zeitstempel / Blockchain)  
- Policy-Verbund und aggregierte ZK-Proofs  
- Auditor-API für Verifikation im Browser

---

© 2025 Confidential Assurance Protocol – Core Engineering  
Alle Rechte vorbehalten.
