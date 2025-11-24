#!/bin/bash
set -e

echo "🎯 Kompiliere PolicyV2 für WebUI..."
echo ""

# Token generieren
TOKEN=$(cargo run --example generate_mock_token 2>&1 | grep "^eyJ" | head -1)
echo "Token: ${TOKEN:0:50}..."
echo ""

# Policy Hash aus Manifest
POLICY_HASH="0xd490be94f6f182bd6a00930c65f6f1f5fab70ddb29116235ae344f064f9b52b3"

# PolicyV2 Format erstellen und kompilieren
echo "📝 Kompiliere PolicyV2 mit persist=true..."
RESPONSE=$(curl -s -X POST http://localhost:8080/policy/compile \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"policy\": {
      \"id\": \"$POLICY_HASH\",
      \"version\": \"lksg.v1\",
      \"name\": \"LkSG Demo Policy\",
      \"owner\": \"demo-company\",
      \"created_at\": \"2025-10-25T09:00:00Z\",
      \"rules\": [
        {
          \"id\": \"require_at_least_one_ubo\",
          \"type\": \"constraint\",
          \"description\": \"At least one UBO required\",
          \"logic\": \"company.ubos.length >= 1\"
        },
        {
          \"id\": \"supplier_count_max\",
          \"type\": \"constraint\",
          \"description\": \"Maximum 10 suppliers\",
          \"logic\": \"company.suppliers.length <= 10\"
        }
      ]
    },
    \"lint_mode\": \"relaxed\",
    \"persist\": true
  }")

echo "$RESPONSE" | jq '.'
echo ""

# Policy ID extrahieren
POLICY_ID=$(echo "$RESPONSE" | jq -r '.policy_id')

if [ "$POLICY_ID" = "null" ] || [ -z "$POLICY_ID" ]; then
  echo "❌ Policy Compile fehlgeschlagen!"
  exit 1
fi

echo "✅ PolicyV2 erfolgreich gespeichert:"
echo "   - Policy ID: $POLICY_ID"
echo "   - Policy Hash: $POLICY_HASH"
echo ""
echo "🌐 WebUI kann jetzt verifizieren!"
