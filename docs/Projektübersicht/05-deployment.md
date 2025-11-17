# 05 - Deployment & Betrieb

## 📖 Über dieses Kapitel

Nachdem Sie nun wissen, **was** das System macht, **wie** es aufgebaut ist, **welche Teile** es hat und **wie man es bedient**, erklärt dieses Kapitel **wie man es in Betrieb nimmt**.

**Für wen ist dieses Kapitel?**
- **Management:** Die Deployment-Optionen-Übersicht mit Kosten/Nutzen
- **IT-Leiter:** Entscheidungshilfe für die richtige Deployment-Methode
- **IT-Administratoren:** Detaillierte Installations- und Betriebsanleitungen
- **DevOps-Engineers:** Kubernetes, Docker, CI/CD-Konfigurationen

**Was Sie lernen werden:**
1. Welche 4 Deployment-Optionen es gibt (und welche für Sie passt)
2. Wie man das System installiert
3. Wie man es überwacht und wartet
4. Wie man Backups erstellt

**Analogie:** Dies ist die **Installationsanleitung** - wie bei einem Gerät die Anleitung zum Aufbauen und Anschließen.

---

## 👔 Für Management: Welche Installations-Methode?

Das System kann auf **4 verschiedene Arten** installiert werden. Hier eine Entscheidungshilfe:

| Methode | Für wen? | Kosten | Komplexität | Skalierbarkeit |
|---------|----------|--------|-------------|----------------|
| **1. Binary** | Kleine Firmen, Tests | € | ⭐ Einfach | ⭐ Limitiert |
| **2. Docker** | Mittlere Firmen | €€ | ⭐⭐ Mittel | ⭐⭐ Gut |
| **3. Kubernetes** | Konzerne, Cloud | €€€€ | ⭐⭐⭐⭐ Komplex | ⭐⭐⭐⭐ Exzellent |
| **4. Systemd** | Linux-Server | € | ⭐⭐ Mittel | ⭐⭐ Gut |

### Empfehlungen nach Unternehmensgröße:

**Kleine Unternehmen (< 50 Mitarbeiter):**
- ✅ **Binary** oder **Systemd** auf einem Linux-Server
- Einfach, kostengünstig, ausreichend
- *Analogie:* Wie ein Desktop-PC statt Server-Rack

**Mittlere Unternehmen (50-500 Mitarbeiter):**
- ✅ **Docker** mit Docker Compose
- Balance zwischen Einfachheit und Professionalität
- Einfaches Update-Management
- *Analogie:* Wie ein kleines Server-Rack mit wenigen Geräten

**Große Unternehmen / Konzerne (> 500 Mitarbeiter):**
- ✅ **Kubernetes** in der Cloud oder On-Premise
- Hochverfügbarkeit, automatische Skalierung
- Integration in bestehende K8s-Infrastruktur
- *Analogie:* Wie ein Rechenzentrum mit automatischer Verwaltung

### Kosten-Vergleich (geschätzte Jahreskosten):

| Methode | Hardware | Software | Wartung | Gesamt/Jahr |
|---------|----------|----------|---------|-------------|
| Binary | 1 Server (~2.000€) | Kostenlos | 5 PT (~10.000€) | ~12.000€ |
| Docker | 1-2 Server (~4.000€) | Kostenlos | 8 PT (~16.000€) | ~20.000€ |
| Kubernetes | Cloud oder 3+ Server (~15.000€) | Kostenlos | 20 PT (~40.000€) | ~55.000€ |

**Hinweis:** PT = Personentage (angenommener Tagessatz: 2.000€)

---

## Deployment-Optionen

Der LsKG-Agent kann auf verschiedene Arten deployed werden:

1. **Binary Deployment** - Direktes Ausführen des Rust-Binaries (einfachste Methode)
2. **Docker Container** - Containerisierte Anwendung (empfohlen für Produktion)
3. **Kubernetes** - Orchestrierte Container in einem Cluster (für Enterprise)
4. **Systemd Service** - Systemd-managed Service auf Linux (klassischer Ansatz)

---

## 1. Binary Deployment

### Voraussetzungen

**System:**
- Linux (x86_64) oder macOS (x86_64/ARM64)
- 4 GB RAM minimum, 8 GB empfohlen
- 2 CPU Cores minimum, 4 Cores empfohlen
- 10 GB Festplatte

**Software:**
- Rust 1.75+ (für Build)
- OpenSSL 1.1+ (für TLS)
- SQLite 3.35+ (optional, für Registry)

### Build from Source

```bash
# Repository klonen
git clone https://github.com/your-org/LsKG-Agent.git
cd LsKG-Agent/agent

# Dependencies installieren
cargo fetch

# Release Build erstellen
cargo build --release

# Binary ist verfügbar unter:
# target/release/cap (CLI)
# target/release/cap-verifier-api (API Server)
```

### Installation

```bash
# Binary in System-Pfad installieren
sudo cp target/release/cap /usr/local/bin/
sudo cp target/release/cap-verifier-api /usr/local/bin/

# Executable permissions setzen
sudo chmod +x /usr/local/bin/cap
sudo chmod +x /usr/local/bin/cap-verifier-api

# Verifizieren
cap --version
cap-verifier-api --version
```

### Konfiguration

```bash
# Konfigurationsverzeichnis erstellen
sudo mkdir -p /etc/cap-verifier
sudo mkdir -p /var/lib/cap-verifier
sudo mkdir -p /var/log/cap-verifier

# Konfigurationsdateien kopieren
sudo cp config/app.yaml /etc/cap-verifier/
sudo cp config/auth.yaml /etc/cap-verifier/
sudo cp config/tls.yaml /etc/cap-verifier/

# Permissions setzen
sudo chown -R cap-verifier:cap-verifier /etc/cap-verifier
sudo chown -R cap-verifier:cap-verifier /var/lib/cap-verifier
sudo chown -R cap-verifier:cap-verifier /var/log/cap-verifier
```

### Systemd Service

**Service-Datei erstellen:**
```bash
sudo nano /etc/systemd/system/cap-verifier.service
```

**Inhalt:**
```ini
[Unit]
Description=CAP Verifier API Server
After=network.target

[Service]
Type=simple
User=cap-verifier
Group=cap-verifier
WorkingDirectory=/var/lib/cap-verifier
ExecStart=/usr/local/bin/cap-verifier-api \
    --config /etc/cap-verifier/app.yaml \
    --tls \
    --tls-cert /etc/cap-verifier/certs/server.crt \
    --tls-key /etc/cap-verifier/certs/server.key
Restart=on-failure
RestartSec=10
StandardOutput=journal
StandardError=journal

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/cap-verifier /var/log/cap-verifier

[Install]
WantedBy=multi-user.target
```

**Service aktivieren:**
```bash
sudo systemctl daemon-reload
sudo systemctl enable cap-verifier
sudo systemctl start cap-verifier
sudo systemctl status cap-verifier
```

**Logs ansehen:**
```bash
sudo journalctl -u cap-verifier -f
```

---

## 2. Docker Deployment

### Dockerfile

Das Projekt enthält bereits ein Multi-Stage Dockerfile:

**agent/Dockerfile:**
```dockerfile
# Stage 1: Build
FROM rust:1.75-bullseye AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    musl-tools \
    && rm -rf /var/lib/apt/lists/*

# Add musl target
RUN rustup target add x86_64-unknown-linux-musl

# Create app directory
WORKDIR /app

# Copy source code
COPY . .

# Build release binary
RUN cargo build --release --target x86_64-unknown-linux-musl

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 cap-verifier

# Copy binary from builder
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/cap-verifier-api /usr/local/bin/

# Create directories
RUN mkdir -p /config /certs /data && \
    chown -R cap-verifier:cap-verifier /config /certs /data

# Switch to non-root user
USER cap-verifier

# Expose ports
EXPOSE 8080 8443

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/healthz || exit 1

# Start server
CMD ["cap-verifier-api", "--config", "/config/app.yaml"]
```

### Docker Build

```bash
# Im agent/ Verzeichnis
docker build -t cap-agent:0.11.0 .

# Tag für Registry
docker tag cap-agent:0.11.0 registry.example.com/cap-agent:0.11.0

# Push zu Registry
docker push registry.example.com/cap-agent:0.11.0
```

### Docker Run

**HTTP Mode (Development):**
```bash
docker run -d \
  --name cap-verifier \
  -p 8080:8080 \
  -v $(pwd)/config:/config:ro \
  -v $(pwd)/data:/data \
  cap-agent:0.11.0
```

**HTTPS Mode (Production):**
```bash
docker run -d \
  --name cap-verifier \
  -p 8443:8443 \
  -v $(pwd)/config:/config:ro \
  -v $(pwd)/certs:/certs:ro \
  -v $(pwd)/data:/data \
  -e TLS_MODE=tls \
  cap-agent:0.11.0
```

### Docker Compose

**docker-compose.yml:**
```yaml
version: '3.8'

services:
  cap-verifier:
    image: cap-agent:0.11.0
    container_name: cap-verifier
    restart: unless-stopped
    ports:
      - "8443:8443"
    volumes:
      - ./config:/config:ro
      - ./certs:/certs:ro
      - cap-data:/data
    environment:
      - TLS_MODE=tls
      - RUST_LOG=info
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/healthz"]
      interval: 30s
      timeout: 3s
      retries: 3
      start_period: 5s
    networks:
      - cap-network

  prometheus:
    image: prom/prometheus:latest
    container_name: prometheus
    restart: unless-stopped
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
    networks:
      - cap-network

  grafana:
    image: grafana/grafana:latest
    container_name: grafana
    restart: unless-stopped
    ports:
      - "3000:3000"
    volumes:
      - grafana-data:/var/lib/grafana
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    networks:
      - cap-network

volumes:
  cap-data:
  prometheus-data:
  grafana-data:

networks:
  cap-network:
    driver: bridge
```

**Starten:**
```bash
docker-compose up -d
```

**Logs:**
```bash
docker-compose logs -f cap-verifier
```

**Stoppen:**
```bash
docker-compose down
```

---

## 3. Kubernetes Deployment

### Namespace erstellen

```bash
kubectl create namespace cap-system
```

### ConfigMap

**config/configmap.yaml:**
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: cap-config
  namespace: cap-system
data:
  app.yaml: |
    server:
      bind: "0.0.0.0:8443"
      log_level: "info"
    oauth:
      issuer: "https://auth.example.com"
      audience: "cap-verifier"
      required_scopes:
        - "verify:read"
    verification:
      backend: "mock"
      timeout_seconds: 30
    policy:
      cache_enabled: true
      cache_ttl_seconds: 3600

  auth.yaml: |
    oauth2:
      issuer: "https://auth.example.com"
      audience: "cap-verifier"
      public_key_path: "/config/public.pem"
      required_scopes:
        - "verify:read"
```

**Anwenden:**
```bash
kubectl apply -f config/configmap.yaml
```

### Secret (TLS Certificates)

```bash
kubectl create secret tls cap-tls-certs \
  --cert=certs/server.crt \
  --key=certs/server.key \
  --namespace=cap-system
```

### Deployment

**k8s/deployment.yaml:**
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cap-verifier
  namespace: cap-system
  labels:
    app: cap-verifier
spec:
  replicas: 3
  selector:
    matchLabels:
      app: cap-verifier
  template:
    metadata:
      labels:
        app: cap-verifier
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8080"
        prometheus.io/path: "/metrics"
    spec:
      containers:
      - name: cap-verifier
        image: registry.example.com/cap-agent:0.11.0
        imagePullPolicy: IfNotPresent
        ports:
        - containerPort: 8080
          name: http
          protocol: TCP
        - containerPort: 8443
          name: https
          protocol: TCP
        env:
        - name: TLS_MODE
          value: "tls"
        - name: RUST_LOG
          value: "info"
        volumeMounts:
        - name: config
          mountPath: /config
          readOnly: true
        - name: tls-certs
          mountPath: /certs
          readOnly: true
        - name: data
          mountPath: /data
        resources:
          requests:
            cpu: 500m
            memory: 512Mi
          limits:
            cpu: 2000m
            memory: 2Gi
        livenessProbe:
          httpGet:
            path: /healthz
            port: 8080
            scheme: HTTP
          initialDelaySeconds: 10
          periodSeconds: 30
          timeoutSeconds: 3
          successThreshold: 1
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /readyz
            port: 8080
            scheme: HTTP
          initialDelaySeconds: 5
          periodSeconds: 10
          timeoutSeconds: 3
          successThreshold: 1
          failureThreshold: 3
        securityContext:
          runAsNonRoot: true
          runAsUser: 1000
          allowPrivilegeEscalation: false
          readOnlyRootFilesystem: false
          capabilities:
            drop:
            - ALL
      volumes:
      - name: config
        configMap:
          name: cap-config
      - name: tls-certs
        secret:
          secretName: cap-tls-certs
      - name: data
        persistentVolumeClaim:
          claimName: cap-data-pvc
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 100
            podAffinityTerm:
              labelSelector:
                matchExpressions:
                - key: app
                  operator: In
                  values:
                  - cap-verifier
              topologyKey: kubernetes.io/hostname
```

**Anwenden:**
```bash
kubectl apply -f k8s/deployment.yaml
```

### Service

**k8s/service.yaml:**
```yaml
apiVersion: v1
kind: Service
metadata:
  name: cap-verifier-svc
  namespace: cap-system
  labels:
    app: cap-verifier
spec:
  type: ClusterIP
  selector:
    app: cap-verifier
  ports:
  - name: http
    port: 8080
    targetPort: 8080
    protocol: TCP
  - name: https
    port: 8443
    targetPort: 8443
    protocol: TCP
```

**Anwenden:**
```bash
kubectl apply -f k8s/service.yaml
```

### Ingress

**k8s/ingress.yaml:**
```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: cap-verifier-ingress
  namespace: cap-system
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/backend-protocol: "HTTPS"
spec:
  ingressClassName: nginx
  tls:
  - hosts:
    - verifier.example.com
    secretName: cap-verifier-tls
  rules:
  - host: verifier.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: cap-verifier-svc
            port:
              number: 8443
```

**Anwenden:**
```bash
kubectl apply -f k8s/ingress.yaml
```

### PersistentVolumeClaim

**k8s/pvc.yaml:**
```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: cap-data-pvc
  namespace: cap-system
spec:
  accessModes:
  - ReadWriteOnce
  resources:
    requests:
      storage: 10Gi
  storageClassName: fast-ssd
```

**Anwenden:**
```bash
kubectl apply -f k8s/pvc.yaml
```

### Deployment überprüfen

```bash
# Pods ansehen
kubectl get pods -n cap-system

# Logs eines Pods
kubectl logs -n cap-system cap-verifier-xxx-yyy -f

# Service testen
kubectl port-forward -n cap-system svc/cap-verifier-svc 8443:8443

# Dann in einem anderen Terminal:
curl -k https://localhost:8443/healthz
```

---

## 4. Monitoring & Observability

> ⭐ **NEU (Week 2):** Ein vollständiger Production-Ready Monitoring Stack ist jetzt verfügbar!
>
> **Umfasst:** Prometheus, Grafana (2 Dashboards), Loki, Promtail, Jaeger, Node Exporter, cAdvisor
> **Status:** ✅ 8/8 Container running, 5/5 healthy
> **Dokumentation:** `/Users/tomwesselmann/Desktop/LsKG-Agent/agent/monitoring/README.md`
> **Quick Start:**
> ```bash
> cd /Users/tomwesselmann/Desktop/LsKG-Agent/agent/monitoring
> docker compose up -d
> ./test-monitoring.sh
> ```
>
> **Service URLs:**
> - Grafana: http://localhost:3000 (admin/admin)
> - Prometheus: http://localhost:9090
> - Jaeger UI: http://localhost:16686
>
> **Für Details siehe:** [monitoring/README.md](/Users/tomwesselmann/Desktop/LsKG-Agent/agent/monitoring/README.md) und [07-status-und-roadmap.md](./07-status-und-roadmap.md)

---

### Basic Prometheus Configuration (für Custom Setups)

**prometheus.yml:**
```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'cap-verifier'
    static_configs:
      - targets: ['cap-verifier:8080']
    metrics_path: '/metrics'
```

### Grafana Dashboard

**Dashboard erstellen:**
1. Grafana öffnen (http://localhost:3000)
2. Login: admin / admin
3. Add Data Source → Prometheus → http://prometheus:9090
4. Create Dashboard → Add Panel

**Wichtige Metriken:**

**Request Rate:**
```promql
rate(cap_verifier_requests_total[5m])
```

**Success Rate:**
```promql
sum(rate(cap_verifier_requests_total{result="ok"}[5m])) /
sum(rate(cap_verifier_requests_total[5m]))
```

**Latency (P50, P95, P99):**
```promql
histogram_quantile(0.50, rate(cap_verifier_request_duration_seconds_bucket[5m]))
histogram_quantile(0.95, rate(cap_verifier_request_duration_seconds_bucket[5m]))
histogram_quantile(0.99, rate(cap_verifier_request_duration_seconds_bucket[5m]))
```

**Auth Failures:**
```promql
rate(cap_auth_token_validation_failures_total[5m])
```

**Cache Hit Ratio:**
```promql
cap_cache_hit_ratio
```

---

## 5. Backup & Recovery

### Daten, die gesichert werden müssen

1. **Registry Database** (`registry.db`)
2. **BLOB Store** (`blobs/`)
3. **Keys** (`keys/`)
4. **Konfiguration** (`config/`)
5. **Certificates** (`certs/`)

### Backup Script

**scripts/backup.sh:**
```bash
#!/bin/bash

BACKUP_DIR="/backups/cap-verifier"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_PATH="$BACKUP_DIR/backup_$TIMESTAMP"

# Create backup directory
mkdir -p "$BACKUP_PATH"

# Backup registry
echo "Backing up registry..."
sqlite3 /data/registry.db ".backup '$BACKUP_PATH/registry.db'"

# Backup BLOB store
echo "Backing up BLOB store..."
rsync -a /data/blobs/ "$BACKUP_PATH/blobs/"

# Backup keys
echo "Backing up keys..."
cp -r /data/keys/ "$BACKUP_PATH/keys/"

# Backup config
echo "Backing up config..."
cp -r /etc/cap-verifier/ "$BACKUP_PATH/config/"

# Create tarball
echo "Creating tarball..."
cd "$BACKUP_DIR"
tar -czf "backup_$TIMESTAMP.tar.gz" "backup_$TIMESTAMP"
rm -rf "backup_$TIMESTAMP"

# Remove old backups (keep last 7 days)
find "$BACKUP_DIR" -name "backup_*.tar.gz" -mtime +7 -delete

echo "Backup complete: $BACKUP_DIR/backup_$TIMESTAMP.tar.gz"
```

### Restore Script

**scripts/restore.sh:**
```bash
#!/bin/bash

BACKUP_FILE="$1"

if [ -z "$BACKUP_FILE" ]; then
  echo "Usage: $0 <backup-file.tar.gz>"
  exit 1
fi

# Stop service
systemctl stop cap-verifier

# Extract backup
TEMP_DIR=$(mktemp -d)
tar -xzf "$BACKUP_FILE" -C "$TEMP_DIR"

# Restore registry
echo "Restoring registry..."
cp "$TEMP_DIR/backup_*/registry.db" /data/registry.db

# Restore BLOB store
echo "Restoring BLOB store..."
rsync -a "$TEMP_DIR/backup_*/blobs/" /data/blobs/

# Restore keys
echo "Restoring keys..."
cp -r "$TEMP_DIR/backup_*/keys/" /data/keys/

# Restore config
echo "Restoring config..."
cp -r "$TEMP_DIR/backup_*/config/" /etc/cap-verifier/

# Cleanup
rm -rf "$TEMP_DIR"

# Start service
systemctl start cap-verifier

echo "Restore complete"
```

### Automated Backups (Cron)

```bash
# Edit crontab
crontab -e

# Add daily backup at 2 AM
0 2 * * * /usr/local/bin/cap-backup.sh >> /var/log/cap-backup.log 2>&1
```

---

## 6. Sicherheit

### TLS/mTLS Configuration

**Generiere Self-Signed Certificate (Development):**
```bash
openssl req -x509 -newkey rsa:4096 -keyout server.key -out server.crt -days 365 -nodes \
  -subj "/C=DE/ST=Berlin/L=Berlin/O=Example/CN=localhost"
```

**Production Certificate (Let's Encrypt):**
```bash
# Mit cert-manager in Kubernetes (siehe Ingress Annotation)
# oder mit certbot:
sudo certbot certonly --standalone -d verifier.example.com
```

### OAuth2 Public Key

**Public Key aus JWK extrahieren:**
```bash
# JWK von Auth Server abrufen
curl https://auth.example.com/.well-known/jwks.json | jq '.keys[0]'

# In PEM konvertieren (z.B. mit jwt.io oder online tool)
# Dann in /config/public.pem speichern
```

### Firewall Rules

**UFW (Ubuntu):**
```bash
# Allow SSH
sudo ufw allow 22/tcp

# Allow HTTPS
sudo ufw allow 8443/tcp

# Enable firewall
sudo ufw enable
```

**iptables:**
```bash
# Allow HTTPS
sudo iptables -A INPUT -p tcp --dport 8443 -j ACCEPT

# Save rules
sudo iptables-save > /etc/iptables/rules.v4
```

---

## 7. Performance Tuning

### Rust Runtime

**Environment Variables:**
```bash
# Thread pool size
export TOKIO_WORKER_THREADS=4

# Stack size
export RUST_MIN_STACK=8388608
```

### SQLite Tuning

**In registry.db:**
```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = -64000;  -- 64 MB
PRAGMA temp_store = MEMORY;
```

### Kubernetes Resources

**Adjust based on load:**
```yaml
resources:
  requests:
    cpu: 1000m       # 1 CPU
    memory: 1Gi
  limits:
    cpu: 4000m       # 4 CPU
    memory: 4Gi
```

### Horizontal Pod Autoscaler

**k8s/hpa.yaml:**
```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: cap-verifier-hpa
  namespace: cap-system
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: cap-verifier
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

---

## 8. Logging

### Structured Logging (tracing)

**Log Levels:**
- `error` - Fehler, die sofortiges Handeln erfordern
- `warn` - Warnungen, die überwacht werden sollten
- `info` - Allgemeine Informationen (Standard)
- `debug` - Detaillierte Debug-Informationen
- `trace` - Sehr detaillierte Trace-Informationen

**Environment Variable:**
```bash
export RUST_LOG=info
# Oder für Module-spezifisch:
export RUST_LOG=cap_agent=debug,tower_http=info
```

### Log Aggregation (ELK Stack)

**Filebeat Configuration:**
```yaml
filebeat.inputs:
- type: container
  paths:
    - '/var/lib/docker/containers/*/*.log'
  processors:
    - add_kubernetes_metadata:
        host: ${NODE_NAME}
        matchers:
        - logs_path:
            logs_path: "/var/lib/docker/containers/"

output.elasticsearch:
  hosts: ["elasticsearch:9200"]
```

---

## 9. Health Checks

### Endpoints

- **Liveness:** `GET /healthz` - Prüft ob Service läuft
- **Readiness:** `GET /readyz` - Prüft ob Service bereit ist Requests zu akzeptieren

### Kubernetes Probes

```yaml
livenessProbe:
  httpGet:
    path: /healthz
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 30

readinessProbe:
  httpGet:
    path: /readyz
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 10
```

---

## Zusammenfassung

Der LsKG-Agent kann flexibel deployed werden:
- ✅ Binary + Systemd für einfache Deployments
- ✅ Docker für Container-basierte Deployments
- ✅ Kubernetes für Enterprise-Skalierung
- ✅ Monitoring mit Prometheus + Grafana
- ✅ Automated Backups
- ✅ Security Hardening
- ✅ Performance Tuning
