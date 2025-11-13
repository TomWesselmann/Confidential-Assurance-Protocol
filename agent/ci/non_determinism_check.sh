#!/usr/bin/env bash
#
# Non-Determinism Sentinel Check (Week 3)
#
# Compiles the same policy 100 times and verifies all IR hashes are identical.
# If ANY hash differs, this indicates non-determinism and fails the build.
#
# Usage: ./ci/non_determinism_check.sh
# Exit codes: 0 (pass), 1 (non-determinism detected)

set -euo pipefail

echo "🔍 Non-Determinism Sentinel Check (100 iterations)"
echo "=================================================="

POLICY_FILE="examples/lksg_v1.policy.yml"
TMP_DIR=$(mktemp -d)
trap "rm -rf $TMP_DIR" EXIT

if [ ! -f "$POLICY_FILE" ]; then
    echo "❌ Policy file not found: $POLICY_FILE"
    exit 1
fi

echo "📄 Policy: $POLICY_FILE"
echo "🔄 Compiling 100 times..."

# Compile 100 times
for i in $(seq 1 100); do
    cargo run --quiet --bin cap-agent -- policy compile \
        "$POLICY_FILE" \
        -o "$TMP_DIR/ir_${i}.json" \
        > /dev/null 2>&1 || {
        echo "❌ Compilation failed at iteration $i"
        exit 1
    }
done

echo "✅ All 100 compilations succeeded"

# Extract IR hashes
echo "📊 Extracting IR hashes..."

for i in $(seq 1 100); do
    jq -r '.ir_hash' "$TMP_DIR/ir_${i}.json" >> "$TMP_DIR/hashes.txt"
done

# Count unique hashes
UNIQUE_HASHES=$(sort "$TMP_DIR/hashes.txt" | uniq | wc -l | tr -d ' ')

if [ "$UNIQUE_HASHES" -eq 1 ]; then
    HASH=$(head -n 1 "$TMP_DIR/hashes.txt")
    echo "✅ PASS: All 100 IR hashes are identical"
    echo "   Hash: $HASH"
    exit 0
else
    echo "❌ FAIL: Found $UNIQUE_HASHES unique hashes in 100 runs"
    echo "   This indicates NON-DETERMINISTIC compilation!"
    echo ""
    echo "Unique hashes found:"
    sort "$TMP_DIR/hashes.txt" | uniq -c
    exit 1
fi
