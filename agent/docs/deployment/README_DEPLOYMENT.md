# CAP Agent - Deployment Guide

**Version:** v0.11.0
**Stand:** 2025-11-10

---

## Inhaltsverzeichnis

1. [Schnellstart](#schnellstart)
2. [Docker Deployment](#docker-deployment)
3. [Kubernetes Deployment](#kubernetes-deployment)
4. [Konfiguration](#konfiguration)
5. [TLS/mTLS Setup](#tlsmtls-setup)
6. [Monitoring & Logging](#monitoring--logging)
7. [Troubleshooting](#troubleshooting)

---

## Schnellstart

### Voraussetzungen

- Docker 20.10+
- Docker Compose 2.0+
- Kubernetes 1.25+ (optional)
- kubectl (optional)
- Helm 3.0+ (optional)

### Lokaler Test (Docker)

```bash
# 1. Repository klonen
git clone <repo-url>
cd agent

# 2. Docker Image bauen
docker build -t cap-agent:0.11.0 .

# 3. API starten
docker-compose up -d cap-api

# 4. Health Check
curl http://localhost:8080/healthz

# 5. Logs ansehen
docker-compose logs -f cap-api
```

---

## Docker Deployment

### 1. Image bauen

```bash
# Standard Build
docker build -t cap-agent:0.11.0 .

# Build mit spezifischer Platform
docker build --platform linux/amd64 -t cap-agent:0.11.0 .

# Multi-Platform Build (für ARM64 + AMD64)
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t cap-agent:0.11.0 \
  --push \
  .
```

### 2. Container starten

#### Standalone API

```bash
docker run -d \
  --name cap-api \
  -p 8080:8080 \
  -v $(pwd)/build:/app/build \
  -v $(pwd)/keys:/app/keys:ro \
  -e RUST_LOG=info,cap_agent=debug \
  cap-agent:0.11.0
```

#### Mit Docker Compose

```bash
# Start API
docker-compose up -d cap-api

# Check Status
docker-compose ps

# View Logs
docker-compose logs -f cap-api

# Stop
docker-compose down
```

### 3. CLI-Kommandos ausführen

```bash
# Via Docker Run
docker run --rm \
  -v $(pwd)/build:/app/build \
  -v $(pwd)/keys:/app/keys:ro \
  cap-agent:0.11.0 \
  cap-agent version

# Via Docker Compose
docker-compose run --rm cap-cli version

# Audit Chain verifizieren
docker-compose run --rm cap-cli \
  audit verify --file /app/build/audit_chain.jsonl

# Registry auflisten
docker-compose run --rm cap-cli \
  registry list --backend sqlite
```

### 4. Volumes

**Persistente Daten:**
- `/app/build` - Registry, BLOB Store, Audit Log
- `/app/keys` - Ed25519 Keys (read-only empfohlen)

**Read-Only:**
- `/app/config` - Konfigurationsdateien
- `/app/examples` - Beispieldaten

**Empfehlung:**
```yaml
volumes:
  - ./build:/app/build              # Read-Write (Datenbank)
  - ./keys:/app/keys:ro             # Read-Only (Keys)
  - ./config:/app/config:ro         # Read-Only (Config)
```

### 5. Umgebungsvariablen

```bash
# Logging
RUST_LOG=info,cap_agent=debug
RUST_BACKTRACE=1

# API Binding
CAP_BIND_ADDRESS=0.0.0.0:8080

# Storage Paths
CAP_REGISTRY_PATH=/app/build/registry.sqlite
CAP_BLOB_STORE_PATH=/app/build/test_blobs.sqlite
CAP_AUDIT_LOG=/app/build/audit_chain.jsonl
CAP_KEYS_DIR=/app/keys

# OAuth2 (optional)
OAUTH2_ISSUER=https://auth.example.com
OAUTH2_AUDIENCE=cap-verifier
```

---

## Kubernetes Deployment

### 1. Namespace erstellen

```bash
kubectl apply -f k8s/namespace.yaml
```

### 2. ConfigMap & Secrets

```bash
# ConfigMap
kubectl apply -f k8s/configmap.yaml

# Secrets (Keys) - ACHTUNG: Niemals in Git committen!
kubectl create secret generic cap-keys \
  --from-file=company.ed25519=./keys/company.ed25519 \
  --from-file=company.pub=./keys/company.pub \
  -n cap-system
```

### 3. PersistentVolumeClaim

```bash
# Erstelle PVC für Datenbank
kubectl apply -f k8s/pvc.yaml

# Check Status
kubectl get pvc -n cap-system
```

### 4. Deployment

```bash
# Deploy API
kubectl apply -f k8s/deployment.yaml

# Check Pods
kubectl get pods -n cap-system

# Check Logs
kubectl logs -f deployment/cap-verifier-api -n cap-system

# Check Events
kubectl describe deployment cap-verifier-api -n cap-system
```

### 5. Service

```bash
# Erstelle Service
kubectl apply -f k8s/service.yaml

# Port-Forward für lokalen Test
kubectl port-forward svc/cap-verifier-api 8080:80 -n cap-system

# Test
curl http://localhost:8080/healthz
```

### 6. Ingress (mit TLS)

```bash
# Cert-Manager installieren (für Let's Encrypt)
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.0/cert-manager.yaml

# ClusterIssuer erstellen
cat <<EOF | kubectl apply -f -
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: admin@example.com
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
    - http01:
        ingress:
          class: nginx
EOF

# Ingress erstellen
kubectl apply -f k8s/ingress.yaml

# Check Ingress
kubectl get ingress -n cap-system
kubectl describe ingress cap-verifier-api -n cap-system

# Check Certificate
kubectl get certificate -n cap-system
```

### 7. Skalierung

```bash
# Horizontal Scaling
kubectl scale deployment cap-verifier-api --replicas=5 -n cap-system

# Autoscaling (HPA)
kubectl autoscale deployment cap-verifier-api \
  --cpu-percent=70 \
  --min=3 \
  --max=10 \
  -n cap-system

# Check HPA
kubectl get hpa -n cap-system
```

### 8. Rolling Update

```bash
# Update Image
kubectl set image deployment/cap-verifier-api \
  cap-api=cap-agent:0.12.0 \
  -n cap-system

# Check Rollout Status
kubectl rollout status deployment/cap-verifier-api -n cap-system

# Rollback (if needed)
kubectl rollout undo deployment/cap-verifier-api -n cap-system
```

---

## Konfiguration

### docker-compose.yml Anpassungen

```yaml
services:
  cap-api:
    # Custom Port
    ports:
      - "9090:8080"  # External:Internal

    # Custom Registry Backend
    environment:
      - CAP_REGISTRY_BACKEND=postgres
      - DATABASE_URL=postgres://user:pass@db:5432/cap

    # Resource Limits
    deploy:
      resources:
        limits:
          cpus: '4.0'
          memory: 4G
```

### Kubernetes ConfigMap Anpassungen

```yaml
# k8s/configmap.yaml
data:
  # Production Logging
  RUST_LOG: "warn,cap_agent=info"

  # Custom Storage
  CAP_REGISTRY_PATH: "/mnt/nfs/registry.sqlite"

  # Performance Tuning
  MAX_CONCURRENT_REQUESTS: "200"
  DB_POOL_SIZE: "20"
```

---

## TLS/mTLS Setup

### Option 1: TLS via Ingress (Empfohlen)

TLS wird vom Kubernetes Ingress Controller (Nginx) terminiert:

```yaml
# k8s/ingress.yaml
spec:
  tls:
  - hosts:
    - cap-api.example.com
    secretName: cap-api-tls  # Auto-generiert via cert-manager
```

**Vorteile:**
- Automatische Zertifikat-Verwaltung (Let's Encrypt)
- Kein TLS-Code in der Anwendung
- Zentrale TLS-Konfiguration

### Option 2: TLS im Container

**ACHTUNG:** TLS im Container ist NICHT implementiert (v0.11.0)

**Benötigte Änderungen:**
1. `rustls` crate Integration
2. TLS Config in `src/bin/verifier_api.rs`
3. Certificate Loading
4. Port 8443 aktivieren

**TODO (zukünftig):**
```rust
// src/bin/verifier_api.rs (nicht implementiert)
use rustls::ServerConfig;

let tls_config = ServerConfig::builder()
    .with_safe_defaults()
    .with_no_client_auth()
    .with_single_cert(certs, key)?;

// Bind mit TLS
axum_server::bind_rustls("0.0.0.0:8443".parse()?, tls_config)
    .serve(app.into_make_service())
    .await?;
```

### mTLS (Client Certificate Validation)

**ACHTUNG:** mTLS ist NICHT implementiert (v0.11.0)

**Benötigte Änderungen:**
1. Client Certificate Validation
2. CA Trust Store
3. Certificate Revocation List (CRL)

**Workaround:** mTLS via Ingress
```yaml
# k8s/ingress.yaml (Nginx mTLS)
metadata:
  annotations:
    nginx.ingress.kubernetes.io/auth-tls-verify-client: "on"
    nginx.ingress.kubernetes.io/auth-tls-secret: "cap-system/ca-secret"
```

---

## Monitoring & Logging

### Health Checks

```bash
# Health Check
curl http://localhost:8080/healthz

# Readiness Check
curl http://localhost:8080/readyz

# Kubernetes Probes
kubectl get pods -n cap-system
kubectl describe pod <pod-name> -n cap-system
```

### Prometheus Metrics

**ACHTUNG:** Prometheus Metrics sind NICHT implementiert (v0.11.0)

**TODO (zukünftig):**
- `/metrics` Endpoint
- Prometheus Integration
- Grafana Dashboard

**Workaround:** Log-basiertes Monitoring

### Logging

```bash
# Docker Logs
docker-compose logs -f cap-api

# Kubernetes Logs
kubectl logs -f deployment/cap-verifier-api -n cap-system

# Alle Pods
kubectl logs -f -l app=cap-verifier-api -n cap-system

# Vorheriger Container (nach Crash)
kubectl logs -p <pod-name> -n cap-system
```

### Log-Aggregation

**ELK Stack:**
```yaml
# Filebeat Sidecar (k8s/deployment.yaml)
- name: filebeat
  image: elastic/filebeat:8.0.0
  volumeMounts:
  - name: logs
    mountPath: /var/log/cap
```

**Loki + Promtail:**
```bash
# Helm Install
helm repo add grafana https://grafana.github.io/helm-charts
helm install loki grafana/loki-stack
```

---

## Troubleshooting

### Problem 1: Container startet nicht

**Symptome:**
```bash
docker-compose up
# Error: failed to start container
```

**Diagnose:**
```bash
# Check Logs
docker-compose logs cap-api

# Check Image
docker images | grep cap-agent

# Check Volumes
docker volume ls
```

**Lösung:**
```bash
# Rebuild ohne Cache
docker-compose build --no-cache

# Check Ports
lsof -i :8080  # Port belegt?

# Check Volumes
ls -la build/ keys/  # Permissions?
```

### Problem 2: Health Check failed

**Symptome:**
```bash
curl http://localhost:8080/healthz
# Connection refused
```

**Diagnose:**
```bash
# Check Container Status
docker ps -a

# Check Logs
docker logs cap-api

# Check Binding
docker exec cap-api ss -tlnp | grep 8080
```

**Lösung:**
```bash
# Check Environment
docker exec cap-api env | grep CAP_BIND

# Restart
docker-compose restart cap-api
```

### Problem 3: Kubernetes Pod CrashLoopBackOff

**Symptome:**
```bash
kubectl get pods -n cap-system
# NAME                   READY   STATUS             RESTARTS
# cap-verifier-api-xxx   0/1     CrashLoopBackOff   5
```

**Diagnose:**
```bash
# Check Logs
kubectl logs cap-verifier-api-xxx -n cap-system

# Check Events
kubectl describe pod cap-verifier-api-xxx -n cap-system

# Check Previous Logs
kubectl logs -p cap-verifier-api-xxx -n cap-system
```

**Lösung:**
```bash
# Check ConfigMap
kubectl get cm cap-config -n cap-system -o yaml

# Check Secrets
kubectl get secrets cap-keys -n cap-system

# Check PVC
kubectl get pvc -n cap-system
kubectl describe pvc cap-data-pvc -n cap-system

# Restart Pod
kubectl delete pod cap-verifier-api-xxx -n cap-system
```

### Problem 4: TLS Certificate Error

**Symptome:**
```bash
curl https://cap-api.example.com/healthz
# SSL certificate problem
```

**Diagnose:**
```bash
# Check Certificate
kubectl get certificate -n cap-system
kubectl describe certificate cap-api-tls -n cap-system

# Check cert-manager Logs
kubectl logs -n cert-manager deployment/cert-manager
```

**Lösung:**
```bash
# Delete & Recreate Certificate
kubectl delete certificate cap-api-tls -n cap-system
kubectl delete secret cap-api-tls -n cap-system

# Reapply Ingress
kubectl delete ingress cap-verifier-api -n cap-system
kubectl apply -f k8s/ingress.yaml

# Wait for cert-manager
kubectl get certificate -n cap-system -w
```

### Problem 5: Performance Issues

**Symptome:**
- Langsame Antwortzeiten
- High CPU/Memory
- OOM Kills

**Diagnose:**
```bash
# Docker Stats
docker stats cap-api

# Kubernetes Metrics
kubectl top pods -n cap-system
kubectl top nodes
```

**Lösung:**
```bash
# Increase Resources (Docker)
# Edit docker-compose.yml:
deploy:
  resources:
    limits:
      cpus: '4.0'
      memory: 4G

# Increase Resources (K8s)
# Edit k8s/deployment.yaml:
resources:
  limits:
    cpu: "4000m"
    memory: "4Gi"

# Apply Changes
kubectl apply -f k8s/deployment.yaml
```

---

## Checkliste: Production-Ready

### Vor Deployment

- [ ] Dockerfile gebaut und getestet
- [ ] docker-compose.yml konfiguriert
- [ ] Volumes gemountet (build/, keys/)
- [ ] Environment Variables gesetzt
- [ ] Health Checks funktionieren
- [ ] Logs lesbar (RUST_LOG konfiguriert)

### Kubernetes

- [ ] Namespace erstellt
- [ ] ConfigMap & Secrets erstellt
- [ ] PVC provisioniert
- [ ] Deployment deployed
- [ ] Service erstellt
- [ ] Ingress konfiguriert (mit TLS)
- [ ] Health Probes funktionieren
- [ ] Resource Limits gesetzt
- [ ] Autoscaling konfiguriert (optional)

### Sicherheit

- [ ] Keys NICHT in Git committed
- [ ] Secrets via Kubernetes Secrets
- [ ] TLS aktiviert (Ingress oder Container)
- [ ] OAuth2 konfiguriert
- [ ] RBAC konfiguriert (Kubernetes)
- [ ] Network Policies (optional)

### Monitoring

- [ ] Health Checks erreichbar
- [ ] Logs aggregiert (ELK/Loki)
- [ ] Prometheus Metrics (TODO)
- [ ] Grafana Dashboard (TODO)
- [ ] Alerts konfiguriert

---

## Nächste Schritte

1. **TLS/mTLS implementieren** (siehe BLOCKER #9 in CAP_ROADMAP_TODO.md)
2. **Prometheus Metrics** (siehe TODO in SYSTEM_ARCHITECTURE.md)
3. **Helm Chart** erstellen
4. **CI/CD Pipeline** (GitHub Actions / GitLab CI)

---

**Stand:** 2025-11-10
**Version:** v0.11.0
**Verantwortlich:** DevOps Team
