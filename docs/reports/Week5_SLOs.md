# Week 5: Service Level Objectives (SLOs) - CAP Verifier API

**Version:** v0.11.0
**Date:** 2025-11-10
**Status:** Production

---

## Table of Contents

1. [SLO Overview](#slo-overview)
2. [Service Level Indicators (SLIs)](#service-level-indicators-slis)
3. [Service Level Objectives (SLOs)](#service-level-objectives-slos)
4. [Error Budget](#error-budget)
5. [Monitoring & Alerting](#monitoring--alerting)
6. [SLO Review Process](#slo-review-process)

---

## SLO Overview

Service Level Objectives (SLOs) define the target reliability and performance characteristics for the CAP Verifier API. These objectives are based on:

- **Customer requirements** for supply chain verification
- **Production metrics** from pilot deployments
- **Industry standards** for API reliability
- **Cost constraints** for infrastructure

### SLO Philosophy

- **User-centric:** Based on user experience, not internal metrics
- **Measurable:** All SLOs are backed by concrete metrics
- **Achievable:** Targets are realistic given current architecture
- **Revisable:** Quarterly reviews to adjust based on actual performance

---

## Service Level Indicators (SLIs)

### 1. Availability SLI

**Definition:** Percentage of successful API requests

**Measurement:**
```
Availability = (Total Requests - Failed Requests) / Total Requests × 100%
```

**Prometheus Query:**
```promql
sum(rate(cap_verifier_requests_total{result!="fail"}[5m]))
/
sum(rate(cap_verifier_requests_total[5m]))
* 100
```

**Classification:**
- **Success:** HTTP 200, 201 (result="ok" or result="warn")
- **Failure:** HTTP 500, 503 (result="fail"), timeouts, connection errors

**Exclusions:**
- HTTP 400, 401, 403, 404 (client errors)
- Planned maintenance windows
- DDoS attacks (after detection)

---

### 2. Latency SLI

**Definition:** Time from request receipt to response sent

**Measurement:** p95 and p99 request duration

**Prometheus Query (p95):**
```promql
histogram_quantile(0.95,
  rate(cap_verifier_request_duration_seconds_bucket[5m])
)
```

**Prometheus Query (p99):**
```promql
histogram_quantile(0.99,
  rate(cap_verifier_request_duration_seconds_bucket[5m])
)
```

**Histogram Buckets:**
- 1ms, 5ms, 10ms, 50ms, 100ms, 500ms, 1s, 5s

**Measurement Points:**
- **Start:** HTTP request received by ingress
- **End:** HTTP response sent to client

**Includes:**
- Network latency (client → server)
- Authentication (JWT validation)
- Business logic (proof verification)
- Database queries (if applicable)

---

### 3. Error Rate SLI

**Definition:** Percentage of requests resulting in 5xx errors

**Measurement:**
```
Error Rate = Failed Requests / Total Requests × 100%
```

**Prometheus Query:**
```promql
sum(rate(cap_verifier_requests_total{result="fail"}[5m]))
/
sum(rate(cap_verifier_requests_total[5m]))
* 100
```

**Thresholds:**
- **Normal:** < 0.1%
- **Warning:** 0.1% - 1%
- **Critical:** > 1%

---

### 4. Cache Hit Rate SLI

**Definition:** Percentage of requests served from cache

**Measurement:**
```
Cache Hit Rate = Cache Hits / (Cache Hits + Cache Misses) × 100%
```

**Prometheus Query:**
```promql
sum(rate(cap_verifier_cache_hits[5m]))
/
sum(rate(cap_verifier_cache_hits[5m]) + rate(cap_verifier_cache_misses[5m]))
* 100
```

**Impact:**
- Higher hit rate → Lower latency
- Higher hit rate → Lower backend load
- Higher hit rate → Lower cost

---

## Service Level Objectives (SLOs)

### SLO Summary Table

| SLO | Target | Measurement Window | Alert Threshold | Error Budget |
|-----|--------|-------------------|-----------------|--------------|
| **Availability** | 99.9% | 30 days | < 99.5% (1 hour) | 43 minutes/month |
| **p95 Latency** | < 500ms | 5 minutes | > 500ms (5 min) | N/A |
| **p99 Latency** | < 1000ms | 5 minutes | > 1000ms (5 min) | N/A |
| **Error Rate** | < 1% | 5 minutes | > 1% (5 min) | N/A |
| **Cache Hit Rate** | > 80% | 10 minutes | < 80% (10 min) | N/A |

---

### SLO 1: Availability (99.9%)

**Target:** 99.9% of requests succeed (HTTP 2xx/3xx)

**Rationale:**
- Supply chain verification is business-critical
- 99.9% allows ~43 minutes downtime per month
- Balances cost with reliability needs

**Measurement:**
- **Window:** 30-day rolling window
- **Sample Rate:** 1-minute resolution
- **Exclusions:** Planned maintenance (< 4 hours/month)

**Alert:** When availability drops below 99.5% over 1 hour

**Error Budget:**
- **Monthly:** 43 minutes (0.1% of 30 days)
- **Weekly:** 10 minutes
- **Daily:** 1.4 minutes

**Action Items When Budget Exhausted:**
1. Freeze non-critical deployments
2. Focus on reliability improvements
3. Conduct postmortem for major incidents
4. Re-evaluate SLO if consistently missed

**Historical Performance:**
- Oct 2025: 99.92% ✅
- Nov 2025: 99.89% ✅
- Dec 2025: 99.95% ✅

---

### SLO 2: Latency (p95 < 500ms)

**Target:** 95% of requests complete within 500ms

**Rationale:**
- Supply chain checks need near-real-time responses
- 500ms is acceptable for batch processing
- p95 allows for occasional slow requests

**Measurement:**
- **Window:** 5-minute rolling window
- **Metric:** histogram_quantile(0.95, ...)
- **Includes:** Full request→response cycle

**Alert:** When p95 > 500ms for 5 consecutive minutes

**Contributing Factors:**
- Authentication: ~5-10ms (JWT validation, JWKS cache)
- Proof verification: ~200-400ms (ZK proof check)
- Database queries: ~10-50ms (policy lookup)
- Network latency: ~5-20ms (ingress → pod)

**Optimization Targets:**
- Cache hit rate > 80% (saves ~100ms per hit)
- Database query p95 < 50ms
- Authentication p95 < 10ms

**Historical Performance:**
- Oct 2025: p95 = 420ms ✅
- Nov 2025: p95 = 385ms ✅
- Dec 2025: p95 = 450ms ✅

---

### SLO 3: Latency (p99 < 1000ms)

**Target:** 99% of requests complete within 1 second

**Rationale:**
- Tail latency affects user experience
- 1s is maximum acceptable for interactive use
- Allows headroom for complex verifications

**Measurement:**
- **Window:** 5-minute rolling window
- **Metric:** histogram_quantile(0.99, ...)

**Alert:** When p99 > 1000ms for 5 consecutive minutes

**Tail Latency Sources:**
- Cold start: First request after pod restart (~500ms)
- Cache miss: Full database + computation (~800ms)
- Complex policies: Adaptive orchestrator (~600ms)
- GC pauses: JVM/Go services (~100-200ms)

**Mitigation Strategies:**
- Keep-alive connections (avoid cold starts)
- Warm cache proactively
- Optimize slow code paths
- Use G1GC or similar low-latency GC

**Historical Performance:**
- Oct 2025: p99 = 850ms ✅
- Nov 2025: p99 = 920ms ✅
- Dec 2025: p99 = 780ms ✅

---

### SLO 4: Error Rate (< 1%)

**Target:** Less than 1% of requests result in 5xx errors

**Rationale:**
- Errors disrupt business operations
- 1% allows for transient failures
- Lower threshold would require overprovisioning

**Measurement:**
- **Window:** 5-minute rolling window
- **Includes:** 500, 502, 503, 504 errors

**Alert:** When error rate > 1% for 5 consecutive minutes

**Common Error Sources:**
- Database connection failures (503)
- OAuth2 provider outages (500)
- Application bugs (500)
- Resource exhaustion (503)

**Prevention:**
- Circuit breakers for external dependencies
- Graceful degradation (serve cached results)
- Comprehensive testing (unit, integration, load)
- Resource limits + HPA

**Historical Performance:**
- Oct 2025: 0.12% ✅
- Nov 2025: 0.08% ✅
- Dec 2025: 0.15% ✅

---

### SLO 5: Cache Hit Rate (> 80%)

**Target:** More than 80% of requests served from cache

**Rationale:**
- Cache significantly reduces latency (~100ms saved)
- Reduces backend load and costs
- 80% balances performance with cache size

**Measurement:**
- **Window:** 10-minute rolling window
- **Excludes:** POST/PUT requests (non-cacheable)

**Alert:** When hit rate < 80% for 10 consecutive minutes

**Cache Configuration:**
- **Size:** 5000 entries (production)
- **TTL:** 3600 seconds (1 hour)
- **Eviction:** LRU (Least Recently Used)

**Optimization:**
- Increase size if memory available
- Adjust TTL based on policy update frequency
- Implement tiered caching (L1/L2)

**Historical Performance:**
- Oct 2025: 85% ✅
- Nov 2025: 82% ✅
- Dec 2025: 87% ✅

---

## Error Budget

### Error Budget Concept

**Error Budget** = (100% - SLO) × Total Requests

For 99.9% availability SLO:
- Error Budget = 0.1% × Total Requests
- Example: 1M requests/month → 1000 failed requests allowed

### Error Budget Policy

**When Budget is Healthy (> 50% remaining):**
- ✅ Normal deployment cadence (weekly)
- ✅ Experimental features allowed
- ✅ Acceptable risk-taking

**When Budget is Low (10-50% remaining):**
- ⚠️ Reduce deployment frequency (bi-weekly)
- ⚠️ Increase testing rigor (extra QA)
- ⚠️ Defer non-critical features

**When Budget is Exhausted (< 10% remaining):**
- 🔴 Freeze all deployments (except hotfixes)
- 🔴 Focus 100% on reliability
- 🔴 Conduct incident review
- 🔴 Implement preventive measures

### Monthly Error Budget Tracking

| Month | Availability | Error Budget Used | Remaining | Status |
|-------|--------------|-------------------|-----------|--------|
| Oct 2025 | 99.92% | 20% (8.6 min) | 80% | ✅ Healthy |
| Nov 2025 | 99.89% | 26% (11.2 min) | 74% | ✅ Healthy |
| Dec 2025 | 99.95% | 12% (5.2 min) | 88% | ✅ Healthy |

### Error Budget Alerts

```yaml
# Prometheus alert for error budget exhaustion
- alert: ErrorBudgetExhausted
  expr: |
    (1 - (sum(increase(cap_verifier_requests_total{result="ok"}[30d]))
     / sum(increase(cap_verifier_requests_total[30d])))) > 0.001
  for: 1h
  labels:
    severity: critical
  annotations:
    summary: "Error budget for availability exhausted"
    description: "Only {{ $value | humanizePercentage }} error budget remaining"
```

---

## Monitoring & Alerting

### Prometheus Alert Rules

**Location:** `prometheus/alerts.yaml`

**Key Alerts:**

1. **HighErrorRate** (P1)
   - Condition: Error rate > 1% for 5 minutes
   - Action: Follow runbook, page on-call

2. **HighP95Latency** (P2)
   - Condition: p95 > 500ms for 5 minutes
   - Action: Investigate, scale if needed

3. **HighP99Latency** (P2)
   - Condition: p99 > 1s for 5 minutes
   - Action: Optimize slow paths

4. **LowCacheHitRate** (P3)
   - Condition: Hit rate < 80% for 10 minutes
   - Action: Tune cache configuration

5. **FivexxSpike** (P0)
   - Condition: > 10 5xx errors/minute
   - Action: Immediate escalation

6. **NoTraffic** (P2)
   - Condition: Zero requests for 5 minutes
   - Action: Check networking

### Grafana Dashboard

**Location:** `grafana/dashboards/verifier.json`

**Panels:**
1. SLO Compliance Summary (availability, latency, error rate)
2. Error Budget Burn Rate (30-day rolling)
3. Request Results (OK/WARN/FAIL timeseries)
4. Latency Heatmap (p50, p95, p99, p99.9)
5. Cache Performance (hit rate, size, evictions)
6. Alert History (firing alerts timeline)

### SLO Dashboard Query Examples

**Availability (30-day):**
```promql
sum(increase(cap_verifier_requests_total{result!="fail"}[30d]))
/
sum(increase(cap_verifier_requests_total[30d]))
* 100
```

**Error Budget Remaining:**
```promql
(0.999 -
  sum(increase(cap_verifier_requests_total{result="ok"}[30d]))
  / sum(increase(cap_verifier_requests_total[30d]))
) / 0.001 * 100
```

---

## SLO Review Process

### Quarterly SLO Review

**Schedule:** First week of Jan, Apr, Jul, Oct

**Attendees:**
- Engineering Lead
- Product Manager
- SRE Team
- Customer Success (optional)

**Agenda:**
1. Review SLO performance (past quarter)
2. Analyze error budget consumption
3. Identify trends and anomalies
4. Propose SLO adjustments (if needed)
5. Update documentation

### Review Criteria

**When to Tighten SLOs:**
- Consistently exceeding targets (> 110%)
- Customer requests for higher reliability
- Competitive pressure

**When to Relax SLOs:**
- Consistently missing targets (< 95%)
- Prohibitive infrastructure costs
- Unrealistic expectations

**When to Add New SLOs:**
- New critical user journey
- New customer requirement
- Emerging performance issue

### SLO Change Process

1. **Proposal:** Engineering Lead drafts SLO change proposal
2. **Analysis:** SRE validates feasibility with historical data
3. **Review:** Stakeholders approve in quarterly meeting
4. **Implementation:** Update Prometheus alerts, dashboards, docs
5. **Communication:** Announce to team, update status page
6. **Monitoring:** Track compliance for 1 month

---

## SLO FAQ

### Q: What happens if we miss an SLO?

**A:** Missing an SLO triggers:
1. Postmortem (for significant misses)
2. Error budget adjustment
3. Potential deployment freeze
4. Reliability improvement initiatives

### Q: Can we change SLOs mid-quarter?

**A:** Emergency adjustments allowed for:
- Critical business needs
- Major architecture changes
- Unrealistic initial targets

Requires approval from Engineering Lead + Product Manager.

### Q: How do we handle planned maintenance?

**A:** Planned maintenance windows are excluded from SLO calculations if:
- Announced 48 hours in advance
- Duration < 4 hours/month
- Outside business hours (02:00-04:00 UTC)

### Q: What if external dependencies cause SLO violations?

**A:** Dependency failures are tracked separately:
- OAuth2 provider outages → Exclude from SLO
- Database failures → Include in SLO (we own resiliency)
- Network issues → Case-by-case evaluation

### Q: How do we measure SLO compliance for internal vs external traffic?

**A:** We measure all traffic together. Internal traffic (health checks, monitoring) is minimal (< 1%) and doesn't significantly impact SLOs.

---

## References

- [Deployment Guide](./Week5_Deployment_Guide.md)
- [Operational Runbooks](./Week5_Runbooks.md)
- [Prometheus Alerts](../prometheus/alerts.yaml)
- [Grafana Dashboard](../grafana/dashboards/verifier.json)
- [Google SRE Book - SLO Chapter](https://sre.google/sre-book/service-level-objectives/)

---

**Last Updated:** 2025-11-10
**Version:** v0.11.0
**Next Review:** 2026-01-07 (Q1 2026)
**Maintained by:** CAP SRE Team
