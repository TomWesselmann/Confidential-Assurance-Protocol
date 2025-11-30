# Week 5: Operational Runbooks - CAP Verifier API

**Version:** v0.11.0
**Date:** 2025-11-10
**Status:** Production

---

## Table of Contents

1. [Incident Response](#incident-response)
2. [Alert Runbooks](#alert-runbooks)
3. [Common Operations](#common-operations)
4. [Escalation](#escalation)

---

## Incident Response

### Severity Levels

| Severity | Impact | Response Time | Escalation |
|----------|--------|---------------|------------|
| **P0 - Critical** | Service down, data loss | 15 min | Immediate |
| **P1 - High** | Degraded service, SLO breach | 1 hour | After 2 hours |
| **P2 - Medium** | Minor degradation | 4 hours | After 12 hours |
| **P3 - Low** | Cosmetic, no impact | 24 hours | After 48 hours |

### Incident Commander Checklist

1. ☐ Acknowledge alert in PagerDuty/Opsgenie
2. ☐ Assess severity (P0-P3)
3. ☐ Start incident channel (#incident-YYYYMMDD-NNN)
4. ☐ Begin investigation (follow relevant runbook)
5. ☐ Update status page (if customer-facing)
6. ☐ Implement fix or workaround
7. ☐ Verify resolution
8. ☐ Document postmortem (for P0/P1)
9. ☐ Close incident

---

## Alert Runbooks

### 🔴 HighErrorRate

**Alert:** Error rate > 1% over 5 minutes

**Severity:** P1 (High)

**Symptoms:**
- `cap_verifier_requests_total{result="fail"}` increasing
- Users report verification failures
- Dashboard shows red error rate panel

**Investigation Steps:**

1. **Check recent deployments**
   ```bash
   kubectl rollout history deployment/cap-verifier -n cap-verifier

   # If recent deployment, consider rollback
   kubectl rollout undo deployment/cap-verifier -n cap-verifier
   ```

2. **Review error logs**
   ```bash
   kubectl logs deployment/cap-verifier -n cap-verifier --tail=100 | grep ERROR
   ```

3. **Check error types**
   ```bash
   # Sample recent errors
   kubectl logs deployment/cap-verifier -n cap-verifier --tail=500 \
     | jq 'select(.level=="ERROR") | {timestamp, error, request_id}'
   ```

4. **Verify OAuth2 integration**
   ```bash
   # Test JWKS endpoint
   curl -v https://idp.example.com/.well-known/jwks.json

   # Check auth failures
   curl http://cap-verifier-service:8080/metrics | grep auth_failures
   ```

**Common Root Causes:**
- OAuth2 provider outage → Check status page
- Invalid JWT tokens → Verify issuer/audience config
- Database connection issues → Check backend services
- Bug in recent deployment → Rollback

**Resolution:**
- Rollback deployment if caused by recent changes
- Update OAuth2 configuration if authentication issue
- Scale up pods if capacity issue
- Apply hotfix for bugs

**Verification:**
```bash
# Error rate should drop below 1%
watch -n 5 'curl -s http://cap-verifier-service:8080/metrics \
  | grep cap_verifier_requests_total'
```

---

### ⚠️ HighP95Latency

**Alert:** p95 latency > 500ms for 5 minutes

**Severity:** P2 (Medium)

**Symptoms:**
- Slow API responses
- Users report timeouts
- Grafana dashboard shows elevated p95/p99

**Investigation Steps:**

1. **Check pod resource usage**
   ```bash
   kubectl top pods -n cap-verifier

   # Look for CPU/memory saturation
   # CPU > 80% or Memory > 90% indicates resource pressure
   ```

2. **Review HPA status**
   ```bash
   kubectl get hpa -n cap-verifier

   # Check current/desired replicas
   # If maxed out, increase maxReplicas
   ```

3. **Analyze slow requests**
   ```bash
   kubectl logs deployment/cap-verifier -n cap-verifier --tail=100 \
     | jq 'select(.duration_ms > 500) | {path, method, duration_ms, timestamp}'
   ```

4. **Check cache hit rate**
   ```bash
   curl http://cap-verifier-service:8080/metrics | grep cache_hit_ratio

   # If < 0.8, cache is underperforming
   ```

**Common Root Causes:**
- High load → Scale up replicas
- Cache misses → Increase cache size/TTL
- Database slow queries → Optimize queries
- Resource starvation → Increase CPU/memory limits

**Resolution:**
```bash
# Option 1: Scale up replicas
kubectl scale deployment/cap-verifier --replicas=10 -n cap-verifier

# Option 2: Increase resource limits
helm upgrade cap-verifier . --reuse-values \
  --set resources.limits.cpu=2000m \
  --set resources.limits.memory=2Gi

# Option 3: Increase cache size
helm upgrade cap-verifier . --reuse-values \
  --set cache.size=10000 \
  --set cache.ttl_seconds=7200
```

**Verification:**
```bash
# p95 should drop below 500ms
watch -n 10 'curl -s http://cap-verifier-service:8080/metrics \
  | grep request_duration_seconds | grep 0.95'
```

---

### 🔴 FivexxSpike

**Alert:** > 10 5xx errors per minute

**Severity:** P0 (Critical)

**Symptoms:**
- Internal server errors (500, 503)
- Potential service outage
- Database or backend failures

**Investigation Steps:**

1. **Check pod health**
   ```bash
   kubectl get pods -n cap-verifier

   # Look for CrashLoopBackOff, Error, or Pending
   kubectl describe pod <failing-pod> -n cap-verifier
   ```

2. **Review panic/crash logs**
   ```bash
   kubectl logs deployment/cap-verifier -n cap-verifier --previous

   # Look for stack traces, panics, or fatal errors
   ```

3. **Check dependencies**
   ```bash
   # Test OAuth2 provider
   curl -v https://idp.example.com/.well-known/jwks.json

   # Test database connection (if applicable)
   kubectl exec -it deployment/cap-verifier -n cap-verifier -- \
     /bin/sh -c 'nc -zv postgres-service 5432'
   ```

4. **Check readiness/liveness probes**
   ```bash
   kubectl describe pod <pod-name> -n cap-verifier \
     | grep -A 10 "Liveness\|Readiness"
   ```

**Common Root Causes:**
- Database connection failure → Restore database
- OAuth2 provider down → Wait for recovery or disable auth (emergency)
- OOM (Out of Memory) → Increase memory limits
- Application bug → Deploy hotfix

**Resolution:**
```bash
# Emergency: Rollback to last known good version
kubectl rollout undo deployment/cap-verifier -n cap-verifier

# Restart pods (if transient issue)
kubectl delete pods -l app=cap-verifier -n cap-verifier

# Scale down to zero and back up (nuclear option)
kubectl scale deployment/cap-verifier --replicas=0 -n cap-verifier
kubectl scale deployment/cap-verifier --replicas=3 -n cap-verifier
```

**Escalation:**
- Page on-call engineer immediately
- Escalate to L2 after 30 minutes if unresolved
- Engage vendor support for dependency failures

---

### ⚠️ LowCacheHitRate

**Alert:** Cache hit rate < 80% for 10 minutes

**Severity:** P3 (Low)

**Symptoms:**
- Increased backend load
- Slightly elevated latency
- More database queries

**Investigation Steps:**

1. **Check cache metrics**
   ```bash
   curl http://cap-verifier-service:8080/metrics | grep cache

   # Look at:
   # - cap_verifier_cache_hits
   # - cap_verifier_cache_misses
   # - Ratio = hits / (hits + misses)
   ```

2. **Review cache configuration**
   ```bash
   kubectl get configmap cap-verifier-config -n cap-verifier \
     -o jsonpath='{.data.cache\.yaml}'
   ```

3. **Check cache eviction patterns**
   ```bash
   kubectl logs deployment/cap-verifier -n cap-verifier \
     | grep cache | grep evict
   ```

**Common Root Causes:**
- Cache size too small → Increase size
- TTL too short → Increase TTL
- Cache disabled → Enable cache
- New traffic pattern → Expected (wait for warmup)

**Resolution:**
```bash
# Increase cache size
helm upgrade cap-verifier . --reuse-values \
  --set cache.size=10000 \
  --set cache.ttl_seconds=7200

# Verify cache is enabled
kubectl get configmap cap-verifier-config -n cap-verifier \
  -o jsonpath='{.data.cache\.yaml}' | grep enabled
```

**Verification:**
```bash
# Wait 10 minutes for cache to warm up
sleep 600

# Check hit rate
curl http://cap-verifier-service:8080/metrics | grep cache_hit_ratio
# Should be > 0.8
```

---

### 🔴 HighAuthFailureRate

**Alert:** Auth failure rate > 10% for 5 minutes

**Severity:** P1 (High)

**Symptoms:**
- Users receiving 401 Unauthorized
- `cap_verifier_auth_failures_total` increasing rapidly
- Possible OAuth2 provider issue

**Investigation Steps:**

1. **Check OAuth2 provider status**
   ```bash
   # Test JWKS endpoint
   curl -I https://idp.example.com/.well-known/jwks.json

   # Expect: HTTP 200 OK
   ```

2. **Review auth failure logs**
   ```bash
   kubectl logs deployment/cap-verifier -n cap-verifier --tail=100 \
     | grep "auth_failure"
   ```

3. **Verify configuration**
   ```bash
   kubectl get configmap cap-verifier-config -n cap-verifier \
     -o jsonpath='{.data.auth\.yaml}'
   ```

4. **Test token validation**
   ```bash
   # Obtain fresh token
   TOKEN=$(curl -X POST https://auth.example.com/oauth/token \
     -d "grant_type=client_credentials" \
     -d "client_id=test-client" \
     -d "client_secret=test-secret" \
     | jq -r '.access_token')

   # Test with API
   curl -H "Authorization: Bearer $TOKEN" \
     https://cap-verifier.example.com/healthz
   ```

**Common Root Causes:**
- OAuth2 provider outage → Wait for recovery
- JWKS key rotation → Clear cache, wait for refresh
- Misconfigured issuer/audience → Update config
- Clock skew → Sync NTP

**Resolution:**
```bash
# Option 1: Restart pods to refresh JWKS cache
kubectl delete pods -l app=cap-verifier -n cap-verifier

# Option 2: Update OAuth2 config if misconfigured
helm upgrade cap-verifier . --reuse-values \
  --set oauth2.issuer="https://correct-issuer.example.com"

# Emergency: Temporarily disable auth (DANGEROUS!)
helm upgrade cap-verifier . --reuse-values \
  --set oauth2.enabled=false
# NOTE: Only use this in emergency, restore immediately after fix
```

**Escalation:**
- Engage OAuth2 provider support immediately
- Coordinate with security team for auth bypass (if critical)

---

### ⚠️ NoTraffic

**Alert:** No requests received for 5 minutes

**Severity:** P2 (Medium)

**Symptoms:**
- `cap_verifier_requests_total` not increasing
- Service appears unhealthy
- Potential networking issue

**Investigation Steps:**

1. **Check Ingress/LoadBalancer**
   ```bash
   kubectl get ingress -n cap-verifier
   kubectl describe ingress cap-verifier -n cap-verifier

   # Verify endpoints are registered
   ```

2. **Test internal service**
   ```bash
   kubectl run curl-test --image=curlimages/curl:latest -it --rm -- \
     curl http://cap-verifier-service:8080/healthz

   # Should return 200 OK
   ```

3. **Check NetworkPolicy**
   ```bash
   kubectl get networkpolicy -n cap-verifier

   # Ensure ingress rules allow traffic
   ```

4. **Verify DNS resolution**
   ```bash
   nslookup cap-verifier.example.com

   # Should resolve to LoadBalancer IP
   ```

**Common Root Causes:**
- Ingress misconfiguration → Fix ingress rules
- NetworkPolicy blocking traffic → Update policy
- DNS not pointing to service → Update DNS
- Certificate expired (TLS) → Renew certificate

**Resolution:**
```bash
# Check ingress configuration
kubectl get ingress cap-verifier -n cap-verifier -o yaml

# Test direct pod access
kubectl port-forward deployment/cap-verifier 8080:8080 -n cap-verifier
curl http://localhost:8080/healthz

# Fix ingress if needed
kubectl apply -f helm/templates/ingress.yaml -n cap-verifier
```

---

## Common Operations

### Graceful Restart

```bash
# Rolling restart (zero downtime)
kubectl rollout restart deployment/cap-verifier -n cap-verifier

# Wait for completion
kubectl rollout status deployment/cap-verifier -n cap-verifier
```

### Scale Up/Down

```bash
# Manual scaling
kubectl scale deployment/cap-verifier --replicas=10 -n cap-verifier

# Update HPA
helm upgrade cap-verifier . --reuse-values \
  --set autoscaling.maxReplicas=20
```

### View Logs (Structured)

```bash
# Real-time logs
kubectl logs -f deployment/cap-verifier -n cap-verifier

# Filter by level
kubectl logs deployment/cap-verifier -n cap-verifier \
  | jq 'select(.level=="ERROR")'

# Filter by time range
kubectl logs deployment/cap-verifier -n cap-verifier --since=1h

# Search for specific request_id
kubectl logs deployment/cap-verifier -n cap-verifier \
  | jq 'select(.request_id=="abc-123")'
```

### Update Configuration

```bash
# Edit ConfigMap
kubectl edit configmap cap-verifier-config -n cap-verifier

# Restart pods to pick up changes
kubectl rollout restart deployment/cap-verifier -n cap-verifier
```

### Certificate Rotation

```bash
# Create new secret
kubectl create secret tls cap-verifier-tls-new \
  --cert=new-server.crt \
  --key=new-server.key \
  --namespace cap-verifier

# Update deployment to use new secret
helm upgrade cap-verifier . --reuse-values \
  --set tls.secretName=cap-verifier-tls-new

# Delete old secret after verification
kubectl delete secret cap-verifier-tls -n cap-verifier
```

### Backup Registry

```bash
# Exec into pod
kubectl exec -it deployment/cap-verifier -n cap-verifier -- /bin/sh

# Inside pod: Copy registry database
cp /data/registry.sqlite /tmp/registry-backup.sqlite
exit

# Copy to local machine
kubectl cp cap-verifier-pod:/tmp/registry-backup.sqlite \
  ./backup/registry-$(date +%Y%m%d).sqlite

# Upload to S3
aws s3 cp ./backup/registry-$(date +%Y%m%d).sqlite \
  s3://backups/cap-verifier/
```

---

## Escalation

### Escalation Path

**Level 1 (L1) - On-Call Engineer**
- Initial investigation
- Follow runbooks
- Implement known fixes
- Escalate after 30 min (P0), 2 hours (P1)

**Level 2 (L2) - Senior Engineer/Team Lead**
- Deep debugging
- Code changes/hotfixes
- Architecture decisions
- Escalate after 2 hours (P0), 8 hours (P1)

**Level 3 (L3) - Engineering Manager**
- Resource allocation
- Vendor engagement
- Customer communication
- Executive escalation

### Contact Information

| Role | Contact | Availability |
|------|---------|--------------|
| On-Call Engineer | PagerDuty rotation | 24/7 |
| Team Lead | Slack #cap-verifier-ops | Business hours |
| Engineering Manager | email@example.com | Escalations only |
| OAuth2 Provider Support | vendor-support@example.com | 24/7 |

### Escalation Criteria

**Immediate Escalation (P0):**
- Service down > 15 minutes
- Data loss or corruption
- Security breach suspected

**Escalate to L2:**
- P0 unresolved after 30 minutes
- P1 unresolved after 2 hours
- Requires code changes

**Escalate to L3:**
- P0 unresolved after 2 hours
- Requires vendor engagement
- Customer-facing outage > 4 hours

---

## Maintenance Windows

### Planned Maintenance

**Frequency:** Monthly (first Tuesday, 02:00-04:00 UTC)

**Pre-maintenance Checklist:**
1. ☐ Announce in #cap-verifier-ops 48 hours ahead
2. ☐ Update status page
3. ☐ Backup all data
4. ☐ Test rollback procedure
5. ☐ Prepare runbook for maintenance

**During Maintenance:**
1. Start maintenance window in status page
2. Apply updates/changes
3. Verify all health checks pass
4. Monitor metrics for 30 minutes
5. Close maintenance window

**Post-Maintenance:**
1. Confirm service is healthy
2. Update documentation if needed
3. Post summary in #cap-verifier-ops

---

## Disaster Recovery

### Full Service Restore

**Scenario:** Complete cluster failure

**Steps:**
1. Provision new Kubernetes cluster
2. Install Helm with production values
3. Restore registry backup from S3
4. Verify DNS points to new LoadBalancer
5. Test end-to-end verification flow
6. Update status page

**RTO (Recovery Time Objective):** 4 hours
**RPO (Recovery Point Objective):** 24 hours (daily backups)

---

**Last Updated:** 2025-11-10
**Version:** v0.11.0
**Maintained by:** CAP SRE Team
