# 04 - API Reference

## 📖 Über dieses Kapitel

Nachdem Sie in den vorherigen Kapiteln gelernt haben, **was** das System macht ([Kapitel 01](./01-overview.md)), **wie** es aufgebaut ist ([Kapitel 02](./02-architecture.md)), und **welche Teile** es gibt ([Kapitel 03](./03-components.md)), zeigt dieses Kapitel **wie man es bedient**.

**Für wen ist dieses Kapitel?**
- **Management:** Die vereinfachten Erklärungen zu REST API und CLI
- **Compliance-Beauftragte:** Die CLI-Befehls-Beispiele für tägliche Arbeit
- **Entwickler:** Die detaillierte REST API Dokumentation
- **IT-Administratoren:** Die Konfiguration und Betriebsparameter

**Was Sie lernen werden:**
1. Wie man mit dem System über REST API kommuniziert (für andere Programme)
2. Wie man das System über CLI bedient (für Menschen)
3. Welche Befehle und Endpunkte es gibt
4. Welche Datenformate verwendet werden

**Analogie:** Dies ist die **Bedienungsanleitung** - wie bei einem Auto das Handbuch mit allen Knöpfen, Schaltern und Funktionen.

---

## 👔 Für Management: Zwei Arten der Bedienung

Das System hat **zwei Schnittstellen** (wie ein Auto mit Lenkrad UND Tempomat):

### 1. REST API (für andere Programme)
**Was ist das?** Eine "Steckdose für Software" - andere Programme (z.B. SAP) können das System automatisch nutzen

**Beispiel-Anwendung:**
- Ihr SAP-System sendet jeden Abend automatisch die Lieferanten-Liste
- Das System prüft automatisch und sendet das Ergebnis zurück
- Kein manuelles Eingreifen nötig

**Vorteile:**
✅ Automatisierung (kein manuelles Kopieren/Einfügen)
✅ Integration in bestehende IT-Systeme
✅ Schnell (Millisekunden statt Minuten)

### 2. CLI (für Menschen via Terminal)
**Was ist das?** Textbasierte Befehle - wie alte DOS-Befehle oder Unix-Kommandos

**Beispiel-Anwendung:**
- Compliance-Beauftragter öffnet Terminal
- Gibt Befehl ein: `cap prepare --suppliers suppliers.csv --ubos ubos.csv`
- System erstellt Nachweis

**Vorteile:**
✅ Schnelle Ad-hoc-Prüfungen
✅ Keine grafische Oberfläche nötig
✅ Skriptfähig (automatisierbar)

**Wann wird was genutzt?**
- **REST API:** Für Integration in Unternehmenssoftware (SAP, ERP-Systeme)
- **CLI:** Für manuelle Operationen, Tests, Administration

---

## REST API v0.11.0

**Für Management:** REST API ist wie ein Online-Formular, das andere Programme ausfüllen können - statt Menschen.

### Server Information

**Base URL:** `https://verifier.example.com` (production)
**Local Development:** `http://localhost:8080` (HTTP) or `https://localhost:8443` (HTTPS)
**Protocol:** HTTP/1.1, HTTP/2
**Authentication:** OAuth2 Bearer Token (JWT RS256)
**Content-Type:** `application/json`

---

## Authentication

### OAuth2 Bearer Token

Alle geschützten Endpunkte erfordern ein gültiges JWT-Token im Authorization-Header.

**Header Format:**
```
Authorization: Bearer <JWT_TOKEN>
```

**Token Format:** JWT RS256

**Required Claims:**
```json
{
  "sub": "client-id",
  "iss": "https://auth.example.com",
  "aud": "cap-verifier",
  "exp": 1700000000,
  "iat": 1699996400,
  "scope": "verify:read"
}
```

**Token Validation:**
1. Signature verification (RS256)
2. Expiration check (`exp` > now)
3. Issuer validation (`iss` matches config)
4. Audience validation (`aud` == "cap-verifier")
5. Scope validation ("verify:read" required)

**Error Responses:**
- `401 Unauthorized` - Missing or invalid token
- `403 Forbidden` - Valid token but insufficient scopes

---

## Public Endpoints

### GET /healthz

Health check endpoint (keine Authentifizierung erforderlich).

**Request:**
```bash
curl http://localhost:8080/healthz
```

**Response (200 OK):**
```json
{
  "status": "OK",
  "version": "0.1.0",
  "build_hash": null
}
```

**Status Codes:**
- `200 OK` - Service is healthy
- `503 Service Unavailable` - Service is down

---

### GET /readyz

Readiness check endpoint (keine Authentifizierung erforderlich).

**Request:**
```bash
curl http://localhost:8080/readyz
```

**Response (200 OK):**
```json
{
  "status": "OK",
  "checks": [
    {
      "name": "verifier_core",
      "status": "OK"
    },
    {
      "name": "crypto",
      "status": "OK"
    }
  ]
}
```

**Status Codes:**
- `200 OK` - Service is ready
- `503 Service Unavailable` - Service is not ready

**Use Case:** Kubernetes readiness probe

---

### GET /metrics

Prometheus metrics endpoint (keine Authentifizierung erforderlich).

**Request:**
```bash
curl http://localhost:8080/metrics
```

**Response (200 OK):**
```
# HELP cap_verifier_requests_total Total verification requests
# TYPE cap_verifier_requests_total counter
cap_verifier_requests_total{result="ok"} 42
cap_verifier_requests_total{result="fail"} 3
cap_verifier_requests_total{result="warn"} 1

# HELP cap_auth_token_validation_failures_total Authentication failures
# TYPE cap_auth_token_validation_failures_total counter
cap_auth_token_validation_failures_total 2

# HELP cap_cache_hit_ratio Cache effectiveness ratio
# TYPE cap_cache_hit_ratio gauge
cap_cache_hit_ratio 0.87

# HELP cap_verifier_request_duration_seconds Request latency
# TYPE cap_verifier_request_duration_seconds histogram
cap_verifier_request_duration_seconds_bucket{le="0.01"} 15
cap_verifier_request_duration_seconds_bucket{le="0.1"} 38
cap_verifier_request_duration_seconds_bucket{le="1.0"} 45
cap_verifier_request_duration_seconds_bucket{le="+Inf"} 46
cap_verifier_request_duration_seconds_sum 12.5
cap_verifier_request_duration_seconds_count 46
```

**Content-Type:** `text/plain; version=0.0.4`

**Metrics:**
- `cap_verifier_requests_total` - Counter, labels: `{result="ok|fail|warn"}`
- `cap_auth_token_validation_failures_total` - Counter
- `cap_cache_hit_ratio` - Gauge (0.0 - 1.0)
- `cap_verifier_request_duration_seconds` - Histogram

---

## Protected Endpoints

### POST /verify

Verifiziert einen Compliance-Proof.

**Authentication:** Required (Bearer Token)

**Request:**
```bash
curl -X POST https://verifier.example.com/verify \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d @verify-request.json
```

**Request Body:**
```json
{
  "policy_id": "0x0afcb402e74ff6a11601863fc4ae2f2d756124db71bc703f5889ecefcd371ff4",
  "context": {
    "manifest": {
      "version": "manifest.v1.0",
      "created_at": "2025-11-17T10:00:00Z",
      "supplier_root": "0x83a8779ddef4567890123456789012345678901234567890123456789012345678",
      "ubo_root": "0x0afcb402e74ff6a11601863fc4ae2f2d756124db71bc703f5889ecefcd371ff4",
      "company_commitment_root": "0xd490be94abc123def456789012345678901234567890123456789012345678901",
      "policy": {
        "name": "Test Policy",
        "version": "lksg.v1",
        "hash": "0x0afcb402e74ff6a11601863fc4ae2f2d756124db71bc703f5889ecefcd371ff4"
      },
      "audit": {
        "tail_digest": "0x1234567890abcdef",
        "events_count": 5
      },
      "proof": {
        "proof_type": "mock",
        "status": "ok"
      },
      "signatures": [],
      "time_anchor": null
    },
    "proof": {
      "version": "proof.v0",
      "type": "mock",
      "statement": "0x0afcb402e74ff6a11601863fc4ae2f2d756124db71bc703f5889ecefcd371ff4",
      "manifest_hash": "0xd490be94abc123def456789012345678901234567890123456789012345678901",
      "policy_hash": "0x0afcb402e74ff6a11601863fc4ae2f2d756124db71bc703f5889ecefcd371ff4",
      "proof_data": {
        "checked_constraints": [
          {
            "name": "require_at_least_one_ubo",
            "ok": true
          },
          {
            "name": "supplier_count_max",
            "ok": true
          }
        ]
      },
      "status": "ok"
    }
  },
  "backend": "mock",
  "options": null
}
```

**Request Schema:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `policy_id` | string | Yes | SHA3-256 hash of policy (0x-prefixed) |
| `context.manifest` | object | Yes | Complete manifest object |
| `context.proof` | object | No | Proof object (optional for manifest-only verification) |
| `backend` | string | Yes | Proof backend: "mock", "zkvm", "halo2" |
| `options` | object | No | Verification options (reserved) |

**Response (200 OK):**
```json
{
  "result": "ok",
  "manifest_hash": "0xd490be94abc123def456789012345678901234567890123456789012345678901",
  "proof_hash": "0x83a8779ddef456789012345678901234567890123456789012345678901234567",
  "trace": null,
  "signature": null,
  "timestamp": null,
  "report": {
    "status": "ok",
    "manifest_hash": "0xd490be94abc123def456789012345678901234567890123456789012345678901",
    "proof_hash": "0x83a8779ddef456789012345678901234567890123456789012345678901234567",
    "signature_valid": false,
    "details": []
  }
}
```

**Response Schema:**

| Field | Type | Description |
|-------|------|-------------|
| `result` | string | "ok", "fail", "warn" |
| `manifest_hash` | string | SHA3-256 hash of manifest |
| `proof_hash` | string | SHA3-256 hash of proof |
| `report.status` | string | Verification status |
| `report.signature_valid` | boolean | Whether signature is valid (if present) |
| `report.details` | array | Detailed verification messages |

**Error Responses:**

**400 Bad Request:**
```json
{
  "error": "invalid_request",
  "message": "Missing required field: policy_id"
}
```

**401 Unauthorized:**
```json
{
  "error": "unauthorized",
  "message": "Missing or invalid authorization token"
}
```

**403 Forbidden:**
```json
{
  "error": "forbidden",
  "message": "Insufficient scopes"
}
```

**500 Internal Server Error:**
```json
{
  "error": "internal_error",
  "message": "Verification failed"
}
```

---

### POST /policy/compile

Kompiliert und validiert eine Policy.

**Authentication:** Required (Bearer Token)

**Request:**
```bash
curl -X POST https://verifier.example.com/policy/compile \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "policy": {
      "version": "lksg.v1",
      "name": "Test Policy",
      "created_at": "2025-11-06T10:00:00Z",
      "constraints": {
        "require_at_least_one_ubo": true,
        "supplier_count_max": 10
      },
      "notes": ""
    }
  }'
```

**Request Body:**
```json
{
  "policy": {
    "version": "lksg.v1",
    "name": "Test Policy",
    "created_at": "2025-11-06T10:00:00Z",
    "constraints": {
      "require_at_least_one_ubo": true,
      "supplier_count_max": 10,
      "ubo_count_min": null,
      "require_statement_roots": null
    },
    "notes": ""
  }
}
```

**Request Schema:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `policy.version` | string | Yes | Policy version (e.g., "lksg.v1") |
| `policy.name` | string | Yes | Human-readable policy name |
| `policy.created_at` | string | Yes | RFC3339 timestamp |
| `policy.constraints` | object | Yes | Policy constraints |
| `policy.notes` | string | No | Additional notes |

**Constraints Schema:**

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `require_at_least_one_ubo` | boolean | false | At least one UBO required |
| `supplier_count_max` | integer | - | Maximum number of suppliers |
| `ubo_count_min` | integer | null | Minimum number of UBOs |
| `require_statement_roots` | array | null | Required statement roots |

**Response (200 OK):**
```json
{
  "policy_hash": "0x0afcb402e74ff6a11601863fc4ae2f2d756124db71bc703f5889ecefcd371ff4",
  "policy_info": {
    "name": "Test Policy",
    "version": "lksg.v1",
    "hash": "0x0afcb402e74ff6a11601863fc4ae2f2d756124db71bc703f5889ecefcd371ff4"
  },
  "status": "compiled"
}
```

**Response Schema:**

| Field | Type | Description |
|-------|------|-------------|
| `policy_hash` | string | SHA3-256 hash of policy |
| `policy_info` | object | Policy metadata |
| `status` | string | Compilation status ("compiled") |

**Error Responses:**

**400 Bad Request:**
```json
{
  "error": "validation_error",
  "message": "Invalid policy version: must match 'lksg.v*'"
}
```

---

### GET /policy/:id

Ruft eine kompilierte Policy ab.

**Authentication:** Required (Bearer Token)

**Request:**
```bash
curl -H "Authorization: Bearer $TOKEN" \
  https://verifier.example.com/policy/0x0afcb402e74ff6a11601863fc4ae2f2d756124db71bc703f5889ecefcd371ff4
```

**URL Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string | Policy hash (SHA3-256, 0x-prefixed) |

**Response (200 OK):**
```json
{
  "policy_hash": "0x0afcb402e74ff6a11601863fc4ae2f2d756124db71bc703f5889ecefcd371ff4",
  "policy": {
    "version": "lksg.v1",
    "name": "Test Policy",
    "created_at": "2025-11-06T10:00:00Z",
    "constraints": {
      "require_at_least_one_ubo": true,
      "supplier_count_max": 10
    },
    "notes": ""
  }
}
```

**Error Responses:**

**404 Not Found:**
```json
{
  "error": "not_found",
  "message": "Policy not found"
}
```

---

## CLI Reference

### cap prepare

Importiert CSV-Daten und berechnet Commitments.

**Usage:**
```bash
cap prepare \
  --suppliers suppliers.csv \
  --ubos ubos.csv \
  --output build/
```

**Options:**

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--suppliers` | path | Yes | Path to suppliers CSV |
| `--ubos` | path | Yes | Path to UBOs CSV |
| `--output` | path | No | Output directory (default: build/) |
| `--company` | string | No | Company data (JSON string) |

**Output:**
- `build/commitments.json` - Merkle roots
- `build/agent.audit.jsonl` - Audit log

**Example CSV (suppliers.csv):**
```csv
name,jurisdiction,tier
"Supplier A",DE,TIER_1
"Supplier B",FR,TIER_2
```

**Example CSV (ubos.csv):**
```csv
name,birthdate,citizenship
"John Doe","1980-01-15",US
"Jane Smith","1975-03-22",GB
```

---

### cap policy validate

Validiert eine Policy-Datei.

**Usage:**
```bash
cap policy validate --policy policy.lksg.v1.yml
```

**Options:**

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--policy` | path | Yes | Path to policy YAML |

**Output:**
```
Policy validation: OK
Policy hash: 0x0afcb402e74ff6a11601863fc4ae2f2d756124db71bc703f5889ecefcd371ff4
```

---

### cap manifest build

Erstellt ein Manifest aus Commitments und Policy.

**Usage:**
```bash
cap manifest build \
  --commitments build/commitments.json \
  --policy policy.lksg.v1.yml \
  --output build/manifest.json
```

**Options:**

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--commitments` | path | Yes | Path to commitments.json |
| `--policy` | path | Yes | Path to policy YAML |
| `--output` | path | No | Output path (default: build/manifest.json) |
| `--time-anchor` | string | No | Time anchor reference |

**Output:**
- `build/manifest.json`

---

### cap proof build

Generiert einen Proof.

**Usage:**
```bash
cap proof build \
  --manifest build/manifest.json \
  --policy policy.lksg.v1.yml \
  --backend mock \
  --output build/proof.dat
```

**Options:**

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--manifest` | path | Yes | Path to manifest.json |
| `--policy` | path | Yes | Path to policy YAML |
| `--backend` | string | No | Proof backend (default: mock) |
| `--output` | path | No | Output path (default: build/proof.dat) |

**Backends:**
- `mock` - Mock proof (Phase 1)
- `zkvm` - ZK-VM proof (Phase 3)
- `halo2` - Halo2 proof (Phase 3)

---

### cap proof verify

Verifiziert einen Proof.

**Usage:**
```bash
cap proof verify \
  --proof build/proof.dat \
  --manifest build/manifest.json
```

**Options:**

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--proof` | path | Yes | Path to proof.dat |
| `--manifest` | path | Yes | Path to manifest.json |

**Output:**
```
Verification: OK
Manifest hash: 0xd490be94abc...
Proof hash: 0x83a8779ddef...
```

---

### cap sign manifest

Signiert ein Manifest mit Ed25519.

**Usage:**
```bash
cap sign manifest \
  --manifest build/manifest.json \
  --key keys/private.ed25519 \
  --output build/signature.json
```

**Options:**

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--manifest` | path | Yes | Path to manifest.json |
| `--key` | path | Yes | Path to Ed25519 private key |
| `--output` | path | No | Output path (default: build/signature.json) |

---

### cap export

Exportiert ein standardisiertes Proof-Paket.

**Usage:**
```bash
cap export \
  --manifest build/manifest.json \
  --proof build/proof.dat \
  --output build/cap-proof/
```

**Options:**

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--manifest` | path | Yes | Path to manifest.json |
| `--proof` | path | Yes | Path to proof.dat |
| `--output` | path | Yes | Output directory |
| `--timestamp` | path | No | Path to timestamp file |
| `--registry` | path | No | Path to registry entry |

**Package Structure:**
```
cap-proof/
├── _meta.json
├── manifest.json
├── proof.dat
├── proof.json
├── timestamp.txt (optional)
├── registry.json (optional)
└── README.md
```

---

### cap registry add

Fügt einen Eintrag zur Registry hinzu.

**Usage:**
```bash
cap registry add \
  --manifest build/manifest.json \
  --proof build/proof.dat \
  --registry registry.db \
  --signing-key keys/company.ed25519
```

**Options:**

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--manifest` | path | Yes | Path to manifest.json |
| `--proof` | path | Yes | Path to proof.dat |
| `--registry` | path | Yes | Path to registry (JSON or SQLite) |
| `--signing-key` | path | No | Path to signing key (for signature) |

---

### cap registry find

Sucht einen Registry-Eintrag.

**Usage:**
```bash
cap registry find \
  --registry registry.db \
  --manifest-hash 0xd490be94abc... \
  --proof-hash 0x83a8779ddef...
```

**Options:**

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--registry` | path | Yes | Path to registry |
| `--manifest-hash` | string | Yes | Manifest hash |
| `--proof-hash` | string | Yes | Proof hash |

---

### cap keys keygen

Generiert ein Ed25519-Schlüsselpaar.

**Usage:**
```bash
cap keys keygen \
  --owner "Company Name" \
  --output keys/company \
  --valid-days 365
```

**Options:**

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--owner` | string | Yes | Key owner name |
| `--output` | path | Yes | Output path (without extension) |
| `--valid-days` | integer | No | Validity period (default: 365) |
| `--usage` | string | No | Key usage (default: "signing,registry") |

**Output:**
- `keys/company.json` - Metadata (cap-key.v1)
- `keys/company.ed25519` - Private key (32 bytes)
- `keys/company.pub` - Public key (32 bytes)

---

### cap keys rotate

Rotiert einen Schlüssel (retire old, activate new).

**Usage:**
```bash
cap keys rotate \
  --current keys/old-key.json \
  --new keys/new-key.json
```

**Options:**

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--current` | path | Yes | Current key metadata |
| `--new` | path | Yes | New key metadata |

**Actions:**
1. Mark `current` status as "retired"
2. Move `current` to `keys/archive/`
3. Update `new` status to "active"

---

### cap keys attest

Erstellt eine Chain-of-Trust Attestierung.

**Usage:**
```bash
cap keys attest \
  --signer keys/old-key.json \
  --subject keys/new-key.json \
  --output keys/attestation.json
```

**Options:**

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--signer` | path | Yes | Signer key metadata |
| `--subject` | path | Yes | Subject key metadata |
| `--output` | path | Yes | Output attestation file |

**Output:**
- Attestation file (cap-attestation.v1) mit Ed25519 Signature

---

### cap verifier run

Verifiziert ein Proof-Paket.

**Usage:**
```bash
cap verifier run --package build/cap-proof/
```

**Options:**

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--package` | path | Yes | Path to proof package |

**Output:**
```
Verification Report:
Status: OK
Manifest Hash: 0xd490be94abc...
Proof Hash: 0x83a8779ddef...
Signature: Valid
```

---

## Data Formats

### Manifest Format (manifest.v1.0)

```json
{
  "version": "manifest.v1.0",
  "created_at": "2025-11-17T10:00:00Z",
  "supplier_root": "0x...",
  "ubo_root": "0x...",
  "company_commitment_root": "0x...",
  "policy": {
    "name": "Policy Name",
    "version": "lksg.v1",
    "hash": "0x..."
  },
  "audit": {
    "tail_digest": "0x...",
    "events_count": 5
  },
  "proof": {
    "proof_type": "mock",
    "status": "ok"
  },
  "signatures": [],
  "time_anchor": null
}
```

### Proof Format (proof.v0)

```json
{
  "version": "proof.v0",
  "type": "mock",
  "statement": "0x...",
  "manifest_hash": "0x...",
  "policy_hash": "0x...",
  "proof_data": {
    "checked_constraints": [
      { "name": "require_at_least_one_ubo", "ok": true }
    ]
  },
  "status": "ok"
}
```

### Policy Format (lksg.v1)

```yaml
version: lksg.v1
name: My Policy
created_at: "2025-11-17T10:00:00Z"
constraints:
  require_at_least_one_ubo: true
  supplier_count_max: 100
  ubo_count_min: 1
notes: ""
```

---

## Error Codes

| Code | Status | Description |
|------|--------|-------------|
| 200 | OK | Request successful |
| 400 | Bad Request | Invalid request body or parameters |
| 401 | Unauthorized | Missing or invalid JWT token |
| 403 | Forbidden | Valid token but insufficient scopes |
| 404 | Not Found | Resource not found |
| 500 | Internal Server Error | Server-side error |
| 503 | Service Unavailable | Service is down or not ready |

---

## Rate Limiting

**Status:** Planned (Phase 3)

**Planned Limits:**
- 100 requests/minute per client_id
- 1000 requests/hour per client_id

---

## Pagination

**Status:** Not implemented (all endpoints return full results)

**Planned:** Phase 3 for `/registry/list`

---

## Versioning

**API Version:** v0.11.0
**Manifest Version:** manifest.v1.0
**Proof Version:** proof.v0
**Policy Version:** lksg.v1
**Key Metadata Version:** cap-key.v1
**Attestation Version:** cap-attestation.v1

---

## SDKs

**Status:** Planned (Phase 4)

**Planned Languages:**
- Python
- TypeScript
- Go
- Java
