#!/bin/bash
set -e

echo "📦 Vorbereitung für WebUI Verifikation..."
echo ""

# OAuth2 Token generieren
echo "🔐 Generiere OAuth2 Token..."
TOKEN=$(cargo run --example generate_mock_token 2>&1 | grep "^eyJ" | head -1)
echo "Token: ${TOKEN:0:50}..."
echo ""

# Policy kompilieren und im Policy Store speichern
echo "📝 Kompiliere Policy und speichere im Policy Store..."
POLICY_RESPONSE=$(curl -s -X POST http://localhost:8080/policy/compile \
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
      "notes": "Demo policy for testing Tag 3 proof system"
    }
  }')

echo "$POLICY_RESPONSE" | jq '.'
echo ""

# Policy Hash extrahieren
POLICY_HASH=$(echo "$POLICY_RESPONSE" | jq -r '.policy_hash')
POLICY_ID=$(echo "$POLICY_RESPONSE" | jq -r '.policy_id')

echo "✅ Policy gespeichert:"
echo "   - Policy ID:   $POLICY_ID"
echo "   - Policy Hash: $POLICY_HASH"
echo ""

# Proof Package Info
echo "📂 Proof Package liegt bereit:"
echo "   - Pfad: build/proof_package/"
echo "   - Manifest: build/proof_package/manifest.json"
echo "   - Proof: build/proof_package/proof.dat"
echo ""

echo "🌐 WebUI Zugriff:"
echo "   - URL: http://localhost:5173/"
echo "   - API: http://localhost:8080/"
echo ""

echo "📋 Nächste Schritte:"
echo "1. Öffne http://localhost:5173/ im Browser"
echo "2. Lade das Proof Package (build/proof_package/) hoch"
echo "3. Die WebUI wird automatisch verifizieren"
echo ""
echo "✅ Setup abgeschlossen!"
