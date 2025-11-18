#!/bin/bash
set -e

echo "🎯 Kompiliere PolicyV2 für WebUI..."
echo ""

# Token generieren
echo "🔐 Generiere OAuth2 Token..."
TOKEN=$(cargo run --example generate_mock_token 2>&1 | grep "^eyJ" | head -1)
echo "Token: ${TOKEN:0:50}..."
echo ""

# PolicyV2 YAML laden und base64-kodieren
POLICY_FILE="examples/policy.lksg.v2.yml"
echo "📖 Lade Policy aus $POLICY_FILE..."

if [ ! -f "$POLICY_FILE" ]; then
  echo "❌ Policy-Datei nicht gefunden: $POLICY_FILE"
  exit 1
fi

POLICY_YAML_B64=$(cat "$POLICY_FILE" | base64)
echo "📦 Policy base64-kodiert (${#POLICY_YAML_B64} chars)"
echo ""

# Policy kompilieren mit persist=true
echo "📝 Kompiliere PolicyV2 (persist=true)..."
RESPONSE=$(curl -s -X POST http://localhost:8080/policy/compile \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"policy_yaml\": \"$POLICY_YAML_B64\",
    \"lint_mode\": \"relaxed\",
    \"persist\": true
  }")

echo "$RESPONSE" | jq '.'
echo ""

# Policy ID und Hash extrahieren
POLICY_ID=$(echo "$RESPONSE" | jq -r '.policy_id')
POLICY_HASH=$(echo "$RESPONSE" | jq -r '.policy_hash')
STORED=$(echo "$RESPONSE" | jq -r '.stored')

if [ "$POLICY_ID" = "null" ] || [ -z "$POLICY_ID" ]; then
  echo "❌ Policy Compile fehlgeschlagen!"
  exit 1
fi

if [ "$STORED" != "true" ]; then
  echo "⚠️  Policy wurde nicht gespeichert (stored=$STORED)"
  exit 1
fi

echo "✅ PolicyV2 erfolgreich kompiliert und gespeichert:"
echo "   - Policy ID: $POLICY_ID"
echo "   - Policy Hash: $POLICY_HASH"
echo ""
echo "🔍 Teste Policy-Abruf..."
curl -s -X GET "http://localhost:8080/policy/$POLICY_ID" \
  -H "Authorization: Bearer $TOKEN" | jq '.policy_id, .policy_hash'
echo ""
echo "🌐 WebUI kann jetzt mit policy_id=\"$POLICY_ID\" verifizieren!"
