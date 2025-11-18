#!/bin/bash
set -e

echo "🧪 Test PolicyV2 Compile (JSON direct)..."
echo ""

TOKEN=$(cargo run --example generate_mock_token 2>&1 | grep "^eyJ" | head -1)

echo "📝 Sende Policy als direktes JSON-Objekt..."
curl -i -X POST http://localhost:8080/policy/compile \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "policy": {
      "id": "lksg.demo.v1",
      "version": "1.0",
      "legal_basis": [
        {
          "directive": "LkSG",
          "article": "§3 Abs. 1"
        }
      ],
      "description": "LkSG Demo Policy",
      "inputs": {
        "supplier_hashes": {
          "type": "array",
          "items": "string"
        },
        "ubo_hashes": {
          "type": "array",
          "items": "string"
        }
      },
      "rules": [
        {
          "id": "require_at_least_one_ubo",
          "op": ">=",
          "lhs": {"var": "ubo_hashes.length"},
          "rhs": 1
        },
        {
          "id": "supplier_count_max",
          "op": "<=",
          "lhs": {"var": "supplier_hashes.length"},
          "rhs": 10
        }
      ]
    },
    "lint_mode": "relaxed",
    "persist": true
  }'

echo ""
