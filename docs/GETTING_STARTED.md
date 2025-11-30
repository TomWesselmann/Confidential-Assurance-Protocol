# Getting Started – Anleitung für Einsteiger

Diese Anleitung richtet sich an **Personen ohne Vorkenntnisse** in GitHub oder Software-Testing. Du lernst Schritt für Schritt, wie du mit diesem Projekt arbeiten kannst.

---

## 📋 Inhaltsverzeichnis

1. [Was ist dieses Projekt?](#was-ist-dieses-projekt)
2. [🚀 Schnellstart mit WebUI (Einfachster Einstieg!)](#schnellstart-mit-webui-einfachster-einstieg)
3. [Voraussetzungen installieren](#voraussetzungen-installieren)
4. [Projekt herunterladen (Clone)](#projekt-herunterladen-clone)
5. [Projekt bauen (Build)](#projekt-bauen-build)
6. [Tests ausführen](#tests-ausführen)
7. [CLI verwenden](#cli-verwenden)
8. [Änderungen vornehmen](#änderungen-vornehmen)
9. [Probleme melden (Issues)](#probleme-melden-issues)
10. [Häufige Fehler und Lösungen](#häufige-fehler-und-lösungen)

---

## Was ist dieses Projekt?

**CAP Agent** (Confidential Assurance Protocol) ist ein Rust-basiertes Command-Line-Tool für die Erstellung und Verifikation von kryptographischen Nachweisen im Kontext des deutschen Lieferkettensorgfaltspflichtengesetzes (LkSG).

**Einfach gesagt:** Es hilft Unternehmen, ihre Lieferketten nachweisbar zu dokumentieren und zu verifizieren.

---

## 🚀 Schnellstart mit WebUI (Einfachster Einstieg!)

**Neu in v0.11.0:** Die grafische Benutzeroberfläche (WebUI) ist der einfachste Weg, um CAP ohne Kommandozeilen-Kenntnisse zu verwenden!

### Was ist die WebUI?

Die WebUI ist eine **browserbasierte Oberfläche**, mit der du:
- ✅ Proof Packages per **Drag & Drop** hochladen kannst
- ✅ Manifest-Daten **visuell** anzeigen lassen kannst
- ✅ Proofs mit **einem Klick** verifizieren kannst
- ✅ Verification Results mit **farbcodierten Status-Badges** siehst

**Keine Terminal-Befehle nötig!**

### Voraussetzungen (nur einmalig)

**Du brauchst:**
1. **Node.js** (für WebUI) - [Download hier](https://nodejs.org/)
2. **Rust** (für Backend API) - [Installationsanleitung siehe unten](#schritt-1-rust-installieren)

### Schritt-für-Schritt Anleitung

#### 1. Projekt herunterladen

```bash
# Terminal öffnen (macOS: Cmd+Space → "Terminal")
git clone https://github.com/TomWesselmann/Confidential-Assurance-Protocol.git
cd Confidential-Assurance-Protocol
```

#### 2. Backend API starten

```bash
# Terminal 1: Backend API starten
cd agent
cargo run --bin cap-verifier-api

# Warte bis du siehst:
# 🚀 Starting CAP Verifier API v0.1.0
# 🎧 Listening on http://127.0.0.1:8080
```

**Was passiert?**
- Das Backend wird kompiliert (beim ersten Mal 3-5 Minuten)
- Die REST API startet auf Port 8080
- Die API ist bereit, Requests zu empfangen

#### 3. WebUI starten

```bash
# Terminal 2: Neues Terminal-Fenster öffnen
cd webui
npm install  # Nur beim ersten Mal
npm run dev

# Warte bis du siehst:
# ➜  Local:   http://localhost:5173/
```

**Was passiert?**
- Node.js Dependencies werden installiert (nur beim ersten Mal)
- Der Dev-Server startet
- Die WebUI ist bereit unter http://localhost:5173

#### 4. Policy vorbereiten

```bash
# Terminal 3: Neues Terminal-Fenster öffnen
# Führe diesen Befehl aus, um die Demo-Policy zu kompilieren:

TOKEN="admin-tom"
curl -X POST http://localhost:8080/policy/v2/compile \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "policy": {
      "id": "lksg.demo.v1",
      "version": "1.0.0",
      "legal_basis": [{"directive": "LkSG", "article": "§3"}],
      "description": "Demo policy for WebUI testing",
      "inputs": {
        "ubo_count": {"type": "integer"},
        "supplier_count": {"type": "integer"}
      },
      "rules": [
        {"id": "rule_ubo_exists", "op": "range_min", "lhs": {"var": "ubo_count"}, "rhs": 1}
      ]
    },
    "persist": true,
    "lint_mode": "relaxed"
  }'
```

**Erwartete Ausgabe:**
```json
{
  "policy_id": "lksg.demo.v1",
  "policy_hash": "0x...",
  "stored": true
}
```

**Was ist passiert?**
- Eine Demo-Policy wurde im Backend gespeichert
- Diese Policy prüft, ob mindestens 1 UBO (Ultimate Beneficial Owner) vorhanden ist

#### 5. WebUI öffnen und verwenden

1. **Browser öffnen:** http://localhost:5173

2. **Proof Package hochladen:**
   - Ziehe eine ZIP-Datei in das Upload-Feld
   - Oder klicke auf "Datei auswählen"
   - Das Manifest wird automatisch angezeigt

3. **Manifest ansehen:**
   - Company Commitment Root (kryptographischer Hash)
   - Policy Name, Version, Hash
   - Audit Event Count
   - Erstellungsdatum

4. **Proof verifizieren:**
   - Klicke auf den blauen Button "Proof Verifizieren"
   - Warte 1-2 Sekunden
   - Sieh das Verification Result:
     - ✅ **OK** = Proof ist gültig
     - ⚠️ **WARN** = Warnung (Proof gültig, aber Hinweise)
     - ❌ **FAIL** = Proof ungültig

5. **Details ansehen:**
   - Manifest Hash
   - Proof Hash
   - Signatur-Status
   - Detaillierter Report

### Beispiel-Screenshot (Beschreibung)

**Upload Screen:**
```
┌─────────────────────────────────────────┐
│  CAP Verifier WebUI                     │
├─────────────────────────────────────────┤
│                                         │
│  ┌───────────────────────────────────┐ │
│  │  Drag & Drop Proof Package hier  │ │
│  │  oder klicke, um Datei zu wählen │ │
│  └───────────────────────────────────┘ │
│                                         │
└─────────────────────────────────────────┘
```

**Verification Screen:**
```
┌─────────────────────────────────────────┐
│  Manifest Viewer                        │
├─────────────────────────────────────────┤
│  Company Commitment: 0x1234...          │
│  Policy: LkSG Demo v1.0                 │
│  Status: Bereit zur Verifikation        │
│                                         │
│  [ Proof Verifizieren ]  [Reset]        │
├─────────────────────────────────────────┤
│  Verification Result: ✅ OK              │
│  Manifest Hash: 0xd490...               │
│  Proof Hash: 0x83a8...                  │
│  Signature: ✓ Valid                     │
└─────────────────────────────────────────┘
```

### Vorteile der WebUI

| Feature | CLI (Terminal) | WebUI (Browser) |
|---------|----------------|-----------------|
| **Einstieg** | Rust & Terminal-Kenntnisse nötig | Browser genügt |
| **Upload** | Manuelle Dateipfade | Drag & Drop |
| **Visualisierung** | JSON in Terminal | Grafische Darstellung |
| **Verifikation** | Mehrere Befehle | Ein Klick |
| **Ergebnis** | Text-Output | Farbcodierte Badges |

### Wichtige Hinweise für Einsteiger

1. **"admin-tom" Token:**
   - Dies ist ein **Entwicklungs-Token** (nur für lokales Testing!)
   - In Production wird ein echter OAuth2 Token verwendet
   - Du siehst diesen Token normalerweise nicht, er ist im Hintergrund aktiv

2. **Demo Proof Packages:**
   - Enthalten oft **keine echten UBO/Supplier-Daten**
   - Verification kann **"FAIL"** anzeigen (das ist korrekt!)
   - Für echte Verifikation benötigst du echte Proof Packages mit Daten

3. **Backend muss laufen:**
   - WebUI funktioniert **nur**, wenn Backend API läuft (Terminal 1)
   - Wenn Backend stoppt, zeigt WebUI Fehler

4. **Policy muss kompiliert sein:**
   - Schritt 4 (Policy kompilieren) ist **wichtig**!
   - Ohne Policy zeigt Verifikation "Policy not found" Fehler

### Nächste Schritte

- ✅ **CLI lernen:** Für fortgeschrittene Nutzer (siehe [CLI verwenden](#cli-verwenden))
- ✅ **Eigene Proofs erstellen:** Mit CSV-Daten und Policy-Dateien
- ✅ **Dokumentation lesen:** `agent/CLAUDE.md` für vollständige Doku
- ✅ **Tests ausführen:** `cargo test` im agent-Verzeichnis

---

## Voraussetzungen installieren

### Schritt 1: Rust installieren

Rust ist die Programmiersprache, in der dieses Projekt geschrieben ist.

#### Auf macOS / Linux:

1. Öffne das **Terminal** (auf macOS: `Cmd + Space`, dann "Terminal" eingeben)
2. Kopiere diesen Befehl und drücke Enter:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
3. Folge den Anweisungen auf dem Bildschirm (meistens einfach Enter drücken)
4. Starte das Terminal neu oder führe aus:
   ```bash
   source $HOME/.cargo/env
   ```

#### Auf Windows:

1. Gehe zu: https://rustup.rs/
2. Lade `rustup-init.exe` herunter
3. Führe die Datei aus und folge den Anweisungen
4. Starte die Eingabeaufforderung (CMD) neu

#### Prüfen, ob Rust installiert ist:

```bash
rustc --version
cargo --version
```

**Erwartete Ausgabe:**
```
rustc 1.75.0 (oder höher)
cargo 1.75.0 (oder höher)
```

### Schritt 2: Git installieren

Git ist ein Versionskontrollsystem, mit dem du den Code herunterladen kannst.

#### Auf macOS:

Git ist meist vorinstalliert. Prüfe mit:
```bash
git --version
```

Falls nicht installiert:
```bash
xcode-select --install
```

#### Auf Linux:

```bash
# Ubuntu/Debian
sudo apt-get install git

# Fedora
sudo dnf install git
```

#### Auf Windows:

1. Gehe zu: https://git-scm.com/download/win
2. Lade den Installer herunter
3. Führe den Installer aus (alle Standard-Optionen sind OK)

#### Prüfen, ob Git installiert ist:

```bash
git --version
```

**Erwartete Ausgabe:**
```
git version 2.x.x
```

---

## Projekt herunterladen (Clone)

### Schritt 1: GitHub-Repository klonen

1. Öffne das **Terminal** (macOS/Linux) oder die **Eingabeaufforderung** (Windows)
2. Navigiere zu einem Ordner, wo du das Projekt speichern möchtest:
   ```bash
   cd ~/Desktop
   ```
3. Klone das Repository:
   ```bash
   git clone https://github.com/TomWesselmann/Confidential-Assurance-Protocol.git
   ```
4. Wechsel in das Projekt-Verzeichnis:
   ```bash
   cd Confidential-Assurance-Protocol/agent
   ```

**Was ist passiert?**
- Git hat alle Dateien des Projekts in einen Ordner `Confidential-Assurance-Protocol` heruntergeladen
- Du befindest dich jetzt im `agent`-Unterordner, wo der Hauptcode liegt

### Schritt 2: Projekt-Struktur ansehen

```bash
ls
```

**Du solltest sehen:**
```
Cargo.toml          # Projekt-Konfiguration
src/                # Quellcode
tests/              # Tests
examples/           # Beispiel-Dateien
docs/               # Dokumentation
build/              # Output-Verzeichnis
```

---

## Projekt bauen (Build)

### Was bedeutet "bauen"?

**Bauen** (oder "Build") bedeutet, den Rust-Quellcode in ein ausführbares Programm zu kompilieren.

### Schritt 1: Projekt kompilieren

```bash
cargo build
```

**Was passiert?**
- Cargo (Rusts Paketmanager) lädt alle Abhängigkeiten herunter
- Der Code wird kompiliert
- Ein ausführbares Programm wird in `target/debug/cap-agent` erstellt

**Dauer:** Beim ersten Mal 2-5 Minuten (je nach Internet und Computer)

**Erwartete Ausgabe:**
```
   Compiling cap-agent v0.11.0
    Finished dev [unoptimized + debuginfo] target(s) in 3m 42s
```

### Schritt 2: Release-Version bauen (optional)

Für optimierte Performance:
```bash
cargo build --release
```

**Unterschied:**
- `cargo build` → Schnell kompilieren, langsamer ausführen (für Entwicklung)
- `cargo build --release` → Langsamer kompilieren, schneller ausführen (für Production)

---

## Tests ausführen

### Was sind Tests?

Tests sind kleine Programme, die prüfen, ob der Code korrekt funktioniert. Sie laufen automatisch und zeigen dir, ob etwas kaputt ist.

### Schritt 1: Alle Tests ausführen

```bash
cargo test
```

**Was passiert?**
- Cargo kompiliert den Code
- Alle Tests werden ausgeführt
- Du siehst eine Zusammenfassung (z.B. "150 tests passed")

**Erwartete Ausgabe:**
```
running 150 tests
test crypto::tests::test_sha3_256 ... ok
test commitment::tests::test_merkle_root ... ok
...
test result: ok. 150 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Schritt 2: Einzelne Tests ausführen

```bash
# Nur Tests mit "crypto" im Namen
cargo test crypto

# Nur einen spezifischen Test
cargo test test_sha3_256
```

### Schritt 3: Tests mit mehr Ausgaben

```bash
cargo test -- --nocapture
```

**Tipp:** Das `-- --nocapture` zeigt auch `println!()` Ausgaben während Tests.

### Schritt 4: Ignorierte Tests ausführen

Manche Tests sind als "ignoriert" markiert (weil sie lange dauern):

```bash
cargo test -- --ignored
```

---

## CLI verwenden

### Was ist eine CLI?

**CLI** = Command Line Interface (Kommandozeilenschnittstelle)

Du gibst Befehle im Terminal ein, statt mit der Maus zu klicken.

### Schritt 1: Hilfe anzeigen

```bash
cargo run -- --help
```

**Erwartete Ausgabe:**
```
CAP Agent - Confidential Assurance Protocol

Usage: cap-agent <COMMAND>

Commands:
  prepare         Berechnet Commitments aus CSV-Daten
  policy          Policy-Verwaltung
  manifest        Manifest-Verwaltung
  proof           Proof-Erstellung und -Verifikation
  ...
```

### Schritt 2: Version anzeigen

```bash
cargo run -- version
```

**Erwartete Ausgabe:**
```
cap-agent v0.11.0
```

### Schritt 3: Beispiel-Pipeline ausführen

#### 3.1 Commitments berechnen

```bash
cargo run -- prepare \
  --suppliers examples/suppliers.csv \
  --ubos examples/ubos.csv
```

**Was passiert?**
- CSV-Dateien werden gelesen
- Merkle-Roots werden berechnet
- Ergebnis wird in `build/commitments.json` gespeichert

**Erwartete Ausgabe:**
```
✅ Commitments generiert
📊 Supplier Root: 0x1234...
📊 UBO Root: 0x5678...
📄 Gespeichert in: build/commitments.json
```

#### 3.2 Policy validieren

```bash
cargo run -- policy validate \
  --file examples/policy.lksg.v1.yml
```

**Erwartete Ausgabe:**
```
✅ Policy ist gültig
🔑 Policy Hash: 0xabc123...
```

#### 3.3 Manifest erstellen

```bash
cargo run -- manifest build \
  --policy examples/policy.lksg.v1.yml
```

**Voraussetzung:** `build/commitments.json` muss existieren (siehe 3.1)

**Erwartete Ausgabe:**
```
✅ Manifest erstellt
📄 Gespeichert in: build/manifest.json
```

#### 3.4 Proof erstellen

```bash
cargo run -- proof build \
  --manifest build/manifest.json \
  --policy examples/policy.lksg.v1.yml
```

**Erwartete Ausgabe:**
```
✅ Proof erstellt
📄 Gespeichert in: build/proof.dat
📄 Lesbare Version: build/proof.json
```

#### 3.5 Proof verifizieren

```bash
cargo run -- proof verify \
  --proof build/proof.dat \
  --manifest build/manifest.json
```

**Erwartete Ausgabe:**
```
✅ Proof ist gültig
📊 Manifest Hash: 0xd490...
📊 Policy Hash: 0x0afc...
📊 Status: OK
```

### Schritt 4: Output-Dateien ansehen

Alle generierten Dateien sind im `build/` Verzeichnis:

```bash
ls -lh build/
```

**Du solltest sehen:**
```
commitments.json       # Merkle Roots
manifest.json          # Compliance Manifest
proof.dat              # Binary Proof
proof.json             # Lesbare Proof-Version
agent.audit.jsonl      # Audit Log
```

**Dateien ansehen:**
```bash
# JSON-Dateien sind lesbar
cat build/commitments.json

# Mit Pretty-Print (falls jq installiert ist)
cat build/commitments.json | jq .
```

---

## Änderungen vornehmen

### Schritt 1: Einen Branch erstellen

**Was ist ein Branch?**
Ein Branch ist wie eine Kopie des Codes, in der du Änderungen machen kannst, ohne den Hauptcode zu beeinflussen.

```bash
# Aktuellen Branch anzeigen
git branch

# Neuen Branch erstellen und wechseln
git checkout -b mein-feature
```

### Schritt 2: Änderungen machen

1. Öffne eine Datei in einem Texteditor (z.B. VS Code, Sublime Text)
2. Mache deine Änderungen
3. Speichere die Datei

### Schritt 3: Änderungen testen

```bash
# Code neu kompilieren
cargo build

# Tests ausführen
cargo test

# Clippy (Linter) ausführen
cargo clippy
```

### Schritt 4: Änderungen committen

**Was ist ein Commit?**
Ein Commit ist ein Snapshot deiner Änderungen mit einer Beschreibung.

```bash
# Zeige, was geändert wurde
git status

# Alle Änderungen zur Staging Area hinzufügen
git add .

# Commit erstellen mit Nachricht
git commit -m "Beschreibe deine Änderung hier"
```

**Beispiel:**
```bash
git commit -m "Fix typo in README.md"
```

### Schritt 5: Änderungen hochladen (Push)

```bash
# Beim ersten Mal
git push -u origin mein-feature

# Bei weiteren Commits
git push
```

### Schritt 6: Pull Request erstellen

1. Gehe zu: https://github.com/TomWesselmann/Confidential-Assurance-Protocol
2. Du siehst einen Button: **"Compare & pull request"**
3. Klicke darauf
4. Beschreibe deine Änderungen
5. Klicke **"Create pull request"**

**Was ist ein Pull Request?**
Ein Pull Request (PR) ist eine Anfrage, deine Änderungen in den Hauptcode zu integrieren. Andere können deinen Code reviewen.

---

## Probleme melden (Issues)

### Schritt 1: Zum Issues-Bereich gehen

1. Gehe zu: https://github.com/TomWesselmann/Confidential-Assurance-Protocol/issues
2. Klicke auf **"New issue"**

### Schritt 2: Issue beschreiben

**Gute Issue:**
```
Titel: cargo test schlägt fehl mit "file not found" Fehler

Beschreibung:
Wenn ich `cargo test` ausführe, bekomme ich diesen Fehler:

```
Error: No such file or directory (os error 2)
```

**Meine Umgebung:**
- Betriebssystem: macOS 14.0
- Rust Version: 1.75.0
- Projekt Version: v0.11.0

**Schritte zum Reproduzieren:**
1. `git clone` das Repository
2. `cd agent`
3. `cargo test`

**Erwartetes Verhalten:**
Tests sollten erfolgreich durchlaufen.

**Tatsächliches Verhalten:**
Tests schlagen fehl mit Fehler.
```

### Schritt 3: Issue absenden

Klicke auf **"Submit new issue"**

---

## Häufige Fehler und Lösungen

### Fehler 1: "cargo: command not found"

**Problem:** Rust ist nicht installiert oder nicht im PATH.

**Lösung:**
```bash
# Rust installieren (siehe oben)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Terminal neu starten oder
source $HOME/.cargo/env
```

### Fehler 2: "error: linker `cc` not found"

**Problem:** C-Compiler fehlt (wird für manche Rust-Pakete benötigt).

**Lösung (macOS):**
```bash
xcode-select --install
```

**Lösung (Linux):**
```bash
# Ubuntu/Debian
sudo apt-get install build-essential

# Fedora
sudo dnf install gcc
```

**Lösung (Windows):**
- Installiere Visual Studio Build Tools: https://visualstudio.microsoft.com/downloads/

### Fehler 3: "test result: FAILED. X passed; Y failed"

**Problem:** Manche Tests schlagen fehl.

**Lösung:**
1. Lies die Fehlermeldung genau
2. Prüfe, ob du die richtige Rust-Version hast: `rustc --version` (mindestens 1.70)
3. Prüfe, ob alle Dateien vorhanden sind: `ls examples/`
4. Erstelle ein Issue auf GitHub (siehe oben)

### Fehler 4: "No such file or directory: build/commitments.json"

**Problem:** Du versuchst ein Kommando auszuführen, das eine vorherige Datei benötigt.

**Lösung:**
```bash
# Führe die Pipeline in der richtigen Reihenfolge aus:
cargo run -- prepare --suppliers examples/suppliers.csv --ubos examples/ubos.csv
cargo run -- manifest build --policy examples/policy.lksg.v1.yml
```

### Fehler 5: "error: could not compile `cap-agent`"

**Problem:** Kompilierungsfehler im Code.

**Lösung:**
1. Stelle sicher, dass du die neueste Version hast:
   ```bash
   git pull origin main
   ```
2. Lösche Build-Cache und versuche erneut:
   ```bash
   cargo clean
   cargo build
   ```
3. Wenn das nicht hilft, erstelle ein Issue

---

## Nützliche Kommandos (Cheat Sheet)

### Git

```bash
# Status anzeigen (was wurde geändert?)
git status

# Neueste Änderungen herunterladen
git pull

# Alle Änderungen hinzufügen
git add .

# Commit erstellen
git commit -m "Beschreibung"

# Hochladen
git push

# Branch wechseln
git checkout main
```

### Cargo

```bash
# Projekt kompilieren
cargo build

# Tests ausführen
cargo test

# Programm ausführen
cargo run -- <ARGUMENTE>

# Code-Qualität prüfen
cargo clippy

# Code formatieren
cargo fmt

# Dokumentation generieren
cargo doc --open

# Build-Cache löschen
cargo clean
```

### Nützliche Befehle

```bash
# Projektstruktur anzeigen
tree -L 2

# Dateigröße anzeigen
ls -lh build/

# JSON formatiert anzeigen (benötigt jq)
cat build/manifest.json | jq .

# Logs ansehen
tail -f build/agent.audit.jsonl
```

---

## Weitere Ressourcen

### Dokumentation

- **Projekt-Dokumentation:** `docs/` Verzeichnis
- **CLAUDE.md:** Vollständige System-Dokumentation
- **MANAGING_LINT_WARNINGS.md:** Anleitung für Code-Qualität
- **LINT_POLICY.md:** Lint-Richtlinien

### Externe Links

- **Rust Buch (Deutsch):** https://rust-lang-de.github.io/rustbook-de/
- **Cargo Buch:** https://doc.rust-lang.org/cargo/
- **Git Tutorial:** https://git-scm.com/book/de/v2
- **GitHub Guides:** https://guides.github.com/

### Hilfe bekommen

- **GitHub Issues:** https://github.com/TomWesselmann/Confidential-Assurance-Protocol/issues
- **Rust Community:** https://users.rust-lang.org/
- **Stack Overflow:** https://stackoverflow.com/questions/tagged/rust

---

## Glossar

### Wichtige Begriffe

- **Repository (Repo):** Ein Projekt mit allen Dateien und Versionsgeschichte
- **Clone:** Eine Kopie eines Repositories herunterladen
- **Commit:** Ein Snapshot von Änderungen mit Beschreibung
- **Branch:** Eine parallele Version des Codes
- **Pull Request (PR):** Anfrage, Änderungen zu integrieren
- **Issue:** Ein gemeldetes Problem oder Feature-Request
- **Cargo:** Rust's Paketmanager und Build-Tool
- **Crate:** Ein Rust-Paket (Library oder Binary)
- **Compiler:** Programm, das Code in ausführbare Dateien umwandelt
- **Test:** Automatisierte Prüfung, ob Code korrekt funktioniert
- **CI/CD:** Continuous Integration / Continuous Deployment (Automatisierung)

---

## Nächste Schritte

Jetzt, wo du die Grundlagen kennst:

1. ✅ **Probiere die Beispiel-Pipeline aus** (siehe [CLI verwenden](#cli-verwenden))
2. ✅ **Führe Tests aus** (siehe [Tests ausführen](#tests-ausführen))
3. ✅ **Lies die Projekt-Dokumentation** (`CLAUDE.md`)
4. ✅ **Mach deine erste Änderung** (siehe [Änderungen vornehmen](#änderungen-vornehmen))
5. ✅ **Melde ein Issue, wenn du Probleme findest**

**Viel Erfolg! 🚀**

---

**Dokumentation erstellt:** 2025-11-13
**Version:** v0.11.0
**Für Fragen:** GitHub Issues oder Maintainer kontaktieren
