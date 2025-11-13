# WEEK7_INDEX — Plan & Pacing

**Scope:** Week 7 focuses on (S) Keys/HSM, (R) Registry v1.1, (A) Audit‑Hash‑Chain, and (P) optional `/proof/adapt` API.

## Reihenfolge & Abhängigkeiten
1) **S — Keys/HSM** (kann parallel zu R starten)
2) **R — Registry v1.1** (liefert `schema_version`; A hängt minimal davon ab)
3) **A — Audit‑Hash‑Chain** (beginnt mit stabilem Event‑Schema; `schema_version` einbinden)
4) **P — /proof/adapt** (optional; sobald A/B‑Gleichheit zur CLI validiert)

## Owners
- S (Keys/HSM): Security & Platform
- R (Registry): Data/Storage
- A (Audit): App/Backend
- P (API): App/Backend

## Akzeptanz-Quick‑Check (DoD je Track)
- **S:** Provider‑Abstraktion produktionsreif, Rotation‑Suite grün
- **R:** v1.1 Schema live & kompatibel; Migration idempotent
- **A:** Hash‑Chain verifizierbar; Tamper‑Test schlägt an; keine PII
- **P:** (optional) /proof/adapt A/B‑gleich zur CLI; p95 < 200ms

## Test-Sammelkommandos
```bash
# Keys
cargo test --test key_provider_unit -- --nocapture
SOFTHSM2_CONF=tests/softhsm2.conf PKCS11_PIN=1234       cargo test --test key_provider_pkcs11_it -- --ignored --nocapture
cargo test --test rotation -- --nocapture

# Registry
cargo test --test registry_migration -- --nocapture
cargo test --test registry_compat -- --nocapture

# Audit
cargo test --test audit_chain_unit -- --nocapture
cargo test --test audit_chain_it -- --nocapture
cargo test --test audit_chain_tamper -- --nocapture

# Optional /proof/adapt
cargo run --bin cap-verifier-api &
cargo test --test adapt_http_it -- --ignored --nocapture
```