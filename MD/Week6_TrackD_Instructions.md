# 🛡️ Week 6 — Track D Instruction Pack: Backup/Restore Drill & Key Rotation

**Context:** Track A (prod cutover), B1 (CLI+metrics), B2 (drift+gate), C1/C2 (SAP docs + IT) ✅.
**Goal of Track D:** Beweissichere **Wiederherstellung** (identische Hashes/ETags) + **Schlüsselrotation** ohne Ausfälle (alte & neue KIDs kompatibel).
**Operating mode:** fail‑closed, deterministisch, **keine PII** in Backups/Logs, reproduzierbar.

---

## 🎯 Deliverables (Ende Track D)
- **Backup/Restore Runbook** (`docs/runbook_restore.md`) – Schritt‑für‑Schritt inkl. Prüfhaken.
- **Key‑Rotation Runbook** (`docs/runbook_rotation.md`) – Phasenmodell + Rollback.
- **Automations‑Skripte** (`scripts/backup.sh`, `scripts/restore.sh`, `scripts/key_rotate.sh`).
- **Tests** (`tests/backup_restore.rs`, `tests/rotation.rs`) – CI‑fähig (wo möglich), sonst `#[ignore]` mit Env‑Variablen.
- **DoD‑Nachweise:** ETag/Hashes identisch nach Restore; Verify akzeptiert alt **und** neu während Kompat‑Fenster; nach Decom wird alt abgelehnt.

---

## 🔒 Backup Scope (minimal, aber ausreichend)
Artefakte (read‑only, ohne PII): IR‑Registry (JSON/SQLite), OpenAPI/Configs (ohne Secrets), Dashboards/Alerts, Docs. Manifest `backup.manifest.json` mit sha3‑256 über alle Dateien.

## ♻️ Restore Drill (neuer Namespace)
Deploy in leerem Namespace, Artefakte einspielen, verifizieren: `GET /policy/:id` gleicher `ir_hash` & ETag; `/readyz` 200; Schemathesis OK.

## 🔁 Key Rotation (KID‑basiert)
Phasen: Vorbereitung → Dual‑Accept → Sign‑Switch → Decom. Vor T1 alt+neu OK; nach T1 alt FAIL, neu OK.

## 🧪 Tests
- backup_restore.rs: `restored_ir_hash_matches`, `smoke_ready_after_restore`.
- rotation.rs: `accepts_old_and_new_before_T1`, `rejects_old_after_T1`.

## ✅ DoD
Hashes/ETag identisch; Dual‑Accept/Decom korrekt; Skripte & Runbooks vorhanden; keine PII.
