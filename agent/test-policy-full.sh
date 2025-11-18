#!/bin/bash
set -e

echo "🧪 Teste Policy Compile mit vollständigen Constraints..."

# Token generieren
TOKEN=$(cargo run --example generate_mock_token 2>&1 | grep "^eyJ" | head -1)
echo "Token: ${TOKEN:0:50}..."
echo ""

# Policy compile mit allen Feldern
echo "📝 Sende Policy Compile Request..."
RESPONSE=$(curl -s -X POST http://localhost:8080/policy/compile \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "policy": {
      "version": "lksg.v1",
      "name": "LkSG Demo Policy",
      "created_at": "2025-10-25T09:00:00Z",
      "constraints": {
        "require_at_least_one_ubo": true,
        "supplier_count_max": 10
      },
      "notes": "Demo policy for testing"
    }
  }')

echo "$RESPONSE" | jq '.'
echo ""

# Policy Hash extrahieren
POLICY_HASH=$(echo "$RESPONSE" | jq -r '.policy_hash')
POLICY_ID=$(echo "$RESPONSE" | jq -r '.policy_id')

if [ "$POLICY_HASH" = "null" ]; then
  echo "❌ Policy Compile fehlgeschlagen!"
  exit 1
fi

echo "✅ Policy erfolgreich gespeichert:"
echo "   - Policy ID: $POLICY_ID"
echo "   - Policy Hash: $POLICY_HASH"
echo ""

# Jetzt Policy abrufen
echo "🔍 Teste Policy Retrieval..."
curl -s -X GET "http://localhost:8080/policy/$POLICY_HASH" \
  -H "Authorization: Bearer $TOKEN" | jq '.'

echo ""
echo "✅ Alle Tests erfolgreich!"
