
# 🧭 Week 6 — **B2 Instruction Pack**: Drift‑Analyse & Readiness‑Gate (für Claude Code)

**Kontext:** Week‑6 Track A (Prod Cutover) und B1 (Enforcer + CLI/Flags + Basis‑Metriken) sind fertig.  
**Ziel B2:** Implementiere **Drift‑Analyse** (rolling windows, Prometheus‑Export) und eine **optionale Readiness‑Schranke** (Gate) für `/readyz`, damit progressive Enforcement‑Rollouts sicher bleiben.

**Leitplanken:** fail‑closed, deterministisch, **keine PII** in Logs/Metriken, geringe Latenz (< 1 ms p95 je Aufnahme).

---

## ✅ Deliverables (Ende B2)

1. **Drift‑Modul** `src/orchestrator/drift.rs`  
   - Rolling Ratio für **5m / 15m / 60m** (konfigurierbar, Bounded‑Ringbuffer)  
   - API: `record(shadow, enforced, policy_id)`; `ratio_5m()`, `ratio_15m()`, `ratio_60m()`  

2. **Readiness‑Gate** `src/orchestrator/gate.rs` (optional per Flag)  
   - Schaltet `/readyz` → **503**, wenn `drift_ratio > drift_max_ratio`  
   - Helm‑Flag: `enforce_readiness_gate: {enabled: true, window: "5m"}`  

3. **Prometheus‑Metriken** (Exporter bereits vorhanden)  
   - `adapt_drift_events_total{policy_id}` (Counter)  
   - `adapt_drift_ratio{window="5m|15m|60m"}` (Gauge)  
   - `adapt_enforce_rollout_percent` (Gauge; B1) – **weiter benutzen**  

4. **Grafana‑Patch** (verifier.json)  
   - Panels: **Drift Ratio (5m/15m/60m)**, **Drift Events (rate 5m)**, **Rollout %**  

5. **Tests**  
   - `tests/orchestrator_drift_metrics.rs`  
   - `tests/orchestrator_enforce_gate.rs`  

6. **Runbook‑Updates**  
   - `docs/runbook_rollout.md` (Gate & Schwellen), `docs/runbook_restore.md` (keine Änderung), `docs/runbook_rotation.md` (keine Änderung)  

---

## 🗂️ Neue/Geänderte Dateien

```
src/
  orchestrator/
    drift.rs     # NEU – Rolling windows + Prometheus Binding
    gate.rs      # NEU – Readiness Gate
    enforcer.rs  # B1 – ruft jetzt drift::record(...) auf
  http/
    readyz.rs    # Gate‑Hook (optional) in Readiness‑Pfad
tests/
  orchestrator_drift_metrics.rs   # NEU – Metriken/Ratio
  orchestrator_enforce_gate.rs    # NEU – Gate Verhalten
grafana/
  dashboards/verifier.json        # Panels ergänzen
helm/
  values-prod.yaml                # Flags: enforce_readiness_gate, drift_max_ratio, window
docs/
  runbook_rollout.md              # Gate‑Prozeduren & Grenzwerte
```

---

## 🧠 Spezifikation: Drift‑Tracking

**Definition**  
Drift entsteht, wenn **Shadow‑Verdikt ≠ Enforced‑Verdikt**. Drift‑Ratio = `drift_events / total_requests` in einem Zeitfenster.

**Rolling Windows**  
- 5 Minuten: 60 Buckets × 5‑Sekunden (empfohlen)  
- 15 Minuten: 60 Buckets × 15‑Sekunden **oder** 90 × 10‑Sekunden  
- 60 Minuten: 60 Buckets × 60‑Sekunden  

**Anforderungen**  
- O(1) Update pro Request, lock‑light (RwLock/Atomic).  
- Keine PII: Aggregation nur pro **policy_id** (Label), keine Context‑Details.

**Rust‑Skeleton `src/orchestrator/drift.rs`:**
```rust
use std::time::{Duration, Instant};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict { Ok, Warn, Fail }

#[derive(Default, Clone)]
struct Bucket { total: u64, drift: u64 }

#[derive(Clone)]
pub struct RollingWindow {
    buckets: Arc<RwLock<Vec<Bucket>>>,
    head: Arc<RwLock<usize>>,
    step: Duration,         // z.B. 5s
    last_advance: Arc<RwLock<Instant>>,
}

impl RollingWindow {
    pub fn new(bucket_count: usize, step: Duration) -> Self {
        Self {
            buckets: Arc::new(RwLock::new(vec![Bucket::default(); bucket_count])),
            head: Arc::new(RwLock::new(0)),
            step,
            last_advance: Arc::new(RwLock::new(Instant::now())),
        }
    }
    fn advance_if_needed(&self) {
        let mut last = self.last_advance.write().unwrap();
        let mut head = self.head.write().unwrap();
        while last.elapsed() >= self.step {
            *last += self.step;
            *head = (*head + 1) % self.buckets.read().unwrap().len();
            self.buckets.write().unwrap()[*head] = Bucket::default();
        }
    }
    pub fn record(&self, is_drift: bool) {
        self.advance_if_needed();
        let head = *self.head.read().unwrap();
        let mut buckets = self.buckets.write().unwrap();
        let b = &mut buckets[head];
        b.total += 1;
        if is_drift { b.drift += 1; }
    }
    pub fn ratio(&self) -> f64 {
        self.advance_if_needed();
        let buckets = self.buckets.read().unwrap();
        let (mut t, mut d) = (0u64, 0u64);
        for b in buckets.iter() { t += b.total; d += b.drift; }
        if t == 0 { 0.0 } else { d as f64 / t as f64 }
    }
}

pub struct DriftTracker {
    pub w5m: RollingWindow,
    pub w15m: RollingWindow,
    pub w60m: RollingWindow,
    // optional: per-policy map
    pub per_policy: RwLock<HashMap<String, RollingWindow>>,
}

impl DriftTracker {
    pub fn new() -> Self {
        Self {
            w5m: RollingWindow::new(60, Duration::from_secs(5)),
            w15m: RollingWindow::new(60, Duration::from_secs(15)),
            w60m: RollingWindow::new(60, Duration::from_secs(60)),
            per_policy: RwLock::new(HashMap::new()),
        }
    }
    pub fn record(&self, policy_id: &str, shadow: Verdict, enforced: Verdict) -> bool {
        let is_drift = shadow != enforced;
        self.w5m.record(is_drift);
        self.w15m.record(is_drift);
        self.w60m.record(is_drift);
        // policy-spezifisch (lazy init)
        let mut map = self.per_policy.write().unwrap();
        let entry = map.entry(policy_id.to_string())
            .or_insert_with(|| RollingWindow::new(60, Duration::from_secs(5)));
        entry.record(is_drift);
        is_drift
    }
    pub fn ratio_5m(&self) -> f64 { self.w5m.ratio() }
    pub fn ratio_15m(&self) -> f64 { self.w15m.ratio() }
    pub fn ratio_60m(&self) -> f64 { self.w60m.ratio() }
    pub fn ratio_5m_policy(&self, policy_id: &str) -> f64 {
        self.per_policy.read().unwrap()
            .get(policy_id).map(|w| w.ratio()).unwrap_or(0.0)
    }
}
```

**Prometheus‑Binding (Beispiel in Enforcer‑Pfad):**
```rust
use crate::orchestrator::metrics::{ADAPT_DRIFT_EVENTS_TOTAL, ADAPT_DRIFT_RATIO_5M, ADAPT_DRIFT_RATIO_15M, ADAPT_DRIFT_RATIO_60M};

let is_drift = tracker.record(&policy_id, shadow, enforced);
if is_drift {
    ADAPT_DRIFT_EVENTS_TOTAL.with_label_values(&[&policy_id]).inc();
}
ADAPT_DRIFT_RATIO_5M.set(tracker.ratio_5m());
ADAPT_DRIFT_RATIO_15M.set(tracker.ratio_15m());
ADAPT_DRIFT_RATIO_60M.set(tracker.ratio_60m());
```

---

## 🧰 Readiness‑Gate (optional)

**Zweck:** Schützt Prod bei Drift‑Peaks während des Enforce‑Rollouts.

**Konfiguration (Helm):**
```yaml
enforce_readiness_gate:
  enabled: true
  window: "5m"        # zulässig: 5m|15m|60m
drift_max_ratio: 0.005  # 0.5 %
```

**Verhalten:**  
- Wenn `enabled=true` **und** `ratio(window) > drift_max_ratio` → `/readyz` liefert **503** + kurze Reason (`"DRIFT_GATE"`) ohne PII.  
- Feature muss **abschaltbar** sein (Blue‑Green, Wartung).

**Rust‑Skeleton `src/orchestrator/gate.rs`:**
```rust
pub struct GateCfg { pub enabled: bool, pub window: String, pub max_ratio: f64 }

pub fn readiness_gate_ok(cfg: &GateCfg, tracker: &DriftTracker) -> (bool, &'static str) {
    if !cfg.enabled { return (true, "OK"); }
    let ratio = match cfg.window.as_str() {
        "15m" => tracker.ratio_15m(),
        "60m" => tracker.ratio_60m(),
        _     => tracker.ratio_5m(),
    };
    if ratio > cfg.max_ratio {
        (false, "DRIFT_GATE")
    } else {
        (true, "OK")
    }
}
```

**HTTP‑Hook (pseudo):**
```rust
// in /readyz Handler
let (ok, reason) = gate::readiness_gate_ok(&cfg.gate, &tracker);
if ok { return 200; } else { return 503 with json { "reason": reason } }
```

---

## 📊 Prometheus – Metriken (Ergänzung)

```rust
// src/orchestrator/metrics.rs (erweitern)
pub static ADAPT_DRIFT_RATIO_15M: Lazy<Gauge> =
    Lazy::new(|| register_gauge!("adapt_drift_ratio_15m", "Drift ratio (15m)").unwrap());
pub static ADAPT_DRIFT_RATIO_60M: Lazy<Gauge> =
    Lazy::new(|| register_gauge!("adapt_drift_ratio_60m", "Drift ratio (60m)").unwrap());
// ADAPT_DRIFT_RATIO_5M existiert bereits (B1) – alternativ umbenennen in *_5m
```

**Grafana‑Patches (Query‑Beispiele):**
- **Drift Ratio 5m**: `avg_over_time(adapt_drift_ratio_5m[5m])`
- **Drift Ratio 15m**: `avg_over_time(adapt_drift_ratio_15m[15m])`
- **Drift Events rate 5m**: `sum(rate(adapt_drift_events_total[5m])) by (policy_id)`
- **Rollout %**: `adapt_enforce_rollout_percent`

---

## 🧪 Tests

**A) `tests/orchestrator_drift_metrics.rs`**
```rust
#[test]
fn records_and_exports_ratios() {
    // init tracker
    // feed 90 requests, 3 drifts → expect ratio ~ 0.033 in 5m window
    // assert Prometheus gauges updated (>= 0.03 && <= 0.04)
}
#[test]
fn per_policy_ratio_independent() {
    // record different drift patterns per policy_id → ratios independent
}
```

**B) `tests/orchestrator_enforce_gate.rs`**
```rust
#[test]
fn gate_closed_on_high_drift() {
    // cfg.enabled=true, window=5m, max_ratio=0.005
    // inject >0.5% drift → readiness_gate_ok returns (false, "DRIFT_GATE")
}
#[test]
fn gate_open_when_disabled() {
    // cfg.enabled=false → always OK
}
```

**Befehle**
```bash
cargo test --test orchestrator_drift_metrics -- --nocapture
cargo test --test orchestrator_enforce_gate -- --nocapture
```

---

## ⚙️ Helm & Config

`helm/values-prod.yaml` (Ergänzen/Prüfen):
```yaml
enforce_enabled: true          # B1
enforce_rollout_percent: 25    # B1
drift_max_ratio: 0.005
enforce_readiness_gate:
  enabled: true
  window: "5m"
```

Rollout‑Änderungen per `helm upgrade` → Controller lädt ConfigMap neu (Config‑Reload Sidecar o. ä.).

---

## 📘 Runbook‑Update (docs/runbook_rollout.md)

- **Gate aktivieren:** `enforce_readiness_gate.enabled=true`  
- **Canary 25%**, 30–60 min Beobachtung; Kriterien:  
  - `adapt_drift_ratio_5m ≤ drift_max_ratio`  
  - `p95(verify) < 600 ms`, `error_rate < 1%`  
- **Rollback**, wenn Ratio > 2× Limit oder KPI‑Bruch.  
- **Troubleshooting:** Label‑Cardinality niedrig halten; Cache Invalidation prüfen.

---

## ✅ Definition of Done (B2)

1. Drift‑Tracker exportiert **5m/15m/60m** Ratio‑Gauges, Events‑Counter.  
2. Readiness‑Gate (optional) setzt `/readyz` auf **503** über Schwelle.  
3. Grafana zeigt 3 Drift‑Panels + Rollout %.  
4. Tests `orchestrator_drift_metrics.rs` & `orchestrator_enforce_gate.rs` **grün**.  
5. Runbook aktualisiert; Helm‑Flags übernehmen die Steuerung.

---

## ▶️ Sammelkommandos

```bash
# Unit
cargo test --test orchestrator_drift_metrics -- --nocapture
cargo test --test orchestrator_enforce_gate -- --nocapture

# Helm Patch (prod/stage)
helm upgrade --install cap helm/ -f helm/values-prod.yaml --wait
```

**Ergebnis:** Mit B2 erhältst du **sichtbare, belastbare Drift‑Metriken** und eine **sichere Readiness‑Schranke** für progressive Enforcement‑Rollouts in Produktion.
