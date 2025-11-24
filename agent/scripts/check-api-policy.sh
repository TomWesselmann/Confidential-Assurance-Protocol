#!/bin/bash
set -e

echo "🔍 Prüfe ob Policy im API-Server Store ist..."
echo ""

TOKEN=$(cargo run --example generate_mock_token 2>&1 | grep "^eyJ" | head -1)
POLICY_HASH="0xd490be94f6f182bd6a00930c65f6f1f5fab70ddb29116235ae344f064f9b52b3"

echo "📡 GET /policy/$POLICY_HASH"
curl -s -X GET "http://localhost:8080/policy/$POLICY_HASH" \
  -H "Authorization: Bearer $TOKEN" \
  | jq '.'

echo ""
echo "✅ Check abgeschlossen!"
