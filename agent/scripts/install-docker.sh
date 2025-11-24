#!/bin/bash
# Docker Desktop Installation (mit sudo)
set -e

echo "🐳 Docker Desktop Installation..."
echo ""
echo "⚠️  Du wirst nach deinem Admin-Passwort gefragt!"
echo ""

# Mit sudo installieren
brew install --cask docker

echo ""
echo "✅ Docker Desktop installiert!"
echo ""
echo "🚀 Nächste Schritte:"
echo "   1. Docker Desktop starten:"
echo "      open -a Docker"
echo ""
echo "   2. Warte bis Docker läuft (grünes Icon in Menüleiste)"
echo ""
echo "   3. Dann dieses Skript ausführen:"
echo "      ./test-docker.sh"
echo ""
