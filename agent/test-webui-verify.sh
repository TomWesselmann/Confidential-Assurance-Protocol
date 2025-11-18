#!/bin/bash
set -e

echo "🧪 Teste WebUI Verify Request Format..."
echo ""

# Token generieren
TOKEN=$(cargo run --example generate_mock_token 2>&1 | grep "^eyJ" | head -1)
echo "Token: ${TOKEN:0:50}..."
echo ""

# Manifest laden
MANIFEST=$(cat build/proof_package/manifest.json | jq -c '.')

# Verify Request mit WebUI-Format
echo "📝 Sende Verify Request (WebUI Format)..."
curl -s -X POST http://localhost:8080/verify \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"policy_id\": \"0xd490be94f6f182bd6a00930c65f6f1f5fab70ddb29116235ae344f064f9b52b3\",
    \"context\": {
      \"manifest\": $MANIFEST
    },
    \"backend\": \"mock\",
    \"options\": {
      \"check_timestamp\": false,
      \"check_registry\": false
    }
  }" | jq '.'

echo ""
echo "✅ Test abgeschlossen!"
