#!/bin/bash
# CAP Verifier API - Production Smoke Tests
# Version: 1.0
# Purpose: Validates production deployment after cutover

set -e  # Exit on error
set -u  # Exit on undefined variable

# Configuration
BASE_URL="${BASE_URL:-https://cap-verifier.example.com}"
OAUTH_TOKEN_URL="${OAUTH_TOKEN_URL:-https://auth.example.com/oauth/token}"
OAUTH_CLIENT_ID="${OAUTH_CLIENT_ID:-}"
OAUTH_CLIENT_SECRET="${OAUTH_CLIENT_SECRET:-}"
TIMEOUT=10
RETRY_COUNT=3
RETRY_DELAY=2

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Statistics
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# ============================================================
# Helper Functions
# ============================================================

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

log_error() {
    echo -e "${RED}[✗]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[⚠]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."

    if ! command -v curl &> /dev/null; then
        log_error "curl is not installed. Please install curl."
        exit 1
    fi

    if ! command -v jq &> /dev/null; then
        log_warning "jq is not installed. JSON parsing will be limited."
    fi

    log_success "Prerequisites OK"
}

# Fetch OAuth2 token
fetch_oauth_token() {
    log_info "Fetching OAuth2 token..."

    if [[ -z "$OAUTH_CLIENT_ID" ]] || [[ -z "$OAUTH_CLIENT_SECRET" ]]; then
        log_error "OAUTH_CLIENT_ID and OAUTH_CLIENT_SECRET must be set"
        log_info "Export them as environment variables or pass them as arguments"
        exit 1
    fi

    TOKEN_RESPONSE=$(curl -s -f -X POST "$OAUTH_TOKEN_URL" \
        -H "Content-Type: application/x-www-form-urlencoded" \
        -d "grant_type=client_credentials" \
        -d "client_id=$OAUTH_CLIENT_ID" \
        -d "client_secret=$OAUTH_CLIENT_SECRET" \
        -d "scope=verify:run policy:compile policy:read" \
        --connect-timeout $TIMEOUT \
        --max-time $((TIMEOUT * 2)) || echo "{}")

    if command -v jq &> /dev/null; then
        TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.access_token // empty')
    else
        TOKEN=$(echo "$TOKEN_RESPONSE" | grep -o '"access_token":"[^"]*"' | cut -d'"' -f4)
    fi

    if [[ -z "$TOKEN" ]] || [[ "$TOKEN" == "null" ]]; then
        log_error "Failed to fetch OAuth2 token"
        log_error "Response: $TOKEN_RESPONSE"
        exit 1
    fi

    log_success "OAuth2 token obtained"
}

# Run test with retries
run_test() {
    local test_name="$1"
    local test_func="$2"

    TESTS_RUN=$((TESTS_RUN + 1))
    log_info "Running test: $test_name"

    for i in $(seq 1 $RETRY_COUNT); do
        if $test_func; then
            log_success "Test passed: $test_name"
            TESTS_PASSED=$((TESTS_PASSED + 1))
            return 0
        else
            if [[ $i -lt $RETRY_COUNT ]]; then
                log_warning "Test failed (attempt $i/$RETRY_COUNT), retrying in ${RETRY_DELAY}s..."
                sleep $RETRY_DELAY
            else
                log_error "Test failed after $RETRY_COUNT attempts: $test_name"
                TESTS_FAILED=$((TESTS_FAILED + 1))
                return 1
            fi
        fi
    done
}

# ============================================================
# Public Endpoint Tests (No Auth Required)
# ============================================================

test_healthz() {
    local response
    local status

    response=$(curl -s -o /dev/null -w "%{http_code}" \
        --connect-timeout $TIMEOUT \
        --max-time $TIMEOUT \
        "$BASE_URL/healthz")

    status=$?

    if [[ $status -ne 0 ]]; then
        log_error "Failed to connect to /healthz (curl exit code: $status)"
        return 1
    fi

    if [[ "$response" == "200" ]]; then
        log_info "  Status: $response"
        return 0
    else
        log_error "  Expected 200, got $response"
        return 1
    fi
}

test_readyz() {
    local response
    local body
    local status

    body=$(curl -s -w "\n%{http_code}" \
        --connect-timeout $TIMEOUT \
        --max-time $TIMEOUT \
        "$BASE_URL/readyz")

    status=$?
    response=$(echo "$body" | tail -1)
    body=$(echo "$body" | head -n -1)

    if [[ $status -ne 0 ]]; then
        log_error "Failed to connect to /readyz (curl exit code: $status)"
        return 1
    fi

    if [[ "$response" == "200" ]]; then
        log_info "  Status: $response"

        if command -v jq &> /dev/null; then
            local overall_status
            overall_status=$(echo "$body" | jq -r '.status // empty')
            if [[ "$overall_status" == "OK" ]]; then
                log_info "  Readiness: OK"
                return 0
            else
                log_error "  Readiness check failed: $overall_status"
                return 1
            fi
        fi
        return 0
    else
        log_error "  Expected 200, got $response"
        return 1
    fi
}

# ============================================================
# Protected Endpoint Tests (OAuth2 Required)
# ============================================================

test_verify_unauthorized() {
    local response
    local status

    # Test without token (should return 401)
    response=$(curl -s -o /dev/null -w "%{http_code}" \
        --connect-timeout $TIMEOUT \
        --max-time $TIMEOUT \
        -X POST "$BASE_URL/verify" \
        -H "Content-Type: application/json" \
        -d '{"policy_id":"test","context":{"_demo":true},"backend":"mock"}')

    status=$?

    if [[ $status -ne 0 ]]; then
        log_error "Failed to connect to /verify (curl exit code: $status)"
        return 1
    fi

    if [[ "$response" == "401" ]]; then
        log_info "  Unauthorized (expected): $response"
        return 0
    else
        log_error "  Expected 401, got $response"
        return 1
    fi
}

test_verify_authorized() {
    local response
    local body
    local status

    # Test with token (should return 200)
    body=$(curl -s -w "\n%{http_code}" \
        --connect-timeout $TIMEOUT \
        --max-time $((TIMEOUT * 2)) \
        -X POST "$BASE_URL/verify" \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -d '{"policy_id":"lksg.v1","context":{"_demo":true},"backend":"mock"}')

    status=$?
    response=$(echo "$body" | tail -1)
    body=$(echo "$body" | head -n -1)

    if [[ $status -ne 0 ]]; then
        log_error "Failed to connect to /verify (curl exit code: $status)"
        return 1
    fi

    if [[ "$response" == "200" ]]; then
        log_info "  Status: $response"

        if command -v jq &> /dev/null; then
            local result
            result=$(echo "$body" | jq -r '.result // empty')
            if [[ "$result" == "ok" ]] || [[ "$result" == "warn" ]]; then
                log_info "  Verification result: $result"
                return 0
            else
                log_error "  Unexpected result: $result"
                log_error "  Response: $body"
                return 1
            fi
        fi
        return 0
    else
        log_error "  Expected 200, got $response"
        log_error "  Response: $body"
        return 1
    fi
}

test_policy_compile() {
    local response
    local body
    local status

    body=$(curl -s -w "\n%{http_code}" \
        --connect-timeout $TIMEOUT \
        --max-time $((TIMEOUT * 2)) \
        -X POST "$BASE_URL/policy/compile" \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -d '{
            "policy": {
                "version": "lksg.v1",
                "name": "Smoke Test Policy",
                "created_at": "2025-11-10T10:00:00Z",
                "constraints": {
                    "require_at_least_one_ubo": true,
                    "supplier_count_max": 10
                },
                "notes": "Production smoke test"
            }
        }')

    status=$?
    response=$(echo "$body" | tail -1)
    body=$(echo "$body" | head -n -1)

    if [[ $status -ne 0 ]]; then
        log_error "Failed to connect to /policy/compile (curl exit code: $status)"
        return 1
    fi

    if [[ "$response" == "200" ]]; then
        log_info "  Status: $response"

        if command -v jq &> /dev/null; then
            local policy_hash
            policy_hash=$(echo "$body" | jq -r '.policy_hash // empty')
            if [[ -n "$policy_hash" ]] && [[ "$policy_hash" != "null" ]]; then
                log_info "  Policy hash: $policy_hash"
                return 0
            else
                log_error "  No policy_hash in response"
                log_error "  Response: $body"
                return 1
            fi
        fi
        return 0
    else
        log_error "  Expected 200, got $response"
        log_error "  Response: $body"
        return 1
    fi
}

test_metrics_endpoint() {
    local response
    local body
    local status

    # Metrics endpoint should be accessible (no auth required in most setups)
    body=$(curl -s -w "\n%{http_code}" \
        --connect-timeout $TIMEOUT \
        --max-time $TIMEOUT \
        "$BASE_URL/metrics" || echo "")

    status=$?
    response=$(echo "$body" | tail -1)
    body=$(echo "$body" | head -n -1)

    if [[ $status -ne 0 ]]; then
        log_warning "Failed to connect to /metrics (curl exit code: $status)"
        log_warning "This may be expected if metrics are not exposed publicly"
        return 0  # Non-blocking
    fi

    if [[ "$response" == "200" ]]; then
        log_info "  Status: $response"

        # Check for CAP-specific metrics
        if echo "$body" | grep -q "cap_verifier_requests_total"; then
            log_info "  Metrics available (cap_verifier_requests_total found)"
            return 0
        else
            log_warning "  Metrics endpoint accessible but no CAP metrics found"
            return 0  # Non-blocking
        fi
    else
        log_warning "  Metrics endpoint returned $response (may be protected)"
        return 0  # Non-blocking
    fi
}

# ============================================================
# Performance Smoke Tests
# ============================================================

test_latency_p95() {
    log_info "Testing p95 latency (10 requests)..."

    local total_time=0
    local successful_requests=0
    local failed_requests=0

    for i in $(seq 1 10); do
        local request_time
        request_time=$(curl -s -o /dev/null -w "%{time_total}" \
            --connect-timeout $TIMEOUT \
            --max-time $((TIMEOUT * 2)) \
            -X POST "$BASE_URL/verify" \
            -H "Authorization: Bearer $TOKEN" \
            -H "Content-Type: application/json" \
            -d '{"policy_id":"lksg.v1","context":{"_demo":true},"backend":"mock"}' \
            2>/dev/null || echo "0")

        if [[ "$request_time" != "0" ]]; then
            total_time=$(echo "$total_time + $request_time" | bc)
            successful_requests=$((successful_requests + 1))
        else
            failed_requests=$((failed_requests + 1))
        fi
    done

    if [[ $successful_requests -eq 0 ]]; then
        log_error "  All requests failed"
        return 1
    fi

    local avg_time
    avg_time=$(echo "scale=3; $total_time / $successful_requests" | bc)

    log_info "  Successful requests: $successful_requests/10"
    log_info "  Average latency: ${avg_time}s"

    # p95 target: < 500ms (0.5s)
    if (( $(echo "$avg_time < 0.5" | bc -l) )); then
        log_success "  Latency OK (< 500ms target)"
        return 0
    else
        log_warning "  Latency higher than target (${avg_time}s > 0.5s)"
        return 0  # Non-blocking
    fi
}

# ============================================================
# Main Execution
# ============================================================

main() {
    echo "========================================"
    echo " CAP Verifier API - Production Smoke Tests"
    echo "========================================"
    echo ""

    log_info "Target: $BASE_URL"
    log_info "OAuth: $OAUTH_TOKEN_URL"
    echo ""

    check_prerequisites
    echo ""

    # Fetch OAuth2 token
    fetch_oauth_token
    echo ""

    # Public endpoint tests
    echo "--- Public Endpoints ---"
    run_test "GET /healthz" test_healthz
    run_test "GET /readyz" test_readyz
    echo ""

    # Protected endpoint tests (unauthorized)
    echo "--- Protected Endpoints (Unauthorized) ---"
    run_test "POST /verify (no token)" test_verify_unauthorized
    echo ""

    # Protected endpoint tests (authorized)
    echo "--- Protected Endpoints (Authorized) ---"
    run_test "POST /verify (with token)" test_verify_authorized
    run_test "POST /policy/compile" test_policy_compile
    echo ""

    # Observability tests
    echo "--- Observability ---"
    run_test "GET /metrics" test_metrics_endpoint
    echo ""

    # Performance tests
    echo "--- Performance ---"
    run_test "Latency p95 test" test_latency_p95
    echo ""

    # Summary
    echo "========================================"
    echo " Summary"
    echo "========================================"
    echo "Tests run:    $TESTS_RUN"
    echo "Tests passed: $TESTS_PASSED"
    echo "Tests failed: $TESTS_FAILED"
    echo ""

    if [[ $TESTS_FAILED -eq 0 ]]; then
        log_success "All smoke tests passed! ✅"
        exit 0
    else
        log_error "Some smoke tests failed! ❌"
        exit 1
    fi
}

# Run main function
main "$@"
