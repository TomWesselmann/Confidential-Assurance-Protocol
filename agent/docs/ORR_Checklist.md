# Operational Readiness Review (ORR) Checklist

**Project:** CAP Verifier API v0.11.0
**Date:** 2025-11-10
**Status:** Ready for Production Cutover
**Review Type:** Week 6 Production Go-Live

---

## Document Control

| Field | Value |
|-------|-------|
| **Document Version** | 1.0 |
| **Review Date** | 2025-11-10 |
| **Approved By** | [Tech Lead Signature] |
| **Next Review** | 2026-02-10 (90 days) |
| **Classification** | Internal/Confidential |

---

## Purpose

This Operational Readiness Review (ORR) checklist verifies that all components of the CAP Verifier API are production-ready before the Week 6 Go-Live Cutover. The review covers infrastructure, security, observability, and operational procedures established in Week 5.

**Go/No-Go Criteria:**
- All ☐ items must be checked ✅ for production deployment approval
- Any ⚠️ items require documented mitigation plans
- Any 🔴 items are blockers for production deployment

---

## 1. Infrastructure Readiness

### 1.1 Kubernetes Cluster

- ☐ **Production cluster provisioned**
  - [ ] Minimum 3 worker nodes (high availability)
  - [ ] Node capacity: 8 CPU cores, 16GB RAM per node
  - [ ] Anti-affinity rules configured (pod distribution)
  - [ ] Kubernetes version ≥ 1.24

- ☐ **Namespace configuration**
  - [ ] `cap-verifier` namespace created
  - [ ] ResourceQuotas configured (CPU/Memory limits)
  - [ ] LimitRanges defined (pod-level constraints)

- ☐ **Storage**
  - [ ] Persistent Volumes available (if needed for registry)
  - [ ] Storage class supports ReadWriteMany (if SQLite registry)
  - [ ] Backup strategy documented

**Verification Commands:**
```bash
kubectl get nodes -o wide
kubectl describe namespace cap-verifier
kubectl get pv,pvc -n cap-verifier
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

### 1.2 Ingress & Load Balancer

- ☐ **Ingress Controller installed**
  - [ ] NGINX Ingress or equivalent deployed
  - [ ] External LoadBalancer IP assigned
  - [ ] DNS record configured (e.g., cap-verifier.example.com)

- ☐ **TLS Termination**
  - [ ] Valid TLS certificate provisioned (Let's Encrypt or CA)
  - [ ] Certificate expiration > 30 days
  - [ ] Auto-renewal configured (cert-manager)

- ☐ **Ingress Rules**
  - [ ] `cap-verifier-ingress` resource deployed
  - [ ] Backend service: `cap-verifier-service:8080`
  - [ ] Health check paths configured (`/healthz`, `/readyz`)

**Verification Commands:**
```bash
kubectl get ingress -n cap-verifier
kubectl describe ingress cap-verifier -n cap-verifier
curl -I https://cap-verifier.example.com/healthz
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

### 1.3 Horizontal Pod Autoscaler (HPA)

- ☐ **HPA configured**
  - [ ] Metrics Server installed in cluster
  - [ ] HPA resource deployed for `cap-verifier` deployment
  - [ ] Min replicas: 3
  - [ ] Max replicas: 20
  - [ ] Target CPU utilization: 60%
  - [ ] Target Memory utilization: 70%

- ☐ **Scaling behavior**
  - [ ] Scale-up behavior: 5 pods every 60 seconds (max)
  - [ ] Scale-down behavior: 1 pod every 300 seconds (gradual)
  - [ ] Stabilization window: 300 seconds

**Verification Commands:**
```bash
kubectl get hpa -n cap-verifier
kubectl describe hpa cap-verifier -n cap-verifier
kubectl top pods -n cap-verifier
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

### 1.4 Network Policies

- ☐ **NetworkPolicy deployed**
  - [ ] Ingress policy: Allow traffic only from Ingress Controller
  - [ ] Egress policy: Allow DNS, OAuth2 provider, Prometheus
  - [ ] Deny-all default policy (if applicable)

- ☐ **Policy validation**
  - [ ] Test internal pod-to-pod communication (should be blocked)
  - [ ] Test external communication (OAuth2 JWKS fetch works)

**Verification Commands:**
```bash
kubectl get networkpolicy -n cap-verifier
kubectl describe networkpolicy cap-verifier -n cap-verifier
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

### 1.5 Pod Security

- ☐ **Security Context**
  - [ ] `runAsNonRoot: true`
  - [ ] `runAsUser: 1000`
  - [ ] `readOnlyRootFilesystem: true`
  - [ ] `allowPrivilegeEscalation: false`
  - [ ] `capabilities.drop: [ALL]`

- ☐ **Pod Disruption Budget (PDB)**
  - [ ] PDB deployed (minAvailable: 2)
  - [ ] Ensures availability during node drain/upgrade

**Verification Commands:**
```bash
kubectl get pods -n cap-verifier -o jsonpath='{range .items[*]}{.metadata.name}{"\t"}{.spec.securityContext}{"\n"}{end}'
kubectl get pdb -n cap-verifier
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

### 1.6 Resource Limits

- ☐ **Resource Requests & Limits**
  - [ ] Requests: 500m CPU, 512Mi memory
  - [ ] Limits: 2000m CPU, 1Gi memory
  - [ ] No OOMKilled events in past 7 days (staging)

**Verification Commands:**
```bash
kubectl describe pod <pod-name> -n cap-verifier | grep -A 10 "Limits"
kubectl get events -n cap-verifier --field-selector reason=OOMKilled
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

## 2. Security Readiness

### 2.1 OAuth2 Client Credentials Flow

- ☐ **OAuth2 Provider (Production IdP)**
  - [ ] Production IdP URL configured (e.g., https://auth.example.com)
  - [ ] Client ID registered for CAP Verifier API
  - [ ] Client Secret stored in Kubernetes Secret
  - [ ] JWKS URL reachable: `https://auth.example.com/.well-known/jwks.json`

- ☐ **JWT Validation**
  - [ ] Issuer validation enabled (`oauth2.issuer`)
  - [ ] Audience validation enabled (`oauth2.audience`)
  - [ ] Algorithm: RS256 (asymmetric)
  - [ ] JWKS cache TTL: 600 seconds

- ☐ **Scopes**
  - [ ] Scopes defined: `verify:run`, `policy:compile`, `policy:read`, `metrics:read`
  - [ ] Endpoints require correct scopes
  - [ ] Test token with invalid scope returns 403 Forbidden

**Verification Commands:**
```bash
# Test JWKS endpoint
curl -s https://auth.example.com/.well-known/jwks.json | jq .

# Test OAuth2 token fetch
TOKEN=$(curl -X POST https://auth.example.com/oauth/token \
  -d "grant_type=client_credentials" \
  -d "client_id=<client_id>" \
  -d "client_secret=<secret>" \
  -d "scope=verify:run policy:compile policy:read" \
  | jq -r '.access_token')

# Test authenticated endpoint
curl -H "Authorization: Bearer $TOKEN" \
  https://cap-verifier.example.com/readyz
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

### 2.2 TLS/mTLS Configuration

- ☐ **TLS (Ingress Termination)**
  - [ ] TLS 1.3 enforced (minimum version)
  - [ ] Modern cipher suite: `ECDHE+AESGCM`, `ECDHE+CHACHA20`, `DHE+AESGCM`
  - [ ] Certificate chain valid (Root + Intermediate + Leaf)
  - [ ] No mixed content warnings

- ☐ **mTLS (Optional, Environment-Specific)**
  - [ ] mTLS enabled/disabled per environment (`require_mtls` flag)
  - [ ] Client CA bundle configured (if mTLS enabled)
  - [ ] SAN validation: `allowed_client_sans` configured
  - [ ] Test mTLS connection with valid client cert

**Verification Commands:**
```bash
# Test TLS handshake
openssl s_client -connect cap-verifier.example.com:443 -showcerts

# Test TLS version
curl -I --tlsv1.3 https://cap-verifier.example.com/healthz

# Test mTLS (if enabled)
curl --cert client.crt --key client.key \
  https://cap-verifier.example.com/healthz
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

### 2.3 Secrets Management

- ☐ **Kubernetes Secrets**
  - [ ] `cap-verifier-oauth2` Secret created (client_secret)
  - [ ] `cap-verifier-tls` Secret created (TLS cert/key)
  - [ ] `cap-verifier-client-ca` Secret created (Client CA, if mTLS)
  - [ ] Secrets encrypted at rest (KMS integration)

- ☐ **Secret Rotation**
  - [ ] Rotation procedure documented (OAuth2 client secret)
  - [ ] TLS certificate auto-renewal (cert-manager)
  - [ ] No secrets hardcoded in ConfigMaps or code

**Verification Commands:**
```bash
kubectl get secrets -n cap-verifier
kubectl describe secret cap-verifier-oauth2 -n cap-verifier
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

### 2.4 Image Security

- ☐ **Container Image**
  - [ ] Image signed with Cosign or Notary v2
  - [ ] Image scanned for vulnerabilities (Trivy, Snyk, Grype)
  - [ ] No critical vulnerabilities (CVSS ≥ 9.0)
  - [ ] Base image: Minimal (Alpine, Distroless, or Scratch)

- ☐ **Registry Security**
  - [ ] Image stored in private registry (authenticated pull)
  - [ ] ImagePullSecrets configured in namespace
  - [ ] Image tag: Immutable (SHA256 digest, not `latest`)

**Verification Commands:**
```bash
# Scan image
trivy image your-registry.example.com/cap-verifier-api:0.11.0

# Verify image signature
cosign verify --key cosign.pub \
  your-registry.example.com/cap-verifier-api:0.11.0
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

## 3. Observability Readiness

### 3.1 Prometheus Metrics

- ☐ **Metrics Endpoint**
  - [ ] `/metrics` endpoint exposed on port 8080
  - [ ] Prometheus scraping configured (ServiceMonitor or PodMonitor)
  - [ ] Scrape interval: 30 seconds

- ☐ **Key Metrics Available**
  - [ ] `cap_verifier_requests_total{result="ok|warn|fail"}`
  - [ ] `cap_verifier_request_duration_seconds` (histogram)
  - [ ] `cap_verifier_auth_failures_total`
  - [ ] `cap_verifier_cache_hit_ratio`

**Verification Commands:**
```bash
# Test metrics endpoint
curl http://cap-verifier-service:8080/metrics | grep cap_verifier

# Check Prometheus targets
kubectl port-forward -n monitoring svc/prometheus 9090:9090
# Open http://localhost:9090/targets
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

### 3.2 Grafana Dashboards

- ☐ **Dashboard Deployed**
  - [ ] `verifier.json` imported into Grafana
  - [ ] Datasource configured (Prometheus)
  - [ ] Dashboard panels display data correctly

- ☐ **Key Panels**
  - [ ] Request Results (OK/WARN/FAIL rates)
  - [ ] Request Duration (p95/p99 latency)
  - [ ] Error Rate (with 1% alert threshold)
  - [ ] Cache Hit Rate (with 80% threshold)
  - [ ] Total Requests (sparkline)

**Verification Commands:**
```bash
# Access Grafana
kubectl port-forward -n monitoring svc/grafana 3000:3000
# Open http://localhost:3000/dashboards
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

### 3.3 Alerting Rules

- ☐ **Prometheus Alerts Configured**
  - [ ] `HighErrorRate` (P1): Error rate > 1% for 5 min
  - [ ] `HighP95Latency` (P2): p95 > 500ms for 5 min
  - [ ] `HighP99Latency` (P2): p99 > 1s for 5 min
  - [ ] `FivexxSpike` (P0): > 10 5xx errors/min
  - [ ] `LowCacheHitRate` (P3): Hit rate < 80% for 10 min
  - [ ] `HighAuthFailureRate` (P1): Auth failures > 10% for 5 min
  - [ ] `NoTraffic` (P2): Zero requests for 5 min

- ☐ **Alertmanager Integration**
  - [ ] AlertManager configured (routing, receivers)
  - [ ] PagerDuty/Opsgenie integration tested
  - [ ] Alert silences configured (maintenance windows)

**Verification Commands:**
```bash
# Check Prometheus alerts
kubectl port-forward -n monitoring svc/prometheus 9090:9090
# Open http://localhost:9090/alerts

# Test alert firing
# (Trigger condition manually in staging)
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

### 3.4 Structured Logging

- ☐ **Log Format**
  - [ ] JSON structured logs (machine-readable)
  - [ ] Fields: `timestamp`, `level`, `message`, `request_id`, `duration_ms`
  - [ ] No PII in logs (GDPR/Privacy compliance)

- ☐ **Log Levels**
  - [ ] Production: `INFO` level
  - [ ] Staging: `DEBUG` level
  - [ ] No `DEBUG` logs in production (performance)

- ☐ **Log Aggregation**
  - [ ] Logs forwarded to centralized system (ELK, Loki, Splunk)
  - [ ] Log retention: 90 days
  - [ ] Query test: Find logs by `request_id`

**Verification Commands:**
```bash
# Check log format
kubectl logs deployment/cap-verifier -n cap-verifier --tail=10

# Test log query
kubectl logs deployment/cap-verifier -n cap-verifier | grep '"level":"ERROR"'
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

### 3.5 SLOs Documented

- ☐ **SLO Definitions**
  - [ ] Availability: 99.9% (43 min downtime/month)
  - [ ] p95 Latency: < 500ms
  - [ ] p99 Latency: < 1000ms
  - [ ] Error Rate: < 1%
  - [ ] Cache Hit Rate: > 80%

- ☐ **SLO Tracking**
  - [ ] Prometheus queries defined (docs/Week5_SLOs.md)
  - [ ] Error budget calculated (30-day rolling window)
  - [ ] Quarterly SLO review scheduled (Q1 2026)

**Verification:**
```bash
# Check SLO compliance (manual query)
# Availability:
promql: sum(rate(cap_verifier_requests_total{result!="fail"}[30d])) / sum(rate(cap_verifier_requests_total[30d])) * 100
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

## 4. Operational Readiness

### 4.1 Runbooks

- ☐ **Alert Runbooks Available**
  - [ ] `docs/Week5_Runbooks.md` completed
  - [ ] Runbooks for each alert (HighErrorRate, HighP95Latency, FivexxSpike, etc.)
  - [ ] Each runbook contains:
    - Severity level (P0-P3)
    - Investigation steps with commands
    - Common root causes
    - Resolution procedures
    - Escalation criteria

- ☐ **Operational Runbooks**
  - [ ] Rollback procedure (Helm rollback, kubectl undo)
  - [ ] Scaling procedure (HPA adjustment, manual scaling)
  - [ ] Certificate rotation (cert-manager renewal)
  - [ ] Secret rotation (OAuth2 client secret, TLS certs)
  - [ ] Cache flush procedure (pod restart, configmap update)

**Verification:**
```bash
# Review runbooks
cat docs/Week5_Runbooks.md | grep "### 🔴\\|### ⚠️"
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

### 4.2 Incident Escalation

- ☐ **Escalation Path Defined**
  - [ ] L1 (On-Call Engineer): Initial investigation (30 min P0, 2 hours P1)
  - [ ] L2 (Senior Engineer/Team Lead): Deep debugging (2 hours P0, 8 hours P1)
  - [ ] L3 (Engineering Manager): Resource allocation, vendor engagement

- ☐ **Contact Information**
  - [ ] On-call rotation configured (PagerDuty/Opsgenie)
  - [ ] Team Lead contact (Slack, Email, Phone)
  - [ ] Engineering Manager contact (Escalations only)
  - [ ] OAuth2 Provider Support (24/7 contact)

**Verification:**
```bash
# Test PagerDuty integration
# (Trigger test alert manually)
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

### 4.3 Deployment Procedures

- ☐ **Helm Deployment**
  - [ ] Production values file: `helm/values-prod.yaml`
  - [ ] Deployment command documented (docs/Week5_Deployment_Guide.md)
  - [ ] Rollback command tested in staging
  - [ ] Blue-Green or Canary strategy defined (if applicable)

- ☐ **CI/CD Pipeline**
  - [ ] Pipeline defined (GitHub Actions, GitLab CI, Jenkins)
  - [ ] Build stage: Docker image build + push
  - [ ] Test stage: Unit tests, integration tests
  - [ ] Deploy stage: Helm upgrade with --wait
  - [ ] Post-deploy: Smoke tests

**Verification Commands:**
```bash
# Test Helm deployment (dry-run)
helm upgrade --install cap-verifier helm/ \
  -f helm/values-prod.yaml \
  --dry-run --debug

# Test Helm rollback
kubectl rollout undo deployment/cap-verifier -n cap-verifier
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

### 4.4 Backup & Restore

- ☐ **Backup Strategy**
  - [ ] Registry SQLite backup (if applicable)
  - [ ] Policy blobs backup (if applicable)
  - [ ] ConfigMaps/Secrets backup
  - [ ] Backup frequency: Daily
  - [ ] Backup retention: 30 days
  - [ ] Backup storage: S3/GCS/Azure Blob

- ☐ **Restore Procedure**
  - [ ] Restore runbook documented (docs/runbook_restore.md, Week 6)
  - [ ] Restore tested in staging
  - [ ] RTO (Recovery Time Objective): 4 hours
  - [ ] RPO (Recovery Point Objective): 24 hours

**Verification Commands:**
```bash
# Test backup (example)
kubectl cp cap-verifier-pod:/data/registry.sqlite ./backup/registry-$(date +%Y%m%d).sqlite

# Test restore (example)
kubectl cp ./backup/registry-20251110.sqlite cap-verifier-pod:/data/registry.sqlite
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

### 4.5 Maintenance Windows

- ☐ **Planned Maintenance**
  - [ ] Frequency: Monthly (first Tuesday, 02:00-04:00 UTC)
  - [ ] Announcement: 48 hours in advance (#cap-verifier-ops Slack)
  - [ ] Status page updated (if applicable)
  - [ ] Pre-maintenance checklist defined (backup, test rollback)

- ☐ **Emergency Maintenance**
  - [ ] Emergency window: Outside business hours (preferred)
  - [ ] Approval: Engineering Manager (P0), Team Lead (P1)
  - [ ] Communication: Slack #cap-verifier-ops + Status page

**Verification:**
```bash
# Review maintenance schedule
cat docs/Week5_Runbooks.md | grep -A 20 "## Maintenance Windows"
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

### 4.6 Cache Configuration

- ☐ **Cache Settings**
  - [ ] Cache enabled: `cache.enabled: true`
  - [ ] Cache size: 5000 entries (production)
  - [ ] Cache TTL: 3600 seconds (1 hour)
  - [ ] Eviction policy: LRU (Least Recently Used)

- ☐ **Cache Flush Procedure**
  - [ ] Documented in runbooks
  - [ ] Method: Pod restart or ConfigMap update
  - [ ] Test cache flush in staging

**Verification Commands:**
```bash
# Check cache config
kubectl get configmap cap-verifier-config -n cap-verifier \
  -o jsonpath='{.data.cache\.yaml}'

# Flush cache (restart pods)
kubectl rollout restart deployment/cap-verifier -n cap-verifier
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

## 5. Testing & Validation

### 5.1 Smoke Tests

- ☐ **Health Checks**
  - [ ] `GET /healthz` returns 200 OK
  - [ ] `GET /readyz` returns 200 OK
  - [ ] Response includes version and status

- ☐ **Protected Endpoints**
  - [ ] `POST /verify` returns 401 without token
  - [ ] `POST /verify` returns 200 with valid token
  - [ ] `POST /policy/compile` returns 401 without token
  - [ ] `POST /policy/compile` returns 200 with valid token
  - [ ] `GET /policy/:id` returns 404 for non-existent policy
  - [ ] `GET /policy/:id` returns 200 for existing policy

- ☐ **Load Test (Spot Check)**
  - [ ] 10 RPS for 5 minutes
  - [ ] p95 latency < 500ms
  - [ ] Error rate < 1%
  - [ ] No OOMKilled events

**Verification Commands:**
```bash
# Smoke test script (docs/scripts/smoke_prod.sh, Week 6)
./scripts/smoke_prod.sh
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

### 5.2 Integration Tests

- ☐ **Contract Tests**
  - [ ] OpenAPI spec validated (openapi/openapi.yaml)
  - [ ] Schemathesis tests pass (all endpoints)
  - [ ] No breaking API changes

**Verification Commands:**
```bash
# Run schemathesis
schemathesis run openapi/openapi.yaml \
  --base-url=https://cap-verifier.example.com \
  --checks all
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

### 5.3 Security Tests

- ☐ **Authentication Tests**
  - [ ] Invalid JWT returns 401 Unauthorized
  - [ ] Expired JWT returns 401 Unauthorized
  - [ ] Missing scope returns 403 Forbidden
  - [ ] Tampered JWT returns 401 Unauthorized

- ☐ **TLS Tests**
  - [ ] TLS 1.2 connection rejected (if TLS 1.3 only)
  - [ ] Weak cipher suites rejected
  - [ ] Certificate chain valid

**Verification Commands:**
```bash
# Test expired token
curl -H "Authorization: Bearer <expired_token>" \
  https://cap-verifier.example.com/verify

# Test TLS version
curl --tlsv1.2 --max-tls 1.2 https://cap-verifier.example.com/healthz
# Should fail if TLS 1.3 enforced
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

## 6. Documentation

### 6.1 Deployment Guide

- ☐ **docs/Week5_Deployment_Guide.md**
  - [ ] Complete deployment guide available
  - [ ] Prerequisites documented
  - [ ] Helm installation steps (dev/staging/prod)
  - [ ] Configuration options documented
  - [ ] Troubleshooting section included

**Verification:**
```bash
# Review deployment guide
cat docs/Week5_Deployment_Guide.md | grep "## Table of Contents" -A 20
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

### 6.2 Operational Runbooks

- ☐ **docs/Week5_Runbooks.md**
  - [ ] Complete runbooks available
  - [ ] Incident response procedures documented
  - [ ] 8+ alert runbooks (HighErrorRate, FivexxSpike, etc.)
  - [ ] Common operations documented (restart, scale, update)
  - [ ] Escalation path documented

**Verification:**
```bash
# Review runbooks
cat docs/Week5_Runbooks.md | grep "## Table of Contents" -A 20
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

### 6.3 SLO Documentation

- ☐ **docs/Week5_SLOs.md**
  - [ ] Complete SLO documentation available
  - [ ] SLI definitions documented
  - [ ] SLO targets documented (99.9% availability, etc.)
  - [ ] Error budget calculations documented
  - [ ] Prometheus queries included

**Verification:**
```bash
# Review SLOs
cat docs/Week5_SLOs.md | grep "## Table of Contents" -A 20
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

### 6.4 OpenAPI Specification

- ☐ **openapi/openapi.yaml**
  - [ ] Complete OpenAPI 3.0 spec
  - [ ] OAuth2 security scheme defined (Client Credentials flow)
  - [ ] Scopes documented (`verify:run`, `policy:compile`, `policy:read`)
  - [ ] All endpoints documented
  - [ ] Request/response schemas defined

**Verification Commands:**
```bash
# Validate OpenAPI spec
docker run --rm -v $(pwd):/local openapitools/openapi-generator-cli validate \
  -i /local/openapi/openapi.yaml
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

## 7. Compliance & Governance

### 7.1 Data Privacy

- ☐ **PII Handling**
  - [ ] No PII logged (GDPR/Privacy compliance)
  - [ ] Data retention policy documented (logs: 90 days)
  - [ ] Data minimization principle applied

- ☐ **Data Encryption**
  - [ ] Data in transit: TLS 1.3
  - [ ] Data at rest: Kubernetes Secrets encrypted (KMS)
  - [ ] No plaintext secrets in ConfigMaps

**Verification:**
```bash
# Check logs for PII leaks
kubectl logs deployment/cap-verifier -n cap-verifier --tail=100 | grep -E "email|phone|ssn|passport"
# Should return nothing
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

### 7.2 Audit Trail

- ☐ **Access Logs**
  - [ ] All API requests logged (request_id, timestamp, user, endpoint)
  - [ ] OAuth2 authentication events logged
  - [ ] Failed authentication logged

- ☐ **Change Logs**
  - [ ] Deployment events logged (who, when, what)
  - [ ] ConfigMap/Secret changes tracked
  - [ ] Kubectl audit logs enabled (if applicable)

**Verification Commands:**
```bash
# Check access logs
kubectl logs deployment/cap-verifier -n cap-verifier | grep '"level":"INFO"'
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

## 8. Canary Rollout Plan (Week 6)

### 8.1 Canary Strategy

- ☐ **Canary Configuration**
  - [ ] Canary deployment configured (Istio, Flagger, or manual)
  - [ ] Traffic split: 5% → 25% → 50% → 100%
  - [ ] Rollout duration: 15 minutes per stage
  - [ ] Automatic rollback on SLO breach

- ☐ **Monitoring During Canary**
  - [ ] Error rate monitored (< 1%)
  - [ ] p95 latency monitored (< 500ms)
  - [ ] 5xx errors monitored (< 10/min)
  - [ ] Rollback trigger: Any SLO breach for > 5 minutes

**Verification:**
```bash
# Check canary deployment (if Flagger)
kubectl get canary -n cap-verifier
kubectl describe canary cap-verifier -n cap-verifier
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

### 8.2 Rollback Plan

- ☐ **Rollback Procedure**
  - [ ] Helm rollback command documented
  - [ ] Rollback tested in staging
  - [ ] Rollback time: < 5 minutes
  - [ ] Post-rollback verification: Smoke tests

**Verification Commands:**
```bash
# Test rollback (dry-run)
helm rollback cap-verifier 0 --dry-run --debug -n cap-verifier

# Actual rollback command
kubectl rollout undo deployment/cap-verifier -n cap-verifier
```

**Status:** ☐ Approved ☐ Requires Mitigation ☐ Blocker

---

## 9. Final Go/No-Go Decision

### 9.1 Summary

**Total Checklist Items:** 100+

**Completed Items:** ☐ [Count]

**Mitigation Required:** ☐ [Count]

**Blockers:** ☐ [Count]

### 9.2 Risk Assessment

| Risk | Severity | Mitigation | Owner |
|------|----------|------------|-------|
| OAuth2 provider outage | High | Cached JWKS keys (600s TTL), graceful degradation | DevOps |
| Database connection failures | Medium | Connection pooling, retry logic | Backend |
| TLS certificate expiration | Low | Auto-renewal (cert-manager), 30-day alerting | Security |
| HPA maxing out (20 pods) | Medium | Monitor replica count, increase maxReplicas if needed | SRE |

### 9.3 Go/No-Go Criteria

- ☐ **GO:** All checklist items ✅, no blockers 🔴, mitigation plans for ⚠️ items
- ☐ **NO-GO:** Any blocker 🔴 items remaining

---

## 10. Sign-Off

| Role | Name | Signature | Date |
|------|------|-----------|------|
| **Tech Lead** | [Name] | [Signature] | 2025-11-10 |
| **DevOps Lead** | [Name] | [Signature] | 2025-11-10 |
| **Security Lead** | [Name] | [Signature] | 2025-11-10 |
| **Product Manager** | [Name] | [Signature] | 2025-11-10 |

---

## 11. Post-Production Review

**Scheduled:** 2025-11-17 (7 days post-deployment)

**Agenda:**
1. Review SLO compliance (first week)
2. Analyze incident reports
3. Review error budget consumption
4. Identify improvement areas
5. Update runbooks if needed

---

**Document End**

**Next Steps:**
1. Complete all checklist items
2. Obtain sign-offs from Tech Lead, DevOps Lead, Security Lead, Product Manager
3. Schedule production deployment (Canary rollout)
4. Monitor closely for 15 minutes (no traffic)
5. Execute smoke tests
6. Proceed with traffic rollout (5% → 25% → 50% → 100%)

**Emergency Contact:**
- On-Call Engineer: PagerDuty rotation
- Escalation: #cap-verifier-ops Slack channel

---

**Generated by:** Claude Code (Anthropic)
**Date:** 2025-11-10
**Version:** 1.0
