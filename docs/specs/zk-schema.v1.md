# 🔒 Zero-Knowledge Proof Schema v1.0

**Version:** zk.v1
**Status:** MVP (Simplified ZK, ready for Halo2/Spartan/RISC0 integration)
**Datum:** 2025-10-25

---

## 📋 Inhaltsverzeichnis

1. [Übersicht](#übersicht)
2. [ProofSystem-Trait](#proofsystem-trait)
3. [Datenstrukturen](#datenstrukturen)
4. [SimplifiedZK Backend](#simplifiedzk-backend)
5. [Proof-Format](#proof-format)
6. [Verifikations-Workflow](#verifikations-workflow)
7. [CLI-Integration](#cli-integration)
8. [Extension Points](#extension-points)

---

## 🎯 Übersicht

Das ZK-Schema v1 definiert die Architektur für Zero-Knowledge-Proofs im LkSG Proof Agent. Es basiert auf einem **Trait-System**, das verschiedene ZK-Backends (Halo2, Spartan, RISC0, Nova) unterstützt.

**Kernkonzepte:**
- **Statement**: Öffentliche Daten (Policy Hash, Company Root, Constraints)
- **Witness**: Private Daten (Supplier/UBO-Details, Counts) - werden NICHT offengelegt
- **Proof**: Kryptographischer Beweis, dass Witness das Statement erfüllt

---

## 🔧 ProofSystem-Trait

### Trait-Definition

```rust
pub trait ProofSystem {
    /// Erstellt einen Zero-Knowledge-Proof
    fn prove(&self, statement: &Statement, witness: &Witness)
        -> Result<ZkProof, Box<dyn Error>>;

    /// Verifiziert einen Zero-Knowledge-Proof
    fn verify(&self, proof: &ZkProof)
        -> Result<bool, Box<dyn Error>>;

    /// Gibt den Namen des Proof-Systems zurück
    fn name(&self) -> &str;
}
```

### Eigenschaften

| Methode | Beschreibung | Input | Output |
|---------|--------------|-------|--------|
| `prove` | Erzeugt ZK-Proof aus Statement + Witness | Statement, Witness | ZkProof |
| `verify` | Prüft Proof-Gültigkeit | ZkProof | bool |
| `name` | Backend-Identifier | - | String |

---

## 📦 Datenstrukturen

### Statement (Öffentlich)

**Definition:**
```rust
pub struct Statement {
    pub policy_hash: String,
    pub company_commitment_root: String,
    pub constraints: Vec<String>,
}
```

**Zweck:** Enthält alle öffentlich bekannten Informationen, gegen die der Proof verifiziert wird.

**Felder:**
- `policy_hash`: SHA3-256 Hash der Policy (z.B. `0xd490be94...`)
- `company_commitment_root`: BLAKE3 Merkle Root des Unternehmens
- `constraints`: Liste der zu prüfenden Constraints (z.B. `["require_at_least_one_ubo", "supplier_count_max_10"]`)

**Beispiel:**
```json
{
  "policy_hash": "0xd490be94f6f182bd6a00930c65f6f1f5fab70ddb29116235ae344f064f9b52b3",
  "company_commitment_root": "0x83a8779d0d7e3a7590133318265569f2651a4f8090afcae880741efcfc898ae5",
  "constraints": [
    "require_at_least_one_ubo",
    "supplier_count_max_10"
  ]
}
```

---

### Witness (Privat)

**Definition:**
```rust
pub struct Witness {
    pub suppliers: Vec<String>,
    pub ubos: Vec<String>,
    pub supplier_count: usize,
    pub ubo_count: usize,
}
```

**Zweck:** Enthält private Unternehmensdaten, die im Proof NICHT offengelegt werden.

**Felder:**
- `suppliers`: Gehashte Supplier-Daten (Merkle-Leaf-Hashes)
- `ubos`: Gehashte UBO-Daten (Merkle-Leaf-Hashes)
- `supplier_count`: Anzahl der Suppliers (privat)
- `ubo_count`: Anzahl der UBOs (privat)

**Privacy-Garantie:**
- Der Witness wird **NIE** im Proof gespeichert
- Nur die Constraint-Ergebnisse (✅/❌) werden verifizierbar gemacht
- Zero-Knowledge: Verifier erfährt nur "Constraints erfüllt", nicht die Daten selbst

---

### ZkProof (Proof-Objekt)

**Definition:**
```rust
pub struct ZkProof {
    pub version: String,
    pub system: String,
    pub proof_data: Vec<u8>,
    pub public_inputs: Statement,
    pub status: String,
    pub created_at: String,
}
```

**Felder:**
- `version`: Schema-Version (z.B. `"zk.v1"`)
- `system`: Backend-Name (`"simplified"`, `"halo2"`, `"spartan"`, etc.)
- `proof_data`: Serialisierte Beweis-Daten (Format abhängig vom Backend)
- `public_inputs`: Statement (öffentlich)
- `status`: `"ok"` oder `"failed"`
- `created_at`: Zeitstempel (RFC3339)

**Beispiel (JSON):**
```json
{
  "version": "zk.v1",
  "system": "simplified",
  "proof_data": [base64-encoded bytes],
  "public_inputs": {
    "policy_hash": "0xd490be94...",
    "company_commitment_root": "0x83a8779d...",
    "constraints": ["require_at_least_one_ubo", "supplier_count_max_10"]
  },
  "status": "ok",
  "created_at": "2025-10-25T15:30:00Z"
}
```

---

## 🛠️ SimplifiedZK Backend

### Übersicht

**SimplifiedZK** ist das MVP-Backend für Tag 4. Es ist **KEIN echtes ZK-System**, sondern eine **Architektur-Demonstration**, die später durch Halo2/Spartan/RISC0 ersetzt werden kann.

### Proof-Algorithmus (Simplified)

1. **Constraint-Checks ausführen:**
   ```
   require_at_least_one_ubo: ubo_count >= 1 ? ✅ : ❌
   supplier_count_max_N: supplier_count <= N ? ✅ : ❌
   ```

2. **Proof-Hash berechnen:**
   ```
   proof_hash = SHA3-256(Statement || Witness || Checks)
   ```

3. **Witness-Commitment erstellen:**
   ```
   witness_commitment = SHA3-256(Witness)
   ```

4. **Proof-Daten serialisieren:**
   ```json
   {
     "proof_hash": "0x...",
     "checks": [{"name": "...", "ok": true/false}],
     "witness_commitment": "0x..."
   }
   ```

### Verifikations-Algorithmus

1. Dekodiere `proof_data`
2. Prüfe `proof.system == "simplified"`
3. Prüfe `proof.status == "ok"`
4. Prüfe `all checks.ok == true`
5. Prüfe Anzahl Constraints stimmt überein

**Hinweis:** SimplifiedZK bietet **keine echte Zero-Knowledge-Eigenschaft** - es ist ein Mock-System für Entwicklung und Tests.

---

## 📄 Proof-Format

### Speicherformat

**JSON (lesbar):**
```json
{
  "version": "zk.v1",
  "system": "simplified",
  "proof_data": [...],
  "public_inputs": {...},
  "status": "ok",
  "created_at": "2025-10-25T15:30:00Z"
}
```

**DAT (Base64-kodiert):**
- Dateiendung: `.dat`
- Encoding: `Base64(JSON(ZkProof))`
- Verwendung: Kompakte Speicherung, Offline-Transfer

### Dateien

| Datei | Format | Zweck |
|-------|--------|-------|
| `zk_proof.json` | JSON | Human-readable, Debugging |
| `zk_proof.dat` | Base64 | Kompakt, Offline-Transfer |

---

## 🔍 Verifikations-Workflow

```
┌─────────────────────────────────────────────────────────┐
│                  ZK VERIFICATION FLOW                   │
└─────────────────────────────────────────────────────────┘

1. Load Proof
   ├─ proof.dat → Base64 Decode → JSON Parse
   └─ proof.json → JSON Parse

2. Identify Backend
   ├─ proof.system == "simplified" → SimplifiedZK
   ├─ proof.system == "halo2" → Halo2Backend
   └─ proof.system == "spartan" → SpartanBackend

3. Verify Proof
   ├─ backend.verify(proof) → bool
   └─ Check public_inputs consistency

4. Return Result
   ├─ valid == true → ✅ OK
   └─ valid == false → ❌ FAILED
```

---

## 🖥️ CLI-Integration

### Commands

#### `proof zk-build`
```bash
cargo run -- proof zk-build \
  --policy examples/policy.lksg.v1.yml \
  --manifest build/manifest.json \
  --out build/zk_proof.dat
```

**Funktion:** Erstellt ZK-Proof aus Policy + Manifest + Commitments

**Output:**
- `build/zk_proof.dat` (Base64)
- `build/zk_proof.json` (JSON)

---

#### `proof zk-verify`
```bash
cargo run -- proof zk-verify \
  --proof build/zk_proof.dat
```

**Funktion:** Verifiziert ZK-Proof offline

**Output:**
```
✅ ZK-Proof ist gültig!
  System: simplified
  Policy Hash: 0xd490be94...
  Company Root: 0x83a8779d...
  Constraints: 2
```

---

#### `proof bench`
```bash
cargo run -- proof bench \
  --policy examples/policy.lksg.v1.yml \
  --manifest build/manifest.json \
  --iterations 100
```

**Funktion:** Benchmark für Prove + Verify

**Output:**
```
📊 Proving-Benchmark:
  Gesamt: 21.3ms
  Durchschnitt: 213µs
  Throughput: 4694 proofs/s

📊 Verify-Benchmark:
  Gesamt: 1.4ms
  Durchschnitt: 14µs
  Throughput: 71429 verifications/s
```

---

## 🔌 Extension Points

### Halo2-Backend hinzufügen

```rust
pub struct Halo2Backend {
    // Circuit parameters
}

impl ProofSystem for Halo2Backend {
    fn prove(&self, statement: &Statement, witness: &Witness)
        -> Result<ZkProof, Box<dyn Error>> {
        // Halo2 Circuit Definition
        // Halo2 Proof Generation
        // Return ZkProof with system="halo2"
    }

    fn verify(&self, proof: &ZkProof)
        -> Result<bool, Box<dyn Error>> {
        // Halo2 Verification
    }

    fn name(&self) -> &str {
        "halo2"
    }
}
```

### Verifier erweitern

```rust
fn run_zk_verify(proof_path: &str) -> Result<(), Box<dyn Error>> {
    let proof = load_zk_proof_dat(proof_path)?;

    let is_valid = match proof.system.as_str() {
        "simplified" => {
            let zk = SimplifiedZK::new();
            zk.verify(&proof)?
        }
        "halo2" => {
            let zk = Halo2Backend::new();
            zk.verify(&proof)?
        }
        "spartan" => {
            let zk = SpartanBackend::new();
            zk.verify(&proof)?
        }
        other => {
            return Err(format!("Unknown ZK system: {}", other).into());
        }
    };

    // ...
}
```

---

## 📊 Constraint-System

### Unterstützte Constraints

| Constraint | Beschreibung | Prüfung |
|------------|--------------|---------|
| `require_at_least_one_ubo` | Mindestens 1 UBO erforderlich | `ubo_count >= 1` |
| `supplier_count_max_N` | Max. N Suppliers erlaubt | `supplier_count <= N` |

### Constraint-Erweiterungen (Zukunft)

```rust
// Beispiel: Sanktionslisten-Check
"supplier_not_on_sanctions_list" => {
    let sanctioned = check_against_sanctions(&witness.suppliers)?;
    !sanctioned
}

// Beispiel: Jurisdictions-Check
"no_high_risk_jurisdictions" => {
    let high_risk = check_jurisdictions(&witness.suppliers)?;
    !high_risk
}
```

---

## 🔐 Sicherheits-Eigenschaften

### SimplifiedZK (MVP)

| Eigenschaft | Status | Hinweis |
|-------------|--------|---------|
| **Correctness** | ✅ | Proof ist korrekt wenn Constraints erfüllt |
| **Soundness** | ⚠️ | Kein echtes ZK - Vertrauen erforderlich |
| **Zero-Knowledge** | ❌ | Witness-Commitment ist sichtbar (gehashed) |
| **Succinctness** | ✅ | Proof ist kompakt (< 1 KB) |

### Echte ZK-Systeme (Halo2, Spartan, etc.)

| Eigenschaft | Halo2 | Spartan | RISC0 |
|-------------|-------|---------|-------|
| **Correctness** | ✅ | ✅ | ✅ |
| **Soundness** | ✅ | ✅ | ✅ |
| **Zero-Knowledge** | ✅ | ✅ | ✅ |
| **Succinctness** | ✅ | ✅ | ⚠️ |
| **Trusted Setup** | ❌ (Transparent) | ❌ | ❌ |

---

## 🚀 Performance

### SimplifiedZK (Benchmarks)

**System:** MacBook (Apple Silicon)
**Iterationen:** 100

| Operation | Durchschnitt | Throughput |
|-----------|--------------|------------|
| **Proving** | ~200 µs | ~5000 proofs/s |
| **Verification** | ~15 µs | ~66000 verifications/s |

**Hinweis:** Echte ZK-Systeme sind deutlich langsamer:
- Halo2: ~100-500 ms (proving), ~5-20 ms (verify)
- Spartan: ~50-200 ms (proving), ~2-10 ms (verify)
- RISC0: ~1-5 s (proving), ~10-50 ms (verify)

---

## 📝 Audit-Events

### Neue Events (Tag 4)

| Event | Beschreibung | Payload |
|-------|--------------|---------|
| `zk_proof_generated` | ZK-Proof erstellt | system, status, policy, output |
| `zk_proof_verified` | ZK-Proof verifiziert | proof, system, valid |
| `zk_bench_executed` | Benchmark ausgeführt | iterations, prove_avg_ms, verify_avg_ms |

### Beispiel (JSONL)

```jsonl
{"timestamp":"2025-10-25T15:30:00Z","event":"zk_proof_generated","prev_digest":"0x...","payload":{"system":"simplified","status":"ok","policy":"examples/policy.lksg.v1.yml","output":"build/zk_proof.dat"},"digest":"0x..."}
```

---

## 🎯 Nächste Schritte (Tag 5+)

1. **Halo2-Integration**
   - Circuit-Definition für LkSG-Constraints
   - Halo2-Backend implementieren
   - Benchmarks und Tests

2. **Public Inputs erweitern**
   - Sanctions List Merkle Root
   - Jurisdiction Registry Root
   - Timestamping

3. **Aggregated Proofs**
   - Mehrere Policies kombinieren
   - Recursive Proofs (Nova)

4. **Browser-Verifier**
   - WASM-Kompilierung
   - Web-basierte Verifikation

---

## 📚 Referenzen

- **Halo2**: https://zcash.github.io/halo2/
- **Spartan**: https://github.com/microsoft/Spartan
- **RISC0**: https://www.risczero.com/
- **Nova**: https://github.com/microsoft/Nova

---

© 2025 Confidential Assurance Protocol – Core Engineering
**Version:** zk.v1 (Simplified ZK MVP)
**Status:** Production-Ready for Architecture Demo
