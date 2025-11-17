# Docker Deployment Guide - CAP Agent v0.11.0

## 📦 Was wurde erstellt?

### Dateien:
- ✅ `Dockerfile.optimized` - Production-optimiertes Multi-Stage Dockerfile (Alpine-based)
- ✅ `.dockerignore` - Optimierter Build-Context
- ✅ `docker-compose.yml` - Lokales Testing-Setup (API + Prometheus + Grafana)
- ✅ `monitoring/prometheus.yml` - Prometheus Scrape-Konfiguration
- ✅ `monitoring/grafana/datasources/prometheus.yml` - Grafana Datasource
- ✅ `monitoring/grafana/dashboards/dashboard.yml` - Dashboard Provisioning

---

## 🚀 Quick Start (Lokales Testing)

### Voraussetzungen:
```bash
# Docker Desktop für Mac installieren
# https://docs.docker.com/desktop/install/mac-install/

# Docker & Docker Compose prüfen
docker --version
docker-compose --version
```

### 1. Docker Image bauen
```bash
cd /Users/tomwesselmann/Desktop/LsKG-Agent/agent

# Optimiertes Alpine-Image bauen
docker build -f Dockerfile.optimized -t cap-agent:v0.11.0-alpine .

# Image-Größe prüfen (Ziel: <100 MB)
docker images cap-agent:v0.11.0-alpine
```

### 2. Stack starten (API + Prometheus + Grafana)
```bash
# Alle Services starten
docker-compose up -d

# Logs anschauen
docker-compose logs -f api

# Status prüfen
docker-compose ps
```

### 3. Services testen

#### CAP Verifier API
```bash
# Health Check
curl http://localhost:8080/healthz

# Readiness Check
curl http://localhost:8080/readyz

# Metrics Endpoint
curl http://localhost:8080/metrics
```

#### Prometheus
- URL: http://localhost:9090
- Targets: http://localhost:9090/targets
- Queries: `up`, `adapt_requests_total`, etc.

#### Grafana
- URL: http://localhost:3000
- Login: `admin` / `admin`
- Datasource: Prometheus (auto-configured)

---

## 🔧 Entwicklung

### Rebuild nach Code-Änderungen
```bash
# Services stoppen
docker-compose down

# Image neu bauen
docker build -f Dockerfile.optimized -t cap-agent:v0.11.0-alpine .

# Services neu starten
docker-compose up -d
```

### Logs & Debugging
```bash
# Alle Logs
docker-compose logs -f

# Nur API Logs
docker-compose logs -f api

# In Container shell gehen
docker-compose exec api sh

# Registry & Build-Verzeichnis inspizieren
docker-compose exec api ls -la /app/build
```

---

## 📊 Monitoring

### Prometheus Queries (Beispiele)

```promql
# Request Rate (QPS)
rate(adapt_requests_total[5m])

# Drift Events
rate(adapt_drift_events_total[5m])

# Selection Latency (P95)
histogram_quantile(0.95, rate(adapt_selection_latency_seconds_bucket[5m]))

# Enforcement Rollout Percentage
adapt_enforce_rollout_percent
```

### Grafana Dashboard erstellen

1. Gehe zu http://localhost:3000
2. Login: `admin` / `admin`
3. Dashboards → New → Add Visualization
4. Wähle Prometheus Datasource
5. Query eingeben (z.B. `rate(adapt_requests_total[5m])`)
6. Panel konfigurieren (Graph, Gauge, etc.)

---

## 🐳 Production Deployment

### 1. Image zu Registry pushen
```bash
# Tag für Registry
docker tag cap-agent:v0.11.0-alpine your-registry.com/cap-agent:v0.11.0

# Push
docker push your-registry.com/cap-agent:v0.11.0
```

### 2. Environment Variables (Production)
```bash
# TLS aktivieren
docker run -e TLS_MODE=tls \
  -e TLS_CERT=/certs/server.crt \
  -e TLS_KEY=/certs/server.key \
  -v /path/to/certs:/certs:ro \
  cap-agent:v0.11.0-alpine
```

### 3. Volumes (Persistent Data)
```yaml
volumes:
  - /data/registry:/app/build  # Registry SQLite
  - /data/keys:/app/keys:ro    # Ed25519 Keys
  - /data/config:/app/config:ro # Config Files
```

---

## 📈 Performance-Ziele

| Metrik | Ziel | Aktuell |
|--------|------|---------|
| Image Size | <100 MB | TBD (nach Build) |
| Startup Time | <10s | TBD |
| Health Check | <500ms | TBD |
| API Latency (P95) | <2s | TBD |

---

## 🚨 Troubleshooting

### Problem: Container startet nicht
```bash
# Logs prüfen
docker-compose logs api

# Health Check manuell
docker-compose exec api curl http://localhost:8080/healthz
```

### Problem: Prometheus scraped keine Metrics
```bash
# Prometheus Targets prüfen
open http://localhost:9090/targets

# Metrics Endpoint manuell testen
curl http://localhost:8080/metrics
```

### Problem: Grafana zeigt keine Daten
```bash
# Datasource testen: Grafana UI → Configuration → Data Sources → Prometheus → Test

# Query in Prometheus UI testen zuerst
open http://localhost:9090
```

---

## ✅ Nächste Schritte

1. [ ] Docker Image bauen und testen
2. [ ] Stack lokal starten (`docker-compose up`)
3. [ ] Alle Services testen (API, Prometheus, Grafana)
4. [ ] Image-Größe optimieren (Ziel: <100 MB)
5. [ ] Grafana Dashboard erstellen
6. [ ] CI/CD Pipeline für Docker Build erweitern
7. [ ] Kubernetes Manifests erstellen

---

**Erstellt:** 17. November 2025  
**Version:** v0.11.0  
**Woche 1 Tag 1-2:** Docker & Monitoring Setup
