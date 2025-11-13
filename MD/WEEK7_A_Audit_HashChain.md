# WEEK7_A — Structured Audit‑Log / Hash‑Chain

**Ziel:** Append‑only Audit‑Log mit Hash‑Kette, verifizierbar, ohne PII.

## Design
- Speicher: `audit.jsonl` (JSON‑Zeile/Event) **oder** `audit.sqlite` (Tabelle `events`)
- Event (vereinfacht):
  ```json
  {
    "ts": "2025-11-10T12:34:56Z",
    "event": "verify_response",
    "policy_id": "lksg.v1",
    "ir_hash": "...",
    "manifest_hash": "...",
    "result": "OK|WARN|FAIL",
    "run_id": "uuid",
    "prev_hash": "...",
    "self_hash": "..."
  }
  ```
- Kettenregel: `self_hash = H(canonical_json(event_without_self_hash))`; `prev_hash` zeigt auf vorheriges `self_hash`

## CLI
- `cap audit append --event <json>` (intern genutzt)
- `cap audit verify --file audit.jsonl|audit.sqlite` → OK/NOK mit Fehlerindex
- `cap audit export --from TS1 --to TS2 --policy lksg.v1`

## Akzeptanzkriterien (DoD)
1. **Tamper‑Nachweis:** jede Änderung früherer Events wird erkannt (Verify FAIL mit Index)
2. **Determinismus:** kanonische Serialisierung, ISO‑Zeitformat (Z)
3. **Leistung:** Verify 10k Events ≤ 3s; Append p95 ≤ 0.5ms
4. **Privatsphäre:** keine PII; nur IDs/Hashes

## Tests & Befehle
```bash
cargo test --test audit_chain_unit -- --nocapture
cargo test --test audit_chain_it -- --nocapture
cargo test --test audit_chain_tamper -- --nocapture
```

## Dateien (neu/ändern)
```
src/audit/mod.rs
src/audit/hash_chain.rs
src/bin/cap.rs          # audit‑CLI
tests/audit_chain_unit.rs
tests/audit_chain_it.rs
tests/audit_chain_tamper.rs
```