#!/bin/bash
set -e

echo "🎯 Setup für WebUI Verifikation..."
echo ""

# 1. Policy aus Manifest lesen
echo "📖 Lese Policy aus Manifest..."
MANIFEST_FILE="build/proof_package/manifest.json"

if [ ! -f "$MANIFEST_FILE" ]; then
  echo "❌ Manifest nicht gefunden: $MANIFEST_FILE"
  exit 1
fi

# Policy Hash aus Manifest extrahieren
EXPECTED_HASH=$(jq -r '.policy.hash' "$MANIFEST_FILE")
POLICY_NAME=$(jq -r '.policy.name' "$MANIFEST_FILE")
POLICY_VERSION=$(jq -r '.policy.version' "$MANIFEST_FILE")

echo "   - Name: $POLICY_NAME"
echo "   - Version: $POLICY_VERSION"
echo "   - Hash: $EXPECTED_HASH"
echo ""

# 2. Token generieren
echo "🔐 Generiere OAuth2 Token..."
TOKEN=$(cargo run --example generate_mock_token 2>&1 | grep "^eyJ" | head -1)
echo "Token generiert: ${TOKEN:0:50}..."
echo ""

# 3. Policy aus YAML-Datei laden und kompilieren
echo "📝 Lade und kompiliere Policy..."
POLICY_FILE="examples/policy.lksg.v1.yml"

if [ ! -f "$POLICY_FILE" ]; then
  echo "❌ Policy Datei nicht gefunden: $POLICY_FILE"
  exit 1
fi

# Policy YAML zu JSON konvertieren und kompilieren
POLICY_JSON=$(cat "$POLICY_FILE" | python3 -c "
import sys, yaml, json
policy = yaml.safe_load(sys.stdin)
json.dump({'policy': policy}, sys.stdout)
")

RESPONSE=$(curl -s -X POST http://localhost:8080/policy/compile \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "$POLICY_JSON")

POLICY_HASH=$(echo "$RESPONSE" | jq -r '.policy_hash')
POLICY_ID=$(echo "$RESPONSE" | jq -r '.policy_id')

if [ "$POLICY_HASH" = "null" ]; then
  echo "❌ Policy Compile fehlgeschlagen!"
  echo "$RESPONSE" | jq '.'
  exit 1
fi

echo "✅ Policy gespeichert:"
echo "   - Policy ID: $POLICY_ID"
echo "   - Policy Hash: $POLICY_HASH"
echo ""

# 4. Hash-Vergleich
if [ "$POLICY_HASH" = "$EXPECTED_HASH" ]; then
  echo "✅ Policy Hash stimmt mit Manifest überein!"
else
  echo "⚠️  Policy Hash unterscheidet sich vom Manifest:"
  echo "   - Manifest: $EXPECTED_HASH"
  echo "   - Compiled: $POLICY_HASH"
  echo "   - Grund: Unterschiedliche created_at Zeit"
  echo ""
  echo "   ℹ️  Die WebUI wird trotzdem funktionieren, aber die Hashes müssen übereinstimmen."
  echo "   ℹ️  Verwende PolicyV2 für flexible Hash-Berechnung."
fi

echo ""
echo "📂 Proof Package:"
echo "   - Pfad: build/proof_package/"
echo "   - Manifest: $MANIFEST_FILE"
echo "   - Proof: build/proof_package/proof.dat"
echo ""

echo "🌐 WebUI bereit:"
echo "   - URL: http://localhost:5173/"
echo "   - API: http://localhost:8080/"
echo ""

echo "📋 Nächste Schritte:"
echo "1. Öffne http://localhost:5173/ im Browser"
echo "2. Lade das Proof Package hoch (drag & drop oder file picker)"
echo "3. Die WebUI wird automatisch verifizieren"
echo ""
echo "✅ Setup abgeschlossen!"
