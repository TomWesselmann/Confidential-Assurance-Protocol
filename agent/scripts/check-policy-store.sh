#!/bin/bash
set -e

echo "🔍 Prüfe ob Policy im Store ist..."

TOKEN=$(cargo run --example generate_mock_token 2>&1 | grep "^eyJ" | head -1)
POLICY_HASH="0xd490be94f6f182bd6a00930c65f6f1f5fab70ddb29116235ae344f064f9b52b3"

RESPONSE=$(curl -s -X GET "http://localhost:8080/policy/$POLICY_HASH" \
  -H "Authorization: Bearer $TOKEN")

POLICY_NAME=$(echo "$RESPONSE" | jq -r '.policy.name')

if [ "$POLICY_NAME" = "null" ]; then
  echo "❌ Policy nicht gefunden im Store!"
  echo "$RESPONSE" | jq '.'
  exit 1
else
  echo "✅ Policy gefunden: $POLICY_NAME"
  echo ""
  echo "Policy Details:"
  echo "$RESPONSE" | jq '.'
fi
