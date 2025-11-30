#!/bin/bash

# test-monitoring.sh - Test Script für CAP Verifier Monitoring Stack
# Startet den vollständigen Monitoring Stack und führt Health Checks durch

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Farben für Output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo "🚀 CAP Verifier Monitoring Stack - Test Script"
echo "================================================"
echo ""

# Check Docker
if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ Docker ist nicht installiert${NC}"
    exit 1
fi

# Check Docker Compose
if ! docker compose version &> /dev/null; then
    echo -e "${RED}❌ Docker Compose ist nicht installiert${NC}"
    exit 1
fi

echo "✅ Docker und Docker Compose verfügbar"
echo ""

# Cleanup alte Container
echo "🧹 Cleanup alte Container..."
docker compose down -v 2>/dev/null || true
echo ""

# Start Monitoring Stack
echo "🚀 Starte Monitoring Stack..."
docker compose up -d

echo ""
echo "⏳ Warte auf Container-Start (30 Sekunden)..."
sleep 30

echo ""
echo "📊 Container Status:"
docker compose ps

echo ""
echo "🔍 Health Checks:"
echo "================================================"

# Function für Health Check
check_health() {
    local service=$1
    local url=$2
    local name=$3

    echo -n "Prüfe $name... "

    if curl -f -s "$url" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ OK${NC}"
        return 0
    else
        echo -e "${RED}❌ FAIL${NC}"
        return 1
    fi
}

# Health Checks
FAILED=0

check_health "cap-verifier-api" "http://localhost:8080/healthz" "CAP Verifier API" || FAILED=$((FAILED+1))
check_health "prometheus" "http://localhost:9090/-/healthy" "Prometheus" || FAILED=$((FAILED+1))
check_health "grafana" "http://localhost:3000/api/health" "Grafana" || FAILED=$((FAILED+1))
check_health "loki" "http://localhost:3100/ready" "Loki" || FAILED=$((FAILED+1))
check_health "jaeger" "http://localhost:14269/" "Jaeger" || FAILED=$((FAILED+1))

echo ""
echo "================================================"

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ Alle Health Checks erfolgreich!${NC}"
    echo ""
    echo "📡 Service URLs:"
    echo "  - CAP Verifier API: http://localhost:8080"
    echo "  - Prometheus:       http://localhost:9090"
    echo "  - Grafana:          http://localhost:3000 (admin/admin)"
    echo "  - Loki:             http://localhost:3100"
    echo "  - Jaeger UI:        http://localhost:16686"
    echo ""
    echo "📊 Grafana Dashboards:"
    echo "  - Main Dashboard:   http://localhost:3000/d/cap-verifier-api"
    echo "  - SLO Dashboard:    http://localhost:3000/d/slo-monitoring"
    echo ""
    echo "🧪 Test Requests senden:"
    echo "  curl http://localhost:8080/healthz"
    echo "  curl http://localhost:8080/readyz"
    echo ""
    echo "🛑 Stack stoppen:"
    echo "  docker compose down"
    echo ""
    exit 0
else
    echo -e "${RED}❌ $FAILED Health Checks fehlgeschlagen${NC}"
    echo ""
    echo "🔍 Logs prüfen:"
    echo "  docker compose logs cap-verifier-api"
    echo "  docker compose logs prometheus"
    echo "  docker compose logs grafana"
    echo ""
    exit 1
fi
