# 🧩 PRD – REST-Verifier-API (`/verify`, `/policy`)

**Ziel:** Proof-Prüfung per API (deterministisch, offline-fähig, sicher) – implementierbar in Rust/Go/Node.  
**Scope:** Minimal Viable API für CAP-Pilot (BASF/SPA-Connector).  
**Security:** mTLS + OAuth2 (Client Credentials). **Keine** ausgehenden Internetverbindungen.

---

## 🔧 Architektur & Ordnerstruktur

```
verifier/
├─ src/
│  ├─ main.rs
│  ├─ api/
│  │  ├─ verify.rs
│  │  ├─ policy.rs
│  │  └─ health.rs
│  ├─ core/
│  │  ├─ manifest.rs
│  │  ├─ policy_ir.rs
│  │  ├─ compiler.rs
│  │  ├─ prover_mock.rs
│  │  └─ verify_engine.rs
│  ├─ security/
│  │  ├─ oauth.rs
│  │  └─ mtls.rs
│  └─ util/
│     ├─ hash.rs
│     └─ time.rs
├─ config/
│  ├─ app.yaml
│  └─ policies/
├─ openapi/
│  └─ verifier.v1.yaml
└─ tests/
   ├─ verify_ok.rs
   ├─ verify_fail.rs
   └─ policy_compile.rs
```

---

## 🔒 Sicherheit
- Transport: TLS ≥ 1.2, mTLS (Client-Zertifikat Pflicht im Pilot).  
- AuthZ: OAuth2 Client Credentials.  
- Rate Limits: global + pro Client.  
- Keine PII – Hashes only.  
- Offline-fähig (keine Outbounds).

---

## 🛣️ Endpunkte (v1)

### `POST /verify`
Verifiziert Proof-Kontext gegen Policy-IR, erzeugt Manifest & Signatur.

**Request:**
```json
{
  "policy_id": "lksg.v1",
  "context": {
    "supplier_hashes": ["0xabc..."],
    "sanctions_root": "0x54e..."
  },
  "backend": "mock",
  "options": {"adaptive": true}
}
```

**Response:**
```json
{
  "result": "OK",
  "manifest_hash": "0xa43b8c...",
  "trace": {"risk_tier": "HIGH","active_rules": ["no_sanctions"]},
  "signature": "base64(ed25519)",
  "timestamp": "RFC3161"
}
```

### `POST /policy/compile`
Kompiliert YAML-Policy → IR v1.

### `GET /policy/:id`
Liest kompiliertes IR + Metadaten.

### `GET /healthz`, `GET /readyz`
Health / Readiness Endpunkte.

---

## 🧪 Verifikationslogik
- `non_membership(lhs, rhs_root)` – Merkle proof (mocked).  
- `range_min(lhs, min)` – lhs ≥ min.  
- `eq(a,b)` – Hash equality.

Adaptive Mode → aktiviert Regeln basierend auf Kontext (Rule-Trace im Manifest).

---

## 🧰 Config (`config/app.yaml`)

```yaml
server:
  port: 8443
  tls_cert: /etc/certs/server.crt
  tls_key: /etc/certs/server.key
auth:
  issuer: https://auth.example
  audience: cap-verifier
limits:
  rps_per_client: 20
policy:
  registry_dir: ./config/policies
crypto:
  sign_key_path: ./keys/agent.ed25519
```

---

## 🧪 Tests
- Unit: Parser, legal_basis Pflicht, IR-Hash deterministisch.  
- Integration: verify OK/FAIL/WARN.  
- Security: Auth Fail = 401, Scope Fail = 403.  
- Load: 100 RPS, 95p < 500ms (Mock).

---

## ✅ Abnahmekriterien
1. OpenAPI v1 validiert.  
2. Deterministische Ergebnisse.  
3. Rule-Trace im Manifest.  
4. Kein PII in Logs.  
5. mTLS + OAuth2 aktiv.  
6. Alle Tests grün.  
7. Docker-Image signiert (Build-Hash in `/healthz`).

---

**Ergebnis:**  
Minimal-API für sichere, reproduzierbare Proof-Prüfung – bereit für SAP/BASF Integration.
