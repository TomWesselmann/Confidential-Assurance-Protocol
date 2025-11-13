
# 🧭 Week 6 — B1 Focus Pack: Enforcer CLI + Prometheus Drift Metrics

**Ziel:** Enforcer endgültig **bedienbar über CLI** (`proof adapt …`) machen und **Produktions‑Metriken** (Prometheus) für Rollout & Drift freischalten.  
**Kontext:** Enforcer‑Modul v0.1 ist implementiert (Shadow/Enforce + Sampling + DriftTracker). Jetzt: **CLI‑Integration + Metrics**.  
**Leitplanken:** fail‑closed, deterministisch, keine PII in Logs/Metriken.

---

## ✅ Deliverables (Ende B1)

1) **CLI‑Flags im Binary `proof`** (Subcommand `adapt`)
- `--enforce` (bool, default=false)  → Enforcement aktivieren
- `--rollout <u8>` (0–100, default=0) → Progressive Aktivierung
- `--drift-max <f64>` (0.0–1.0, default=0.005) → Gate‑Schwelle
- Bestehende Flags weiter unterstützen: `--policy <id> | --ir <file>`, `--context <file>`, `--selector <basic|weighted>`, `--weights <file>`, `--dry-run`, `-o <file>`

2) **Prometheus‑Metriken (exportiert über /metrics im Verifier)**  
Library‑seitig registriert; CLI nutzt denselben Registry‑Pfad.
- `adapt_enforce_rollout_percent` (Gauge)  
- `adapt_requests_total{mode="shadow|enforced",policy_id}` (Counter)  
- `adapt_drift_events_total{policy_id}` (Counter)  
- `adapt_drift_ratio{window="5m"}` (Gauge) – aus DriftTracker (rolling window)  
- Optional: `adapt_selection_latency_seconds` (Histogram)

3) **Grafana‑Panels (Minimal‑Erweiterung)**
- Stat: **Rollout %** (aktuell)
- Graph: **Drift Ratio (5m)**
- Stat: **Drift Events (rate 5m)**

4) **Tests & DoD**
- Unit: Flag‑Parsing & Defaults, Metrik‑Inkrementierung, deterministische Sampling‑Treffer.  
- DoD: Flags funktionieren, Metriken sichtbar & korrekt, kein PII, deterministisch.

---

## 🗂️ Änderungen an Dateien/Modulen

```
src/
  orchestrator/
    enforcer.rs          # (vorhanden)
    metrics.rs           # (NEU) Metrik‑Definition+Registry‑Helper
  bin/
    proof.rs             # (ANPASSEN) Clap‑Flags → EnforceOptions
tests/
  enforcer_cli.rs        # (NEU) Flag‑Parsing, Defaults, Dry‑Run vs Enforce
  enforcer_metrics.rs    # (NEU) Metriken erhöhen sich erwartungsgemäß
grafana/
  dashboards/verifier.json  # Panels für Rollout & Drift ergänzen (minimal)
docs/
  runbook_rollout.md     # Flag‑Tabelle & Beispiel‑Kommandos (Update)
```

---

## 🧩 Spezifikation – CLI (`proof adapt`)

**Beispiele**
```bash
# Shadow‑Mode (nur Beobachtung)
proof adapt --policy lksg.v1 --context examples/context_ok.json --rollout 0 --drift-max 0.005 --dry-run

# Canary 25% (enforce an)
proof adapt --policy lksg.v1 --context examples/context_ok.json --enforce --rollout 25 --drift-max 0.005 -o plan.json

# Weighted‑Selector mit Kosten
proof adapt --policy lksg.v1 --context ctx.json --selector weighted --weights examples/rule_weights.yaml --enforce --rollout 25
```

**Clap‑Skizze**
```rust
#[derive(clap::Args)]
pub struct AdaptArgs {
  #[arg(long)] pub policy: Option<String>,
  #[arg(long)] pub ir: Option<PathBuf>,
  #[arg(long)] pub context: PathBuf,
  #[arg(long, default_value_t=false)] pub enforce: bool,
  #[arg(long, default_value_t=0)] pub rollout: u8,
  #[arg(long, default_value_t=0.005)] pub drift_max: f64,
  #[arg(long, default_value_t=String::from("basic"))] pub selector: String,
  #[arg(long)] pub weights: Option<PathBuf>,
  #[arg(long)] pub dry_run: bool,
  #[arg(short='o')] pub out: Option<PathBuf>,
}
```

**Mapping → EnforceOptions**
```rust
let opts = EnforceOptions { enforce: args.enforce, rollout_percent: args.rollout, drift_max_ratio: args.drift_max };
let VerdictPair { shadow, enforced } = enforcer::decide(&ir, &ctx, &opts);
```

---

## 📊 Prometheus‑Metriken – Definition

```rust
// src/orchestrator/metrics.rs
use once_cell::sync::Lazy;
use prometheus::{register_int_counter_vec, register_gauge, register_histogram, IntCounterVec, Gauge, Histogram};

pub static ADAPT_ROLLOUT_PERCENT: Lazy<Gauge> =
    Lazy::new(|| register_gauge!("adapt_enforce_rollout_percent", "Current enforce rollout percent").unwrap());

pub static ADAPT_REQUESTS_TOTAL: Lazy<IntCounterVec> = Lazy::new(||
    register_int_counter_vec!("adapt_requests_total", "Adapt requests by mode and policy", &["mode","policy_id"]).unwrap());

pub static ADAPT_DRIFT_EVENTS_TOTAL: Lazy<IntCounterVec> = Lazy::new(||
    register_int_counter_vec!("adapt_drift_events_total", "Drift events by policy", &["policy_id"]).unwrap());

pub static ADAPT_SELECTION_LATENCY: Lazy<Histogram> = Lazy::new(||
    register_histogram!("adapt_selection_latency_seconds", "Selection latency seconds").unwrap());

// Drift ratio wird als Gauge aktualisiert (rolling window in DriftTracker)
pub static ADAPT_DRIFT_RATIO_5M: Lazy<Gauge> =
    Lazy::new(|| register_gauge!("adapt_drift_ratio", "Drift ratio (5m)").unwrap());
```

**Aktualisierung (im Enforcer‑Pfad)**
```rust
ADAPT_ROLLOUT_PERCENT.set(opts.rollout_percent as f64);
ADAPT_REQUESTS_TOTAL.with_label_values(&["shadow", &policy_id]).inc();
// ggf. für enforced:
ADAPT_REQUESTS_TOTAL.with_label_values(&["enforced", &policy_id]).inc();
if drift_detected { ADAPT_DRIFT_EVENTS_TOTAL.with_label_values(&[&policy_id]).inc(); }
ADAPT_DRIFT_RATIO_5M.set(tracker.ratio_5m());
```

> **Hinweis:** Keine PII in Labels (nur `policy_id`, `mode`).

---

## 🧪 Tests

### `tests/enforcer_cli.rs`
- `defaults_shadow_mode()` → `--enforce` false, `--rollout` 0, `--drift-max` 0.005.
- `parse_enforce_rollout_drift()` → Flags korrekt gemappt in `EnforceOptions`.
- `deterministic_sampling()` → Für fixen Seed/Hash die gleiche Entscheidung bei `rollout=25`.

### `tests/enforcer_metrics.rs`
- **Setup:** Eigene Prometheus‑Registry injizieren (oder global resetten).  
- `metrics_increment_shadow_only()` → `adapt_requests_total{mode="shadow"}` steigt, kein `enforced`.  
- `metrics_increment_enforced()` → Beide Pfade zählen, Rollout‑Gauge gesetzt.  
- `drift_event_increments_counter()` → künstliche Abweichung → `adapt_drift_events_total++`.  
- `drift_ratio_updates()` → Tracker simuliert 5m‑Fenster → Gauge > 0.

**Befehle**
```bash
cargo test --test enforcer_cli -- --nocapture
cargo test --test enforcer_metrics -- --nocapture
```

---

## 🖥️ Grafana – Minimaler Patch
- **Stat**: `adapt_enforce_rollout_percent`
- **Graph**: `adapt_drift_ratio` (legend: 5m)
- **Stat**: `sum(rate(adapt_drift_events_total[5m])) by (policy_id)`

---

## 🧾 Runbook (Auszug) – docs/runbook_rollout.md
- Start: `enforce=false`, `rollout=0`, Erwartung **Drift=0**.  
- Canary: `enforce=true`, `rollout=25`; Beobachtung 30–60 min → **p95**, **5xx**, **Drift ≤ drift_max**.  
- Ramp‑up: `rollout=100`; **Rollback** bei Drift > 2× `drift_max` oder KPI‑Bruch.

---

## ✅ Definition of Done (B1)
1. `proof adapt` akzeptiert Flags & erzeugt korrekte `EnforceOptions`.  
2. Metriken werden bei Shadow/Enforce/Drift sauber erhöht; **keine PII**.  
3. Grafana zeigt Rollout %, Drift Ratio, Drift Events (rate).  
4. Tests `enforcer_cli.rs` & `enforcer_metrics.rs` **grün**.  
5. Runbook‑Abschnitt aktualisiert.

---

## ▶️ Sammelkommandos
```bash
# Unit
cargo test --test enforcer_cli -- --nocapture
cargo test --test enforcer_metrics -- --nocapture

# Manuell
proof adapt --policy lksg.v1 --context examples/context_ok.json --enforce --rollout 25 --drift-max 0.005 -o plan.json
```
