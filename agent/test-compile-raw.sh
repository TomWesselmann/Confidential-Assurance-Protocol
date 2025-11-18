#!/bin/bash
set -e

echo "🧪 Test PolicyV2 Compile (raw output)..."
echo ""

TOKEN=$(cargo run --example generate_mock_token 2>&1 | grep "^eyJ" | head -1)
POLICY_YAML_B64=$(cat examples/policy.lksg.v2.yml | base64)

echo "📝 Request:"
echo "{\"policy_yaml\":\"${POLICY_YAML_B64:0:50}...\",\"lint_mode\":\"relaxed\",\"persist\":true}"
echo ""

echo "📡 Response:"
curl -i -X POST http://localhost:8080/policy/compile \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"policy_yaml\": \"$POLICY_YAML_B64\",
    \"lint_mode\": \"relaxed\",
    \"persist\": true
  }"

echo ""
