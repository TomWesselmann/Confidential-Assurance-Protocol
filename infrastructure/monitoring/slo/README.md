# SLO/SLI Monitoring System

## Übersicht

Dieses Verzeichnis enthält die Service Level Objective (SLO) und Service Level Indicator (SLI) Konfiguration für die CAP Verifier API. Das System basiert auf den Prinzipien aus dem Google SRE Workbook.

## Konzepte

### Service Level Indicators (SLIs)

SLIs sind quantitative Messungen der Service-Qualität:

1. **Availability SLI**: Prozentsatz erfolgreicher Requests
   - Metrik: `ok_requests / total_requests`
   - Ziel: > 99.9%

2. **Error Rate SLI**: Prozentsatz fehlgeschlagener Requests
   - Metrik: `fail_requests / total_requests`
   - Ziel: < 0.1%

3. **Auth Success SLI**: Erfolgsrate der Authentifizierung
   - Metrik: `(total_requests - auth_failures) / total_requests`
   - Ziel: > 99.95%

4. **Cache Hit Rate SLI**: Cache-Effizienz
   - Metrik: `cache_hit_ratio`
   - Ziel: > 70%

### Service Level Objectives (SLOs)

SLOs definieren das angestrebte Service-Level über einen Zeitraum:

| SLO Name | Target | Time Window | Error Budget |
|----------|--------|-------------|--------------|
| availability_999 | 99.9% | 30 days | 43.2 min/month |
| error_rate_001 | < 0.1% | 30 days | 0.1% |
| auth_success_9995 | 99.95% | 30 days | 0.05% |
| cache_hit_rate_70 | > 70% | 7 days | 30% |

### Error Budget

Das Error Budget ist die erlaubte Fehlerrate innerhalb eines Zeitfensters:

```
Error Budget = 100% - SLO Target
```

**Beispiel für Availability SLO (99.9%):**
- Error Budget: 0.1% (1 - 0.999)
- Bei 30 Tagen: 43.2 Minuten Downtime erlaubt
- Bei 1 Million Requests: 1000 Fehler erlaubt

### Burn Rate

Die Burn Rate misst, wie schnell das Error Budget verbraucht wird:

```
Burn Rate = (Current Error Rate) / (Error Budget Rate)
```

**Burn Rate Interpretation:**
- `1.0` = Normal (Budget wird in der geplanten Zeit verbraucht)
- `> 6.0` = Warnung (Budget verbrennt 6x schneller als geplant)
- `> 14.4` = Kritisch (Budget verbrennt 14.4x schneller als geplant)

## Dateien

### slo-config.yml

Zentrale Konfiguration aller SLOs und SLIs:

```yaml
version: "slo.v1"
service_name: "cap-verifier-api"

slis:
  - name: "availability"
    description: "Ratio of successful requests"
    metric: |
      sum(rate(cap_verifier_requests_total{result="ok"}[5m]))
      /
      sum(rate(cap_verifier_requests_total[5m]))

slos:
  - name: "availability_999"
    sli: "availability"
    target: 0.999  # 99.9%
    time_window: "30_days"
    error_budget: 0.001
```

### Grafana Dashboard (slo-monitoring.json)

Das Dashboard visualisiert:

1. **SLO Compliance Overview**
   - Aktuelle SLI-Werte für alle SLOs
   - Farbkodierung: Grün (OK), Gelb (Warning), Rot (Violation)

2. **Error Budget Status**
   - Verbleibendes Error Budget (Gauge Charts)
   - Berechnung: `(1 - actual_error_rate / slo_error_budget) * 100`

3. **Error Budget Burn Rate**
   - 1h und 6h Burn Rate Trends
   - Alert-Schwellenwerte visualisiert

4. **SLI Trends (30 Days)**
   - Historische SLI-Werte
   - SLO-Threshold-Linien

## Prometheus Queries

### Availability SLI
```promql
sum(rate(cap_verifier_requests_total{result="ok"}[5m]))
/
sum(rate(cap_verifier_requests_total[5m]))
```

### Error Budget Remaining (Availability)
```promql
clamp_min(
  (1 - (1 - (
    sum(rate(cap_verifier_requests_total{result="ok"}[30d]))
    /
    sum(rate(cap_verifier_requests_total[30d]))
  )) / 0.001) * 100,
  0
)
```

### Burn Rate (1h Lookback)
```promql
(1 - (
  sum(rate(cap_verifier_requests_total{result="ok"}[1h]))
  /
  sum(rate(cap_verifier_requests_total[1h]))
)) / 0.001
```

## Alerting Rules

Die Prometheus Alerting Rules (bereits in `monitoring/prometheus/alerts/cap-verifier-rules.yml` definiert) enthalten:

### CAPVerifierSLOErrorBudgetBurning
```yaml
- alert: CAPVerifierSLOErrorBudgetBurning
  expr: |
    (1 - (
      sum(rate(cap_verifier_requests_total{result="ok"}[1h]))
      /
      sum(rate(cap_verifier_requests_total[1h]))
    )) > 0.001
  for: 5m
  labels:
    severity: warning
    slo: "availability"
  annotations:
    summary: "CAP Verifier API SLO error budget is burning fast"
    description: "Current error rate exceeds SLO target (99.9% availability)"
```

## Error Budget Policies

### Slow Rollout Policy
**Trigger:** Error Budget < 25% remaining

**Actions:**
- Pause automated deployments
- Require manual approval for releases
- Increase monitoring cadence

### Emergency Freeze Policy
**Trigger:** Error Budget < 5% remaining

**Actions:**
- Freeze all deployments
- Activate incident response team
- Require root cause analysis before next release

## Integration

### Grafana
1. Dashboard automatisch provisioniert via `provisioning/dashboards/dashboards.yml`
2. Zugriff: http://grafana:3000/d/slo-monitoring

### Prometheus
1. Metrics werden von API exportiert (Port 8080/metrics)
2. Scrape-Interval: 15s (konfiguriert in `prometheus.yml`)

### Alertmanager
1. Alerts werden an Alertmanager weitergeleitet
2. Routing-Konfiguration in `alertmanager.yml` definieren

## Verwendung

### Dashboard öffnen
```bash
# Grafana Dashboard
open http://localhost:3000/d/slo-monitoring

# Prometheus Alerts
open http://localhost:9090/alerts
```

### Error Budget prüfen
```bash
# Availability Error Budget (via PromQL)
curl -G http://localhost:9090/api/v1/query \
  --data-urlencode 'query=clamp_min((1 - (1 - (sum(rate(cap_verifier_requests_total{result="ok"}[30d])) / sum(rate(cap_verifier_requests_total[30d])))) / 0.001) * 100, 0)'
```

### SLO Reports generieren
```bash
# Weekly SLO Report (zukünftiges Feature)
cargo run -- slo report --window 7d --format json > slo-report.json
```

## Best Practices

### 1. Regelmäßige Reviews
- Wöchentliche SLO-Compliance-Reviews
- Monatliche SLO-Target-Anpassungen basierend auf Business-Anforderungen

### 2. Error Budget Tracking
- Error Budget als Entscheidungsgrundlage für Releases nutzen
- Bei niedrigem Error Budget: Stabilität vor Features priorisieren

### 3. Incident Response
- Bei SLO-Verletzung: Incident-Tracking starten
- Post-Mortem bei Error Budget < 10%

### 4. Continuous Improvement
- SLI-Metriken kontinuierlich verfeinern
- SLO-Targets an Business-Needs anpassen
- Neue SLIs für zusätzliche Service-Aspekte definieren

## Erweiterungen (Zukünftig)

1. **Latency SLI**: P50, P95, P99 Latenz-Metriken
2. **Throughput SLI**: Requests pro Sekunde
3. **Data Freshness SLI**: Aktualität von Cache-Daten
4. **Multi-Region SLOs**: Region-spezifische SLOs
5. **User-Journey SLOs**: End-to-End SLOs für kritische Workflows

## Ressourcen

- [Google SRE Workbook: Implementing SLOs](https://sre.google/workbook/implementing-slos/)
- [The Site Reliability Workbook (O'Reilly)](https://sre.google/books/workbook/)
- [Prometheus Recording Rules](https://prometheus.io/docs/prometheus/latest/configuration/recording_rules/)
- [Grafana Alerting](https://grafana.com/docs/grafana/latest/alerting/)

## Lizenz

© 2025 - Alle Rechte vorbehalten
