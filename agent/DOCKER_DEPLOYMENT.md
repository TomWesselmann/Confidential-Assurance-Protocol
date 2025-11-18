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

## 🖥️ WebUI Deployment (v0.11.0)

Die WebUI bietet eine benutzerfreundliche Oberfläche für Proof Upload und Verifikation.

### Voraussetzungen

```bash
# Node.js installieren (für Mac mit Homebrew)
brew install node

# Oder Node.js downloaden von: https://nodejs.org/

# Node & npm Version prüfen
node --version  # v18.0.0 oder höher
npm --version   # v9.0.0 oder höher
```

### Lokales Setup (Development)

#### 1. Backend API starten

```bash
# Terminal 1: Backend API
cd /Users/tomwesselmann/Desktop/LsKG-Agent/agent
cargo run --bin cap-verifier-api

# API läuft auf: http://localhost:8080
```

#### 2. WebUI starten

```bash
# Terminal 2: WebUI Dev Server
cd /Users/tomwesselmann/Desktop/LsKG-Agent/webui
npm install  # Nur beim ersten Mal
npm run dev

# WebUI läuft auf: http://localhost:5173
```

#### 3. Policy kompilieren

```bash
# Terminal 3: Policy für WebUI vorbereiten
TOKEN="admin-tom"
curl -X POST http://localhost:8080/policy/v2/compile \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "policy": {
      "id": "lksg.demo.v1",
      "version": "1.0.0",
      "legal_basis": [{"directive": "LkSG", "article": "§3"}],
      "description": "Demo policy for WebUI testing",
      "inputs": {
        "ubo_count": {"type": "integer"},
        "supplier_count": {"type": "integer"}
      },
      "rules": [
        {
          "id": "rule_ubo_exists",
          "op": "range_min",
          "lhs": {"var": "ubo_count"},
          "rhs": 1
        }
      ]
    },
    "persist": true,
    "lint_mode": "relaxed"
  }'
```

#### 4. WebUI verwenden

1. Browser öffnen: http://localhost:5173
2. Proof Package ZIP hochladen (Drag & Drop)
3. Manifest wird angezeigt
4. "Proof Verifizieren" klicken
5. Verification Result mit Status (OK/WARN/FAIL) wird angezeigt

**Hinweis:** Dev-Modus verwendet `admin-tom` Token für Authentication (NICHT für Production!)

### Production Build (WebUI)

```bash
# WebUI Production Build erstellen
cd webui
npm run build

# Output: webui/dist/
# Enthält optimierte HTML, CSS, JS Dateien
```

### WebUI mit Docker deployen

#### Option 1: Static File Server (nginx)

**Dockerfile.webui:**
```dockerfile
# Build stage
FROM node:18-alpine AS builder
WORKDIR /app
COPY webui/package*.json ./
RUN npm ci
COPY webui/ ./
RUN npm run build

# Production stage
FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/nginx.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

**nginx.conf:**
```nginx
server {
  listen 80;
  root /usr/share/nginx/html;
  index index.html;

  # SPA fallback
  location / {
    try_files $uri $uri/ /index.html;
  }

  # API Proxy (optional)
  location /api/ {
    proxy_pass http://cap-verifier-api:8080/;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
  }
}
```

**Build & Run:**
```bash
# WebUI Image bauen
docker build -f Dockerfile.webui -t cap-webui:v0.11.0 .

# WebUI Container starten
docker run -d -p 3000:80 cap-webui:v0.11.0

# WebUI öffnen: http://localhost:3000
```

#### Option 2: Docker Compose (Full Stack)

**docker-compose.yml erweitern:**
```yaml
services:
  # Bestehende Backend API
  api:
    image: cap-agent:v0.11.0-alpine
    ports:
      - "8080:8080"

  # WebUI hinzufügen
  webui:
    build:
      context: ../
      dockerfile: webui/Dockerfile.webui
    ports:
      - "3000:80"
    depends_on:
      - api
    environment:
      - VITE_API_URL=http://api:8080
```

**Stack starten:**
```bash
docker-compose up -d

# Backend API: http://localhost:8080
# WebUI: http://localhost:3000
```

### Environment Variables (WebUI)

**Development (.env.development):**
```bash
VITE_API_URL=http://localhost:8080
VITE_DEFAULT_TOKEN=admin-tom
```

**Production (.env.production):**
```bash
VITE_API_URL=https://api.your-domain.com
# Token wird vom User eingegeben (kein Default!)
```

### WebUI Features

- ✅ **Drag & Drop Upload:** ZIP Proof Packages hochladen
- ✅ **Manifest Viewer:** Visuelle Darstellung der Manifest-Daten
- ✅ **One-Click Verification:** Proof mit einem Klick verifizieren
- ✅ **Result Display:** Status Badges (OK/WARN/FAIL) mit Details
- ✅ **API Configuration:** Backend URL & Token konfigurierbar

### Security für Production

**⚠️ WICHTIG für Production:**

1. **Admin Token entfernen:**
   - `admin-tom` Token NUR für Development!
   - In Production: OAuth2 JWT Tokens verwenden

2. **CORS Origins einschränken:**
   ```rust
   // agent/src/bin/verifier_api.rs
   .allow_origin("https://your-domain.com".parse().unwrap())
   ```

3. **TLS/HTTPS aktivieren:**
   ```bash
   cargo run --bin cap-verifier-api \
     --bind 0.0.0.0:8443 \
     --tls \
     --tls-cert certs/server.crt \
     --tls-key certs/server.key
   ```

4. **nginx HTTPS:**
   ```nginx
   server {
     listen 443 ssl;
     ssl_certificate /etc/nginx/ssl/cert.pem;
     ssl_certificate_key /etc/nginx/ssl/key.pem;
   }
   ```

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
