# k6 Load Tests - Week 4

## Voraussetzungen

### k6 Installation

**macOS:**
```bash
brew install k6
```

**Linux:**
```bash
# Debian/Ubuntu
sudo gpg -k
sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
sudo apt-get update
sudo apt-get install k6
```

**Windows:**
```bash
choco install k6
```

## Ausführung

### 1. Server starten

```bash
# Terminal 1: Start REST API server
cargo run --bin cap-verifier-api
```

### 2. Load Test ausführen

```bash
# Terminal 2: Run load test
k6 run k6/verify.js
```

### 3. Mit Custom Environment

```bash
# Custom BASE URL and TOKEN
BASE=http://localhost:8080 TOKEN=<your-jwt-token> k6 run k6/verify.js
```

### 4. Mit Summary Export

```bash
# Export results to JSON
mkdir -p reports
k6 run --summary-export=reports/load_week4.json k6/verify.js
```

## Test-Konfiguration

| Parameter | Wert | Beschreibung |
|-----------|------|--------------|
| **RPS** | 50 | Requests Per Second |
| **Duration** | 3min | Test-Dauer |
| **VUs** | 10-50 | Virtual Users (dynamisch) |
| **Executor** | constant-arrival-rate | Konstante Request-Rate |

## Performance-Ziele (DoD)

| Metrik | Ziel | Status |
|--------|------|--------|
| **p95 Latency** | < 500 ms | ✅/❌ |
| **Error Rate** | < 1% | ✅/❌ |
| **HTTP Failures** | < 1% | ✅/❌ |

## Output-Interpretation

### Erfolgreiche Test-Ausgabe:

```
📈 Load Test Summary
==================================================
Total Requests: 9000
Actual RPS: 50.12
p95 Latency: 245.32ms ✅
p99 Latency: 312.45ms
Avg Latency: 128.67ms
Error Rate: 0.02% ✅
Failed Requests: 0.00%
==================================================
```

### Metriken

**http_req_duration**: HTTP Request-Dauer
- `avg`: Durchschnitt
- `p(95)`: 95. Perzentil
- `p(99)`: 99. Perzentil

**errors**: Custom Error Rate (basierend auf Response-Validierung)

**http_req_failed**: HTTP-Fehlerrate (4xx/5xx)

## Test-Payload

Der Test verwendet **Mode B (Embedded IR)** mit Mock-Backend:

```json
{
  "ir": { ... },
  "context": {
    "supplier_hashes": ["0x1234..."],
    "ubo_hashes": ["0x1111..."],
    "sanctions_root": "0x0000..."
  },
  "backend": "mock",
  "options": {
    "adaptive": false,
    "check_timestamp": false,
    "check_registry": false
  }
}
```

## Troubleshooting

### Server nicht erreichbar

**Fehler:**
```
Server not ready: 0
```

**Lösung:**
```bash
# Prüfen ob Server läuft
curl http://localhost:8080/healthz

# Server neu starten
cargo run --bin cap-verifier-api
```

### Hohe Error Rate

**Mögliche Ursachen:**
1. Server überlastet → VUs reduzieren
2. OAuth2 Token ungültig → `TOKEN` env var setzen
3. Mock-Backend-Fehler → Server-Logs prüfen

### p95 Latency > 500ms

**Optimierungen:**
1. Server im Release-Modus starten: `cargo run --release --bin cap-verifier-api`
2. RPS reduzieren für Baseline-Test
3. Hardware-Ressourcen prüfen (CPU, Memory)

## CI Integration

### GitHub Actions Example:

```yaml
- name: Install k6
  run: |
    brew install k6  # macOS runner

- name: Start API Server
  run: |
    cargo run --release --bin cap-verifier-api &
    sleep 5  # Wait for server startup

- name: Run Load Test
  run: |
    k6 run --summary-export=reports/load_week4.json k6/verify.js

- name: Upload Results
  uses: actions/upload-artifact@v4
  with:
    name: load-test-results
    path: reports/load_week4.json
```

## Advanced Usage

### Custom Scenarios

Erweitere `verify.js` mit weiteren Szenarien:

```javascript
export const options = {
  scenarios: {
    smoke: {
      executor: 'constant-vus',
      vus: 1,
      duration: '30s',
    },
    stress: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '2m', target: 100 },
        { duration: '5m', target: 100 },
        { duration: '2m', target: 0 },
      ],
    },
  },
};
```

### Thresholds Anpassen

```javascript
thresholds: {
  'http_req_duration': ['p(95)<300', 'p(99)<500'],  // Strenger
  'errors': ['rate<0.005'],                          // < 0.5%
}
```

## Reports

Alle Load-Test-Reports werden in `reports/` gespeichert:

```
reports/
├── load_week4.json       # Vollständiger k6 JSON-Report
└── load_week4.txt        # Text-Summary (optional)
```

## Referenzen

- [k6 Documentation](https://k6.io/docs/)
- [k6 Metrics](https://k6.io/docs/using-k6/metrics/)
- [k6 Thresholds](https://k6.io/docs/using-k6/thresholds/)
- [Week 4 Execution Guide](/Users/tomwesselmann/Desktop/Week4_Execution.md)
