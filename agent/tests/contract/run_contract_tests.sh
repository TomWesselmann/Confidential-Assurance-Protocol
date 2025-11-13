#!/usr/bin/env bash
#
# Schemathesis Contract Tests for CAP Verifier API
# Week 4 - OpenAPI Contract Validation
#
# Usage:
#   ./run_contract_tests.sh [BASE_URL]
#
# Example:
#   ./run_contract_tests.sh http://localhost:8080
#

set -e

BASE_URL="${1:-http://localhost:8080}"
OPENAPI_SPEC="../../openapi/openapi.yaml"
REPORTS_DIR="../../reports/contract"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "========================================="
echo "  Schemathesis Contract Tests - Week 4  "
echo "========================================="
echo ""
echo "Base URL: $BASE_URL"
echo "OpenAPI Spec: $OPENAPI_SPEC"
echo ""

# Check if server is running
echo "🔍 Checking if server is reachable..."
if curl -s -f "$BASE_URL/healthz" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Server is reachable${NC}"
else
    echo -e "${RED}❌ Server is not reachable at $BASE_URL${NC}"
    echo ""
    echo "Please start the server first:"
    echo "  cargo run --bin cap-verifier-api"
    exit 1
fi

# Create reports directory
mkdir -p "$REPORTS_DIR"

echo ""
echo "========================================="
echo "  Running Schemathesis Contract Tests   "
echo "========================================="
echo ""

# Run Schemathesis with comprehensive checks
schemathesis run "$OPENAPI_SPEC" \
  --url="$BASE_URL" \
  --checks all \
  --report "$REPORTS_DIR/contract_tests.xml" \
  2>&1 | tee "$REPORTS_DIR/contract_tests.log"

EXIT_CODE=${PIPESTATUS[0]}

echo ""
echo "========================================="
echo "  Test Results                          "
echo "========================================="
echo ""

if [ $EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}✅ All contract tests passed!${NC}"
    echo ""
    echo "Reports saved to:"
    echo "  - JUnit XML: $REPORTS_DIR/contract_tests.xml"
    echo "  - Log: $REPORTS_DIR/contract_tests.log"
else
    echo -e "${RED}❌ Contract tests failed with exit code: $EXIT_CODE${NC}"
    echo ""
    echo "Check the logs for details:"
    echo "  $REPORTS_DIR/contract_tests.log"
fi

exit $EXIT_CODE
