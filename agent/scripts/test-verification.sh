#!/bin/bash
set -e

echo "🧪 Teste Verifikation über API..."
echo ""

# OAuth2 Token generieren
TOKEN=$(cargo run --example generate_mock_token 2>&1 | grep "^eyJ" | head -1)

# Manifest und Proof laden
MANIFEST=$(cat build/proof_package/manifest.json | jq -c '.')
PROOF=$(cat build/proof_package/proof.dat)

# Policy Hash aus Manifest extrahieren
POLICY_HASH=$(echo "$MANIFEST" | jq -r '.policy.hash')

echo "📦 Test-Daten:"
echo "   - Policy Hash: $POLICY_HASH"
echo "   - Manifest: build/proof_package/manifest.json"
echo "   - Proof: build/proof_package/proof.dat"
echo ""

# Verifikations-Request
echo "🔍 Sende Verifikations-Request..."
curl -X POST http://localhost:8080/verify \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"policy_id\": \"$POLICY_HASH\",
    \"context\": {
      \"manifest\": $MANIFEST
    },
    \"backend\": \"mock\",
    \"options\": {}
  }" | jq '.'

echo ""
echo "✅ Test abgeschlossen!"
