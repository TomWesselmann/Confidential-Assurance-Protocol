# 🐳 CAP Verifier - Container Deployment Guide

**Version:** v1.0.0
**Status:** Production-Ready
**Target:** On-prem & SAP BTP (Kyma)

---

## 📋 Übersicht

Dieser Guide beschreibt das vollständige Container-Deployment des CAP Verifier REST API für On-Premises und Kubernetes/Kyma Umgebungen.

### Architektur-Komponenten

```
┌─────────────────────────────────────────────────────────────┐
│  Kubernetes Cluster (On-Prem / Kyma)                       │
│                                                             │
│  ┌────────────────────────────────────────────────────┐   │
│  │  Ingress (TLS Termination)                         │   │
│  │  nginx / Istio / Traefik                           │   │
│  └───────────────────┬────────────────────────────────┘   │
│                      │                                     │
│  ┌───────────────────▼────────────────────────────────┐   │
│  │  Service (ClusterIP)                               │   │
│  │  cap-verifier:443 → Pod:8443                       │   │
│  └───────────────────┬────────────────────────────────┘   │
│                      │                                     │
│  ┌───────────────────▼────────────────────────────────┐   │
│  │  Deployment (2 Replicas, HPA-ready)                │   │
│  │                                                     │   │
│  │  ┌──────────────────────────────────────────────┐  │   │
│  │  │  Pod: cap-verifier                           │  │   │
│  │  │  ┌────────────────────────────────────────┐  │  │   │
│  │  │  │  Container: verifier                   │  │  │   │
│  │  │  │  Image: distroless/cc:nonroot          │  │  │   │
│  │  │  │  Binary: /app/cap-verifier-api         │  │  │   │
│  │  │  │  Port: 8443 (HTTP, later TLS)          │  │  │   │
│  │  │  │  User: nonroot (UID 65532)             │  │  │   │
│  │  │  │  CPU: 100m-500m, Mem: 128Mi-512Mi      │  │  │   │
│  │  │  │                                         │  │  │   │
│  │  │  │  Mounts:                                │  │  │   │
│  │  │  │  - /app/config (ConfigMap, RO)         │  │  │   │
│  │  │  │  - /etc/tls (Secret, RO)               │  │  │   │
│  │  │  │  - /etc/mtls (Secret, RO)              │  │  │   │
│  │  │  │  - /etc/keys (Secret, RO)              │  │  │   │
│  │  │  │  - /tmp (emptyDir)                     │  │  │   │
│  │  │  │                                         │  │  │   │
│  │  │  │  Probes:                                │  │  │   │
│  │  │  │  - Liveness: /healthz (30s)            │  │  │   │
│  │  │  │  - Readiness: /readyz (10s)            │  │  │   │
│  │  │  └────────────────────────────────────────┘  │  │   │
│  │  └──────────────────────────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  NetworkPolicy (Ingress/Egress Restrictions)        │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## 🔒 Container-Hardening

### Sicherheitsmerkmale

✅ **Non-Root User** (UID 65532 aus distroless/nonroot)
✅ **Read-Only Root Filesystem**
✅ **Dropped ALL Capabilities**
✅ **Seccomp Profile: RuntimeDefault**
✅ **No Privilege Escalation**
✅ **Distroless Base Image** (gcr.io/distroless/cc-debian12:nonroot)
✅ **Image Size ≤ 100 MB** (optimiert, stripped binary)
✅ **Network Policy** (Ingress/Egress Restrictions)

### Dockerfile (Multi-Stage Build)

```dockerfile
# Build Stage (rust:1.81-bookworm)
FROM rust:1.81-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --bin cap-verifier-api && \
    strip /src/target/release/cap-verifier-api

# Runtime Stage (distroless/cc:nonroot)
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

---

## 📦 Container Build & Push

### 1. Build Image Lokal

```bash
docker build -t cap-verifier:v1.0.0 .

# Größe prüfen
docker images cap-verifier:v1.0.0

# Erwartet: <100 MB
```

### 2. Tag & Push zu Registry

```bash
# Tag für Registry
docker tag cap-verifier:v1.0.0 registry.example.com/cap/verifier:v1.0.0

# Push
docker push registry.example.com/cap/verifier:v1.0.0
```

### 3. Image Signierung (Optional, empfohlen für Production)

```bash
# Mit Cosign signieren
cosign sign --key cosign.key registry.example.com/cap/verifier:v1.0.0

# Signatur verifizieren
cosign verify --key cosign.pub registry.example.com/cap/verifier:v1.0.0
```

---

## ☸️ Kubernetes Deployment

### Verzeichnisstruktur

```
agent/
├── k8s/                          # Plain Kubernetes Manifeste
│   ├── deployment.yaml           # Deployment (2 Replicas)
│   ├── service.yaml              # Service (ClusterIP)
│   ├── configmap.yaml            # App-Konfiguration
│   ├── secrets.example.yaml      # Secret-Templates
│   └── networkpolicy.yaml        # Network Policy
├── helm/                         # Helm Chart
│   └── cap-verifier/
│       ├── Chart.yaml
│       ├── values.yaml
│       ├── templates/
│       │   ├── deployment.yaml
│       │   ├── service.yaml
│       │   ├── configmap.yaml
│       │   ├── serviceaccount.yaml
│       │   ├── networkpolicy.yaml
│       │   ├── ingress.yaml
│       │   ├── hpa.yaml
│       │   └── _helpers.tpl
│       └── README.md
├── Dockerfile
├── .dockerignore
├── config/
│   └── app.yaml                  # Default App-Config
└── openapi/
    └── openapi.yaml              # OpenAPI 3.0 Spec
```

---

## 🚀 Deployment-Methoden

### Option 1: Plain Kubernetes (kubectl)

#### Schritt 1: Secrets erstellen

```bash
# TLS Certificate (Self-Signed für Testing)
openssl req -x509 -newkey rsa:4096 -keyout tls.key -out tls.crt \
  -days 365 -nodes -subj "/CN=cap-verifier"

kubectl create secret tls cap-verifier-tls \
  --cert=tls.crt --key=tls.key

# mTLS CA Certificate
openssl req -x509 -newkey rsa:4096 -keyout ca.key -out ca.crt \
  -days 3650 -nodes -subj "/CN=CAP-CA"

kubectl create secret generic cap-verifier-mtls \
  --from-file=ca.crt=ca.crt

# Ed25519 Keys (using cap-agent)
cargo run -- sign keygen --dir keys

kubectl create secret generic cap-agent-key \
  --from-file=agent.ed25519=keys/company.ed25519 \
  --from-file=agent.pub=keys/company.pub
```

#### Schritt 2: Deployment anwenden

```bash
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/deployment.yaml
kubectl apply -f k8s/service.yaml
kubectl apply -f k8s/networkpolicy.yaml
```

#### Schritt 3: Status prüfen

```bash
# Pods prüfen
kubectl get pods -l app=cap-verifier

# Logs anzeigen
kubectl logs -l app=cap-verifier -f

# Health-Check
kubectl port-forward svc/cap-verifier 8443:443
curl http://localhost:8443/healthz
```

---

### Option 2: Helm Chart (Empfohlen)

#### Installation

```bash
# Secrets erstellen (siehe oben)

# Helm Chart installieren
helm install cap-verifier ./helm/cap-verifier

# Mit custom values
helm install cap-verifier ./helm/cap-verifier -f custom-values.yaml

# In spezifischem Namespace
helm install cap-verifier ./helm/cap-verifier \
  --namespace cap-system --create-namespace
```

#### Custom Values (custom-values.yaml)

```yaml
replicaCount: 3

image:
  repository: registry.example.com/cap/verifier
  tag: "v1.0.0"

resources:
  limits:
    cpu: 1000m
    memory: 1Gi
  requests:
    cpu: 200m
    memory: 256Mi

autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 20
  targetCPUUtilizationPercentage: 70

config:
  oauth:
    issuer: "https://auth.basf.com"
    audience: "cap-verifier-prod"

ingress:
  enabled: true
  className: "nginx"
  hosts:
    - host: cap-verifier.basf.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: cap-verifier-tls
      hosts:
        - cap-verifier.basf.com
```

#### Upgrade

```bash
helm upgrade cap-verifier ./helm/cap-verifier \
  --set image.tag=v1.1.0
```

#### Uninstall

```bash
helm uninstall cap-verifier
```

---

## 🧪 Deployment Smoke Test

### Automatisiertes Smoke-Test-Skript

```bash
./scripts/deploy-smoke-test.sh registry.example.com/cap/verifier v1.0.0
```

**Was wird getestet:**
1. Docker Build
2. Image Push
3. Image-Größe (<100 MB)
4. K8s Deployment
5. Pod Readiness
6. Health/Readiness Probes

### Manuelle Tests

```bash
# Port-Forward
kubectl port-forward svc/cap-verifier 8443:443

# Health-Check
curl http://localhost:8443/healthz
# Expected: {"status":"OK","version":"0.11.0","build_hash":null}

# Readiness-Check
curl http://localhost:8443/readyz
# Expected: {"status":"OK","checks":[...]}

# OAuth2-geschützte Endpoints
TOKEN=$(cargo run --example generate_mock_token 2>&1 | grep "^eyJ" | head -1)

curl -X POST http://localhost:8443/policy/compile \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"policy": {...}}'
```

---

## 📊 Monitoring & Observability

### Logs

```bash
# Alle Pods
kubectl logs -l app=cap-verifier -f

# Einzelner Pod
kubectl logs cap-verifier-<pod-id> -f

# JSON-strukturiert, grep-bar
kubectl logs -l app=cap-verifier | jq .
```

### Metrics (Prometheus)

```yaml
# ServiceMonitor (Prometheus Operator)
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: cap-verifier
spec:
  selector:
    matchLabels:
      app: cap-verifier
  endpoints:
    - port: https
      path: /metrics
      interval: 30s
```

### Traces (Optional)

```bash
# OTLP Exporter konfigurieren (values.yaml)
config:
  tracing:
    enabled: true
    endpoint: "http://jaeger-collector:4317"
    sample_rate: 0.1
```

---

## 🔧 Troubleshooting

### Pods starten nicht

```bash
# Events prüfen
kubectl describe pod <pod-name>

# Secrets prüfen
kubectl get secrets
kubectl describe secret cap-verifier-tls

# ImagePullBackOff
kubectl describe pod <pod-name> | grep -A 10 "Events"
```

### Health Checks fehlschlagen

```bash
# Probe-Logs prüfen
kubectl logs <pod-name> | grep -E "healthz|readyz"

# Manuell testen (exec in Pod)
kubectl exec -it <pod-name> -- wget -qO- http://localhost:8443/healthz
```

### Network Policy Issues

```bash
# NetworkPolicy prüfen
kubectl get networkpolicy
kubectl describe networkpolicy cap-verifier-network-policy

# Debug Pod starten
kubectl run debug --rm -it --image=alpine/curl -- /bin/sh
curl http://cap-verifier.default.svc.cluster.local/healthz
```

---

## ✅ Production-Checkliste

### Pre-Deployment

- [ ] Image in Production Registry gepusht
- [ ] Image signiert (cosign)
- [ ] Secrets erstellt (TLS, mTLS, Ed25519)
- [ ] ConfigMap angepasst (OAuth2 Issuer, Audience)
- [ ] Resource Limits getestet (Load Testing)
- [ ] Network Policy konfiguriert

### Post-Deployment

- [ ] Health & Readiness Probes = 200 OK
- [ ] Logs strukturiert & lesbar (JSON)
- [ ] Metrics exportiert (Prometheus)
- [ ] Alerting konfiguriert (PagerDuty, Slack)
- [ ] Backup-Strategie für Secrets
- [ ] Disaster Recovery Plan dokumentiert

### Security

- [ ] Image-Scan durchgeführt (Trivy, Grype)
- [ ] SBOM generiert (syft)
- [ ] CVEs geprüft & gefixt
- [ ] Network Policy aktiv
- [ ] Pod Security Standards enforced
- [ ] OAuth2 mit Production IdP (Keycloak, Auth0)

### Compliance (BASF/EuroDat)

- [ ] Audit-Log aktiviert
- [ ] TLS/mTLS konfiguriert
- [ ] Data Residency eingehalten (On-Prem)
- [ ] GDPR-Compliance geprüft
- [ ] Penetration Test durchgeführt

---

## 📚 Weitere Ressourcen

- **PRD:** [PRD_Docker_K8s_Container_CAP_Verifier.md](/Users/tomwesselmann/Desktop/PRD_Docker_K8s_Container_CAP_Verifier.md)
- **System Docs:** [CLAUDE.md](CLAUDE.md)
- **Helm Chart:** [helm/cap-verifier/README.md](helm/cap-verifier/README.md)
- **OpenAPI Spec:** [openapi/openapi.yaml](openapi/openapi.yaml)

---

**Status:** ✅ Production-Ready
**Letzte Aktualisierung:** 2025-11-07
**Autor:** CAP Team
