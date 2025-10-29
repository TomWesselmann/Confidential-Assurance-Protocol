# 🧩 PRD – P0 Quick Wins (Timestamp, Statement-Roots, Mini-Policy)

**Projekt:** CAP / Proof Agent  
**Version:** v0.4 → v0.4.1  
**Ziel-Release:** sobald integriert & Tests grün  
**Owner:** Core Engineering (Proof/Policy/Audit)

---

## 0️⃣ Zusammenfassung

Dieses Update erweitert den Proof Agent um drei kleine, aber wirkungsvolle Features:

1. **Öffentlicher Zeitanker** – Manifest kann externen Timestamp oder Blockchain-Ref enthalten.  
2. **Externe Roots im Statement** – ZK-Statement kann `sanctions_root` und `jurisdiction_root` optional aufnehmen.  
3. **Mini-Policy-Erweiterung** – Zusätzliche einfache Constraints für LkSG++ ohne echte ZK-Kryptografie.

Alle Änderungen sind **abwärtskompatibel**, deterministisch und erfordern **keine neuen Libraries**.

---

## 1️⃣ Ziele & Nicht-Ziele

### 🎯 Ziele
- Manifest erhält optionales Feld `time_anchor`.
- Statement unterstützt optionale Roots (`sanctions_root`, `jurisdiction_root`).
- Policy kann einfache Constraints prüfen (`supplier_count_max`, `ubo_count_min`, `require_statement_roots`).

### 🚫 Nicht-Ziele
- Kein echter TSA-Call oder Blockchain-Submit.
- Keine echte ZK-Kryptografie.
- Kein API-Bruch am `ProofSystem`-Trait.

---

## 2️⃣ User Stories

| Rolle | Bedürfnis | Nutzen |
|-------|------------|--------|
| **Compliance Owner** | Zeitanker im Manifest setzen | Nachweis Zeitpunkt Audit-Kette |
| **Auditor** | Öffentliche Inputs (Roots) im Proof sehen | Transparente Beweisprüfung |
| **Policy-Autor** | einfache Regeln formulieren | Policies werden früh validiert |

---

## 3️⃣ CLI-Spezifikation

### 🧮 `audit tip`
**Zweck:** schreibt den aktuellen Hash der Audit-Chain-Spitze (`H_n`) nach `build/audit.head`.

```bash
cap-agent audit tip --out build/audit.head
```

**Output:**  
`build/audit.head` mit Hex-Hash (64 Zeichen).

---

### 🕒 `audit anchor`
**Zweck:** setzt einen Zeitanker im Manifest.  

```bash
cap-agent audit anchor   --kind <tsa|blockchain|file>   --ref <path|txid|uri>   --manifest-in build/manifest.json   --manifest-out build/manifest.anchored.json
```

**Precondition:** `build/audit.head` existiert.  
**Output:** Manifest enthält neues Feld `time_anchor`.

---

### 🧾 `proof zk-build`
**Erweiterte Flags:**
```bash
--sanctions-root <hex>
--jurisdiction-root <hex>
```

→ Felder werden ins Statement geschrieben (optional).

---

### 📜 `policy validate`
**Neue Constraints:**
```yaml
- name: supplier_count_max
  params: { max: 10 }

- name: ubo_count_min
  params: { min: 1 }

- name: require_statement_roots
  params: { keys: ["sanctions_root", "jurisdiction_root"] }
```

Wenn in der Policy geforderte Roots fehlen → Fehler  
`POLICY_E_MISSING_PUBLIC_INPUT`.

---

## 4️⃣ Datenmodelle / Schemas

### 🧱 Manifest (`manifest.json`)
```json
"time_anchor": {
  "kind": "tsa",
  "reference": "./tsa/2025-10-29.tsr",
  "audit_tip_hex": "83a8779d...",
  "created_at": "2025-10-29T09:15:22Z"
}
```

---

### ⚙️ Statement (ZK Public Inputs)
```json
{
  "policy_hash": "d490be94...",
  "company_commitment_root": "83a8779d...",
  "constraints": ["require_at_least_one_ubo"],
  "sanctions_root": "3a1f02bb...",
  "jurisdiction_root": "0c3f99aa..."
}
```

---

### 📚 Policy (YAML)
```yaml
policy:
  id: lksg.v1
  version: 1.1
  constraints:
    - name: require_at_least_one_ubo
    - name: supplier_count_max
      params: { max: 10 }
    - name: ubo_count_min
      params: { min: 1 }
    - name: require_statement_roots
      params: { keys: ["sanctions_root"] }
```

---

## 5️⃣ Code-Änderungen

| Datei | Änderung | Kurzbeschreibung |
|--------|-----------|------------------|
| `audit.rs` | `write_tip()` + `read_tip()` | Audit-Head exportieren & lesen |
| `manifest.rs` | `TimeAnchor` struct | neues Feld im Manifest |
| `zk_system.rs` | `Statement` erweitert | optionale Roots |
| `policy.rs` | neue Constraints | einfache Checks |
| `main.rs` | CLI-Erweiterungen | `audit tip`, `audit anchor`, neue Flags |

---

## 6️⃣ Tests

### ✅ Unit Tests
- `audit::tests::tip_write_and_read_ok`  
- `manifest::tests::time_anchor_roundtrip_ok`  
- `policy::tests::supplier_max_ok_fail`  
- `policy::tests::ubo_min_ok_fail`  
- `policy::tests::require_statement_roots_missing_fails`  
- `zk_system::tests::statement_optional_roots_serialization`

---

### 🧪 Integration Tests
1. **Timestamp Flow**  
   → `prepare` → `policy validate` → `manifest build`  
   → `audit tip` → `audit anchor` → `manifest show`  
   Manifest zeigt Zeitanker.

2. **Statement Roots**  
   → `proof zk-build --sanctions-root <hex>`  
   → `zk-verify` erfolgreich.

3. **Policy Minimal Plus**  
   → Ohne Roots → Fehler  
   → Mit Roots → Pipeline ok.

---

## 7️⃣ Definition of Done (DoD)

- Alle Tests grün  
- `cargo clippy -- -D warnings`  
- CLI `--help` aktualisiert  
- Doku ergänzt (`docs/manifest.md`, `docs/cli-audit.md`, `docs/zk-statement.md`, `docs/policy.lksg.minimal.md`)

---

## 8️⃣ Beispiel-Artefakte

### 🪶 `build/audit.head`
```
83a8779dc1f6a3b0e4d29c0c2d7d0f24e86a7a5a1f5f7b6c9d4b1e2c3f0a9b8c
```

### 🗂️ `manifest.json`
```json
"time_anchor": {
  "kind": "tsa",
  "reference": "./tsa/2025-10-29.tsr",
  "audit_tip_hex": "83a8779d...",
  "created_at": "2025-10-29T09:15:22Z"
}
```

### 🧩 `zk_proof.json`
```json
"public_inputs": {
  "policy_hash": "d490be94...",
  "company_commitment_root": "83a8779d...",
  "constraints": ["require_at_least_one_ubo"],
  "sanctions_root": "3a1f02bb..."
}
```

---

## 9️⃣ Risiken & Abfederung

| Risiko | Gegenmaßnahme |
|--------|----------------|
| Zeitanker unsauber gesetzt | CLI-Validierung & Schema-Check |
| Optionale Roots verwirren User | Policy `require_statement_roots` prüft Pflichtfelder |
| API-Bruch mit echtem ZK später | Felder sind forward-kompatibel (optional) |

---

## 🔟 Changelog (v0.4.1)

- `feat(audit): add tip export & anchor injection into manifest`  
- `feat(zk): optional public inputs sanctions_root & jurisdiction_root`  
- `feat(policy): add supplier_count_max, ubo_count_min, require_statement_roots`  
- `docs: update manifest/cli/statement/policy minimal`  
- `tests: unit & integration for new features`  
- `chore: help texts & examples`

---

© 2025 Confidential Assurance Protocol – Core Engineering
