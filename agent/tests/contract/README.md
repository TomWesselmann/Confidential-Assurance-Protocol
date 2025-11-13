# Contract Tests - Week 4

## Overview

Contract tests validate that the CAP Verifier API conforms to its OpenAPI specification. They use [Schemathesis](https://schemathesis.readthedocs.io/) to automatically generate test cases and verify:

- **Schema Compliance:** All responses match the OpenAPI schema
- **Status Codes:** Correct HTTP status codes (200/304/400/401/403/409/422/500)
- **Content Types:** Correct `Content-Type` headers
- **Response Structure:** All required fields present
- **Data Formats:** Correct data types, patterns, and constraints

## Prerequisites

### 1. Python 3 and Schemathesis

**macOS/Linux:**
```bash
pip3 install schemathesis
```

**Verify Installation:**
```bash
schemathesis --version
# Output: schemathesis, version 4.4.4
```

### 2. Running API Server

The contract tests require a running API server:

```bash
# Terminal 1: Start API server
cargo run --bin cap-verifier-api
```

## Running Contract Tests

### Quick Start

```bash
# From tests/contract directory
./run_contract_tests.sh

# Or specify custom base URL
./run_contract_tests.sh http://localhost:8080
```

### Manual Run

```bash
schemathesis run ../../openapi/openapi.yaml \
  --url=http://localhost:8080 \
  --checks all
```

## Test Configuration

| Parameter | Value | Description |
|-----------|-------|-------------|
| **OpenAPI Spec** | `openapi/openapi.yaml` | API specification |
| **Base URL** | `http://localhost:8080` | Local dev server |
| **Checks** | all | All Schemathesis checks enabled |

## Validated Checks

Schemathesis performs the following checks:

### 1. Schema Validation (`not_a_server_error`)
- No 5xx responses for valid requests
- Server errors only for actual server failures

### 2. Status Code Validation (`status_code_conformance`)
- All status codes match OpenAPI spec
- Correct codes: 200, 304, 400, 401, 403, 409, 422, 500

### 3. Content Type Check (`content_type_conformance`)
- All responses have `Content-Type: application/json`
- Matches OpenAPI schema declarations

### 4. Response Schema (`response_schema_conformance`)
- Response body matches OpenAPI schema
- All required fields present
- Correct data types and patterns

### 5. Header Validation (`response_headers_conformance`)
- Required headers present (e.g., `ETag` for policy endpoints)
- Header values match patterns

## Expected Outcomes

### Success Criteria

```
✅ All contract tests passed!

Hypothesis calls: 150
- Passed: 150
- Failed: 0
- Errors: 0
```

### Common Failures

| Failure | Cause | Fix |
|---------|-------|-----|
| **401 Unauthorized** | Missing/invalid OAuth2 token | Implement auth bypass for tests or provide mock token |
| **Schema Mismatch** | Response doesn't match OpenAPI | Update implementation or OpenAPI spec |
| **Missing Field** | Required field missing in response | Add field to response struct |
| **Wrong Status Code** | Handler returns incorrect code | Fix status code in handler |

## Reports

Contract test results are saved to `reports/contract/`:

```
reports/contract/
├── contract_tests.xml    # JUnit XML (CI integration)
└── contract_tests.log    # Detailed test log
```

## CI Integration

### GitHub Actions Example

```yaml
- name: Install Schemathesis
  run: pip3 install schemathesis

- name: Start API Server
  run: |
    cargo run --bin cap-verifier-api &
    sleep 5  # Wait for server startup

- name: Run Contract Tests
  run: |
    cd tests/contract
    ./run_contract_tests.sh http://localhost:8080

- name: Upload Test Results
  if: always()
  uses: actions/upload-artifact@v4
  with:
    name: contract-test-results
    path: reports/contract/
```

## Advanced Usage

### Custom Checks

```bash
schemathesis run openapi/openapi.yaml \
  --url=http://localhost:8080 \
  --checks not_a_server_error,status_code_conformance
```

### Stateful Testing

```bash
schemathesis run openapi/openapi.yaml \
  --url=http://localhost:8080 \
  --phases stateful
```

### Targeted Endpoint Testing

```bash
# Test only /policy endpoints
schemathesis run openapi/openapi.yaml \
  --url=http://localhost:8080 \
  --include-path-regex='^/policy/'

# Test only /verify endpoint
schemathesis run openapi/openapi.yaml \
  --url=http://localhost:8080 \
  --include-path='/verify'
```

## Troubleshooting

### Server Not Reachable

**Error:**
```
❌ Server is not reachable at http://localhost:8080
```

**Solution:**
```bash
# Check if server is running
curl http://localhost:8080/healthz

# Start server if not running
cargo run --bin cap-verifier-api
```

### Authentication Failures

**Error:**
```
E       status_code_conformance: received 401 (expected: 200)
```

**Temporary Solution (Week 4):**
- Use mock token generation for tests
- Or temporarily disable auth for contract testing

**Production Solution (Future):**
- Implement test OAuth2 token provider
- Use Schemathesis auth hooks

### Schema Validation Failures

**Error:**
```
E       response_schema_conformance: 'ir_hash' is a required property
```

**Solution:**
1. Check OpenAPI spec matches implementation
2. Verify all required fields are present in response
3. Update either spec or implementation to match

## OpenAPI Specification

The contract tests validate against:
- **File:** `openapi/openapi.yaml`
- **Version:** OpenAPI 3.0.3
- **API Version:** 0.11.0

**Endpoints Tested:**
- `GET /healthz` (public)
- `GET /readyz` (public)
- `POST /policy/compile` (protected)
- `GET /policy/{id}` (protected)
- `POST /policy/v2/compile` (protected, Week 3)
- `GET /policy/v2/{id}` (protected, Week 3)
- `POST /verify` (protected)

## Week 4 Definition of Done

| Requirement | Status |
|-------------|--------|
| **Schema Compliance** | ✅ All responses match OpenAPI |
| **Status Codes** | ✅ 200/304/400/401/403/409/422/500 |
| **No 5xx for Valid Requests** | ✅ Server errors only for actual failures |
| **Content Types** | ✅ `application/json` for all endpoints |
| **Required Fields** | ✅ All required fields present |
| **CI Integration** | ✅ JUnit XML reports generated |

## References

- [Schemathesis Documentation](https://schemathesis.readthedocs.io/)
- [OpenAPI 3.0 Specification](https://swagger.io/specification/)
- [Week 4 Execution Guide](/Users/tomwesselmann/Desktop/Week4_Execution.md)
- [OpenAPI Spec](/Users/tomwesselmann/Desktop/LsKG-Agent/agent/openapi/openapi.yaml)
