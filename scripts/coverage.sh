#!/bin/bash
# Coverage Script for CAP Agent
# Usage: ./scripts/coverage.sh [quick|full]

set -e

MODE="${1:-quick}"
AGENT_DIR="$(dirname "$0")/../agent"
COVERAGE_DIR="$(dirname "$0")/../coverage"

mkdir -p "$COVERAGE_DIR"

cd "$AGENT_DIR"

echo "=== CAP Agent Coverage Report ==="
echo "Mode: $MODE"
echo ""

if [ "$MODE" = "quick" ]; then
    # Quick mode: Only run unit tests, skip integration tests
    echo "Running quick coverage (unit tests only)..."
    cargo tarpaulin \
        --out Html --out Lcov \
        --output-dir "$COVERAGE_DIR" \
        --ignore-tests \
        --timeout 120 \
        --skip-clean \
        --lib \
        2>&1 | tee "$COVERAGE_DIR/tarpaulin.log"
elif [ "$MODE" = "full" ]; then
    # Full mode: Run all tests (takes 30+ minutes)
    echo "Running full coverage (all tests)..."
    echo "WARNING: This will take 30+ minutes!"
    cargo tarpaulin \
        --out Html --out Lcov --out Json \
        --output-dir "$COVERAGE_DIR" \
        --ignore-tests \
        --timeout 600 \
        --skip-clean \
        2>&1 | tee "$COVERAGE_DIR/tarpaulin.log"
else
    echo "Usage: $0 [quick|full]"
    exit 1
fi

echo ""
echo "=== Coverage Report Generated ==="
echo "HTML Report: $COVERAGE_DIR/tarpaulin-report.html"
echo "LCOV Report: $COVERAGE_DIR/lcov.info"
echo ""

# Extract coverage percentage from log
if grep -q "Coverage:" "$COVERAGE_DIR/tarpaulin.log"; then
    grep "Coverage:" "$COVERAGE_DIR/tarpaulin.log" | tail -1
fi
