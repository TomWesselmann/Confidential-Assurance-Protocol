# ⚙️ PRD / Design – Registry Performance Benchmarks (v0.8.0)

**Datum:** 2025-10-30  
**Status:** In Planung (P2-Scope)  
**Zielversion:** v0.8.0

---

## 🎯 Ziel
Einführung einer automatisierten Benchmark-Suite, um die Performance der Registry-Implementierung (JSON vs. SQLite) messbar zu machen und Engpässe bei größeren Datenmengen (>1000 Entries) frühzeitig zu erkennen.

---

## 💡 Motivation
- **Nachweisbare Performance:** Quantitative Vergleichswerte zwischen JSON- und SQLite-Backends.
- **Optimierung:** Identifikation von Flaschenhälsen (Parsing, Hashing, IO-Latenz).
- **Skalierung:** Vorbereitung auf produktive Nutzung mit tausenden Proofs.

Bisher: keine reproduzierbaren Messwerte → subjektive Einschätzung der Performance.  
Neu: Criterion-Benchmarks mit deterministischem Setup.

---

## 🧭 Scope (v0.8.0)
**In-Scope**
- Einrichtung Criterion-Benchmark-Suite (`benches/registry_bench.rs`)
- Vergleich JSON vs. SQLite (Insert, Verify, List, Load)
- Skalierungstests mit 100 / 1000 / 10 000 Einträgen
- Ergebnis-Reports (HTML + CSV)

**Out-of-Scope**
- Memory-Profiling / Flamegraphs (v0.9+)
- Netzwerk-Benchmarks (Remote-Registry)

---

## 🏗️ Architektur / Design

### 1) Benchmark-Setup
```rust
// benches/registry_bench.rs
use criterion::{criterion_group, criterion_main, Criterion};
use cap_agent::registry::{Registry, RegistryEntry};

fn bench_registry_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("registry_insert");
    for size in [100, 1000, 10_000] {
        group.bench_with_input(format!("json_{}", size), &size, |b, &n| {
            b.iter(|| run_insert_bench("json", n));
        });
        group.bench_with_input(format!("sqlite_{}", size), &size, |b, &n| {
            b.iter(|| run_insert_bench("sqlite", n));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_registry_insert);
criterion_main!(benches);
```

### 2) Helper-Funktionen
```rust
fn run_insert_bench(backend: &str, entries: usize) {
    let reg = Registry::new_temp(backend);
    for i in 0..entries {
        let e = RegistryEntry::mock(i);
        reg.add_entry(e).unwrap();
    }
}
```

### 3) Benchmark-Metriken
| Kategorie | Messgröße | Einheit |
|------------|------------|----------|
| Insert | Zeit pro 1000 Einträge | ms |
| Verify | Verifikationsrate | ops/s |
| List | Abfragezeit aller Einträge | ms |
| Load | Lesezeit von Registry-Datei | ms |

### 4) Ausgaben
- **Automatisch generiert:** `target/criterion/registry_insert/report/index.html`
- **Zusatzexport:** `--message-format=json` → CSV-Auswertung in CI

---

## ✅ Akzeptanzkriterien
1. Criterion-Benchmarks lauffähig (`cargo bench`)
2. JSON- und SQLite-Backends werden getestet
3. Performance-Reports werden generiert (HTML + JSON)
4. Tests laufen deterministisch mit Mock-Data
5. Dokumentierte Ergebnisse ≥ 1000 Einträge

---

## 🧪 Testplan
- **Smoke-Test:** `cargo bench` läuft ohne Fehler
- **Regression:** Benchmarks wiederholbar (Abweichung < 5 %)
- **CI-Test (optional):** Benchmark-Job in GitHub Actions (nightly)

---

## 🔁 Implementierungsschritte
1. Neues Modul `benches/registry_bench.rs` erstellen
2. Criterion in `Cargo.toml` als Dev-Dependency hinzufügen
3. Mock-Data-Generator (`RegistryEntry::mock`) implementieren
4. Benchmarks für `insert`, `verify`, `list`, `load` schreiben
5. README-Abschnitt „Benchmarking“ hinzufügen

---

## 📈 Beispielausgabe (verkürzt)
```
Benchmarking registry_insert/json_1000: Warming up for 3.0000 s
Benchmarking registry_insert/json_1000: Collecting 10 samples in estimated 5.0000 s
registry_insert/json_1000  time:   [142.33 ms 144.20 ms 146.11 ms]
registry_insert/sqlite_1000  time: [45.00 ms 46.12 ms 47.89 ms]
```
→ **≈ 3× schneller** mit SQLite-Backend.

---

## 📚 Doku-Updates
- **README.md:** Abschnitt „Performance & Benchmarking“ ergänzen
- **SYSTEMARCHITEKTUR.md:** Registry-Layer + Performance-Analyse hinzufügen
- **DEV_GUIDE.md:** Anleitung „cargo bench“ + Report-Auswertung

---

## 📝 Changelog (geplant)
- **Added:** Criterion-Benchmark-Suite für Registry
- **Changed:** CI-Konfiguration um optionalen Performance-Job
- **Docs:** Benchmark-Ergebnisse dokumentiert
- **Tests:** Regression-Toleranz in CI verankert
