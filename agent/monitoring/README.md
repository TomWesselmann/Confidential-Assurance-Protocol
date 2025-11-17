# CAP Verifier API - Monitoring Stack

Vollständiger Production-Ready Monitoring Stack für die CAP Verifier API v0.11.0.

## Stack Components

### Core Services
- **CAP Verifier API** - Die zu überwachende Anwendung
- **Prometheus** - Metrics Collection & Alerting
- **Grafana** - Visualization & Dashboards
- **Loki** - Log Aggregation
- **Promtail** - Log Collection Agent
- **Jaeger** - Distributed Tracing (All-in-One)

### Optional Services
- **Node Exporter** - Host Metrics (CPU, Memory, Disk)
- **cAdvisor** - Container Metrics

## Quick Start

### 1. Voraussetzungen

```bash
# Docker und Docker Compose installieren
docker --version
docker compose version
```

### 2. Stack starten

```bash
cd monitoring
docker compose up -d
```

### 3. Health Checks ausführen

```bash
chmod +x test-monitoring.sh
./test-monitoring.sh
```

### 4. Services öffnen

- **Grafana**: http://localhost:3000 (admin/admin)
  - Main Dashboard: http://localhost:3000/d/cap-verifier-api
  - SLO Dashboard: http://localhost:3000/d/slo-monitoring
- **Prometheus**: http://localhost:9090
- **Jaeger UI**: http://localhost:16686
- **CAP API**: http://localhost:8080

## Verzeichnisstruktur

```
monitoring/
├── docker-compose.yml           # Docker Compose Stack
├── test-monitoring.sh           # Test Script
├── README.md                    # Diese Datei
│
├── prometheus/
│   ├── prometheus.yml           # Prometheus Config
│   └── alerts/
│       └── cap-verifier-rules.yml  # Alerting Rules
│
├── grafana/
│   ├── provisioning/
│   │   ├── dashboards/
│   │   │   └── dashboards.yml   # Dashboard Auto-Provisioning
│   │   └── datasources/
│   │       ├── prometheus.yml   # Prometheus Datasource
│   │       ├── loki.yml         # Loki Datasource
│   │       └── jaeger.yml       # Jaeger Datasource
│   └── dashboards/
│       ├── cap-verifier-api.json   # Main Dashboard
│       └── slo-monitoring.json     # SLO Dashboard
│
├── loki/
│   └── loki-config.yml          # Loki Config
│
├── promtail/
│   └── promtail-config.yml      # Promtail Config
│
├── jaeger/
│   └── jaeger-config.yml        # Jaeger Config
│
└── slo/
    ├── slo-config.yml           # SLO/SLI Definitions
    └── README.md                # SLO Documentation
```

## Dashboards

### 1. Main Dashboard (cap-verifier-api)

**Features:**
- Overview: Total Requests, Request Rate, Error Rate, Cache Hit Ratio
- Request Metrics: Rate by Result, Distribution Pie Chart
- Authentication & Security: Auth Failures, Total Failures
- Cache Performance: Hit Ratio Trends

**Access:** http://localhost:3000/d/cap-verifier-api

### 2. SLO Monitoring Dashboard (slo-monitoring)

**Features:**
- SLO Compliance: Availability (99.9%), Error Rate (< 0.1%), Auth Success (99.95%), Cache Hit Rate (> 70%)
- Error Budget Status: 30-day Rolling Window
- Burn Rate Monitoring: Fast (1h) and Slow (6h) Burn Rates
- SLI Trends: 30-day Historical Data

**Access:** http://localhost:3000/d/slo-monitoring

## Metrics

### Application Metrics (Cap Verifier API)

```promql
# Request Counters
cap_verifier_requests_total{result="ok|warn|fail"}

# Auth Failures
cap_auth_token_validation_failures_total

# Cache Performance
cap_cache_hit_ratio
```

### Prometheus Queries

```bash
# Total Request Rate
sum(rate(cap_verifier_requests_total[5m]))

# Error Rate Percentage
(sum(rate(cap_verifier_requests_total{result="fail"}[5m]))
 /
 sum(rate(cap_verifier_requests_total[5m]))) * 100

# Availability SLI
sum(rate(cap_verifier_requests_total{result="ok"}[5m]))
/
sum(rate(cap_verifier_requests_total[5m]))
```

## Logs

### Loki Query Examples

```logql
# All CAP Verifier Logs
{app="cap-verifier-api"}

# Error Logs Only
{app="cap-verifier-api"} | level="error"

# Auth Failures
{app="cap-verifier-api"} |~ "(?i)auth.*fail"

# Logs with Trace ID
{app="cap-verifier-api"} | json | trace_id!=""
```

### Log Correlation

Logs sind mit Traces korreliert via `trace_id`:
1. Im Loki Dashboard: Klicke auf einen Log-Eintrag
2. "View Trace" Link erscheint automatisch
3. Öffnet Jaeger UI mit dem entsprechenden Trace

## Tracing

### Jaeger UI

**Access:** http://localhost:16686

**Features:**
- Service Dependency Graph (Node Graph)
- Trace Search by Service/Operation/Tags
- Span Details mit Logs/Metrics Correlation

### Trace → Logs Correlation

In Jaeger UI:
1. Öffne einen Trace
2. Klicke auf einen Span
3. "View Logs in Loki" Link erscheint automatisch

## Alerting

### Configured Alerts

**Critical (Pagerduty):**
- `CAPVerifierAPIDown` - API unreachable for 1m
- `CAPVerifierHighErrorRate` - Error rate > 5% for 5m
- `CAPVerifierAuthFailureSpike` - Auth failures > 5/sec for 2m

**Warning (Slack/Email):**
- `CAPVerifierElevatedErrorRate` - Error rate > 1% for 10m
- `CAPVerifierLowCacheHitRatio` - Cache hit < 50% for 15m
- `CAPVerifierAuthFailuresIncreasing` - Auth failures > 1/sec for 10m
- `CAPVerifierNoTrafficDetected` - No traffic for 30m

**Info:**
- `CAPVerifierHighRequestRate` - > 100 req/s (capacity planning)

**SLO-Based:**
- `CAPVerifierSLOErrorBudgetBurning` - Burn rate > threshold

### Viewing Alerts

```bash
# Prometheus Alerts
open http://localhost:9090/alerts

# Check Alert State via API
curl http://localhost:9090/api/v1/alerts | jq
```

## SLO/SLI Monitoring

### Defined SLOs

| SLO | Target | Time Window | Error Budget |
|-----|--------|-------------|--------------|
| Availability | 99.9% | 30 days | 43.2 min/month |
| Error Rate | < 0.1% | 30 days | 0.1% |
| Auth Success | 99.95% | 30 days | 0.05% |
| Cache Hit Rate | > 70% | 7 days | 30% |

### Error Budget Calculation

```promql
# Availability Error Budget Remaining (%)
clamp_min(
  (1 - (1 - (
    sum(rate(cap_verifier_requests_total{result="ok"}[30d]))
    /
    sum(rate(cap_verifier_requests_total[30d]))
  )) / 0.001) * 100,
  0
)
```

### Error Budget Policies

- **< 25% remaining**: Slow rollout (manual approval required)
- **< 5% remaining**: Emergency freeze (all deployments halted)

## Load Testing

### Generate Test Traffic

```bash
# Simple Load Test
for i in {1..1000}; do
  curl -s http://localhost:8080/healthz > /dev/null
  sleep 0.1
done

# Auth Failure Simulation
for i in {1..100}; do
  curl -s -H "Authorization: Bearer invalid-token" \
    http://localhost:8080/verify > /dev/null
  sleep 0.5
done
```

### Expected Results

Nach Load-Test sollten sichtbar sein:
- Request Rate Anstieg in Main Dashboard
- Cache Hit Ratio im SLO Dashboard
- Logs in Loki (mit trace_id)
- Traces in Jaeger UI

## Troubleshooting

### Container startet nicht

```bash
# Logs prüfen
docker compose logs <service-name>

# Beispiel: CAP API Logs
docker compose logs cap-verifier-api
```

### Health Checks fehlgeschlagen

```bash
# Service Status prüfen
docker compose ps

# Einzelnen Service neu starten
docker compose restart <service-name>
```

### Grafana Dashboard fehlt

```bash
# Grafana neu starten (lädt Dashboards)
docker compose restart grafana

# Provisioning Logs prüfen
docker compose logs grafana | grep -i dashboard
```

### Prometheus scraped keine Metriken

```bash
# Targets prüfen
open http://localhost:9090/targets

# Scrape Config testen
curl http://localhost:8080/metrics
```

### Loki empfängt keine Logs

```bash
# Promtail Logs prüfen
docker compose logs promtail

# Loki Ready Check
curl http://localhost:3100/ready

# Docker Socket Access prüfen
docker exec promtail ls -la /var/run/docker.sock
```

## Production Deployment

### Kubernetes Deployment

Für Kubernetes-Deployment siehe:
- `kubernetes/deployment.yml` (CAP API)
- `kubernetes/monitoring/` (Monitoring Stack für K8s)

### Scaling Considerations

1. **Prometheus**:
   - Remote Storage (Cortex/Thanos) für Long-Term Retention
   - Horizontal Sharding für High Cardinality Metrics

2. **Loki**:
   - S3/GCS Backend für Production
   - Compactor für Log Retention Management

3. **Jaeger**:
   - Elasticsearch/Cassandra Backend statt In-Memory
   - Sampling Rate anpassen (100% → 10% in Production)

### Security Hardening

1. **Grafana**:
   - LDAP/OAuth Integration statt Local Users
   - TLS für alle Endpoints
   - Read-Only Dashboards für Viewer Role

2. **Prometheus**:
   - Basic Auth oder OAuth Proxy
   - TLS zwischen Prometheus und Targets

3. **Jaeger**:
   - Token-based Authentication
   - TLS für Collector/Query

## Backup & Recovery

### Grafana Dashboards

```bash
# Export Dashboard
curl -H "Authorization: Bearer <token>" \
  http://localhost:3000/api/dashboards/uid/cap-verifier-api > backup.json

# Import Dashboard
curl -X POST -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d @backup.json \
  http://localhost:3000/api/dashboards/db
```

### Prometheus Data

```bash
# Backup Data (Snapshot)
curl -XPOST http://localhost:9090/api/v1/admin/tsdb/snapshot

# Volume Backup
docker run --rm -v monitoring_prometheus-data:/data \
  -v $(pwd):/backup alpine tar czf /backup/prometheus-backup.tar.gz /data
```

## Maintenance

### Cleanup

```bash
# Stop Stack
docker compose down

# Remove Volumes (deletes all data!)
docker compose down -v

# Remove Images
docker compose down --rmi all
```

### Update Stack

```bash
# Pull neue Images
docker compose pull

# Restart mit neuen Images
docker compose up -d
```

## Support & Resources

- **Project GitHub**: https://github.com/TomWesselmann/Confidential-Assurance-Protocol
- **Prometheus Docs**: https://prometheus.io/docs/
- **Grafana Docs**: https://grafana.com/docs/
- **Loki Docs**: https://grafana.com/docs/loki/
- **Jaeger Docs**: https://www.jaegertracing.io/docs/

## License

© 2025 - Alle Rechte vorbehalten
