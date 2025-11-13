# CAP Proof System – Punkt 6: Scale-Benches (10k) & Batch-Ops

## Ziel
Dieser Abschnitt stellt sicher, dass das CAP-System bei hoher Last **stabil, deterministisch und messbar performant** bleibt.  
Ziel: **10 000 vollständige Self-Verifying-Entries** in unter 10 Minuten (Commodity-Hardware).  
Ergebnisse dienen als **technischer Belastungsnachweis** für Whitepaper & Investoren.

---

## Deliverables

### 1) Benchmark-Suite
- Leistungsmessung für Insert/Verify/Exec-Operationen.
- Fokus auf 10 k-Scale, reproduzierbar, deterministisch.  
- Output: JSON-Reports + Summary-Charts (`bench/results/*.json`).

### 2) Batch-Operations
- CLI-Befehle für große Datenmengen (z. B. 10 000 Proofs in einer Datei).  
- Parallele Verarbeitung (n Threads = CPU-Kerne).  
- Wiederholbar & crash-resilient (Resume-Index).

### 3) Metrics-Collector
- Einheitliche Messung für Zeit, RAM, I/O, CPU-Last.  
- Optional Prometheus-Export (`--metrics-json`).

### 4) Whitepaper-Metriken
- Tabelle mit Latenz / Durchsatz / Footprint-Kennzahlen.  
- Grundlage für Produktbench (Regression).

---

## Datenmodell / Setup

### SQLite-Backends
- **Schema v0.9** mit BLOB-Store & Self-Verify-Feldern  
- Journal-Mode = WAL  
- Pragmas:
  ```sql
  PRAGMA synchronous = NORMAL;
  PRAGMA journal_mode = WAL;
  PRAGMA cache_size = -40000;
  PRAGMA mmap_size = 268435456;
  ```

### Benchmark-Config (`bench/config.toml`)
```toml
[bench]
target_entries = 10000
batch_size = 500
threads = 8
backend = "sqlite"
registry_path = "build/registry.sqlite"
proof_template = "examples/cap-proof/"
```

---

## Batch-Ops CLI

### Batch-Add
```bash
cap-agent registry batch-add   --manifest-dir ./proofs/manifests   --proof-dir ./proofs/data   --verifier ./proofs/verifier.wasm   --abi ./proofs/verifier.abi.json   --signing-key ./keys/company.v2.json   --threads 8   --limit 10000   --backend sqlite --registry ./build/registry.sqlite
```

### Batch-Verify (Regression)
```bash
cap-agent registry batch-verify   --backend sqlite --registry ./build/registry.sqlite   --threads 8   --limit 10000
```

### Benchmark-Runner
```bash
cap-agent bench run --suite scale10k --config bench/config.toml
cap-agent bench report --input bench/results/scale10k.json --out bench/report.md
```

---

## Metriken & Zielwerte

| Testfall | Ziel (8-Core CPU) | Metrik | Kommentar |
|-----------|------------------|---------|------------|
| Insert 10k Entries | < 600 s (≈ 17 ms / Entry) | Durchsatz | SQLite WAL + 8 Threads |
| Verify 10k Entries | < 900 s | Durchsatz | WASM Runtime-Limit 50 M Fuel |
| Disk Footprint | < 1.5 GB | Speicher | Registry + Blobs |
| Memory Peak | < 2 GB | RAM | Parallel-WASM Sandbox |
| Crash-Recovery | 0 Datenverlust | Integrität | Resume-File OK |
| Determinismus | Varianz < 0.5 % | Reproduzierbarkeit | Sandbox-Stable |

---

## Tests & Bench-Plan

### Unit
- `bench::measure_time(fn)`, `bench::write_json_report()`  
- `batch_add::resume()` – prüft korrektes Resume-Verhalten  
- `metrics::cpu/mem` – stabile OS-Metrik  

### Integration
- 100 → 1 000 → 10 000 Entries Pipeline  
- Prüft: Hashing & Sign Determinismus, BLOB Refcounts, Self-Verify  
- Latenz-Logs: `bench/results/*.json`  

### Stress
- Parallele Runs mit 8 Threads  
- Randomized Order → deterministische Outputs  
- Crash-Recovery-Test (kill/restart)

---

## Definition of Done

✅ Benchmark-Suite (`bench/*`) ausführbar  
✅ Batch-CLI (`registry batch-add`, `batch-verify`) stabil  
✅ 10 k Runs deterministisch, ohne Crash  
✅ Performance-Report (MD + JSON) vorhanden  
✅ Doku aktualisiert (`SYSTEMARCHITEKTUR.md`, `CLAUDE.md`)  
✅ Ergebnisse fließen ins Whitepaper („Performance Proof“-Kapitel)

---

## Warum dieser Punkt wichtig ist

| Aspekt | Nutzen |
|--------|--------|
| Whitepaper-Beleg | Objektive Leistungsdaten für Audits & Marketing |
| Produktnachweis | Zeigt, dass Registry & Sandbox skalieren |
| Technische Vertrauensbasis | Belegt Determinismus & Integrität bei Massendaten |
| Benchmark-Infra für Regression | Basis für spätere ZK-Benchmarks |
| Investor Impact | „10 000 Self-Verifying Entries in 10 min“ als Key-Figure |

---

## Whitepaper-Integration
Kapitel: **“Performance Evaluation of CAP Proof Engine”**  
Beinhaltet:
- Diagramme: Insert & Verify Rate (log-scale)  
- Tabellen: Durchsatz & Memory per Entry  
- Kommentar: Sandbox Performance ≈ native ( < 20 % Overhead )  
- Fazit: „CAP registry achieves industrial-grade scalability and deterministic proof integrity at 10 000+ entries.“
