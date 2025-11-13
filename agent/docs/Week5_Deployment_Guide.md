# Week 5: Deployment Guide - CAP Verifier API

**Version:** v0.11.0
**Date:** 2025-11-10
**Status:** Production-Ready

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Environment Setup](#environment-setup)
3. [Helm Deployment](#helm-deployment)
4. [Configuration](#configuration)
5. [TLS/mTLS Setup](#tlsmtls-setup)
6. [OAuth2 Integration](#oauth2-integration)
7. [Monitoring](#monitoring)
8. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### Required Tools

- **Kubernetes:** v1.24+ (with Ingress controller)
- **Helm:** v3.10+
- **kubectl:** v1.24+
- **Docker:** v20.10+ (for building images)

### Infrastructure Requirements

**Development:**
- 1 node, 2 CPU cores, 4GB RAM
- No TLS/mTLS required
- Single replica

**Staging:**
- 2 nodes, 4 CPU cores, 8GB RAM
- TLS termination (Ingress)
- HPA: 2-5 replicas

**Production:**
- 3+ nodes, 8 CPU cores, 16GB RAM
- mTLS required
- HPA: 3-20 replicas
- Anti-affinity rules (pod distribution)

---

## Environment Setup

### Build Docker Image

```bash
# From project root
docker build -t cap-verifier-api:0.11.0 .

# Tag for registry
docker tag cap-verifier-api:0.11.0 your-registry.example.com/cap-verifier-api:0.11.0

# Push to registry
docker push your-registry.example.com/cap-verifier-api:0.11.0
```

### Create Namespace

```bash
kubectl create namespace cap-verifier

# Set context
kubectl config set-context --current --namespace=cap-verifier
```

---

## Helm Deployment

### Install Chart (Development)

```bash
cd helm

helm install cap-verifier . \
  --namespace cap-verifier \
  --values values-dev.yaml \
  --set image.repository=your-registry.example.com/cap-verifier-api \
  --set image.tag=0.11.0
```

**Development Configuration:**
- 1 replica
- No OAuth2 authentication
- No TLS/mTLS
- Debug logging
- No rate limiting

### Install Chart (Staging)

```bash
helm install cap-verifier . \
  --namespace cap-verifier \
  --values values-stage.yaml \
  --set image.repository=your-registry.example.com/cap-verifier-api \
  --set image.tag=0.11.0 \
  --set oauth2.issuer="https://auth-staging.example.com" \
  --set oauth2.audience="cap-verifier-staging" \
  --set oauth2.jwks_url="https://auth-staging.example.com/.well-known/jwks.json"
```

**Staging Configuration:**
- 2 replicas
- OAuth2 enabled
- TLS termination (Ingress)
- HPA: 2-5 pods
- Letsencrypt staging certificates

### Install Chart (Production)

```bash
# Create secrets first
kubectl create secret generic cap-verifier-tls \
  --from-file=tls.crt=path/to/server.crt \
  --from-file=tls.key=path/to/server.key \
  --from-file=ca.crt=path/to/client-ca.crt \
  --namespace cap-verifier

# Install with production values
helm install cap-verifier . \
  --namespace cap-verifier \
  --values values-prod.yaml \
  --set image.repository=your-registry.example.com/cap-verifier-api \
  --set image.tag=0.11.0 \
  --set oauth2.issuer="https://auth.example.com" \
  --set oauth2.audience="cap-verifier" \
  --set oauth2.jwks_url="https://auth.example.com/.well-known/jwks.json"
```

**Production Configuration:**
- 3 replicas
- mTLS required
- HPA: 3-20 pods
- Modern TLS (1.3)
- Pod anti-affinity (required)
- Larger cache (5000 entries)
- Production-grade monitoring

---

## Configuration

### ConfigMap Structure

The Helm chart creates a ConfigMap with 4 configuration files:

**1. auth.yaml** - OAuth2 Configuration
```yaml
issuer: "https://idp.example.com"
audience: "cap-verifier"
jwks_url: "https://idp.example.com/.well-known/jwks.json"
jwks_cache_ttl_sec: 600
required_scopes:
  verify: ["verify:run"]
  policy_compile: ["policy:compile"]
  policy_read: ["policy:read"]
```

**2. tls.yaml** - TLS/mTLS Configuration
```yaml
require_mtls: true
tls_min_version: "1.3"
cipher_profile: "modern"
client_ca_bundle: "/etc/ssl/certs/client-ca.crt"
server_cert: "/etc/ssl/certs/server.crt"
server_key: "/etc/ssl/certs/server.key"
client_cert_validation: "required"
verify_client_san: true
allowed_client_sans:
  - "*.cap-verifier.local"
```

**3. features.yaml** - Feature Flags
```yaml
allow_embedded_ir: true
require_mtls: true
enable_prometheus_metrics: true
```

**4. cache.yaml** - Cache Configuration
```yaml
enabled: true
size: 5000
ttl_seconds: 3600
```

### Override Configuration

```bash
# Update specific values
helm upgrade cap-verifier . \
  --reuse-values \
  --set cache.size=10000 \
  --set oauth2.jwks_cache_ttl_sec=1200
```

---

## TLS/mTLS Setup

### Generate Certificates

**Server Certificate:**
```bash
# Generate private key
openssl genrsa -out server.key 4096

# Generate CSR
openssl req -new -key server.key -out server.csr \
  -subj "/CN=cap-verifier.example.com/O=YourOrg"

# Sign certificate (use your CA)
openssl x509 -req -in server.csr -CA ca.crt -CAkey ca.key \
  -CAcreateserial -out server.crt -days 365 -sha256
```

**Client Certificate (for mTLS):**
```bash
# Generate client key
openssl genrsa -out client.key 4096

# Generate client CSR with SAN
openssl req -new -key client.key -out client.csr \
  -subj "/CN=client.cap-verifier.local/O=ClientOrg" \
  -addext "subjectAltName = DNS:client.cap-verifier.local"

# Sign client certificate
openssl x509 -req -in client.csr -CA ca.crt -CAkey ca.key \
  -CAcreateserial -out client.crt -days 365 -sha256 \
  -extfile <(printf "subjectAltName=DNS:client.cap-verifier.local")
```

### Create TLS Secret

```bash
kubectl create secret tls cap-verifier-tls \
  --cert=server.crt \
  --key=server.key \
  --namespace cap-verifier

# Add client CA for mTLS
kubectl create secret generic cap-verifier-client-ca \
  --from-file=ca.crt=client-ca.crt \
  --namespace cap-verifier
```

### Test mTLS Connection

```bash
# With client certificate
curl https://cap-verifier.example.com/healthz \
  --cert client.crt \
  --key client.key \
  --cacert ca.crt

# Should return 200 OK (with valid cert)
# Should return 403 Forbidden (without cert or with invalid cert)
```

---

## OAuth2 Integration

### Obtain JWT Token (Client Credentials)

```bash
# Request token from OAuth2 provider
TOKEN=$(curl -X POST https://auth.example.com/oauth/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=client_credentials" \
  -d "client_id=your-client-id" \
  -d "client_secret=your-client-secret" \
  -d "scope=verify:run policy:compile policy:read" \
  | jq -r '.access_token')

echo $TOKEN
```

### Test Authenticated Endpoint

```bash
# Without token (should return 401)
curl -X POST https://cap-verifier.example.com/verify \
  -H "Content-Type: application/json" \
  -d '{"policy_id": "test", "context": {}, "backend": "mock"}'

# With token (should return 200)
curl -X POST https://cap-verifier.example.com/verify \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"policy_id": "test", "context": {}, "backend": "mock"}'
```

### Required Scopes

| Endpoint | Required Scope |
|----------|----------------|
| `POST /verify` | `verify:run` |
| `POST /policy/compile` | `policy:compile` |
| `POST /policy/v2/compile` | `policy:compile` |
| `GET /policy/{id}` | `policy:read` |
| `GET /policy/v2/{id}` | `policy:read` |
| `GET /healthz` | (public) |
| `GET /readyz` | (public) |

---

## Monitoring

### Prometheus Metrics Endpoint

The API exposes Prometheus metrics at `/metrics`:

```bash
curl http://cap-verifier-service:8080/metrics
```

### Key Metrics

```
# Request counters
cap_verifier_requests_total{result="ok"}
cap_verifier_requests_total{result="warn"}
cap_verifier_requests_total{result="fail"}

# Request duration histogram
cap_verifier_request_duration_seconds_bucket{le="0.5"}
cap_verifier_request_duration_seconds_bucket{le="1.0"}

# Authentication failures
cap_verifier_auth_failures_total

# Cache performance
cap_verifier_cache_hit_ratio
```

### Grafana Dashboard

Import `grafana/dashboards/verifier.json` into Grafana:

```bash
# Via Grafana UI
1. Go to Dashboards → Import
2. Upload verifier.json
3. Select Prometheus datasource
4. Click Import

# Via CLI (with grafana-cli)
grafana-cli --homepath=/usr/share/grafana \
  dashboard import grafana/dashboards/verifier.json
```

**Dashboard Panels:**
1. Request Results (OK/WARN/FAIL rates)
2. Request Duration (p95/p99 latency)
3. Error Rate (with 1% alert threshold)
4. Cache Hit Rate (gauge with 80% threshold)
5. Total Requests (sparkline)

---

## Troubleshooting

### Pod Fails to Start

**Check logs:**
```bash
kubectl logs -f deployment/cap-verifier --namespace cap-verifier
```

**Common issues:**
- Missing ConfigMap: `kubectl get configmap -n cap-verifier`
- Missing Secrets: `kubectl get secrets -n cap-verifier`
- Image pull error: Check image repository and credentials
- Resource limits: Increase CPU/memory requests

### OAuth2 Authentication Failures

**Symptoms:**
- 401 Unauthorized responses
- `cap_verifier_auth_failures_total` increasing

**Debug steps:**
```bash
# Check JWKS endpoint is reachable
curl https://idp.example.com/.well-known/jwks.json

# Decode JWT to inspect claims
echo $TOKEN | cut -d. -f2 | base64 -d | jq

# Verify issuer/audience match config
kubectl get configmap cap-verifier-config -n cap-verifier -o yaml
```

**Common fixes:**
- Issuer mismatch: Update `oauth2.issuer` in values.yaml
- Audience mismatch: Update `oauth2.audience`
- Expired token: Obtain new token
- Missing scope: Include required scope in token request

### mTLS Connection Issues

**Symptoms:**
- 403 Forbidden with client certificate
- Connection refused

**Debug steps:**
```bash
# Check TLS secret exists
kubectl get secret cap-verifier-tls -n cap-verifier

# Verify certificate validity
openssl x509 -in server.crt -text -noout

# Check SAN in client cert
openssl x509 -in client.crt -text -noout | grep "Subject Alternative Name" -A1

# Test TLS handshake
openssl s_client -connect cap-verifier.example.com:443 \
  -cert client.crt -key client.key
```

**Common fixes:**
- SAN not in allowed list: Update `tls.allowed_client_sans` in values.yaml
- Expired certificate: Renew certificate
- Wrong CA: Ensure client cert is signed by CA in `client_ca_bundle`

### High Latency (p95 > 500ms)

**Investigation:**
```bash
# Check pod CPU/memory usage
kubectl top pods -n cap-verifier

# Check HPA status
kubectl get hpa -n cap-verifier

# Review slow requests in logs
kubectl logs deployment/cap-verifier -n cap-verifier | grep "duration_ms"
```

**Potential fixes:**
- Scale up replicas: `kubectl scale deployment/cap-verifier --replicas=10`
- Increase CPU limits in values.yaml
- Optimize cache size: Increase `cache.size`
- Check database/backend latency

### Cache Hit Rate < 80%

**Investigation:**
```bash
# Check cache metrics
curl http://cap-verifier-service:8080/metrics | grep cache

# Review cache configuration
kubectl get configmap cap-verifier-config -o jsonpath='{.data.cache\.yaml}'
```

**Fixes:**
- Increase cache size: Update `cache.size` in values.yaml
- Increase TTL: Update `cache.ttl_seconds`
- Verify cache is enabled: Check `cache.enabled: true`

---

## Health Checks

### Liveness Probe

```bash
curl http://cap-verifier-service:8080/healthz
```

**Expected:**
```json
{
  "status": "OK",
  "version": "0.11.0",
  "build_hash": null
}
```

### Readiness Probe

```bash
curl http://cap-verifier-service:8080/readyz
```

**Expected:**
```json
{
  "status": "OK",
  "checks": [
    {"name": "verifier_core", "status": "OK"},
    {"name": "crypto", "status": "OK"}
  ]
}
```

---

## Rolling Updates

### Upgrade to New Version

```bash
# Update image tag
helm upgrade cap-verifier . \
  --namespace cap-verifier \
  --reuse-values \
  --set image.tag=0.12.0

# Monitor rollout
kubectl rollout status deployment/cap-verifier -n cap-verifier

# Rollback if needed
kubectl rollout undo deployment/cap-verifier -n cap-verifier
```

### Zero-Downtime Strategy

1. Ensure `maxUnavailable: 0` in deployment.yaml
2. Set `maxSurge: 1` for rolling update
3. Configure readiness probe with 5s delay
4. Use PDB (Pod Disruption Budget) for prod

---

## Backup and Restore

### Backup Registry (SQLite)

```bash
# Copy registry database from pod
kubectl cp cap-verifier-pod:/data/registry.sqlite ./backup/registry-$(date +%Y%m%d).sqlite

# Backup to S3/cloud storage
aws s3 cp ./backup/registry-$(date +%Y%m%d).sqlite s3://backups/cap-verifier/
```

### Restore Registry

```bash
# Copy backup to pod
kubectl cp ./backup/registry-20251110.sqlite cap-verifier-pod:/data/registry.sqlite

# Restart pod to reload
kubectl delete pod -l app=cap-verifier -n cap-verifier
```

---

## Security Hardening

### Network Policies

Apply NetworkPolicy to restrict ingress/egress:

```bash
kubectl apply -f helm/templates/networkpolicy.yaml -n cap-verifier
```

**Allowed traffic:**
- Ingress: Port 8080 (HTTP/HTTPS)
- Egress: OAuth2 provider (JWKS), Prometheus

### Pod Security Standards

Deployment uses restrictive security context:

```yaml
securityContext:
  runAsNonRoot: true
  runAsUser: 1000
  readOnlyRootFilesystem: true
  allowPrivilegeEscalation: false
  capabilities:
    drop:
      - ALL
```

### Secrets Management

**Best Practices:**
- Use Kubernetes Secrets for TLS certs
- Rotate secrets every 90 days
- Use Sealed Secrets or External Secrets Operator for GitOps
- Never commit secrets to Git

---

## Performance Tuning

### HPA Configuration

**Conservative (Staging):**
```yaml
autoscaling:
  minReplicas: 2
  maxReplicas: 5
  targetCPUUtilizationPercentage: 75
  targetMemoryUtilizationPercentage: 80
```

**Aggressive (Production):**
```yaml
autoscaling:
  minReplicas: 3
  maxReplicas: 20
  targetCPUUtilizationPercentage: 60
  targetMemoryUtilizationPercentage: 70
```

### Resource Limits

**Development:**
- Requests: 250m CPU, 256Mi memory
- Limits: 1000m CPU, 512Mi memory

**Production:**
- Requests: 500m CPU, 512Mi memory
- Limits: 2000m CPU, 1Gi memory

---

## Contact and Support

**Documentation:** https://docs.cap-verifier.example.com
**Issues:** https://github.com/cap-verifier/api/issues
**Slack:** #cap-verifier-ops

**On-call:** Refer to runbook for escalation procedures.

---

**Last Updated:** 2025-11-10
**Version:** v0.11.0
**Maintained by:** CAP DevOps Team
