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

## 👔 Für Management: Drei Arten der Bedienung

Das System hat **drei Schnittstellen** (wie ein Auto mit Lenkrad, Tempomat UND App-Steuerung):

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

### 3. Desktop App (für Offline-Nutzung)
**Was ist das?** Eine eigenständige Anwendung wie Word oder Excel - funktioniert komplett offline

**Beispiel-Anwendung:**
- Compliance-Beauftragter startet Desktop App
- Lädt CSV-Dateien per Drag & Drop
- Durchläuft 6-Schritte-Workflow
- Exportiert Compliance-Nachweis als ZIP

**Vorteile:**
✅ Keine Internetverbindung nötig
✅ Alle Daten bleiben auf dem lokalen Rechner
✅ Integrierter Audit-Trail
✅ Native Performance

**Wann wird was genutzt?**
- **REST API:** Für Integration in Unternehmenssoftware (SAP, ERP-Systeme)
- **CLI:** Für manuelle Operationen, Tests, Administration
- **Desktop App:** Für Offline-Workflow, datenschutzkritische Umgebungen, Air-Gapped Systeme

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

Kompiliert, validiert und speichert eine Policy mit automatischer Content-Deduplizierung.

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
  "policy_id": "550e8400-e29b-41d4-a716-446655440000",
  "policy_hash": "0x0afcb402e74ff6a11601863fc4ae2f2d756124db71bc703f5889ecefcd371ff4",
  "metadata": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Test Policy",
    "version": "lksg.v1",
    "hash": "0x0afcb402e74ff6a11601863fc4ae2f2d756124db71bc703f5889ecefcd371ff4",
    "status": "active",
    "created_at": "2025-11-17T10:00:00Z",
    "updated_at": "2025-11-17T10:00:00Z",
    "description": null
  },
  "status": "compiled"
}
```

**Response Schema:**

| Field | Type | Description |
|-------|------|-------------|
| `policy_id` | string | UUID v4 identifier |
| `policy_hash` | string | SHA3-256 hash of policy (0x-prefixed) |
| `metadata.id` | string | UUID v4 identifier (same as policy_id) |
| `metadata.name` | string | Policy name |
| `metadata.version` | string | Policy version |
| `metadata.hash` | string | SHA3-256 hash (for content deduplication) |
| `metadata.status` | string | "active", "deprecated", or "draft" |
| `metadata.created_at` | string | ISO 8601 timestamp (RFC3339) |
| `metadata.updated_at` | string | ISO 8601 timestamp (last modified) |
| `metadata.description` | string/null | Optional description |
| `status` | string | Compilation status ("compiled") |

**Content Deduplication:**
- Wenn eine Policy mit identischem Inhalt bereits existiert (gleicher Hash), wird die existierende Policy-ID zurückgegeben
- Nur das `updated_at` Feld wird aktualisiert
- Dies verhindert Duplikate und spart Speicherplatz

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

Ruft eine kompilierte Policy ab (by UUID oder Hash).

**Authentication:** Required (Bearer Token)

**Request (by UUID):**
```bash
curl -H "Authorization: Bearer $TOKEN" \
  https://verifier.example.com/policy/550e8400-e29b-41d4-a716-446655440000
```

**Request (by Hash):**
```bash
curl -H "Authorization: Bearer $TOKEN" \
  https://verifier.example.com/policy/0x0afcb402e74ff6a11601863fc4ae2f2d756124db71bc703f5889ecefcd371ff4
```

**URL Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string | Policy UUID v4 oder SHA3-256 Hash (0x-prefixed) |

**Response (200 OK):**
```json
{
  "metadata": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Test Policy",
    "version": "lksg.v1",
    "hash": "0x0afcb402e74ff6a11601863fc4ae2f2d756124db71bc703f5889ecefcd371ff4",
    "status": "active",
    "created_at": "2025-11-17T10:00:00Z",
    "updated_at": "2025-11-17T10:00:00Z",
    "description": null
  },
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

**Response Schema:**

| Field | Type | Description |
|-------|------|-------------|
| `metadata` | object | Policy metadata (see POST /policy/compile) |
| `policy` | object | Full policy definition |

**Error Responses:**

**404 Not Found:**
```json
{
  "error": "not_found",
  "message": "Policy not found"
}
```

**Use Cases:**
- Abruf by UUID: Eindeutige Identifikation einer spezifischen Policy-Version
- Abruf by Hash: Content-basiertes Lookup (gleicher Inhalt → gleiche Policy)

---

### POST /proof/upload

Multipart File Upload für Proof Packages (ZIP) - primär für WebUI Integration.

**Authentication:** Required (Bearer Token)

**Request:**
```bash
curl -X POST https://verifier.example.com/proof/upload \
  -H "Authorization: Bearer $TOKEN" \
  -F "file=@proof_package.zip"
```

**Request Schema:**
- Content-Type: `multipart/form-data`
- Field Name: `file`
- File Type: ZIP archive (.zip)
- Max File Size: 100 MB (configurable)

**Package Requirements:**
- Must contain `manifest.json` (required)
- Must contain `proof.dat` or equivalent proof file (required)
- Optional files: `timestamp.tsr`, `registry.json`, `README.txt`

**Response (200 OK):**
```json
{
  "manifest": {
    "version": "manifest.v1.0",
    "created_at": "2025-11-18T10:00:00Z",
    "company_commitment_root": "0x32f0a7411827ac0f0ce8b7c5e70a71badc93c27bbfef064b152d96ed37285ed5",
    "policy": {
      "name": "LkSG Demo Policy",
      "version": "lksg.v1",
      "hash": "0x0afcb402e74ff6a11601863fc4ae2f2d756124db71bc703f5889ecefcd371ff4"
    },
    "audit": {
      "tail_digest": "0x1234567890abcdef",
      "events_count": 5
    }
  },
  "proof_base64": "eyJ2ZXJzaW9uIjoicHJvb2Yud...",
  "package_info": {
    "total_size": 12345,
    "file_count": 2
  }
}
```

**Response Schema:**

| Field | Type | Description |
|-------|------|-------------|
| `manifest` | object | Parsed manifest.json content |
| `proof_base64` | string | Base64-encoded proof.dat content |
| `package_info.total_size` | integer | Total ZIP file size in bytes |
| `package_info.file_count` | integer | Number of files in ZIP |

**Error Responses:**

**400 Bad Request:**
```json
{
  "error": "invalid_package",
  "message": "Missing required file: manifest.json"
}
```

**413 Payload Too Large:**
```json
{
  "error": "file_too_large",
  "message": "File exceeds maximum size of 100 MB"
}
```

**Use Cases:**
- WebUI Drag & Drop Upload
- Automated testing
- Batch upload via scripts

**Integration:** Wird primär vom WebUI verwendet (`webui/src/core/api/client.ts:uploadProofPackage()`)

---

### POST /policy/v2/compile

Kompiliert und validiert PolicyV2 mit erweiterten Linting-Features und Fehler-Kategorien.

**Authentication:** Required (Bearer Token)

**Request:**
```bash
curl -X POST https://verifier.example.com/policy/v2/compile \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "policy": {
      "id": "lksg.demo.v1",
      "version": "1.0.0",
      "legal_basis": [
        {
          "directive": "LkSG",
          "article": "§3"
        }
      ],
      "description": "Demo policy",
      "inputs": {
        "ubo_count": {"type": "integer"},
        "supplier_count": {"type": "integer"}
      },
      "rules": [
        {
          "id": "rule_ubo_exists",
          "op": "range_min",
          "lhs": {"var": "ubo_count"},
          "rhs": 1
        }
      ]
    },
    "persist": true,
    "lint_mode": "relaxed"
  }'
```

**Request Body:**
```json
{
  "policy": {
    "id": "lksg.demo.v1",
    "version": "1.0.0",
    "legal_basis": [
      {
        "directive": "LkSG",
        "article": "§3"
      }
    ],
    "description": "Demo policy for testing",
    "inputs": {
      "ubo_count": {"type": "integer"},
      "supplier_count": {"type": "integer"}
    },
    "rules": [
      {
        "id": "rule_ubo_exists",
        "op": "range_min",
        "lhs": {"var": "ubo_count"},
        "rhs": 1
      },
      {
        "id": "rule_supplier_eq",
        "op": "eq",
        "lhs": {"var": "supplier_count"},
        "rhs": {"var": "supplier_count"}
      }
    ]
  },
  "persist": true,
  "lint_mode": "relaxed"
}
```

**Request Schema:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `policy.id` | string | Yes | Policy identifier (e.g., "lksg.demo.v1") |
| `policy.version` | string | Yes | Semantic version (e.g., "1.0.0") |
| `policy.legal_basis` | array | No | Legal references |
| `policy.description` | string | No | Policy description |
| `policy.inputs` | object | Yes | Input variable definitions |
| `policy.rules` | array | Yes | Rule definitions (see below) |
| `persist` | boolean | No | Store policy in backend (default: false) |
| `lint_mode` | string | No | "strict" or "relaxed" (default: "strict") |

**Rule Schema:**

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique rule identifier |
| `op` | string | Operator: "range_min", "eq", "non_membership" |
| `lhs` | object | Left-hand side ({"var": "name"} or {"const": value}) |
| `rhs` | object/number | Right-hand side |

**Erlaubte Operatoren:**
- `range_min` - Minimum-Check (Ersatz für `>=`)
- `eq` - Equality-Check
- `non_membership` - Blacklist-Check (für Sanctions-Listen)

**⚠️ NICHT erlaubt:** `>=`, `<=`, `>`, `<` (führen zu E2001 Lint-Errors)

**Response (200 OK):**
```json
{
  "policy_id": "lksg.demo.v1",
  "policy_hash": "0x1da941f7026bae3cf8b1bcdc3a8e01e76ea678c32ec6bc2c374fb67b3744571f",
  "stored": true,
  "lints": []
}
```

**Response Schema:**

| Field | Type | Description |
|-------|------|-------------|
| `policy_id` | string | Policy identifier |
| `policy_hash` | string | SHA3-256 hash of policy |
| `stored` | boolean | Whether policy was persisted |
| `lints` | array | Lint warnings/errors (see below) |

**Lint Result Schema:**

| Field | Type | Description |
|-------|------|-------------|
| `code` | string | Error code (e.g., "E2001", "W3001") |
| `level` | string | "error", "warning", "info" |
| `message` | string | Human-readable error message |
| `rule_id` | string | Affected rule ID (optional) |

**Error Responses:**

**400 Bad Request (Validation Error):**
```json
{
  "error": "validation_error",
  "message": "Invalid policy structure",
  "details": {
    "lints": [
      {
        "code": "E1001",
        "level": "error",
        "message": "Missing required field: policy.id"
      }
    ]
  }
}
```

**422 Unprocessable Entity (Lint Errors):**
```json
{
  "error": "lint_error",
  "message": "Policy contains lint errors",
  "details": {
    "lints": [
      {
        "code": "E2001",
        "level": "error",
        "message": "invalid op '>=' (allowed: non_membership, eq, range_min)",
        "rule_id": "rule_ubo_min"
      }
    ]
  }
}
```

**Lint Error Codes:**
- **E1001**: Missing required field
- **E2001**: Invalid operator (nicht erlaubte Operatoren)
- **E3001**: Type mismatch in rule
- **W3001**: Unused variable
- **W4001**: Potentially unreachable rule

**Use Cases:**
- WebUI Policy Compilation (mit Linter-Feedback)
- Automated Policy Validation in CI/CD
- Policy Development mit sofortigem Feedback

**Unterschied zu /policy/compile:**
- PolicyV2 hat erweiterte Rule-Struktur
- Linter-Integration mit Error-Kategorien
- `persist` Flag für optionale Speicherung
- `lint_mode` für flexible Validierung

---

## Desktop App IPC Commands (Tauri 2.0) - v0.12.0

**Für Management:** Die Desktop App kommuniziert intern über "IPC" (Inter-Process Communication) - das Frontend (was Sie sehen) redet mit dem Backend (was die Arbeit macht). Für Endbenutzer unsichtbar, aber wichtig für Entwickler.

### Technische Grundlagen

**Protokoll:** Tauri IPC (invoke/emit Pattern)
**Kommunikation:** TypeScript Frontend → Rust Backend
**Authentifizierung:** Nicht erforderlich (lokale App)
**Format:** JSON-serialisierte Parameter

**TypeScript-Aufruf (Frontend):**
```typescript
import { invoke } from '@tauri-apps/api/core';

// Beispiel: CSV importieren
const result = await invoke<ImportResult>('import_csv', {
  path: '/Users/user/suppliers.csv',
  csvType: 'suppliers',
  project: '/Users/user/cap-workspace/my-project'
});
```

**Rust-Handler (Backend):**
```rust
#[tauri::command]
pub async fn import_csv(
    path: String,
    csv_type: String,
    project: String
) -> Result<ImportResult, String> {
    // Implementation
}
```

---

### Project Management Commands

#### select_workspace

Öffnet einen nativen Ordner-Dialog zur Workspace-Auswahl.

**Aufruf:**
```typescript
const path = await selectWorkspace();
// Returns: "/Users/user/cap-workspace" oder null (abgebrochen)
```

**Response:**
```typescript
string | null  // Ausgewählter Pfad oder null
```

---

#### create_project

Erstellt ein neues Projekt im ausgewählten Workspace.

**Aufruf:**
```typescript
const project = await createProject(workspace, projectName);
```

**Parameter:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `workspace` | string | Pfad zum Workspace |
| `name` | string | Projektname |

**Response:**
```typescript
interface ProjectResult {
  path: string;      // Vollständiger Projektpfad
  name: string;      // Projektname
  createdAt: string; // ISO 8601 Timestamp
}
```

**Erstellt:**
- `{workspace}/{name}/input/`
- `{workspace}/{name}/build/`
- `{workspace}/{name}/export/`
- Audit Event: `project_created`

---

#### get_project_status

Liest den aktuellen Status eines Projekts (welche Schritte bereits abgeschlossen).

**Aufruf:**
```typescript
const status = await getProjectStatus(projectPath);
```

**Parameter:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `project` | string | Projektpfad |

**Response:**
```typescript
interface ProjectStatus {
  hasSuppliersCSv: boolean;
  hasUbosCsv: boolean;
  hasPolicy: boolean;
  hasCommitments: boolean;
  hasManifest: boolean;
  hasProof: boolean;
  currentStep: string;  // "import" | "commitments" | "policy" | ...
  info: ProjectInfo;
}
```

---

### Workflow Step Commands

#### import_csv

Importiert eine CSV-Datei (Suppliers oder UBOs).

**Aufruf:**
```typescript
const result = await importCsv(filePath, csvType, projectPath);
```

**Parameter:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `path` | string | Pfad zur CSV-Datei |
| `csvType` | string | "suppliers" oder "ubos" |
| `project` | string | Projektpfad |

**Response:**
```typescript
interface ImportResult {
  rowCount: number;   // Gesamtzahl Zeilen
  validRows: number;  // Gültige Zeilen
  hash: string;       // BLAKE3 Hash (0x-präfixiert)
  fileType: string;   // "suppliers" oder "ubos"
}
```

**Audit Event:** `csv_imported`

---

#### build_commitments

Berechnet kryptographische Commitments für alle importierten Daten.

**Aufruf:**
```typescript
const result = await buildCommitments(projectPath);
```

**Parameter:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `project` | string | Projektpfad |

**Response:**
```typescript
interface CommitmentsResult {
  supplierRoot: string;   // Merkle Root für Suppliers
  uboRoot: string;        // Merkle Root für UBOs
  supplierCount: number;  // Anzahl Supplier-Records
  uboCount: number;       // Anzahl UBO-Records
}
```

**Erstellt:** `build/commitments.json`
**Audit Event:** `commitments_created`

---

#### load_policy

Lädt eine Policy-Datei oder verwendet die Default-Policy.

**Aufruf:**
```typescript
const info = await loadPolicy(projectPath, policyPath?);
```

**Parameter:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `project` | string | Projektpfad |
| `policyPath` | string? | Optional: Pfad zu eigener Policy |

**Response:**
```typescript
interface PolicyInfo {
  name: string;           // Policy-Name
  version: string;        // z.B. "lksg.v1"
  hash: string;           // SHA3-256 Hash
  constraints: string[];  // Liste der Regeln
}
```

**Erstellt:** `input/policy.yml` (wenn nicht vorhanden)
**Audit Event:** `policy_loaded`

---

#### build_manifest

Erstellt das Compliance-Manifest.

**Aufruf:**
```typescript
const result = await buildManifest(projectPath);
```

**Parameter:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `project` | string | Projektpfad |

**Response:**
```typescript
interface ManifestResult {
  hash: string;       // SHA3-256 des Manifests
  version: string;    // "manifest.v1.0"
  createdAt: string;  // ISO 8601 Timestamp
}
```

**Erstellt:** `build/manifest.json`
**Audit Event:** `manifest_built`

---

#### build_proof

Generiert den Compliance-Proof.

**Aufruf:**
```typescript
const result = await buildProof(projectPath);
```

**Parameter:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `project` | string | Projektpfad |

**Response:**
```typescript
interface ProofResult {
  proofHash: string;  // SHA3-256 des Proofs
  backend: string;    // "mock" (später: "zkvm", "halo2")
  status: string;     // "ok" oder "fail"
}
```

**Erstellt:** `build/proof.capz`
**Audit Event:** `proof_built`

---

#### export_bundle

Exportiert das fertige Bundle als ZIP-Datei.

**Aufruf:**
```typescript
const result = await exportBundle(projectPath, outputPath);
```

**Parameter:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `project` | string | Projektpfad |
| `output` | string | Zielpfad für ZIP |

**Response:**
```typescript
interface ExportResult {
  bundlePath: string;  // Pfad zur erstellten ZIP
  sizeBytes: number;   // Dateigröße
  hash: string;        // BLAKE3 Bundle-Hash
  files: string[];     // Enthaltene Dateien
}
```

**Erstellt:**
- ZIP-Datei am angegebenen Pfad
- Kopie in `export/cap-bundle-{timestamp}.zip`

**Audit Event:** `bundle_exported`

---

### Audit & Verification Commands

#### read_audit_log

Liest den Audit-Trail eines Projekts.

**Aufruf:**
```typescript
const entries = await readAuditLog(projectPath);
```

**Response:**
```typescript
interface AuditEntry {
  seq: number;          // Sequenznummer
  ts: string;           // ISO 8601 Timestamp
  event: string;        // Event-Typ
  details: object;      // Event-spezifische Daten
  prevDigest: string;   // Hash des vorherigen Eintrags
  digest: string;       // SHA3-256 Hash dieses Eintrags
}[]
```

---

#### verify_bundle

Verifiziert ein exportiertes Bundle offline.

**Aufruf:**
```typescript
const result = await verifyBundle(bundlePath);
```

**Parameter:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `bundlePath` | string | Pfad zur ZIP-Datei |

**Response:**
```typescript
interface VerificationResult {
  status: string;        // "ok" | "warn" | "fail"
  manifestHash: string;  // SHA3-256 des Manifests
  proofHash: string;     // SHA3-256 des Proofs
  details: string[];     // Details zur Verifikation
}
```

---

### Audit Event Types

Die Desktop App schreibt folgende Events in den Audit-Trail:

| Event | Trigger | Details |
|-------|---------|---------|
| `project_created` | create_project | `{project_name}` |
| `csv_imported` | import_csv | `{file_type, row_count, file_hash}` |
| `commitments_created` | build_commitments | `{supplier_root, ubo_root}` |
| `policy_loaded` | load_policy | `{policy_name, policy_hash}` |
| `manifest_built` | build_manifest | `{manifest_hash}` |
| `proof_built` | build_proof | `{proof_hash, backend}` |
| `bundle_exported` | export_bundle | `{output_path, hash, size}` |

**Hash-Chain:** Jeder Eintrag enthält `prev_digest` → Manipulationen sofort erkennbar

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

### cap export (cap-bundle.v1 Format) ⭐

Exportiert ein standardisiertes Proof-Paket im **cap-bundle.v1 Format** mit SHA3-256 Hashes und strukturierten Metadaten.

**Usage:**
```bash
cap export \
  --manifest build/manifest.json \
  --proof build/proof.dat \
  --output build/cap-proof/ \
  --force
```

**Options:**

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--manifest` | path | Yes | Path to manifest.json |
| `--proof` | path | Yes | Path to proof.dat |
| `--output` | path | Yes | Output directory |
| `--timestamp` | path | No | Path to timestamp file |
| `--registry` | path | No | Path to registry entry |
| `--force` | flag | No | Overwrite existing output directory |

**Package Structure (cap-bundle.v1):**
```
cap-proof/
├── _meta.json              # Bundle metadata (schema: cap-bundle.v1)
│                           # - SHA3-256 hashes für alle Dateien
│                           # - BundleFileMeta mit role, size, content_type
│                           # - Automatisch extrahierte Policy-Informationen
├── manifest.json           # Compliance manifest (role: "manifest")
├── proof.dat               # Zero-knowledge proof (role: "proof")
├── timestamp.tsr           # Optional: Timestamp (role: "timestamp")
├── registry.json           # Optional: Registry (role: "registry")
├── verification.report.json # Verification report (role: "report")
└── README.txt              # Human-readable instructions
```

**_meta.json Schema (cap-bundle.v1):**
```json
{
  "schema": "cap-bundle.v1",
  "bundle_id": "bundle-1732464123",
  "created_at": "2025-11-24T10:00:00Z",
  "files": {
    "manifest.json": {
      "role": "manifest",
      "hash": "0x1da941f7...",   // SHA3-256
      "size": 1234,
      "content_type": "application/json",
      "optional": false
    },
    "proof.dat": {
      "role": "proof",
      "hash": "0x83a8779d...",
      "size": 5678,
      "content_type": "application/octet-stream",
      "optional": false
    }
  },
  "proof_units": [
    {
      "id": "default",
      "manifest_file": "manifest.json",
      "proof_file": "proof.dat",
      "policy_id": "LkSG Demo Policy",    // Automatisch aus Manifest extrahiert
      "policy_hash": "0xabc123...",       // Automatisch aus Manifest extrahiert
      "backend": "mock"
    }
  ]
}
```

**Features:**
✅ SHA3-256 Hashes für jede Datei (Integritätsprüfung)
✅ Strukturierte Metadaten (Rolle, Typ, Größe)
✅ Policy-Informationen automatisch extrahiert
✅ Standardisiertes Format für Offline-Verifikation
✅ Backward-compatible (alte Pakete werden noch unterstützt)
✅ Sicherheit: Path Traversal Prevention, Cycle Detection, TOCTOU Mitigation
✅ Bundle Type Detection (Modern vs Legacy)

**Analogie (Management):** Wie ein standardisiertes Versandpaket mit Barcode, Tracking-Nummer und Inhaltsliste - jede Datei hat einen eindeutigen "Fingerabdruck" (SHA3-256 Hash)

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

### cap-bundle.v1 Package Format ⭐

**Für Management:** Das cap-bundle.v1 Format ist das standardisierte Paketformat für offline-verifizierbare Compliance-Nachweise mit strukturierten Metadaten und SHA3-256 Integritätsprüfung.

**Problem (vorher):**
- `proof export` erstellte Pakete im alten Format (cap-proof.v1.0)
- `verifier run` erwartete neues Format (cap-bundle.v1)
- **Ergebnis:** Inkompatibilität, Tests schlugen fehl

**Lösung (jetzt):**
- Beide Tools sprechen die gleiche "Sprache" (cap-bundle.v1)
- Strukturierte Metadaten für jede Datei
- SHA3-256 Hashes für Integritätsprüfung
- Automatische Policy-Information-Extraktion aus Manifest

#### Bundle Structure

```
cap-proof/
├─ _meta.json              # Bundle metadata (schema: cap-bundle.v1)
├─ manifest.json           # Compliance manifest (role: "manifest", optional: false)
├─ proof.dat               # Zero-knowledge proof (role: "proof", optional: false)
├─ timestamp.tsr           # Optional: Timestamp (role: "timestamp", optional: true)
├─ registry.json           # Optional: Registry (role: "registry", optional: true)
├─ verification.report.json # Verification report (role: "report", optional: false)
└─ README.txt              # Human-readable instructions
```

#### _meta.json Schema (BundleMeta)

```json
{
  "schema": "cap-bundle.v1",
  "bundle_id": "bundle-1732464123",
  "created_at": "2025-11-24T10:00:00Z",
  "files": {
    "manifest.json": {
      "role": "manifest",
      "hash": "0x1da941f7026bae3cf8b1bcdc3a8e01e76ea678c32ec6bc2c374fb67b3744571f",
      "size": 1234,
      "content_type": "application/json",
      "optional": false
    },
    "proof.dat": {
      "role": "proof",
      "hash": "0x83a8779ddef4567890123456789012345678901234567890123456789012345678",
      "size": 5678,
      "content_type": "application/octet-stream",
      "optional": false
    },
    "timestamp.tsr": {
      "role": "timestamp",
      "hash": "0xabc123def456789012345678901234567890123456789012345678901234567890",
      "size": 901,
      "content_type": "application/timestamp-reply",
      "optional": true
    },
    "registry.json": {
      "role": "registry",
      "hash": "0xdef456789012345678901234567890123456789012345678901234567890123456",
      "size": 234,
      "content_type": "application/json",
      "optional": true
    },
    "verification.report.json": {
      "role": "report",
      "hash": "0x321fed654789012345678901234567890123456789012345678901234567890123",
      "size": 567,
      "content_type": "application/json",
      "optional": false
    }
  },
  "proof_units": [
    {
      "id": "default",
      "manifest_file": "manifest.json",
      "proof_file": "proof.dat",
      "policy_id": "LkSG Demo Policy",
      "policy_hash": "0x0afcb402e74ff6a11601863fc4ae2f2d756124db71bc703f5889ecefcd371ff4",
      "backend": "mock"
    }
  ]
}
```

#### BundleFileMeta Fields

| Field | Type | Description |
|-------|------|-------------|
| `role` | string | File role: "manifest", "proof", "timestamp", "registry", "report" |
| `hash` | string | SHA3-256 hash (0x-prefixed, 64 hex characters) |
| `size` | integer | File size in bytes |
| `content_type` | string | MIME type (optional, can be null) |
| `optional` | boolean | Whether file is optional (false = required) |

#### ProofUnit Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unit identifier (e.g., "default") |
| `manifest_file` | string | Manifest filename |
| `proof_file` | string | Proof filename |
| `policy_id` | string | Policy name (extracted from manifest) |
| `policy_hash` | string | Policy SHA3-256 hash (extracted from manifest) |
| `backend` | string | Proof backend: "mock", "zkvm", "halo2" |

#### Validation Rules

**Required Files:**
- `manifest.json` (role: "manifest", optional: false)
- `proof.dat` (role: "proof", optional: false)
- `_meta.json` (Bundle metadata)

**Optional Files:**
- `timestamp.tsr` (role: "timestamp", optional: true)
- `registry.json` (role: "registry", optional: true)
- `verification.report.json` (role: "report", depends on context)

**Hash Validation:**
- Alle Hashes müssen SHA3-256 sein (0x-präfixiert, 64 hex chars)
- Verifier prüft Hashes gegen tatsächliche Dateiinhalte
- Manipulationen werden sofort erkannt

**Schema Validation:**
- `schema` muss exakt "cap-bundle.v1" sein
- `bundle_id` Format: "bundle-<timestamp>" (empfohlen)
- `created_at` muss RFC3339 Timestamp sein

#### Features

✅ **Integrität** - Jede Datei hat SHA3-256 Hash → Manipulationen sofort erkennbar
✅ **Metadaten** - Strukturierte Informationen (Rolle, Typ, Größe)
✅ **Standardisierung** - Alle Tools verstehen das gleiche Format
✅ **Auditierbarkeit** - Auditoren können jede Datei einzeln prüfen
✅ **Policy-Info** - Policy-Name und Hash automatisch im Paket enthalten
✅ **Multi-Proof-Support** - Mehrere Proof-Units in einem Bundle (vorbereitet)
✅ **Backward-Kompatibilität** - Alte Pakete werden noch unterstützt (Fallback)
✅ **Sicherheit** - Path Traversal Prevention, Cycle Detection, TOCTOU Mitigation
✅ **Bundle Type Detection** - Automatische Erkennung (Modern vs Legacy)

#### Migration von v1.0 zu cap-bundle.v1

**Old Format (cap-proof.v1.0):**
```
cap-proof/
├── manifest.json
├── proof.dat
└── README.txt
```

**New Format (cap-bundle.v1):**
```
cap-proof/
├── _meta.json      # NEU: Bundle metadata mit Hashes
├── manifest.json
├── proof.dat
├── verification.report.json  # NEU: Verification report
└── README.txt
```

**Verifier Behavior:**
- Prüft zuerst auf `_meta.json` Existenz
- Falls vorhanden → cap-bundle.v1 Verifikation (mit Hash-Checks)
- Falls nicht vorhanden → Legacy Fallback (ohne Hash-Checks)

**Analogie (Management):** Wie der Upgrade von einfachen Versandtaschen zu standardisierten Paketen mit Barcode, Tracking-Nummer und Inhaltsliste - jede Datei hat einen eindeutigen "Fingerabdruck"

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

**Status:** ✅ Production Ready (v0.11.0)

### Implementation

Das System verwendet **IP-basierte Rate Limiting** mit Token Bucket Algorithm (GCRA) via `tower_governor` und `governor` crates.

**Middleware:** `api/rate_limit.rs`
- Automatische IP-Extraktion (via X-Forwarded-For oder Socket Address)
- Token Bucket Algorithm (Gradual Capacity Recovery Algorithm)
- Per-Endpoint Rate Limit Konfiguration
- Standard Rate Limit HTTP Headers

### Default Rate Limits

| Endpoint | Limit | Burst | Zweck |
|----------|-------|-------|-------|
| **Global (Default)** | 100 req/min | 120 | Allgemeine API-Nutzung |
| **POST /verify** | 20 req/min | 25 | Proof-Verifikation (moderate) |
| **POST /policy/v2/compile** | 10 req/min | 15 | Policy-Compilation (expensive) |

**Burst:** Erlaubt kurzfristige Spitzen ohne sofortige 429 Errors

### HTTP Response Headers

Bei erfolgreichen Requests:
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
```

Bei Rate Limit Überschreitung (HTTP 429):
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 0
Retry-After: 36
```

**Retry-After:** Sekunden bis nächster Request erlaubt ist

### CLI Configuration

**Verifier API Server Flags:**
```bash
cargo run --bin cap-verifier-api \
  --rate-limit 100 \
  --rate-limit-burst 120
```

**Optionen:**
- `--rate-limit <number>` - Requests pro Minute (default: 100)
- `--rate-limit-burst <number>` - Burst Size (default: 120)

### Konfigurierte Presets

**RateLimitConfig Presets in `api/rate_limit.rs`:**

1. **default_global()**
   - 100 req/min, burst 120
   - Für allgemeine API-Nutzung
   - Balance zwischen Verfügbarkeit und DoS-Schutz

2. **strict()**
   - 10 req/min, burst 15
   - Für teure Operationen (Policy Compilation, WASM-Execution)
   - Verhindert Resource-Exhaustion

3. **moderate()**
   - 20 req/min, burst 25
   - Für normale Operationen (Verification, Upload)
   - Kompromiss zwischen Durchsatz und Schutz

### API Response bei 429 Too Many Requests

**Status Code:** 429 Too Many Requests

**Response Body:**
```json
{
  "error": "Too many requests. Please retry after 36 seconds.",
  "code": "RATE_LIMIT_EXCEEDED"
}
```

### Implementation Details

**Dependencies:**
- `tower_governor = "0.4"` - Tower Middleware Integration
- `governor = "0.6"` - GCRA Token Bucket Implementation

**Middleware Layer:**
```rust
use crate::api::rate_limit::{rate_limiter_layer, RateLimitConfig};

let rate_limit_config = RateLimitConfig::default_global();
let rate_limiter = rate_limiter_layer(rate_limit_config);

Router::new()
    .route("/verify", post(handle_verify))
    .layer(rate_limiter)
```

**SmartIpKeyExtractor:**
- Extrahiert IP automatisch aus X-Forwarded-For Header (wenn vorhanden)
- Fallback auf Socket Address
- Unterstützt Load Balancer und Reverse Proxies

### Per-Endpoint Rate Limits

Für individuell konfigurierte Limits pro Endpoint:

```rust
// Strict limit for expensive operations
let policy_limiter = rate_limiter_layer(RateLimitConfig::strict());

// Moderate limit for normal operations
let verify_limiter = rate_limiter_layer(RateLimitConfig::moderate());

Router::new()
    .route("/policy/v2/compile", post(handle_policy_compile))
    .layer(policy_limiter)
    .route("/verify", post(handle_verify))
    .layer(verify_limiter)
```

### Production Considerations

**Empfehlungen für Production:**
- **Monitoring:** Prometheus Metrics für Rate Limit Hits (`rate_limit_exceeded_total`)
- **Alerting:** Alert bei hoher Rate Limit Hit-Rate (z.B. > 10% aller Requests)
- **Whitelist:** Trusted IPs (interne Services) von Rate Limiting ausschließen
- **Dynamic Limits:** Client-spezifische Limits basierend auf Subscription-Level
- **Distributed Rate Limiting:** Redis-Backend für Multi-Instance Deployments

**Tests:** 4 Unit-Tests in `api/rate_limit.rs`
- Config presets (default_global, strict, moderate)
- Layer creation
- IP extraction logic

---

## Pagination

**Status:** Not implemented (all endpoints return full results)

**Planned:** Phase 3 for `/registry/list`

---

## Versioning

**API Version:** v0.12.0
**Desktop App Version:** v0.12.0
**Manifest Version:** manifest.v1.0
**Proof Version:** proof.v0
**Policy Version:** lksg.v1
**Key Metadata Version:** cap-key.v1
**Attestation Version:** cap-attestation.v1
**Audit Trail Version:** v1.0

---

## SDKs

**Status:** Planned (Phase 4)

**Planned Languages:**
- Python
- TypeScript
- Go
- Java
