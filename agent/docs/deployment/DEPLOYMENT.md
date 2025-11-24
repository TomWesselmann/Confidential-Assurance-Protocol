# CAP Verifier - Deployment Guide

**Version:** v0.11.0
**Status:** Production-Ready
**Last Updated:** 2025-11-24

> Comprehensive deployment guide for CAP Verifier REST API covering Docker, Kubernetes, TLS/mTLS, Monitoring, and Production Best Practices

---

## Table of Contents

1. [Quick Start](#1-quick-start)
2. [Docker Deployment](#2-docker-deployment)
3. [Kubernetes Deployment](#3-kubernetes-deployment)
4. [TLS/mTLS Configuration](#4-tlsmtls-configuration)
5. [Monitoring & Observability](#5-monitoring--observability)
6. [Configuration Reference](#6-configuration-reference)
7. [Troubleshooting](#7-troubleshooting)
8. [Production Checklist](#8-production-checklist)

---

## 1. Quick Start

### 1.1 Prerequisites

- **Docker:** 20.10+ ([Install Docker](https://docs.docker.com/get-docker/))
- **Docker Compose:** 2.0+ (included with Docker Desktop)
- **Rust:** 1.70+ (for local builds)
- **kubectl:** Latest (for Kubernetes deployment)
- **Helm:** 3.0+ (optional, for Helm charts)

**Mac-specific (M1/M2):**
- Docker Desktop for Mac with Apple Silicon support
- Rosetta 2 (automatic on macOS 11+)

### 1.2 Quick Start: Docker (HTTP)

**Fastest path to get API running locally:**

```bash
# 1. Clone repository
git clone <repo-url>
cd agent

# 2. Build Docker image
docker build -t cap-agent:0.11.0 .

# 3. Run container (HTTP-only, Development)
docker run -d \
  --name cap-verifier \
  -p 8080:8080 \
  -v $(pwd)/build:/app/build \
  -v $(pwd)/keys:/app/keys:ro \
  -e RUST_LOG=info,cap_agent=debug \
  cap-agent:0.11.0

# 4. Health check
curl http://localhost:8080/healthz
# Expected: {"status":"OK","version":"0.11.0","build_hash":null}

# 5. View logs
docker logs -f cap-verifier
```

### 1.3 Quick Start: Docker Compose

**Production-like setup with Monitoring Stack:**

```bash
# 1. Start full stack (API + Prometheus + Grafana + Loki + Jaeger)
cd monitoring
docker compose up -d

# 2. Check status
docker compose ps

# 3. Test endpoints
curl http://localhost:8080/healthz     # API Health
open http://localhost:9090             # Prometheus
open http://localhost:3000             # Grafana (admin/admin)
open http://localhost:16686            # Jaeger UI

# 4. Stop stack
docker compose down
```

### 1.4 Quick Start: Kubernetes

**Deploy to existing Kubernetes cluster:**

```bash
# 1. Create namespace
kubectl create namespace cap-system

# 2. Create secrets (TLS, Ed25519 keys)
kubectl create secret tls cap-verifier-tls \
  --cert=certs/server.crt \
  --key=certs/server.key \
  -n cap-system

kubectl create secret generic cap-agent-key \
  --from-file=agent.ed25519=keys/company.ed25519 \
  --from-file=agent.pub=keys/company.pub \
  -n cap-system

# 3. Apply manifests
kubectl apply -f kubernetes/deployment.yml -n cap-system
kubectl apply -f kubernetes/service.yml -n cap-system

# 4. Check pods
kubectl get pods -n cap-system

# 5. Port-forward for testing
kubectl port-forward svc/cap-verifier 8080:80 -n cap-system
curl http://localhost:8080/healthz
```

---

## 2. Docker Deployment

### 2.1 Basic Docker Build

**Standard Dockerfile build:**

```bash
# Build image
docker build -t cap-agent:0.11.0 .

# Verify image size
docker images cap-agent:0.11.0

# Run container
docker run -d -p 8080:8080 cap-agent:0.11.0
```

### 2.2 Multi-Stage Build Strategies

#### Option A: Alpine-Based (Lightweight, ~50 MB)

**Dockerfile.alpine:**

```dockerfile
# Build Stage (Rust 1.81)
FROM rust:1.81-alpine AS builder
WORKDIR /src
RUN apk add --no-cache musl-dev
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --bin cap-verifier-api && \
    strip /src/target/release/cap-verifier-api

# Runtime Stage (Alpine)
FROM alpine:3.18
RUN apk add --no-cache ca-certificates
RUN addgroup -S nonroot && adduser -S nonroot -G nonroot
USER nonroot:nonroot
WORKDIR /app
COPY --from=builder /src/target/release/cap-verifier-api /app/
COPY config /app/config
COPY openapi /app/openapi
EXPOSE 8443
ENTRYPOINT ["/app/cap-verifier-api"]
CMD ["--bind", "0.0.0.0:8443"]
```

**Build:**

```bash
docker build -f Dockerfile.alpine -t cap-agent:v0.11.0-alpine .
```

**Pros:**
- ✅ Smallest image size (~50 MB)
- ✅ Fast startup (<5s)
- ✅ Full shell access (sh)

**Cons:**
- ⚠️ musl libc (compatibility issues possible)
- ⚠️ More attack surface than distroless

#### Option B: Distroless (Security-Hardened, ~80 MB)

**Dockerfile.distroless:**

```dockerfile
# Build Stage (Rust 1.81)
FROM rust:1.81-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --bin cap-verifier-api && \
    strip /src/target/release/cap-verifier-api

# Runtime Stage (Distroless)
FROM gcr.io/distroless/cc-debian12:nonroot
USER nonroot:nonroot
WORKDIR /app
COPY --from=build /src/target/release/cap-verifier-api /app/
COPY config /app/config
COPY openapi /app/openapi
EXPOSE 8443
ENTRYPOINT ["/app/cap-verifier-api"]
CMD ["--bind", "0.0.0.0:8443"]
```

**Build:**

```bash
docker build -f Dockerfile.distroless -t cap-agent:v0.11.0-distroless .
```

**Pros:**
- ✅ Minimal attack surface (no shell, no package manager)
- ✅ Non-root user (UID 65532)
- ✅ Google-maintained base image

**Cons:**
- ⚠️ No shell (debugging harder)
- ⚠️ Slightly larger than Alpine (~80 MB)

**Security Features (Distroless):**
- ✅ Read-Only Root Filesystem
- ✅ Dropped ALL Capabilities
- ✅ Seccomp Profile: RuntimeDefault
- ✅ No Privilege Escalation
- ✅ Image Size ≤ 100 MB

### 2.3 Docker Compose Setup

**docker-compose.yml (Full Stack):**

```yaml
version: '3.8'

services:
  cap-verifier-api:
    build:
      context: .
      dockerfile: Dockerfile.alpine
    image: cap-agent:0.11.0-alpine
    container_name: cap-verifier
    ports:
      - "8080:8080"
    volumes:
      - ./build:/app/build
      - ./keys:/app/keys:ro
      - ./config:/app/config:ro
    environment:
      - RUST_LOG=info,cap_agent=debug
      - CAP_BIND_ADDRESS=0.0.0.0:8080
      - POLICY_STORE_BACKEND=sqlite
      - POLICY_DB_PATH=/app/build/policies.sqlite
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "wget", "-qO-", "http://localhost:8080/healthz"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    networks:
      - cap-network

  prometheus:
    image: prom/prometheus:latest
    container_name: prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus/prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.retention.time=30d'
    restart: unless-stopped
    networks:
      - cap-network

  grafana:
    image: grafana/grafana:latest
    container_name: grafana
    ports:
      - "3000:3000"
    volumes:
      - grafana-data:/var/lib/grafana
      - ./monitoring/grafana/provisioning:/etc/grafana/provisioning:ro
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
      - GF_USERS_ALLOW_SIGN_UP=false
    restart: unless-stopped
    networks:
      - cap-network

  loki:
    image: grafana/loki:latest
    container_name: loki
    ports:
      - "3100:3100"
    volumes:
      - ./monitoring/loki/loki-config.yml:/etc/loki/local-config.yaml:ro
      - loki-data:/loki
    command: -config.file=/etc/loki/local-config.yaml
    restart: unless-stopped
    networks:
      - cap-network

  promtail:
    image: grafana/promtail:latest
    container_name: promtail
    volumes:
      - ./monitoring/promtail/promtail-config.yml:/etc/promtail/config.yml:ro
      - /var/run/docker.sock:/var/run/docker.sock:ro
    command: -config.file=/etc/promtail/config.yml
    restart: unless-stopped
    networks:
      - cap-network

  jaeger:
    image: jaegertracing/all-in-one:latest
    container_name: jaeger
    ports:
      - "16686:16686"  # Jaeger UI
      - "4317:4317"    # OTLP gRPC
      - "4318:4318"    # OTLP HTTP
      - "14268:14268"  # jaeger.thrift
      - "14269:14269"  # Health check
    environment:
      - COLLECTOR_OTLP_ENABLED=true
      - SPAN_STORAGE_TYPE=memory
      - LOG_LEVEL=info
    restart: unless-stopped
    networks:
      - cap-network

  node-exporter:
    image: prom/node-exporter:latest
    container_name: node-exporter
    ports:
      - "9100:9100"
    restart: unless-stopped
    networks:
      - cap-network

  cadvisor:
    image: gcr.io/cadvisor/cadvisor:latest
    container_name: cadvisor
    ports:
      - "8081:8080"
    volumes:
      - /:/rootfs:ro
      - /var/run:/var/run:ro
      - /sys:/sys:ro
      - /var/lib/docker:/var/lib/docker:ro
    restart: unless-stopped
    networks:
      - cap-network

volumes:
  prometheus-data:
  grafana-data:
  loki-data:

networks:
  cap-network:
    driver: bridge
```

**Usage:**

```bash
# Start stack
docker compose up -d

# View logs
docker compose logs -f cap-verifier-api

# Check status
docker compose ps

# Stop stack
docker compose down

# Stop and remove volumes
docker compose down -v
```

### 2.4 Multi-Platform Builds

**Build for multiple architectures (AMD64 + ARM64):**

```bash
# Setup buildx (once)
docker buildx create --name multiplatform --use

# Build and push multi-platform image
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t registry.example.com/cap-agent:0.11.0 \
  --push \
  .

# Verify manifest
docker buildx imagetools inspect registry.example.com/cap-agent:0.11.0
```

### 2.5 Mac-Specific Instructions (M1/M2)

**Apple Silicon considerations:**

```bash
# Build native ARM64 image
docker build --platform linux/arm64 -t cap-agent:0.11.0-arm64 .

# Build AMD64 image (emulated, slower)
docker build --platform linux/amd64 -t cap-agent:0.11.0-amd64 .

# Check platform
docker inspect cap-agent:0.11.0 | grep Architecture

# Run with explicit platform
docker run --platform linux/arm64 -p 8080:8080 cap-agent:0.11.0-arm64
```

**Known Issues:**
- SQLite performance may vary between ARM64/AMD64
- Rosetta 2 emulation adds ~20% overhead for AMD64 images
- Prefer native ARM64 builds for best performance

### 2.6 Container Volumes

**Persistent data storage:**

| Mount Path | Purpose | Access Mode | Required |
|------------|---------|-------------|----------|
| `/app/build` | Registry DB, BLOB Store, Audit Log | Read-Write | ✅ Yes |
| `/app/keys` | Ed25519 Keys | Read-Only | ✅ Yes |
| `/app/config` | Configuration files | Read-Only | ⚠️ Optional |
| `/app/examples` | Example data | Read-Only | ⚠️ Optional |
| `/tmp` | Temporary files | Read-Write | ✅ Yes (tmpfs) |

**Recommended volume configuration:**

```yaml
volumes:
  - ./build:/app/build              # Read-Write (Database)
  - ./keys:/app/keys:ro             # Read-Only (Keys)
  - ./config:/app/config:ro         # Read-Only (Config)
```

### 2.7 Image Size Optimization

**Target: < 100 MB**

| Strategy | Size Reduction | Effort |
|----------|----------------|--------|
| Multi-stage build | ~70% | Low |
| Strip binary (`strip`) | ~30% | Low |
| Alpine base | ~50 MB saved | Medium |
| Distroless base | ~20 MB saved | Medium |
| Compress layers | ~10-15% | High |

**Current image sizes:**
- Alpine: ~50 MB
- Distroless: ~80 MB
- Standard Debian: ~180 MB

---

## 3. Kubernetes Deployment

### 3.1 Basic Kubernetes Deployment

**Namespace:**

```yaml
# kubernetes/namespace.yml
apiVersion: v1
kind: Namespace
metadata:
  name: cap-system
  labels:
    name: cap-system
```

**Deployment:**

```yaml
# kubernetes/deployment.yml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cap-verifier-api
  namespace: cap-system
  labels:
    app: cap-verifier-api
    version: v0.11.0
spec:
  replicas: 2
  selector:
    matchLabels:
      app: cap-verifier-api
  template:
    metadata:
      labels:
        app: cap-verifier-api
        version: v0.11.0
    spec:
      containers:
      - name: cap-api
        image: cap-agent:0.11.0
        imagePullPolicy: IfNotPresent
        ports:
        - containerPort: 8080
          name: http
          protocol: TCP
        env:
        - name: RUST_LOG
          value: "info,cap_agent=debug"
        - name: CAP_BIND_ADDRESS
          value: "0.0.0.0:8080"
        - name: POLICY_STORE_BACKEND
          value: "sqlite"
        - name: POLICY_DB_PATH
          value: "/app/build/policies.sqlite"
        volumeMounts:
        - name: build-data
          mountPath: /app/build
        - name: keys
          mountPath: /app/keys
          readOnly: true
        - name: config
          mountPath: /app/config
          readOnly: true
        - name: tmp
          mountPath: /tmp
        resources:
          limits:
            cpu: "500m"
            memory: "512Mi"
          requests:
            cpu: "100m"
            memory: "128Mi"
        livenessProbe:
          httpGet:
            path: /healthz
            port: http
          initialDelaySeconds: 30
          periodSeconds: 30
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /readyz
            port: http
          initialDelaySeconds: 10
          periodSeconds: 10
          timeoutSeconds: 3
          failureThreshold: 3
      volumes:
      - name: build-data
        persistentVolumeClaim:
          claimName: cap-data-pvc
      - name: keys
        secret:
          secretName: cap-agent-key
      - name: config
        configMap:
          name: cap-config
      - name: tmp
        emptyDir: {}
```

**Service:**

```yaml
# kubernetes/service.yml
apiVersion: v1
kind: Service
metadata:
  name: cap-verifier-api
  namespace: cap-system
  labels:
    app: cap-verifier-api
spec:
  type: ClusterIP
  selector:
    app: cap-verifier-api
  ports:
  - name: http
    port: 80
    targetPort: 8080
    protocol: TCP
```

**PersistentVolumeClaim:**

```yaml
# kubernetes/pvc.yml
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
  storageClassName: standard  # Adjust based on cluster
```

**ConfigMap:**

```yaml
# kubernetes/configmap.yml
apiVersion: v1
kind: ConfigMap
metadata:
  name: cap-config
  namespace: cap-system
data:
  RUST_LOG: "info,cap_agent=debug"
  CAP_BIND_ADDRESS: "0.0.0.0:8080"
  POLICY_STORE_BACKEND: "sqlite"
  POLICY_DB_PATH: "/app/build/policies.sqlite"
```

**Apply manifests:**

```bash
kubectl apply -f kubernetes/namespace.yml
kubectl apply -f kubernetes/configmap.yml
kubectl apply -f kubernetes/pvc.yml
kubectl apply -f kubernetes/deployment.yml
kubectl apply -f kubernetes/service.yml

# Check deployment
kubectl get all -n cap-system
kubectl logs -f deployment/cap-verifier-api -n cap-system
```

### 3.2 Production Kubernetes Deployment

**Production-hardened deployment with Security Context:**

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cap-verifier-api
  namespace: cap-system
spec:
  replicas: 3  # Production: 3+ replicas
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 1
      maxSurge: 1
  template:
    spec:
      serviceAccountName: cap-verifier-sa
      securityContext:
        runAsNonRoot: true
        runAsUser: 65532
        runAsGroup: 65532
        fsGroup: 65532
        seccompProfile:
          type: RuntimeDefault
      containers:
      - name: cap-api
        image: gcr.io/your-registry/cap-agent:0.11.0
        imagePullPolicy: Always
        ports:
        - containerPort: 8443
          name: https
          protocol: TCP
        securityContext:
          allowPrivilegeEscalation: false
          readOnlyRootFilesystem: true
          capabilities:
            drop:
            - ALL
        resources:
          limits:
            cpu: "1000m"
            memory: "1Gi"
          requests:
            cpu: "200m"
            memory: "256Mi"
        livenessProbe:
          httpGet:
            path: /healthz
            port: https
            scheme: HTTPS
          initialDelaySeconds: 30
          periodSeconds: 30
          timeoutSeconds: 10
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /readyz
            port: https
            scheme: HTTPS
          initialDelaySeconds: 10
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        volumeMounts:
        - name: build-data
          mountPath: /app/build
        - name: keys
          mountPath: /app/keys
          readOnly: true
        - name: tls
          mountPath: /etc/tls
          readOnly: true
        - name: mtls
          mountPath: /etc/mtls
          readOnly: true
        - name: tmp
          mountPath: /tmp
      volumes:
      - name: build-data
        persistentVolumeClaim:
          claimName: cap-data-pvc
      - name: keys
        secret:
          secretName: cap-agent-key
      - name: tls
        secret:
          secretName: cap-verifier-tls
      - name: mtls
        secret:
          secretName: cap-verifier-mtls
      - name: tmp
        emptyDir: {}
```

**NetworkPolicy (Ingress/Egress Restrictions):**

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: cap-verifier-network-policy
  namespace: cap-system
spec:
  podSelector:
    matchLabels:
      app: cap-verifier-api
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: ingress-nginx
    ports:
    - protocol: TCP
      port: 8443
  egress:
  - to:
    - namespaceSelector: {}
    ports:
    - protocol: TCP
      port: 53  # DNS
  - to:
    - namespaceSelector:
        matchLabels:
          name: monitoring
    ports:
    - protocol: TCP
      port: 9090  # Prometheus
```

### 3.3 Helm Charts

**Helm Chart structure:**

```
helm/cap-verifier/
├── Chart.yaml
├── values.yaml
├── templates/
│   ├── deployment.yaml
│   ├── service.yaml
│   ├── configmap.yaml
│   ├── serviceaccount.yaml
│   ├── networkpolicy.yaml
│   ├── ingress.yaml
│   ├── hpa.yaml
│   └── _helpers.tpl
└── README.md
```

**Chart.yaml:**

```yaml
apiVersion: v2
name: cap-verifier
description: CAP Verifier REST API Helm Chart
type: application
version: 0.11.0
appVersion: "0.11.0"
keywords:
  - lksg
  - compliance
  - verification
maintainers:
  - name: CAP Team
    email: cap-team@example.com
```

**values.yaml:**

```yaml
replicaCount: 2

image:
  repository: registry.example.com/cap/verifier
  pullPolicy: IfNotPresent
  tag: "v0.11.0"

service:
  type: ClusterIP
  port: 443
  targetPort: 8443

ingress:
  enabled: false
  className: "nginx"
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
  hosts:
    - host: cap-verifier.example.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: cap-verifier-tls
      hosts:
        - cap-verifier.example.com

resources:
  limits:
    cpu: 500m
    memory: 512Mi
  requests:
    cpu: 100m
    memory: 128Mi

autoscaling:
  enabled: false
  minReplicas: 2
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70

config:
  oauth:
    issuer: "https://auth.example.com"
    audience: "cap-verifier"
  policy:
    storeBackend: "sqlite"
    dbPath: "/app/build/policies.sqlite"
```

**Install Helm Chart:**

```bash
# Install
helm install cap-verifier ./helm/cap-verifier \
  --namespace cap-system \
  --create-namespace

# Install with custom values
helm install cap-verifier ./helm/cap-verifier \
  -f custom-values.yaml \
  --namespace cap-system

# Upgrade
helm upgrade cap-verifier ./helm/cap-verifier \
  --set image.tag=v0.12.0

# Uninstall
helm uninstall cap-verifier --namespace cap-system
```

### 3.4 Horizontal Pod Autoscaler (HPA)

```bash
# Create HPA
kubectl autoscale deployment cap-verifier-api \
  --cpu-percent=70 \
  --min=3 \
  --max=10 \
  -n cap-system

# Check HPA status
kubectl get hpa -n cap-system

# Describe HPA
kubectl describe hpa cap-verifier-api -n cap-system
```

**HPA YAML:**

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: cap-verifier-api-hpa
  namespace: cap-system
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: cap-verifier-api
  minReplicas: 3
  maxReplicas: 20
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

### 3.5 Ingress with TLS (Let's Encrypt)

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: cap-verifier-api
  namespace: cap-system
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/force-ssl-redirect: "true"
spec:
  ingressClassName: nginx
  tls:
  - hosts:
    - cap-verifier.example.com
    secretName: cap-verifier-tls  # Auto-generated by cert-manager
  rules:
  - host: cap-verifier.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: cap-verifier-api
            port:
              number: 443
```

**Install cert-manager:**

```bash
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.0/cert-manager.yaml

# Create ClusterIssuer
kubectl apply -f - <<EOF
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
```

### 3.6 Rolling Updates & Rollbacks

```bash
# Update image
kubectl set image deployment/cap-verifier-api \
  cap-api=cap-agent:0.12.0 \
  -n cap-system

# Check rollout status
kubectl rollout status deployment/cap-verifier-api -n cap-system

# Rollout history
kubectl rollout history deployment/cap-verifier-api -n cap-system

# Rollback to previous version
kubectl rollout undo deployment/cap-verifier-api -n cap-system

# Rollback to specific revision
kubectl rollout undo deployment/cap-verifier-api --to-revision=2 -n cap-system
```

### 3.7 Kyma Service Mesh (Future)

**Planned features:**

- Istio integration for Service Mesh
- mTLS via Istio sidecar
- Traffic management (canary deployments, A/B testing)
- Distributed tracing with Jaeger

**Status:** Not yet implemented (see ROADMAP)

### 3.8 Image Signing with Cosign (Future)

**Planned features:**

- Image signing with Cosign
- Signature verification in admission controller
- SBOM generation with syft

**Status:** Not yet implemented (see ROADMAP)

---

## 4. TLS/mTLS Configuration

### 4.1 TLS-Only Setup (Server Certificate)

#### Option A: TLS via Kubernetes Ingress (Recommended)

**TLS termination at Ingress Controller (nginx/Traefik):**

**Advantages:**
- ✅ Automatic certificate management (Let's Encrypt via cert-manager)
- ✅ No TLS code in application
- ✅ Centralized TLS configuration
- ✅ Easy certificate rotation

**Ingress configuration:**

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: cap-verifier-api
  namespace: cap-system
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
spec:
  tls:
  - hosts:
    - cap-api.example.com
    secretName: cap-api-tls  # Auto-generated via cert-manager
  rules:
  - host: cap-api.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: cap-verifier-api
            port:
              number: 80
```

#### Option B: TLS in Container (Advanced)

**TLS termination in CAP Verifier API binary:**

**Prerequisites:**
1. TLS certificate and private key (PEM format)
2. PKCS#8 private key format

**Generate self-signed certificate (Development):**

```bash
# Generate private key
openssl genrsa -out server.key 4096

# Convert to PKCS#8 format (required by rustls)
openssl pkcs8 -topk8 -inform PEM -outform PEM \
  -in server.key -out server-pkcs8.key -nocrypt

# Generate certificate
openssl req -new -x509 -key server-pkcs8.key \
  -out server.crt -days 365 \
  -subj "/CN=cap-verifier"

# Create Kubernetes secret
kubectl create secret tls cap-verifier-tls \
  --cert=server.crt \
  --key=server-pkcs8.key \
  -n cap-system
```

**Run with TLS:**

```bash
# Docker
docker run -d \
  -p 8443:8443 \
  -v $(pwd)/certs:/certs:ro \
  cap-agent:0.11.0 \
  cap-verifier-api \
  --bind 0.0.0.0:8443 \
  --tls \
  --tls-cert /certs/server.crt \
  --tls-key /certs/server-pkcs8.key

# Test TLS
curl https://localhost:8443/healthz --insecure
openssl s_client -connect localhost:8443 -showcerts
```

### 4.2 Mutual TLS (mTLS)

**Client certificate validation:**

#### Step 1: Create CA certificate

```bash
# Generate CA private key
openssl genrsa -out ca.key 4096

# Generate CA certificate (10 years validity)
openssl req -x509 -new -key ca.key \
  -out ca.crt -days 3650 \
  -subj "/CN=CAP-CA"
```

#### Step 2: Generate client certificate

```bash
# Generate client private key
openssl genrsa -out client.key 4096

# Generate CSR
openssl req -new -key client.key \
  -out client.csr \
  -subj "/CN=cap-client"

# Sign with CA
openssl x509 -req -in client.csr \
  -CA ca.crt -CAkey ca.key -CAcreateserial \
  -out client.crt -days 365
```

#### Step 3: Configure mTLS

```bash
# Create CA secret
kubectl create secret generic cap-verifier-mtls \
  --from-file=ca.crt=ca.crt \
  -n cap-system

# Run with mTLS
docker run -d \
  -p 8443:8443 \
  -v $(pwd)/certs:/certs:ro \
  cap-agent:0.11.0 \
  cap-verifier-api \
  --bind 0.0.0.0:8443 \
  --tls \
  --tls-cert /certs/server.crt \
  --tls-key /certs/server-pkcs8.key \
  --mtls \
  --tls-ca /certs/ca.crt

# Test mTLS
curl https://localhost:8443/healthz \
  --cert client.crt \
  --key client.key \
  --cacert ca.crt
```

### 4.3 Certificate Management

#### Let's Encrypt (Production)

**cert-manager ClusterIssuer:**

```yaml
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
```

**Certificate resource:**

```yaml
apiVersion: cert-manager.io/v1
kind: Certificate
metadata:
  name: cap-verifier-tls
  namespace: cap-system
spec:
  secretName: cap-verifier-tls
  issuerRef:
    name: letsencrypt-prod
    kind: ClusterIssuer
  dnsNames:
  - cap-verifier.example.com
```

**Check certificate:**

```bash
kubectl get certificate -n cap-system
kubectl describe certificate cap-verifier-tls -n cap-system
```

#### Self-Signed (Development)

**Quick self-signed certificate:**

```bash
openssl req -x509 -newkey rsa:4096 \
  -keyout server.key -out server.crt \
  -days 365 -nodes \
  -subj "/CN=localhost"
```

#### Certificate Rotation

**Automatic rotation via cert-manager:**

```yaml
apiVersion: cert-manager.io/v1
kind: Certificate
spec:
  renewBefore: 720h  # Renew 30 days before expiry
```

**Manual rotation:**

```bash
# Delete old certificate
kubectl delete certificate cap-verifier-tls -n cap-system
kubectl delete secret cap-verifier-tls -n cap-system

# Reapply certificate resource
kubectl apply -f certificate.yml

# Wait for cert-manager
kubectl get certificate -n cap-system -w
```

### 4.4 Testing TLS Connections

**OpenSSL s_client:**

```bash
# Test TLS handshake
openssl s_client -connect localhost:8443 -showcerts

# Test with SNI
openssl s_client -connect cap-verifier.example.com:443 \
  -servername cap-verifier.example.com

# Test certificate chain
openssl s_client -connect localhost:8443 -showcerts | \
  openssl x509 -text -noout

# Test cipher suites
nmap --script ssl-enum-ciphers -p 8443 localhost
```

**curl with TLS:**

```bash
# Insecure (skip verification)
curl https://localhost:8443/healthz --insecure

# With CA certificate
curl https://localhost:8443/healthz --cacert ca.crt

# With client certificate (mTLS)
curl https://localhost:8443/healthz \
  --cert client.crt \
  --key client.key \
  --cacert ca.crt
```

---

## 5. Monitoring & Observability

### 5.1 Prometheus Setup

**Prometheus scrape configuration:**

`monitoring/prometheus/prometheus.yml`:

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'cap-verifier-api'
    static_configs:
      - targets: ['cap-verifier-api:8080']
    scrape_interval: 10s
    metrics_path: '/metrics'

  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  - job_name: 'node-exporter'
    static_configs:
      - targets: ['node-exporter:9100']

  - job_name: 'cadvisor'
    static_configs:
      - targets: ['cadvisor:8080']
```

**Prometheus Alerting Rules:**

`monitoring/prometheus/alerts/cap-verifier-rules.yml`:

```yaml
groups:
- name: cap_verifier_alerts
  interval: 30s
  rules:
  # Critical Alerts
  - alert: CapVerifierApiDown
    expr: up{job="cap-verifier-api"} == 0
    for: 2m
    labels:
      severity: critical
    annotations:
      summary: "CAP Verifier API is down"
      description: "API has been unavailable for more than 2 minutes."

  - alert: HighErrorRate
    expr: rate(cap_verifier_requests_total{result="fail"}[5m]) > 0.05
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "High error rate detected"
      description: "Error rate is {{ $value | humanizePercentage }}"

  # Warning Alerts
  - alert: ElevatedErrorRate
    expr: rate(cap_verifier_requests_total{result="fail"}[5m]) > 0.01
    for: 10m
    labels:
      severity: warning
    annotations:
      summary: "Elevated error rate"
      description: "Error rate is {{ $value | humanizePercentage }}"

  - alert: LowCacheHitRatio
    expr: cap_cache_hit_ratio < 0.5
    for: 15m
    labels:
      severity: warning
    annotations:
      summary: "Low cache hit ratio"
      description: "Cache hit ratio is {{ $value | humanizePercentage }}"
```

**Deploy Prometheus in Kubernetes:**

```bash
# Using Prometheus Operator
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm install prometheus prometheus-community/kube-prometheus-stack \
  --namespace monitoring \
  --create-namespace
```

**ServiceMonitor (Prometheus Operator):**

```yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: cap-verifier
  namespace: cap-system
spec:
  selector:
    matchLabels:
      app: cap-verifier-api
  endpoints:
  - port: http
    path: /metrics
    interval: 30s
```

### 5.2 Grafana Dashboards

**Grafana datasource configuration:**

`monitoring/grafana/provisioning/datasources/prometheus.yml`:

```yaml
apiVersion: 1
datasources:
- name: Prometheus
  type: prometheus
  access: proxy
  url: http://prometheus:9090
  isDefault: true
  editable: false
```

**Dashboard provisioning:**

`monitoring/grafana/provisioning/dashboards/dashboards.yml`:

```yaml
apiVersion: 1
providers:
- name: 'default'
  orgId: 1
  folder: ''
  type: file
  disableDeletion: false
  updateIntervalSeconds: 10
  allowUiUpdates: true
  options:
    path: /etc/grafana/provisioning/dashboards
```

**Key Panels for CAP Verifier Dashboard:**

1. **Request Rate** (QPS)
   ```promql
   rate(cap_verifier_requests_total[5m])
   ```

2. **Error Rate**
   ```promql
   rate(cap_verifier_requests_total{result="fail"}[5m]) /
   rate(cap_verifier_requests_total[5m])
   ```

3. **Request Latency (P95)**
   ```promql
   histogram_quantile(0.95,
     sum(rate(cap_verifier_request_duration_seconds_bucket[5m])) by (le))
   ```

4. **Cache Hit Ratio**
   ```promql
   cap_cache_hit_ratio
   ```

5. **Auth Failures**
   ```promql
   rate(cap_auth_token_validation_failures_total[5m])
   ```

**Access Grafana:**

```bash
# Port-forward
kubectl port-forward -n monitoring svc/prometheus-grafana 3000:80

# Open browser
open http://localhost:3000

# Login: admin / prom-operator
```

### 5.3 Loki (Log Aggregation)

**Loki configuration:**

`monitoring/loki/loki-config.yml`:

```yaml
auth_enabled: false

server:
  http_listen_port: 3100

ingester:
  lifecycler:
    ring:
      kvstore:
        store: inmemory
      replication_factor: 1
  chunk_idle_period: 5m
  chunk_retain_period: 30s

schema_config:
  configs:
  - from: 2023-01-01
    store: boltdb-shipper
    object_store: filesystem
    schema: v11
    index:
      prefix: index_
      period: 24h

storage_config:
  boltdb_shipper:
    active_index_directory: /loki/boltdb-shipper-active
    cache_location: /loki/boltdb-shipper-cache
    shared_store: filesystem
  filesystem:
    directory: /loki/chunks

compactor:
  working_directory: /loki/boltdb-shipper-compactor
  compaction_interval: 10m
  retention_enabled: true
  retention_delete_delay: 2h
  retention_delete_worker_count: 150

limits_config:
  retention_period: 744h  # 31 days
  max_query_length: 721h  # 30 days
  ingestion_rate_mb: 10
  ingestion_burst_size_mb: 20

chunk_store_config:
  max_look_back_period: 0s

table_manager:
  retention_deletes_enabled: true
  retention_period: 744h

query_range:
  results_cache:
    cache:
      embedded_cache:
        enabled: true
        max_size_mb: 100
```

**Promtail configuration (Log Collection):**

`monitoring/promtail/promtail-config.yml`:

```yaml
server:
  http_listen_port: 9080
  grpc_listen_port: 0

positions:
  filename: /tmp/positions.yaml

clients:
  - url: http://loki:3100/loki/api/v1/push

scrape_configs:
  # Docker logs
  - job_name: cap-verifier-api
    docker_sd_configs:
      - host: unix:///var/run/docker.sock
        refresh_interval: 5s
    relabel_configs:
      - source_labels: ['__meta_docker_container_label_app']
        target_label: 'app'
      - source_labels: ['__meta_docker_container_name']
        target_label: 'container_name'
    pipeline_stages:
      - json:
          expressions:
            timestamp: timestamp
            level: level
            message: message
            target: target
      - labels:
          level:
      - timestamp:
          source: timestamp
          format: RFC3339Nano
      - static_labels:
          app: cap-verifier-api
          component: backend
          environment: production

  # Kubernetes pods
  - job_name: kubernetes-pods
    kubernetes_sd_configs:
      - role: pod
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_label_app]
        action: keep
        regex: cap-verifier-api
      - source_labels: [__meta_kubernetes_namespace]
        target_label: namespace
      - source_labels: [__meta_kubernetes_pod_name]
        target_label: pod_name
```

**Query logs in Grafana:**

```logql
# All logs from cap-verifier-api
{app="cap-verifier-api"}

# Error logs
{app="cap-verifier-api"} |= "level=error"

# Auth failures
{app="cap-verifier-api"} |= "auth_failure"

# Filter by trace_id
{app="cap-verifier-api"} | json | trace_id="abc123"
```

### 5.4 Jaeger (Distributed Tracing)

**Jaeger all-in-one configuration:**

```bash
# Run Jaeger
docker run -d \
  --name jaeger \
  -p 16686:16686 \
  -p 4317:4317 \
  -p 4318:4318 \
  jaegertracing/all-in-one:latest
```

**Access Jaeger UI:**

```bash
open http://localhost:16686
```

**OTLP Exporter configuration (Future):**

```yaml
# values.yaml (Helm)
config:
  tracing:
    enabled: true
    endpoint: "http://jaeger-collector:4317"
    sample_rate: 0.1  # 10% sampling
```

**Correlation: Logs → Traces:**

Grafana derived field for Loki → Jaeger correlation:

```yaml
# Grafana datasource config
derivedFields:
  - name: TraceID
    matcherRegex: "trace_id\":\"(\\w+)"
    url: "http://localhost:16686/trace/$${__value.raw}"
    datasourceUid: jaeger
```

### 5.5 SLO/SLI Monitoring

**Defined SLIs:**

1. **Availability SLI:** `ok_requests / total_requests`
2. **Error Rate SLI:** `fail_requests / total_requests`
3. **Auth Success SLI:** `(total_requests - auth_failures) / total_requests`
4. **Cache Hit Rate SLI:** `cap_cache_hit_ratio`

**Defined SLOs:**

| SLO | Target | Time Window | Error Budget |
|-----|--------|-------------|--------------|
| Availability | 99.9% | 30 days | 43.2 min/month |
| Error Rate | < 0.1% | 30 days | 0.1% |
| Auth Success | 99.95% | 30 days | 0.05% |
| Cache Hit Rate | > 70% | 7 days | 30% |

**Prometheus queries for SLOs:**

```promql
# Availability SLO (99.9%)
sum(rate(cap_verifier_requests_total{result="ok"}[30d])) /
sum(rate(cap_verifier_requests_total[30d]))

# Error Rate SLO (< 0.1%)
sum(rate(cap_verifier_requests_total{result="fail"}[30d])) /
sum(rate(cap_verifier_requests_total[30d]))

# Error Budget Remaining (Availability)
1 - (
  (1 - sum(rate(cap_verifier_requests_total{result="ok"}[30d])) /
       sum(rate(cap_verifier_requests_total[30d]))) / 0.001
)
```

**Grafana SLO Dashboard:**

See `monitoring/grafana/dashboards/slo-monitoring.json` (17 panels)

---

## 6. Configuration Reference

### 6.1 Environment Variables

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `RUST_LOG` | Logging level | `info` | ⚠️ Optional |
| `RUST_BACKTRACE` | Stack traces on panic | `0` | ⚠️ Optional |
| `CAP_BIND_ADDRESS` | API bind address | `127.0.0.1:8080` | ⚠️ Optional |
| `POLICY_STORE_BACKEND` | Policy storage (memory/sqlite) | `memory` | ⚠️ Optional |
| `POLICY_DB_PATH` | SQLite database path | `build/policies.sqlite` | ⚠️ Optional |
| `CAP_REGISTRY_PATH` | Registry database path | `/app/build/registry.sqlite` | ⚠️ Optional |
| `CAP_BLOB_STORE_PATH` | BLOB store path | `/app/build/test_blobs.sqlite` | ⚠️ Optional |
| `CAP_AUDIT_LOG` | Audit log path | `/app/build/audit_chain.jsonl` | ⚠️ Optional |
| `CAP_KEYS_DIR` | Ed25519 keys directory | `/app/keys` | ⚠️ Optional |
| `OAUTH2_ISSUER` | OAuth2 issuer URL | - | ⚠️ Optional |
| `OAUTH2_AUDIENCE` | OAuth2 audience | `cap-verifier` | ⚠️ Optional |

**Example `.env` file:**

```bash
# Logging
RUST_LOG=info,cap_agent=debug
RUST_BACKTRACE=1

# API Binding
CAP_BIND_ADDRESS=0.0.0.0:8080

# Storage Paths
POLICY_STORE_BACKEND=sqlite
POLICY_DB_PATH=/app/build/policies.sqlite
CAP_REGISTRY_PATH=/app/build/registry.sqlite
CAP_BLOB_STORE_PATH=/app/build/test_blobs.sqlite
CAP_AUDIT_LOG=/app/build/audit_chain.jsonl
CAP_KEYS_DIR=/app/keys

# OAuth2 (Production)
OAUTH2_ISSUER=https://auth.example.com
OAUTH2_AUDIENCE=cap-verifier
```

### 6.2 Runtime Flags (CLI)

**cap-verifier-api binary flags:**

| Flag | Description | Example |
|------|-------------|---------|
| `--bind` | Bind address | `--bind 0.0.0.0:8443` |
| `--tls` | Enable TLS | `--tls` |
| `--tls-cert` | TLS certificate path | `--tls-cert certs/server.crt` |
| `--tls-key` | TLS private key path | `--tls-key certs/server.key` |
| `--mtls` | Enable mutual TLS | `--mtls` |
| `--tls-ca` | CA certificate for mTLS | `--tls-ca certs/ca.crt` |
| `--rate-limit` | Requests per minute | `--rate-limit 100` |
| `--rate-limit-burst` | Burst size | `--rate-limit-burst 120` |

**Full example:**

```bash
cap-verifier-api \
  --bind 0.0.0.0:8443 \
  --tls \
  --tls-cert /certs/server.crt \
  --tls-key /certs/server.key \
  --mtls \
  --tls-ca /certs/ca.crt \
  --rate-limit 100 \
  --rate-limit-burst 120
```

### 6.3 Policy Store Backend Selection

**In-Memory (Development):**

```bash
# Environment variable
export POLICY_STORE_BACKEND=memory

# Docker
docker run -e POLICY_STORE_BACKEND=memory cap-agent:0.11.0

# Kubernetes ConfigMap
data:
  POLICY_STORE_BACKEND: "memory"
```

**SQLite (Production):**

```bash
# Environment variable
export POLICY_STORE_BACKEND=sqlite
export POLICY_DB_PATH=/data/policies.sqlite

# Docker with volume
docker run \
  -e POLICY_STORE_BACKEND=sqlite \
  -e POLICY_DB_PATH=/data/policies.sqlite \
  -v /data:/data \
  cap-agent:0.11.0

# Kubernetes with PVC
env:
- name: POLICY_STORE_BACKEND
  value: "sqlite"
- name: POLICY_DB_PATH
  value: "/app/build/policies.sqlite"
volumeMounts:
- name: build-data
  mountPath: /app/build
```

### 6.4 Resource Limits (Kubernetes)

**Recommended resource limits:**

| Environment | CPU Request | CPU Limit | Memory Request | Memory Limit |
|-------------|-------------|-----------|----------------|--------------|
| Development | 100m | 500m | 128Mi | 512Mi |
| Staging | 200m | 1000m | 256Mi | 1Gi |
| Production | 500m | 2000m | 512Mi | 2Gi |
| High Load | 1000m | 4000m | 1Gi | 4Gi |

**Apply limits in deployment.yaml:**

```yaml
resources:
  limits:
    cpu: "2000m"
    memory: "2Gi"
  requests:
    cpu: "500m"
    memory: "512Mi"
```

---

## 7. Troubleshooting

### 7.1 Container Won't Start

**Symptoms:**

```bash
docker compose up
# Error: failed to start container
```

**Diagnosis:**

```bash
# Check logs
docker compose logs cap-verifier-api

# Check image
docker images | grep cap-agent

# Check volumes
docker volume ls

# Check ports
lsof -i :8080  # Port already in use?

# Check volume permissions
ls -la build/ keys/
```

**Solutions:**

```bash
# Rebuild without cache
docker compose build --no-cache

# Check port conflicts
docker ps -a

# Fix volume permissions
chmod -R 755 build/
chmod 600 keys/company.ed25519

# Clean Docker resources
docker system prune -a
```

### 7.2 Health Check Failed

**Symptoms:**

```bash
curl http://localhost:8080/healthz
# Connection refused
```

**Diagnosis:**

```bash
# Check container status
docker ps -a

# Check logs
docker logs cap-verifier-api

# Check binding
docker exec cap-verifier-api ss -tlnp | grep 8080

# Check environment
docker exec cap-verifier-api env | grep CAP_BIND
```

**Solutions:**

```bash
# Restart container
docker compose restart cap-verifier-api

# Check firewall
sudo ufw status

# Test inside container
docker exec -it cap-verifier-api sh
wget -qO- http://localhost:8080/healthz
```

### 7.3 Kubernetes Pod CrashLoopBackOff

**Symptoms:**

```bash
kubectl get pods -n cap-system
# NAME                   READY   STATUS             RESTARTS
# cap-verifier-api-xxx   0/1     CrashLoopBackOff   5
```

**Diagnosis:**

```bash
# Check logs
kubectl logs cap-verifier-api-xxx -n cap-system

# Check previous logs (after crash)
kubectl logs -p cap-verifier-api-xxx -n cap-system

# Check events
kubectl describe pod cap-verifier-api-xxx -n cap-system

# Check configmap
kubectl get cm cap-config -n cap-system -o yaml

# Check secrets
kubectl get secrets cap-agent-key -n cap-system
```

**Solutions:**

```bash
# Check PVC
kubectl get pvc -n cap-system
kubectl describe pvc cap-data-pvc -n cap-system

# Check resource limits
kubectl describe pod cap-verifier-api-xxx -n cap-system | grep -A 10 "Limits"

# Restart pod
kubectl delete pod cap-verifier-api-xxx -n cap-system

# Check secret contents (keys exist?)
kubectl exec -it cap-verifier-api-xxx -n cap-system -- ls -la /app/keys
```

### 7.4 TLS Certificate Error

**Symptoms:**

```bash
curl https://cap-api.example.com/healthz
# SSL certificate problem: unable to get local issuer certificate
```

**Diagnosis:**

```bash
# Check certificate (Kubernetes)
kubectl get certificate -n cap-system
kubectl describe certificate cap-verifier-tls -n cap-system

# Check cert-manager logs
kubectl logs -n cert-manager deployment/cert-manager

# Test certificate
openssl s_client -connect cap-api.example.com:443 -showcerts

# Check certificate expiry
openssl s_client -connect localhost:8443 -showcerts 2>&1 | \
  openssl x509 -noout -dates
```

**Solutions:**

```bash
# Delete and recreate certificate
kubectl delete certificate cap-verifier-tls -n cap-system
kubectl delete secret cap-verifier-tls -n cap-system

# Reapply ingress
kubectl delete ingress cap-verifier-api -n cap-system
kubectl apply -f kubernetes/ingress.yml

# Wait for cert-manager
kubectl get certificate -n cap-system -w

# Check ClusterIssuer
kubectl get clusterissuer
kubectl describe clusterissuer letsencrypt-prod
```

### 7.5 Performance Issues

**Symptoms:**

- Slow API responses (>2s)
- High CPU/Memory usage
- OOM Kills
- Timeouts

**Diagnosis:**

```bash
# Docker Stats
docker stats cap-verifier-api

# Kubernetes Metrics
kubectl top pods -n cap-system
kubectl top nodes

# Check resource limits
kubectl describe pod <pod-name> -n cap-system | grep -A 10 "Limits"

# Check logs for errors
kubectl logs -f deployment/cap-verifier-api -n cap-system | grep -i "error\|panic"
```

**Solutions:**

```bash
# Increase Docker resources
# Edit docker-compose.yml:
deploy:
  resources:
    limits:
      cpus: '4.0'
      memory: 4G

# Increase Kubernetes resources
# Edit kubernetes/deployment.yml:
resources:
  limits:
    cpu: "4000m"
    memory: "4Gi"
  requests:
    cpu: "1000m"
    memory: "1Gi"

# Apply changes
kubectl apply -f kubernetes/deployment.yml

# Scale horizontally
kubectl scale deployment cap-verifier-api --replicas=5 -n cap-system

# Enable HPA
kubectl autoscale deployment cap-verifier-api \
  --cpu-percent=70 \
  --min=3 \
  --max=10 \
  -n cap-system
```

### 7.6 Database Connection Issues

**Symptoms:**

- "Failed to open SQLite database"
- "Database is locked"
- Slow queries

**Diagnosis:**

```bash
# Check SQLite file
docker exec cap-verifier-api ls -la /app/build/policies.sqlite

# Check file permissions
docker exec cap-verifier-api stat /app/build/policies.sqlite

# Check for lock file
docker exec cap-verifier-api ls -la /app/build/*.sqlite-shm /app/build/*.sqlite-wal

# Check disk space
docker exec cap-verifier-api df -h /app/build
```

**Solutions:**

```bash
# Restart container (releases locks)
docker compose restart cap-verifier-api

# Check WAL mode
docker exec cap-verifier-api \
  sqlite3 /app/build/policies.sqlite "PRAGMA journal_mode;"

# Optimize database
docker exec cap-verifier-api \
  sqlite3 /app/build/policies.sqlite "VACUUM; PRAGMA optimize;"

# Backup and restore
docker exec cap-verifier-api \
  sqlite3 /app/build/policies.sqlite ".backup /app/build/policies.backup.sqlite"
```

### 7.7 Network Policy Issues

**Symptoms:**

- Pods can't communicate
- Ingress traffic blocked
- Prometheus can't scrape metrics

**Diagnosis:**

```bash
# Check NetworkPolicy
kubectl get networkpolicy -n cap-system
kubectl describe networkpolicy cap-verifier-network-policy -n cap-system

# Test connectivity
kubectl run debug --rm -it --image=alpine/curl -n cap-system -- /bin/sh
curl http://cap-verifier-api.cap-system.svc.cluster.local/healthz

# Check ingress controller logs
kubectl logs -n ingress-nginx deployment/ingress-nginx-controller
```

**Solutions:**

```bash
# Temporarily disable NetworkPolicy (debugging only!)
kubectl delete networkpolicy cap-verifier-network-policy -n cap-system

# Test connectivity
curl http://cap-verifier-api.cap-system.svc.cluster.local/healthz

# Update NetworkPolicy (allow ingress-nginx)
kubectl apply -f kubernetes/networkpolicy.yml

# Verify ingress labels
kubectl get namespaces --show-labels
kubectl get pods -n ingress-nginx --show-labels
```

### 7.8 WebUI Issues

**Symptoms:**

- WebUI can't connect to API
- CORS errors in browser console
- 401 Unauthorized

**Diagnosis:**

```bash
# Check API accessibility
curl http://localhost:8080/healthz

# Check CORS headers
curl -I http://localhost:8080/healthz

# Check browser console (F12)
# Look for CORS errors or network failures

# Check API logs
docker compose logs -f cap-verifier-api | grep -i "cors\|unauthorized"
```

**Solutions:**

```bash
# Verify API URL in WebUI
# Check webui/.env.development:
VITE_API_URL=http://localhost:8080

# Restart WebUI dev server
cd webui
npm run dev

# Test CORS preflight
curl -X OPTIONS http://localhost:8080/policy/compile \
  -H "Access-Control-Request-Method: POST" \
  -H "Access-Control-Request-Headers: authorization" \
  -v

# Check token validity
TOKEN="admin-tom"
curl -X POST http://localhost:8080/policy/compile \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"policy": {...}}'
```

---

## 8. Production Checklist

### 8.1 Pre-Deployment

**Security:**

- [ ] Keys NOT committed to Git
- [ ] Secrets stored in Kubernetes Secrets or Vault
- [ ] TLS certificates valid and trusted
- [ ] OAuth2 configured with Production IdP
- [ ] Image vulnerability scan passed (Trivy/Grype)
- [ ] SBOM generated (syft)
- [ ] CVEs reviewed and patched

**Infrastructure:**

- [ ] Image pushed to Production Registry
- [ ] Image signed (Cosign) and verified
- [ ] ConfigMaps configured for Production
- [ ] PersistentVolumes provisioned
- [ ] Resource limits tested (Load Testing)
- [ ] Network Policies configured
- [ ] Ingress configured with TLS
- [ ] DNS records created

**Configuration:**

- [ ] Environment variables set correctly
- [ ] Policy Store Backend selected (SQLite recommended)
- [ ] Logging level appropriate (warn/info in Production)
- [ ] Rate limiting configured
- [ ] OAuth2 Issuer/Audience set

### 8.2 Post-Deployment

**Health Checks:**

- [ ] Health probe returns 200 OK
- [ ] Readiness probe returns 200 OK
- [ ] All pods running and ready
- [ ] Ingress accessible externally

**Monitoring:**

- [ ] Logs structured and readable (JSON)
- [ ] Metrics exported to Prometheus
- [ ] Grafana dashboards imported
- [ ] Alerting configured (PagerDuty/Slack)
- [ ] SLO/SLI dashboards working

**Backup & Recovery:**

- [ ] Backup strategy for Secrets
- [ ] Database backups scheduled
- [ ] Disaster Recovery Plan documented
- [ ] Rollback procedure tested

### 8.3 Compliance (LkSG/GDPR)

- [ ] Audit log activated and persistent
- [ ] TLS/mTLS enforced
- [ ] Data residency requirements met
- [ ] GDPR compliance verified
- [ ] Penetration test performed
- [ ] Security audit completed

---

## Appendix: Migration Notes

**For users of older deployment documentation:**

| Old File | New Location | Notes |
|----------|--------------|-------|
| `README_DEPLOYMENT.md` | `DEPLOYMENT.md` Chapters 1, 4, 6, 7 | Consolidated |
| `DOCKER_DEPLOYMENT.md` | `DEPLOYMENT.md` Chapters 2.2, 2.3, 5 | Consolidated |
| Old `DEPLOYMENT.md` | `DEPLOYMENT.md` Chapters 2.2, 3.2, 3.3 | Expanded |

**All unique content has been preserved.**

---

## Further Resources

- **System Documentation:** [CLAUDE.md](../CLAUDE.md)
- **Monitoring Guide:** [monitoring/README.md](../../monitoring/README.md)
- **SLO Configuration:** [monitoring/slo/README.md](../../monitoring/slo/README.md)
- **OpenAPI Spec:** [openapi.yaml](../../openapi.yaml)
- **WebUI Documentation:** [webui/README.md](../../webui/README.md)
- **PRD:** [PRD_Docker_K8s_Container_CAP_Verifier.md](/Users/tomwesselmann/Desktop/PRD_Docker_K8s_Container_CAP_Verifier.md)

---

**Status:** ✅ Production-Ready
**Last Updated:** 2025-11-24
**Version:** v0.11.0
**Maintained By:** CAP Team

---

**Document Consolidation:**
This document consolidates 3 deployment guides (1626 lines total) into a single comprehensive resource.
All unique content from the original documents has been preserved.

**Consolidated:** 2025-11-24 by Claude Code
**Methodology:** CAP_CLAUDE_WORKFLOW_V2
