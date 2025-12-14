# SAP Adapter Pilot E2E Flow

> **Status:** GEPLANT (Future Version v1.0+)
> Dieses Feature ist für zukünftige Enterprise-Versionen geplant.
> Derzeit nicht im Minimal Local Agent (v0.12.2) enthalten.
> Voraussetzung: Wiedereinführung der REST API für Server-Kommunikation.

## Overview

The SAP Adapter pilot demonstrates the end-to-end integration of the CAP Verifier API with SAP S/4HANA systems via OData. This document describes the complete workflow from data extraction to compliance verification to result writeback.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                      SAP Adapter Pilot E2E Flow                     │
└─────────────────────────────────────────────────────────────────────┘

    1. PULL                 2. VERIFY                3. WRITEBACK
    ┌────────┐             ┌────────┐              ┌────────┐
    │  SAP   │   OData     │  CAP   │   REST       │  SAP   │   OData
    │ S/4    │◄────────────│Adapter │─────────────►│ S/4    │◄──────────
    │ HANA   │  Suppliers  │        │   Verify     │ HANA   │  Status
    └────────┘             └────────┘              └────────┘
        │                      │                        │
        │                      │                        │
        ▼                      ▼                        ▼
   Z_CAP_SUPPLIERS      Verifier API         Z_CAP_SUPPLIER_STATUS
   (Source Table)       (REST Endpoint)      (Target Table)
```

## Prerequisites

### SAP System Configuration

**Required OData Services:**
- `Z_CAP_SUPPLIERS_SRV` - Supplier master data extraction
- `Z_CAP_STATUS_SRV` - Compliance status writeback

**Required Custom Tables:**

1. **Z_CAP_SUPPLIERS** (Source Data)
   ```abap
   * Field Name      Data Type    Length    Description
   * SUPPLIER_ID     CHAR         10        Unique supplier identifier
   * SUPPLIER_NAME   CHAR         50        Company name
   * JURISDICTION    CHAR         3         ISO country code
   * TIER            INT          1         Supply chain tier (1-3)
   * SANCTION_FLAG   CHAR         1         'X' if sanctioned
   * CREATED_AT      DATS         8         Creation date
   * CREATED_BY      CHAR         12        User ID
   ```

2. **Z_CAP_SUPPLIER_STATUS** (Target/Writeback)
   ```abap
   * Field Name      Data Type    Length    Description
   * RUN_ID          CHAR         32        Idempotency key
   * SUPPLIER_ID     CHAR         10        Foreign key to Z_CAP_SUPPLIERS
   * MANIFEST_HASH   CHAR         66        SHA3-256 (0x-prefixed)
   * POLICY_HASH     CHAR         66        SHA3-256 (0x-prefixed)
   * IR_HASH         CHAR         66        SHA3-256 (0x-prefixed)
   * VERDICT         CHAR         10        'ok' or 'fail'
   * VERIFIED_AT     DATS         8         Verification date
   * VERIFIED_BY     CHAR         12        System user
   ```

**Authorization Requirements:**
- `S_SERVICE` - OData service execution
- Display authorization on `Z_CAP_SUPPLIERS`
- Create/Update authorization on `Z_CAP_SUPPLIER_STATUS`

### CAP Adapter Configuration

**Environment Variables:**
```bash
# SAP Connection
export SAP_URL="https://sap-s4.example.com:8443/sap/opu/odata/sap/Z_CAP_SUPPLIERS_SRV"
export SAP_CLIENT="100"  # SAP client/mandant
export SAP_USER="CAP_ADAPTER"
export SAP_PASSWORD="<secure-password>"  # Use vault in production

# CAP Verifier API
export CAP_API_BASE="https://cap-verifier.example.com/api/v1"
export CAP_API_TOKEN="<oauth2-jwt-token>"

# Rate Limiting
export ADAPTER_RATE_LIMIT="10"  # Requests per second
export ADAPTER_RETRY_MAX="3"
export ADAPTER_RETRY_BACKOFF="exponential"  # exponential or linear
```

**Adapter Binary:**
```bash
# Build SAP adapter (separate repository)
cd sap-adapter
cargo build --release

# Install
cp target/release/cap-adapter /usr/local/bin/
```

## E2E Workflow

### Phase 1: Data Pull (SAP → Adapter)

#### Command
```bash
cap-adapter pull \
  --odata "$SAP_URL" \
  --client "$SAP_CLIENT" \
  --user "$SAP_USER" \
  --password-env SAP_PASSWORD \
  --filter "TIER le 2" \
  --out context.json \
  --audit /var/log/cap/adapter.audit.jsonl
```

#### OData Query (Generated)
```http
GET /sap/opu/odata/sap/Z_CAP_SUPPLIERS_SRV/SupplierSet
    ?$filter=TIER le 2
    &$format=json
    &sap-client=100
Authorization: Basic <base64(user:password)>
```

#### Output: `context.json`
```json
{
  "policy_id": "lksg.v1",
  "context": {
    "supplier_hashes": [
      "0x1da941f7026bae3cf8b1bcdc3a8e01e76ea678c32ec6bc2c374fb67b3744571f",
      "0x83a8779ddef4567890123456789012345678901234567890123456789012345678"
    ],
    "ubo_hashes": [],
    "company_commitment_root": "0xabc123def456789012345678901234567890123456789012345678901234567890",
    "sanctions_root": null,
    "jurisdiction_root": null,
    "variables": {
      "extraction_timestamp": "2025-11-10T10:00:00Z",
      "sap_client": "100",
      "record_count": 50
    }
  },
  "metadata": {
    "run_id": "RUN_1731232800",
    "source_system": "SAP_S4_PROD",
    "extraction_method": "odata_v2"
  }
}
```

**Validation:**
- ✅ All supplier records extracted (50/50)
- ✅ Hashes computed via BLAKE3
- ✅ No PII in hashes (only commitments)
- ✅ Audit log entry created

### Phase 2: Compliance Verification (Adapter → CAP API → Adapter)

#### Command
```bash
curl -s -k \
  -H "Authorization: Bearer $CAP_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d @context.json \
  "$CAP_API_BASE/verify" \
  | tee /tmp/verify.json
```

#### Request Body (from context.json)
```json
{
  "policy_id": "lksg.v1",
  "context": {
    "supplier_hashes": ["0x1da941f7...", "0x83a8779d..."],
    "ubo_hashes": [],
    "company_commitment_root": "0xabc123def...",
    "sanctions_root": null,
    "jurisdiction_root": null,
    "variables": {
      "extraction_timestamp": "2025-11-10T10:00:00Z",
      "sap_client": "100",
      "record_count": 50
    }
  },
  "backend": "mock",
  "options": {
    "check_timestamp": false,
    "check_registry": false
  }
}
```

#### Response: `/tmp/verify.json`
```json
{
  "result": "ok",
  "manifest_hash": "0xd490be94abc123def456789012345678901234567890123456789012345678901234",
  "proof_hash": "0x83a8779ddef456789012345678901234567890123456789012345678901234567890",
  "trace": null,
  "signature": null,
  "timestamp": null,
  "report": {
    "status": "ok",
    "manifest_hash": "0xd490be94abc123def456789012345678901234567890123456789012345678901234",
    "proof_hash": "0x83a8779ddef456789012345678901234567890123456789012345678901234567890",
    "signature_valid": false,
    "details": [
      {
        "constraint": "require_at_least_one_supplier",
        "status": "ok",
        "message": "Found 50 suppliers"
      },
      {
        "constraint": "supplier_count_max",
        "status": "ok",
        "message": "Supplier count within limit (50 <= 100)"
      }
    ]
  }
}
```

**Validation:**
- ✅ HTTP 200 OK
- ✅ `result: "ok"`
- ✅ All constraints satisfied
- ✅ manifest_hash, proof_hash present

### Phase 3: Writeback (Adapter → SAP)

#### Command
```bash
cap-adapter writeback \
  --in /tmp/verify.json \
  --odata "$SAP_URL" \
  --table Z_CAP_SUPPLIER_STATUS \
  --idempotency "RUN_1731232800" \
  --user "$SAP_USER" \
  --password-env SAP_PASSWORD \
  --batch-size 10 \
  --audit /var/log/cap/adapter.audit.jsonl
```

#### OData Batch Request (Generated)
```http
POST /sap/opu/odata/sap/Z_CAP_STATUS_SRV/$batch
Content-Type: multipart/mixed; boundary=batch_12345
Authorization: Basic <base64(user:password)>

--batch_12345
Content-Type: application/http
Content-Transfer-Encoding: binary

POST SupplierStatusSet HTTP/1.1
Content-Type: application/json

{
  "RUN_ID": "RUN_1731232800",
  "SUPPLIER_ID": "S0001",
  "MANIFEST_HASH": "0xd490be94abc123...",
  "POLICY_HASH": "0x0afcb402e74ff6...",
  "IR_HASH": "0x1234567890abc...",
  "VERDICT": "ok",
  "VERIFIED_AT": "20251110",
  "VERIFIED_BY": "CAP_ADAPTER"
}

--batch_12345
Content-Type: application/http
Content-Transfer-Encoding: binary

POST SupplierStatusSet HTTP/1.1
Content-Type: application/json

{
  "RUN_ID": "RUN_1731232800",
  "SUPPLIER_ID": "S0002",
  "MANIFEST_HASH": "0xd490be94abc123...",
  "POLICY_HASH": "0x0afcb402e74ff6...",
  "IR_HASH": "0x1234567890abc...",
  "VERDICT": "ok",
  "VERIFIED_AT": "20251110",
  "VERIFIED_BY": "CAP_ADAPTER"
}

--batch_12345--
```

**Batch Processing:**
- Batch size: 10 records per request
- Total batches: 5 (50 records / 10)
- Retry logic: Exponential backoff on 429/503

**Idempotency Guarantee:**
- Primary key: `(RUN_ID, SUPPLIER_ID)`
- Duplicate writes: Silently ignored (SAP UPDATE vs INSERT)
- Audit trail: All writes logged with RUN_ID

#### Output (Console)
```
📥 Reading verification result: /tmp/verify.json
✅ Verification successful: manifest_hash=0xd490be94abc...
📊 Total records to write: 50
🔄 Batch 1/5: Writing 10 records...
✅ Batch 1/5: 10 records written
🔄 Batch 2/5: Writing 10 records...
✅ Batch 2/5: 10 records written
🔄 Batch 3/5: Writing 10 records...
✅ Batch 3/5: 10 records written
🔄 Batch 4/5: Writing 10 records...
✅ Batch 4/5: 10 records written
🔄 Batch 5/5: Writing 10 records...
✅ Batch 5/5: 10 records written
✅ Writeback complete: 50/50 records written
📝 Audit log: /var/log/cap/adapter.audit.jsonl
```

**Validation:**
- ✅ 50/50 records written
- ✅ No duplicates on re-run (same RUN_ID)
- ✅ Audit trail contains manifest_hash, policy_hash, ir_hash

## Idempotency Verification

### First Run
```bash
# Execute full E2E
cap-adapter pull --odata "$SAP_URL" --client "$SAP_CLIENT" --out context.json
curl -s -k -H "Authorization: Bearer $CAP_API_TOKEN" -H "Content-Type: application/json" -d @context.json "$CAP_API_BASE/verify" | tee /tmp/verify.json
cap-adapter writeback --in /tmp/verify.json --odata "$SAP_URL" --table Z_CAP_SUPPLIER_STATUS --idempotency "RUN_1731232800"

# Check SAP table
SELECT COUNT(*) FROM Z_CAP_SUPPLIER_STATUS WHERE RUN_ID = 'RUN_1731232800';
-- Result: 50
```

### Second Run (Same RUN_ID)
```bash
# Re-execute writeback with same RUN_ID
cap-adapter writeback --in /tmp/verify.json --odata "$SAP_URL" --table Z_CAP_SUPPLIER_STATUS --idempotency "RUN_1731232800"

# Check SAP table again
SELECT COUNT(*) FROM Z_CAP_SUPPLIER_STATUS WHERE RUN_ID = 'RUN_1731232800';
-- Result: 50 (not 100!)
```

**Expected Behavior:**
- ✅ No duplicates created
- ✅ SAP returns HTTP 200 (not 409)
- ✅ Adapter logs: "Idempotent write detected, no changes"

## Rate Limiting & Retry Logic

### Rate Limit Handling

**SAP Response (429 Too Many Requests):**
```http
HTTP/1.1 429 Too Many Requests
Retry-After: 5
Content-Type: application/json

{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Too many requests, please retry after 5 seconds"
  }
}
```

**Adapter Behavior:**
1. Detect `429` status code
2. Parse `Retry-After` header (seconds)
3. Sleep for specified duration + jitter (0-1s)
4. Retry request (max 3 attempts)
5. Log retry attempt in audit trail

**Exponential Backoff (on other errors):**
```
Attempt 1: Immediate
Attempt 2: Wait 1s + jitter(0-0.5s)
Attempt 3: Wait 2s + jitter(0-1s)
Attempt 4: Wait 4s + jitter(0-2s)
```

### Error Handling

**Non-Retryable Errors (4xx):**
- 400 Bad Request → Log + Abort
- 401 Unauthorized → Log + Abort
- 403 Forbidden → Log + Abort
- 404 Not Found → Log + Abort

**Retryable Errors (5xx):**
- 500 Internal Server Error → Retry (max 3)
- 503 Service Unavailable → Retry (max 3)
- 504 Gateway Timeout → Retry (max 3)

**Network Errors:**
- Connection timeout → Retry (max 3)
- DNS resolution failure → Retry (max 3)
- TLS handshake failure → Log + Abort

## Audit Trail

### Adapter Audit Log Format (JSONL)

**Location:** `/var/log/cap/adapter.audit.jsonl`

**Schema:**
```jsonl
{"timestamp":"2025-11-10T10:00:00.123456Z","event":"pull_start","run_id":"RUN_1731232800","payload":{"odata_url":"https://sap-s4.example.com:8443/sap/opu/odata/sap/Z_CAP_SUPPLIERS_SRV","client":"100","filter":"TIER le 2"}}
{"timestamp":"2025-11-10T10:00:05.789012Z","event":"pull_complete","run_id":"RUN_1731232800","payload":{"record_count":50,"duration_ms":5665}}
{"timestamp":"2025-11-10T10:00:06.012345Z","event":"verify_request","run_id":"RUN_1731232800","payload":{"api_base":"https://cap-verifier.example.com/api/v1","policy_id":"lksg.v1"}}
{"timestamp":"2025-11-10T10:00:06.567890Z","event":"verify_response","run_id":"RUN_1731232800","payload":{"result":"ok","manifest_hash":"0xd490be94abc123...","proof_hash":"0x83a8779ddef456...","duration_ms":555}}
{"timestamp":"2025-11-10T10:00:07.123456Z","event":"writeback_start","run_id":"RUN_1731232800","payload":{"table":"Z_CAP_SUPPLIER_STATUS","batch_size":10,"total_records":50}}
{"timestamp":"2025-11-10T10:00:08.234567Z","event":"writeback_batch","run_id":"RUN_1731232800","payload":{"batch_num":1,"records_written":10,"duration_ms":1111}}
{"timestamp":"2025-11-10T10:00:09.345678Z","event":"writeback_batch","run_id":"RUN_1731232800","payload":{"batch_num":2,"records_written":10,"duration_ms":1111}}
{"timestamp":"2025-11-10T10:00:10.456789Z","event":"writeback_batch","run_id":"RUN_1731232800","payload":{"batch_num":3,"records_written":10,"duration_ms":1111}}
{"timestamp":"2025-11-10T10:00:11.567890Z","event":"writeback_batch","run_id":"RUN_1731232800","payload":{"batch_num":4,"records_written":10,"duration_ms":1111}}
{"timestamp":"2025-11-10T10:00:12.678901Z","event":"writeback_batch","run_id":"RUN_1731232800","payload":{"batch_num":5,"records_written":10,"duration_ms":1111}}
{"timestamp":"2025-11-10T10:00:12.789012Z","event":"writeback_complete","run_id":"RUN_1731232800","payload":{"total_written":50,"duration_ms":5665}}
```

**Event Types:**
- `pull_start` / `pull_complete` - OData extraction
- `verify_request` / `verify_response` - API verification
- `writeback_start` / `writeback_batch` / `writeback_complete` - SAP writeback
- `retry_attempt` - Retry logic triggered
- `error` - Non-retryable errors

## Acceptance Criteria (DoD)

### Functional Requirements
- ✅ **50/50 Records Written**: All supplier records successfully written to SAP
- ✅ **No Duplicates**: Re-running with same RUN_ID does not create duplicates
- ✅ **Audit Trail Complete**: manifest_hash, policy_hash, ir_hash stored in SAP table
- ✅ **Idempotency Verified**: Second run with same RUN_ID yields identical SAP state
- ✅ **Rate Limiting**: Adapter respects 429 responses and Retry-After header
- ✅ **Retry Logic**: Exponential backoff on transient errors (max 3 attempts)

### Non-Functional Requirements
- ✅ **Latency**: End-to-end runtime < 30s for 50 records
- ✅ **Batch Performance**: Each batch (10 records) completes in < 2s
- ✅ **Error Handling**: All errors logged with RUN_ID for traceability
- ✅ **Audit Completeness**: All events (pull, verify, writeback) logged in JSONL
- ✅ **Security**: No plaintext passwords in logs or audit trail
- ✅ **Monitoring**: Metrics exported for Prometheus/Grafana

## Security Considerations

### Credential Management
- ✅ Use environment variables for passwords (never hardcode)
- ✅ Use OAuth2 JWT tokens for CAP API (short-lived, rotatable)
- ✅ Store SAP credentials in vault (HashiCorp Vault, AWS Secrets Manager)
- ✅ Use mTLS for SAP connections in production

### Data Privacy
- ✅ No PII in hashes (commitments only)
- ✅ No supplier names in audit logs
- ✅ Redact sensitive fields in debug logs
- ✅ GDPR compliance: Retain audit logs for minimum required period

### Network Security
- ✅ TLS 1.2+ for all OData connections
- ✅ Certificate pinning for CAP API
- ✅ Network policies: Adapter → SAP (port 8443), Adapter → CAP API (port 443)
- ✅ No direct internet access from adapter (proxy required)

## Monitoring & Alerting

### Prometheus Metrics

**SAP Adapter Metrics:**
```promql
# Request counters
sap_adapter_pull_requests_total{status="success|failure"}
sap_adapter_verify_requests_total{status="success|failure"}
sap_adapter_writeback_requests_total{status="success|failure"}

# Latency histograms
sap_adapter_pull_duration_seconds
sap_adapter_verify_duration_seconds
sap_adapter_writeback_duration_seconds

# Error counters
sap_adapter_errors_total{type="network|auth|rate_limit|server_error"}

# Retry counters
sap_adapter_retries_total{reason="429|5xx|timeout"}

# Record counters
sap_adapter_records_pulled_total
sap_adapter_records_written_total
```

### Grafana Dashboard

**Panels:**
1. **E2E Success Rate**: `rate(sap_adapter_writeback_requests_total{status="success"}[5m])`
2. **Latency P95**: `histogram_quantile(0.95, sap_adapter_pull_duration_seconds)`
3. **Error Rate**: `rate(sap_adapter_errors_total[5m])`
4. **Retry Rate**: `rate(sap_adapter_retries_total[5m])`
5. **Records Throughput**: `rate(sap_adapter_records_written_total[5m])`

### Alerting Rules

**Critical Alerts:**
```yaml
- alert: SAPAdapterE2EFailureRate
  expr: rate(sap_adapter_writeback_requests_total{status="failure"}[5m]) > 0.1
  for: 5m
  labels:
    severity: critical
  annotations:
    summary: "SAP Adapter E2E failure rate > 10%"

- alert: SAPAdapterLatencyHigh
  expr: histogram_quantile(0.95, sap_adapter_pull_duration_seconds) > 10
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "SAP Adapter pull latency P95 > 10s"

- alert: SAPAdapterRateLimitExceeded
  expr: rate(sap_adapter_retries_total{reason="429"}[5m]) > 5
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "SAP Adapter hitting rate limits"
```

## Troubleshooting

### Common Issues

**Issue 1: 401 Unauthorized on OData**
```
Error: HTTP 401 Unauthorized
Cause: Invalid credentials or expired password
Solution:
1. Check SAP_USER and SAP_PASSWORD environment variables
2. Verify user has S_SERVICE authorization
3. Check password expiration in SAP (SU01)
4. Rotate credentials if expired
```

**Issue 2: 429 Rate Limit Exceeded**
```
Error: HTTP 429 Too Many Requests
Cause: Adapter exceeding SAP rate limits
Solution:
1. Reduce ADAPTER_RATE_LIMIT (default: 10 req/s)
2. Increase batch size (reduce total requests)
3. Contact SAP Basis team to increase quota
```

**Issue 3: Duplicate Records Created**
```
Error: 100 records in Z_CAP_SUPPLIER_STATUS (expected 50)
Cause: RUN_ID not unique between runs
Solution:
1. Ensure RUN_ID includes timestamp or UUID
2. Verify idempotency key is passed correctly
3. Check SAP table constraints (primary key: RUN_ID + SUPPLIER_ID)
```

**Issue 4: Verification Timeout**
```
Error: Timeout waiting for /verify response
Cause: CAP API overloaded or network issue
Solution:
1. Check CAP API health: curl $CAP_API_BASE/healthz
2. Verify network connectivity to CAP API
3. Increase timeout: --timeout 30s
4. Check Prometheus: verify_latency_seconds
```

## Testing Strategy

### Unit Tests (Mocked SAP)
- Test OData query generation
- Test batch request formatting
- Test idempotency key generation
- Test retry logic (exponential backoff)

### Integration Tests (Staging SAP)
- Test full E2E flow with staging data
- Test idempotency (re-run with same RUN_ID)
- Test rate limiting (deliberate 429 trigger)
- Test error handling (invalid credentials)

### Load Tests (Production-like)
- 100 RPS sustained for 5 minutes
- Verify latency P95 < 10s
- Verify no data loss (record count match)
- Verify audit log completeness

## Runbook: SAP Adapter Deployment

### Prerequisites
```bash
# 1. Verify SAP connectivity
curl -u "$SAP_USER:$SAP_PASSWORD" "$SAP_URL/SupplierSet?\$top=1"

# 2. Verify CAP API connectivity
curl -H "Authorization: Bearer $CAP_API_TOKEN" "$CAP_API_BASE/healthz"

# 3. Create audit log directory
sudo mkdir -p /var/log/cap
sudo chown cap-adapter:cap-adapter /var/log/cap
```

### Deployment Steps
```bash
# 1. Install adapter binary
sudo cp cap-adapter /usr/local/bin/
sudo chmod +x /usr/local/bin/cap-adapter

# 2. Create systemd service
sudo cat > /etc/systemd/system/cap-adapter.service <<EOF
[Unit]
Description=CAP SAP Adapter
After=network.target

[Service]
Type=oneshot
User=cap-adapter
EnvironmentFile=/etc/cap/adapter.env
ExecStart=/usr/local/bin/cap-adapter pull --odata \$SAP_URL --client \$SAP_CLIENT --out /tmp/context.json
ExecStart=/usr/bin/curl -s -k -H "Authorization: Bearer \$CAP_API_TOKEN" -H "Content-Type: application/json" -d @/tmp/context.json \$CAP_API_BASE/verify -o /tmp/verify.json
ExecStart=/usr/local/bin/cap-adapter writeback --in /tmp/verify.json --odata \$SAP_URL --table Z_CAP_SUPPLIER_STATUS --idempotency RUN_\$(date +%s)
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

# 3. Create environment file
sudo cat > /etc/cap/adapter.env <<EOF
SAP_URL=https://sap-s4.example.com:8443/sap/opu/odata/sap/Z_CAP_SUPPLIERS_SRV
SAP_CLIENT=100
SAP_USER=CAP_ADAPTER
SAP_PASSWORD=<vault:secret/sap/adapter>
CAP_API_BASE=https://cap-verifier.example.com/api/v1
CAP_API_TOKEN=<vault:secret/cap/api-token>
ADAPTER_RATE_LIMIT=10
ADAPTER_RETRY_MAX=3
EOF

# 4. Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable cap-adapter.service
sudo systemctl start cap-adapter.service

# 5. Verify deployment
sudo systemctl status cap-adapter.service
sudo journalctl -u cap-adapter.service -f
```

### Rollback Procedure
```bash
# 1. Stop service
sudo systemctl stop cap-adapter.service

# 2. Restore previous binary
sudo cp /usr/local/bin/cap-adapter.backup /usr/local/bin/cap-adapter

# 3. Restart service
sudo systemctl start cap-adapter.service

# 4. Verify rollback
sudo systemctl status cap-adapter.service
```

## Future Enhancements

### Planned Features
- ✅ **Real-time Streaming**: Replace batch pull with SAP Event Mesh subscriptions
- ✅ **Delta Extraction**: Incremental pull based on `CHANGED_AT` timestamp
- ✅ **Multi-System Support**: Parallel extraction from multiple SAP systems
- ✅ **Advanced Retry**: Circuit breaker pattern for sustained failures
- ✅ **Webhook Callbacks**: Async writeback via SAP Event Mesh publish

### Performance Optimizations
- ✅ **Connection Pooling**: Reuse OData connections for multiple requests
- ✅ **Parallel Batches**: Write batches in parallel (with semaphore)
- ✅ **Compression**: Enable gzip compression for OData responses
- ✅ **Caching**: Cache policy compilations to reduce API calls

## References

- [SAP OData v2 Specification](https://www.odata.org/documentation/odata-version-2-0/)
- [CAP Verifier API Documentation](./REST_API_v1.md)
- [LkSG Compliance Requirements](https://www.csr-in-deutschland.de/EN/Business-Human-Rights/Supply-Chain-Act/supply-chain-act.html)
- [ABAP Custom Table Creation Guide](https://help.sap.com/docs/ABAP)

## Contact & Support

**Technical Owner:** CAP Verifier Team
**SAP Basis Contact:** sap-basis@example.com
**Incident Escalation:** PagerDuty → #cap-alerts Slack channel

---

**Document Version:** 1.1.0
**Last Updated:** 2025-12-14
**Author:** CAP Engineering Team
**Note:** Dieses Dokument beschreibt geplante Features für Enterprise-Versionen (v1.0+).
