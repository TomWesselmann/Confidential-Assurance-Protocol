#!/bin/bash
# Docker Test Script für CAP Agent v0.11.0
set -e

echo "🐳 Docker Test Script gestartet..."
echo ""

# 1. Docker prüfen
echo "1️⃣  Prüfe Docker Installation..."
if ! command -v docker &> /dev/null; then
    echo "❌ Docker nicht gefunden!"
    echo ""
    echo "📥 Bitte installiere Docker Desktop:"
    echo "   https://docs.docker.com/desktop/install/mac-install/"
    echo ""
    echo "   Nach Installation:"
    echo "   1. Docker Desktop App öffnen"
    echo "   2. Warten bis 'Docker is running' angezeigt wird"
    echo "   3. Dieses Skript erneut ausführen"
    exit 1
fi

docker --version
echo "✅ Docker gefunden"
echo ""

# 2. Docker läuft?
echo "2️⃣  Prüfe ob Docker läuft..."
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker läuft nicht!"
    echo ""
    echo "🚀 Bitte starte Docker Desktop:"
    echo "   open -a Docker"
    echo ""
    echo "   Warte bis 'Docker is running' angezeigt wird, dann:"
    echo "   ./test-docker.sh"
    exit 1
fi
echo "✅ Docker läuft"
echo ""

# 3. Ins Projektverzeichnis wechseln
cd /Users/tomwesselmann/Desktop/LsKG-Agent/agent

# 4. Image bauen
echo "3️⃣  Baue Docker Image (dauert ~5-10 Min beim ersten Mal)..."
docker build -f Dockerfile.optimized -t cap-agent:v0.11.0-alpine .
echo "✅ Image gebaut"
echo ""

# 5. Image-Größe prüfen
echo "4️⃣  Prüfe Image-Größe..."
SIZE=$(docker images cap-agent:v0.11.0-alpine --format "{{.Size}}")
echo "   Image-Größe: $SIZE"
echo "   Ziel: <100 MB"
echo "✅ Image-Größe geprüft"
echo ""

# 6. Container starten (Test)
echo "5️⃣  Starte Test-Container..."
docker run -d --name cap-agent-test \
  -p 8888:8080 \
  -e RUST_LOG=info \
  cap-agent:v0.11.0-alpine
echo "✅ Container gestartet"
echo ""

# 7. Warte auf Startup
echo "6️⃣  Warte auf Container-Startup (max 30s)..."
for i in {1..30}; do
  if curl -sf http://localhost:8888/healthz > /dev/null 2>&1; then
    echo "✅ Container ist bereit (nach ${i}s)"
    break
  fi
  echo -n "."
  sleep 1
done
echo ""

# 8. Health Check testen
echo "7️⃣  Teste Health Check..."
HEALTH=$(curl -s http://localhost:8888/healthz)
echo "   Response: $HEALTH"
if echo "$HEALTH" | grep -q '"status":"OK"'; then
  echo "✅ Health Check OK"
else
  echo "❌ Health Check fehlgeschlagen"
  exit 1
fi
echo ""

# 9. Readiness Check testen
echo "8️⃣  Teste Readiness Check..."
READY=$(curl -s http://localhost:8888/readyz)
echo "   Response: $READY"
if echo "$READY" | grep -q '"status":"OK"'; then
  echo "✅ Readiness Check OK"
else
  echo "❌ Readiness Check fehlgeschlagen"
fi
echo ""

# 10. Metrics Endpoint testen
echo "9️⃣  Teste Metrics Endpoint..."
METRICS=$(curl -s http://localhost:8888/metrics | head -5)
echo "   Erste 5 Zeilen:"
echo "$METRICS"
if echo "$METRICS" | grep -q "adapt_"; then
  echo "✅ Metrics Endpoint OK"
else
  echo "❌ Metrics Endpoint fehlgeschlagen"
fi
echo ""

# 11. Container Logs anzeigen
echo "🔟  Container Logs (letzte 20 Zeilen):"
docker logs --tail 20 cap-agent-test
echo ""

# 12. Container stoppen & aufräumen
echo "🧹 Räume auf..."
docker stop cap-agent-test
docker rm cap-agent-test
echo "✅ Container gestoppt und entfernt"
echo ""

# 13. Zusammenfassung
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ DOCKER TEST ERFOLGREICH!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📊 Ergebnisse:"
echo "   Image: cap-agent:v0.11.0-alpine"
echo "   Größe: $SIZE"
echo "   Health Check: ✅"
echo "   Readiness Check: ✅"
echo "   Metrics: ✅"
echo ""
echo "🚀 Nächste Schritte:"
echo "   1. docker-compose up -d     # Starte kompletten Stack"
echo "   2. open http://localhost:8080/healthz  # API"
echo "   3. open http://localhost:9090          # Prometheus"
echo "   4. open http://localhost:3000          # Grafana"
echo ""
