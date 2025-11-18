#!/bin/bash
set -e

echo "🧪 Teste Policy Compile Endpoint..."

# Token generieren
TOKEN=$(cargo run --example generate_mock_token 2>&1 | grep "^eyJ" | head -1)
echo "Token: ${TOKEN:0:50}..."
echo ""

# Policy compile testen
echo "📝 Sende Policy Compile Request..."
curl -v -X POST http://localhost:8080/policy/compile \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "policy": {
      "version": "lksg.v1",
      "name": "Test Policy",
      "created_at": "2025-11-18T10:00:00Z",
      "constraints": {
        "require_at_least_one_ubo": true
      },
      "notes": "Test"
    }
  }'

echo ""
echo "✅ Test abgeschlossen!"
