# 🧩 PRD / Design – Registry Schema Versioning (v0.8.0)

**Datum:** 2025-10-30  
**Status:** In Planung (P1-Scope)  
**Zielversion:** v0.8.0

---

## 🎯 Ziel
Einführung eines konsistenten **Schema-Versionierungssystems** für die Registry. Dadurch können künftige Änderungen an der Registry-Struktur sicher erkannt, migriert und validiert werden.

---

## 💡 Motivation
- **Stabilität:** Änderungen am Registry-Layout (z. B. neue Felder, Signaturen) sollen rückwärtskompatibel bleiben.
- **Migration:** CLI und Tools erkennen alte Schemaversionen automatisch.
- **Audit:** Jeder Registry-Dump enthält maschinenlesbare Versionsinformation.

Bisher: keine explizite Schema-Version → Änderungen erfordern manuelle Prüfungen.  
Neu: `registry_meta.schema_version` Feld + Getter-Funktion + CLI-Anzeige.

---

## 🧭 Scope (v0.8.0)
**In-Scope**
- Hinzufügen von `schema_version` zur Registry-Metaebene (JSON & SQLite)
- Getter/Setter-API (`registry::schema_version()`)
- CLI-Ausgabe `registry info` → zeigt aktuelle Version
- Migration bestehender Dateien (Default `1.0`)

**Out-of-Scope**
- Automatische Down-Migrationen
- Versionsabhängige Formattransformationen

---

## 🏗️ Architektur / Design

### 1) Schema-Erweiterung (JSON)
```json
{
  "registry_version": "1.0",
  "schema_version": "1.0",
  "entries": [ ... ]
}
```

Für SQLite:
```sql
ALTER TABLE registry_meta ADD COLUMN schema_version TEXT DEFAULT '1.0';
```

### 2) Rust-Implementierung
```rust
#[derive(Serialize, Deserialize)]
pub struct RegistryMeta {
    pub registry_version: String,
    pub schema_version: String,
    // ... weitere Felder
}

impl RegistryMeta {
    pub fn schema_version(&self) -> &str {
        &self.schema_version
    }
}
```

### 3) CLI-Unterstützung
```bash
cap-agent registry info
```
Ausgabe:
```
Registry schema version: 1.0
Entries: 128
Backend: SQLite
```

---

## ✅ Akzeptanzkriterien
1. Registry-Dateien enthalten `schema_version`
2. CLI `registry info` zeigt aktuelle Version
3. Migration älterer Dateien setzt `schema_version = '1.0'`
4. Kein Einfluss auf bestehende Einträge oder Hashes
5. Kompatibel mit JSON- und SQLite-Backends

---

## 🧪 Testplan
- **Unit:**
  - `default_schema_version_set()`
  - `schema_version_getter_returns_correct_value()`
  - `migration_adds_schema_version_field()`
- **CLI Smoke:**
  - `registry info` → zeigt Schema-Version
  - Alte Datei ohne Feld → Migration + Warnung

---

## 🔁 Migrationsschritte (Dev)
1. `registry_meta.rs`: Feld `schema_version` hinzufügen
2. Default-Wert `1.0` in Konstruktor setzen
3. SQLite-Migration: ALTER TABLE + Default-Spalte
4. CLI erweitern (`registry info` Ausgabe)
5. Tests & Doku aktualisieren

---

## 🧱 Beispiel
```bash
$ cap-agent registry info
Registry schema version: 1.0
Entries: 42
Backend: JSON
```

---

## 📚 Doku-Updates
- **README.md:** Abschnitt „Registry-Format“ um Schema-Version ergänzen
- **SYSTEMARCHITEKTUR.md:** Registry-Metaebene (Versionierung) aufnehmen
- **CLI.md:** Beispielausgabe für `registry info`

---

## 📝 Changelog (geplant)
- **Added:** Feld `schema_version` in Registry-Metadaten
- **Changed:** CLI `registry info` zeigt Schema-Version
- **Docs:** Registry-Formatbeschreibung aktualisiert
- **Tests:** Unit- und CLI-Schema-Checks
