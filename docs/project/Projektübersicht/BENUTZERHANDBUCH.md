# CAP Agent - Vollständiges Benutzerhandbuch

**Version:** 0.12.0
**Für:** Einsteiger und Fortgeschrittene
**Datum:** 2025-12-04 (aktualisiert mit Enterprise Security)

---

## 🔐 Enterprise Security Status (NEU - Dezember 2025)

> **Wichtig für Unternehmenseinsatz:** Bevor Sie CAP Agent in Produktionsumgebungen einsetzen, beachten Sie bitte die Sicherheitshinweise.

| Metrik | Wert | Bedeutung |
|--------|------|-----------|
| **Enterprise Readiness** | 57% | Aktueller Stand |
| **Ziel** | 95% | Nach 14 Wochen Hardening |
| **Kritische Findings** | 4 | Müssen vor Production behoben werden |

**Was bedeutet das für Sie?**

- ✅ **Für Entwicklung/Test:** Vollständig nutzbar, alle Features funktionieren
- ✅ **Desktop App (offline):** Sicher für lokale Nutzung, Daten bleiben auf Ihrem Rechner
- ⚠️ **Server/API (online):** Zusätzliche Sicherheitsmaßnahmen erforderlich für Production

**Wichtige Sicherheitsdokumente:**
- [Security Audit Report](../../security/SECURITY_AUDIT_REPORT.md) - Vollständiger Sicherheitsbericht
- [Enterprise Roadmap](../../ROADMAP_ENTERPRISE.md) - 14-Wochen Hardening-Plan
- [Troubleshooting](./06-troubleshooting.md#-enterprise-security-troubleshooting-neu---dezember-2025) - Security-Probleme lösen

---

## 📖 Was ist der CAP Agent?

**Kurz gesagt:** Ein Werkzeug, um **nachzuweisen**, dass dein Unternehmen gesetzliche Compliance-Anforderungen erfüllt (z.B. Lieferkettengesetz), **ohne sensible Geschäftsdaten preiszugeben**.

**Das Problem:** Du musst Auditoren beweisen, dass du z.B. alle deine Zulieferer kennst und überprüft hast - aber du willst nicht die Namen und Details deiner Geschäftspartner offenlegen.

**Die Lösung:** Der CAP Agent erstellt **kryptografische Beweise** (Zero-Knowledge Proofs), die bestätigen "Ja, wir haben mindestens 1 UBO dokumentiert" **ohne** zu verraten, wer das ist.

---

## 📑 Inhaltsverzeichnis

1. [Schnellstart](#schnellstart)
2. [Desktop App (NEU v0.12.0)](#desktop-app-neu-v0120)
3. [Server-Commands](#server-commands)
4. [Commitment-Commands](#commitment-commands)
5. [Policy-Commands](#policy-commands)
6. [Manifest-Commands](#manifest-commands)
7. [Proof-Commands](#proof-commands)
8. [Verifier-Commands](#verifier-commands)
9. [Signatur-Commands](#signatur-commands)
10. [Schlüssel-Commands](#schlüssel-commands)
11. [Registry-Commands](#registry-commands)
12. [BLOB-Store-Commands](#blob-store-commands)
13. [WebUI Anleitung](#webui-anleitung)
14. [Monitoring & Observability](#monitoring--observability)
15. [Performance & Metrics](#performance--metrics)
16. [Hilfreiche Tipps](#hilfreiche-tipps)

---

## 🚀 Schnellstart

### Voraussetzungen

**Du brauchst:**
- Rust installiert (`cargo --version` zum Testen)
- Node.js installiert (`node --version` zum Testen)
- 3 Terminal-Fenster

### Minimaler Workflow (CLI - empfohlen!)

**Hinweis:** Du brauchst die WebUI eigentlich nicht! Das CLI ist einfacher für lokale Nutzung.

```bash
cd agent

# Schritt 1: Deine Daten verschlüsseln
cargo run --bin cap-agent -- prepare \
  --suppliers ../examples/suppliers.csv \
  --ubos ../examples/ubos.csv

# Schritt 2: Compliance-Manifest erstellen
cargo run --bin cap-agent -- manifest build \
  --policy ../examples/policy.lksg.v1.yml

# Schritt 3: Proof generieren
cargo run --bin cap-agent -- proof build

# Schritt 4: Proof verifizieren
cargo run --bin cap-agent -- proof verify

# Schritt 5: Package für Auditor erstellen
cargo run --bin cap-agent -- proof export \
  --manifest build/manifest.json \
  --proof build/proof.capz \
  --output build/package

# Schritt 6: ZIP erstellen
cd build/package && zip -r ../proof-package.zip .
```

**Fertig!** Du hast jetzt ein Proof-Package, das du Auditoren geben kannst.

---

### Alternative: Mit WebUI (nur für Demos)

```bash
# Terminal 1: API Server
cd agent
cargo run --bin cap-verifier-api

# Terminal 2: WebUI
cd webui
npm run dev

# Terminal 3: Proof erstellen (siehe oben)

# Dann: Browser öffnen → http://localhost:5173 → ZIP hochladen
```

---

## 🖥️ Desktop App (NEU v0.12.0)

> ⭐ **Empfohlen für Einzelpersonen und Freelancer!** Die Desktop App funktioniert komplett offline - kein Server, keine Kommandozeile nötig.

### Was ist die Desktop App?

Eine **native Desktop-Anwendung** (gebaut mit Tauri 2.0), die alle CAP Agent Funktionen in einer benutzerfreundlichen Oberfläche bietet:

- **Proofer Mode:** 6-Schritt Workflow zum Erstellen von Compliance-Nachweisen
- **Verifier Mode:** Bundles hochladen und offline verifizieren
- **Audit Mode:** Audit-Trail Timeline mit Hash-Chain-Anzeige

### Vorteile gegenüber CLI/WebUI

| Aspekt | Desktop App | CLI | WebUI |
|--------|-------------|-----|-------|
| **Offline** | ✅ Vollständig | ✅ | ❌ Server nötig |
| **Benutzerfreundlich** | ✅ Grafisch | ⚠️ Terminal nötig | ✅ |
| **Installation** | ✅ 1 Datei | ⚠️ Rust + Build | ⚠️ Node + Server |
| **Daten bleiben lokal** | ✅ | ✅ | ⚠️ Server |

### Installation

**macOS:**
```bash
# Option 1: Von Release herunterladen
# (Noch nicht verfügbar - aus Source bauen)

# Option 2: Aus Source bauen
cd src-tauri
cargo tauri build

# App öffnen
open target/release/bundle/macos/CAP\ Desktop\ Proofer.app
```

**Windows:**
```powershell
# Aus Source bauen
cd src-tauri
cargo tauri build

# Installer ausführen
.\target\release\bundle\msi\CAP_Desktop_Proofer.msi
```

**Linux:**
```bash
cd src-tauri
cargo tauri build

# AppImage ausführen
./target/release/bundle/appimage/cap-desktop-proofer.AppImage
```

### Der 6-Schritt Proofer Workflow

Die Desktop App führt dich durch jeden Schritt:

```
┌─────────┐    ┌─────────────┐    ┌────────┐    ┌──────────┐    ┌───────┐    ┌────────┐
│ Import  │ →  │ Commitments │ →  │ Policy │ →  │ Manifest │ →  │ Proof │ →  │ Export │
└─────────┘    └─────────────┘    └────────┘    └──────────┘    └───────┘    └────────┘
     ↓                ↓               ↓              ↓             ↓            ↓
  CSV laden      Hashes          Regeln         Verknüpfen      ZK-Proof    Bundle
  (Suppliers,   berechnen        laden          aller Teile    erstellen   erstellen
   UBOs)
```

### Schritt für Schritt Anleitung

#### 1. Workspace wählen
- Starte die App
- Klicke auf "Neues Projekt erstellen" oder wähle einen Workspace-Ordner
- Alle Projekte werden in deinem gewählten Ordner gespeichert

#### 2. Import (CSV-Dateien hochladen)
- Wähle deine **suppliers.csv** (Lieferantenliste)
- Wähle deine **ubos.csv** (Ultimate Beneficial Owners)
- Die App zeigt dir eine Vorschau der Daten

**CSV-Format:**
```csv
# suppliers.csv
name,jurisdiction,tier
"ACME GmbH","DE","1"
"Parts Inc","US","2"

# ubos.csv
name,birthdate,citizenship
"Max Mustermann","1980-01-15","DE"
"Jane Doe","1975-06-20","US"
```

#### 3. Commitments (Hashes berechnen)
- Klicke auf "Commitments berechnen"
- Die App berechnet BLAKE3 Merkle-Roots
- Deine echten Daten verlassen **nie** den Rechner - nur Hashes!

#### 4. Policy (Regeln definieren)
- Wähle eine vorhandene Policy oder lade eine YAML-Datei
- Die Default-Policy (LkSG) ist bereits integriert

#### 5. Manifest (Alles verknüpfen)
- Klicke auf "Manifest erstellen"
- Das Manifest verknüpft: Commitments + Policy + Metadaten

#### 6. Proof (ZK-Beweis generieren)
- Klicke auf "Proof erstellen"
- Der Progress-Balken zeigt den Fortschritt
- ⚠️ Dies kann einige Sekunden dauern

#### 7. Export (Bundle erstellen)
- Klicke auf "Bundle exportieren"
- Wähle einen Speicherort
- Fertig! Das Bundle enthält:
  - `_meta.json` (Bundle-Metadaten mit SHA3-256 Hashes)
  - `manifest.json`
  - `proof.dat`

### Verifier Mode

1. Wechsle zu **Verifier** (Tab oben rechts)
2. Ziehe ein Bundle (Ordner) per Drag & Drop oder klicke "Bundle wählen"
3. Die App verifiziert:
   - ✅ Alle Dateien vorhanden?
   - ✅ SHA3-256 Hashes korrekt?
   - ✅ Proof gültig gegen Policy?
4. Ergebnis: **OK** (grün) oder **FAIL** (rot)

### Audit Mode

1. Wähle ein Projekt in der Sidebar
2. Wechsle zu **Audit** (Tab oben rechts)
3. Sieh die komplette Timeline aller Aktionen:
   - `project_created` - Projekt angelegt
   - `csv_imported` - CSVs importiert
   - `commitments_built` - Hashes berechnet
   - `policy_loaded` - Policy geladen
   - `manifest_built` - Manifest erstellt
   - `proof_built` - Proof generiert
   - `bundle_exported` - Bundle exportiert

**Hash-Chain:**
Jeder Eintrag enthält einen `prev_hash`, der auf den vorherigen zeigt. Dadurch ist Manipulation erkennbar!

### Tipps & Tricks

**CSV-Probleme:**
- Kodierung muss **UTF-8** sein (nicht Windows-1252)
- Trennzeichen muss **Komma** sein (nicht Semikolon)
- Excel-Tipp: "Speichern unter" → "CSV UTF-8"

**Workflow-Fortschritt:**
- Der Fortschritt wird automatisch gespeichert
- Projekt erneut öffnen → Fortschritt bleibt erhalten
- Verwendet `initializeFromStatus()` um Status zu laden

**DevTools öffnen:**
- macOS: `Cmd+Option+I`
- Windows/Linux: `Ctrl+Shift+I`

**Fehler melden:**
- Logs unter: `~/Library/Logs/CAP Desktop Proofer/` (macOS)
- Oder: `%APPDATA%\CAP Desktop Proofer\logs\` (Windows)

---

## 🖥️ Server-Commands

### `cap-verifier-api` - REST API Server starten

**Wofür brauche ich das?**
Nur wenn du die **WebUI** nutzen willst oder wenn mehrere Leute **remote** auf deine API zugreifen sollen. **Für Einzelpersonen empfehlen wir die Desktop App!**

**Warum ist das wichtig?**
Die WebUI (Browser) kann nicht direkt mit dem Filesystem arbeiten - sie braucht einen Server als "Brücke" zur Rust-Logik.

**Wann nutze ich das?**
- Du willst die WebUI ausprobieren
- Du willst anderen Leuten Zugriff geben (z.B. Auditor soll Proofs hochladen)
- Du testest Remote-Szenarien

**Wann brauche ich das NICHT?**
Wenn du nur lokal mit dem CLI arbeitest! Dann einfach `cargo run --bin cap-agent` nutzen.

**Was macht das?**
Startet einen HTTP-Server (Standard: http://127.0.0.1:8080) mit REST API Endpoints für Proof-Verifikation und Policy-Management.

**Command:**
```bash
cargo run --bin cap-verifier-api
```

**Erweiterte Optionen:**

```bash
# Mit eigenem Port und Token
cargo run --bin cap-verifier-api \
  --bind 127.0.0.1:8080 \
  --token admin-tom

# Mit TLS (Production - verschlüsselte Verbindung)
cargo run --bin cap-verifier-api \
  --bind 0.0.0.0:8443 \
  --tls \
  --tls-cert certs/server.crt \
  --tls-key certs/server.key

# Mit mTLS (Mutual Authentication - beide Seiten authentifizieren sich)
cargo run --bin cap-verifier-api \
  --bind 0.0.0.0:8443 \
  --tls \
  --tls-cert certs/server.crt \
  --tls-key certs/server.key \
  --mtls \
  --tls-ca certs/ca.crt
```

---

### Parameter im Detail

#### `--bind <IP:PORT>`
**Wofür brauche ich das?**
Legt fest, auf welcher IP-Adresse und welchem Port der Server lauscht.

**Warum ist das wichtig?**
- `127.0.0.1` (localhost) = Nur Zugriff vom eigenen Computer (sicher für Entwicklung)
- `0.0.0.0` = Zugriff von überall im Netzwerk (nötig für Production/Remote-Zugriff)
- Port `8080` = HTTP-Standard-Port
- Port `8443` = HTTPS-Standard-Port (mit TLS)

**Wann nutze ich das?**
- **Entwicklung:** `127.0.0.1:8080` (nur lokal, kein TLS nötig)
- **Production:** `0.0.0.0:8443` (Netzwerk-Zugriff mit TLS)
- **Docker/Kubernetes:** `0.0.0.0:8080` (Container-Netzwerk)

**Beispiel:**
```bash
# Nur lokal erreichbar (sicher für Tests)
--bind 127.0.0.1:8080

# Von überall erreichbar (Production)
--bind 0.0.0.0:8443
```

---

#### `--token <STRING>`
**Wofür brauche ich das?**
Ein einfaches Passwort (Bearer Token), das der Client in jedem API-Request mitschicken muss.

**Warum ist das wichtig?**
Ohne Token kann jeder auf deine API zugreifen und Proofs verifizieren oder Policies hochladen. Der Token ist wie ein Schlüssel zur API.

**Wann nutze ich das?**
- **Entwicklung:** Einfacher String wie `admin-tom` (nur zum Testen!)
- **Production:** Langer, zufälliger String (z.B. `openssl rand -base64 32`)
- **Alternative:** OAuth2 JWT Tokens (siehe CLAUDE.md)

**Sicherheitshinweis:**
- ⚠️ NIEMALS in Git einchecken!
- ⚠️ Nicht in Log-Dateien speichern!
- ✅ Als Umgebungsvariable übergeben: `TOKEN=$(cat secret.txt) cargo run ...`

**Beispiel:**
```bash
# Entwicklung (unsicher, nur für Tests)
--token admin-tom

# Production (sicher)
--token $(openssl rand -base64 32)
```

---

#### `--tls`
**Wofür brauche ich das?**
Aktiviert HTTPS statt HTTP, sodass alle Daten verschlüsselt übertragen werden.

**Warum ist das wichtig?**
Ohne TLS werden Daten (inkl. Token!) im Klartext übers Netzwerk geschickt. Jeder, der mithört, kann:
- Deinen Bearer Token stehlen
- Proofs mitlesen
- API-Anfragen manipulieren

**Wann nutze ich das?**
- **IMMER in Production!**
- **Nie in lokaler Entwicklung** (localhost ist schon sicher, TLS unnötig)
- **Immer wenn der Server übers Internet erreichbar ist**

**Technisch:**
- TLS = Transport Layer Security (Nachfolger von SSL)
- Verwendet X.509-Zertifikate
- Verschlüsselt mit RSA/ECC + AES

**Beispiel:**
```bash
# Entwicklung: Kein TLS (HTTP)
cargo run --bin cap-verifier-api --bind 127.0.0.1:8080

# Production: Mit TLS (HTTPS)
cargo run --bin cap-verifier-api \
  --bind 0.0.0.0:8443 \
  --tls \
  --tls-cert certs/server.crt \
  --tls-key certs/server.key
```

---

#### `--tls-cert <FILE>`
**Wofür brauche ich das?**
Das "Personalausweis" deines Servers. Ein X.509-Zertifikat, das beweist, dass dein Server wirklich der ist, für den er sich ausgibt.

**Warum ist das wichtig?**
Ohne Zertifikat würde der Browser/Client eine Warnung anzeigen ("Diese Verbindung ist nicht sicher"). Das Zertifikat bestätigt:
- Der Server gehört wirklich dir
- Die Verbindung ist verschlüsselt
- Niemand hat sich dazwischengeschaltet (Man-in-the-Middle-Schutz)

**Wann nutze ich das?**
Immer wenn du `--tls` nutzt. Das Zertifikat muss zum Server-Hostnamen passen (z.B. `api.example.com`).

**Wo bekomme ich ein Zertifikat her?**
1. **Let's Encrypt** (kostenlos, automatisch): `certbot certonly --standalone`
2. **Self-Signed** (nur für Tests!): `openssl req -x509 -newkey rsa:4096 ...`
3. **Unternehmen:** Von deiner internen PKI/CA

**Format:** PEM (Plain Text mit `-----BEGIN CERTIFICATE-----`)

**Beispiel:**
```bash
# Zertifikat-Datei angeben
--tls-cert certs/server.crt

# Inhalt einer PEM-Datei:
# -----BEGIN CERTIFICATE-----
# MIIDXTCCAkWgAwIBAgIJAKZ...
# -----END CERTIFICATE-----
```

---

#### `--tls-key <FILE>`
**Wofür brauche ich das?**
Der geheime Schlüssel (Private Key), der zum Zertifikat gehört. Damit entschlüsselt der Server die eingehenden Daten.

**Warum ist das wichtig?**
Das ist das Gegenstück zum Zertifikat:
- **Zertifikat** = öffentlich, jeder darf es sehen
- **Private Key** = geheim, nur der Server darf ihn kennen

Wenn jemand deinen Private Key stiehlt, kann er sich als dein Server ausgeben!

**Wann nutze ich das?**
Immer zusammen mit `--tls-cert`. Ohne Private Key kann der Server keine TLS-Verbindungen akzeptieren.

**Sicherheit:**
- ⚠️ NIEMALS in Git einchecken!
- ⚠️ Nur mit `chmod 600` lesbar für den Server-User
- ⚠️ Auf verschlüsseltem Storage speichern
- ✅ In Production: Hardware Security Module (HSM) nutzen

**Format:** PKCS#8 PEM (mit `-----BEGIN PRIVATE KEY-----`)

**Beispiel:**
```bash
# Private Key angeben
--tls-key certs/server.key

# Dateirechte prüfen (nur Owner darf lesen)
ls -l certs/server.key
# -rw------- 1 user group 1704 Nov 20 server.key
```

---

#### `--mtls`
**Wofür brauche ich das?**
Aktiviert "Mutual TLS" (gegenseitige Authentifizierung): Nicht nur der Server beweist seine Identität, sondern **auch der Client** muss ein Zertifikat vorweisen.

**Warum ist das wichtig?**
Standard-TLS (ohne `--mtls`):
- ✅ Server authentifiziert sich (Client weiß: "Ich rede mit dem echten Server")
- ❌ Client ist anonym (Server weiß nicht, wer da verbindet)

Mit mTLS:
- ✅ Server authentifiziert sich
- ✅ Client authentifiziert sich (Server weiß: "Das ist Client XYZ")

**Wann nutze ich das?**
- **Hochsicherheits-Umgebungen** (z.B. Bank, Behörde, kritische Infrastruktur)
- **Machine-to-Machine** (Server-zu-Server-Kommunikation)
- **Zero-Trust-Netzwerke** (jeder muss sich ausweisen)
- **B2B-APIs** (jeder Partner bekommt eigenes Client-Zertifikat)

**Wann NICHT nutzen?**
- Browser-basierte UIs (schwierig für normale User, Client-Zertifikate zu installieren)
- Öffentliche APIs (zu hohe Hürde für Entwickler)

**Beispiel:**
```bash
# Ohne mTLS: Nur Server authentifiziert sich
cargo run --bin cap-verifier-api \
  --bind 0.0.0.0:8443 \
  --tls \
  --tls-cert certs/server.crt \
  --tls-key certs/server.key

# Mit mTLS: Beide Seiten authentifizieren sich
cargo run --bin cap-verifier-api \
  --bind 0.0.0.0:8443 \
  --tls \
  --tls-cert certs/server.crt \
  --tls-key certs/server.key \
  --mtls \
  --tls-ca certs/ca.crt
```

---

#### `--tls-ca <FILE>`
**Wofür brauche ich das?**
Das Zertifikat der "Zertifizierungsstelle" (Certificate Authority, CA), die die Client-Zertifikate ausgestellt hat.

**Warum ist das wichtig?**
Wenn du `--mtls` aktivierst, muss der Server prüfen:
- Ist das Client-Zertifikat echt?
- Wurde es von einer vertrauenswürdigen CA ausgestellt?

Das CA-Zertifikat ist die "Wurzel des Vertrauens". Nur Clients mit Zertifikaten, die von dieser CA signiert wurden, dürfen verbinden.

**Wann nutze ich das?**
Immer zusammen mit `--mtls`. Ohne CA-Zertifikat kann der Server die Client-Zertifikate nicht validieren.

**Wer ist die CA?**
1. **Interne CA:** Dein Unternehmen betreibt eigene PKI (z.B. mit `openssl ca`, `easy-rsa`, oder Windows CA)
2. **Externe CA:** Let's Encrypt, DigiCert, GlobalSign (für öffentliche Zertifikate)
3. **Self-Signed CA:** Für Tests (erstellt mit `openssl req -x509 -new -nodes -key ca.key -sha256 -days 1024 -out ca.crt`)

**Format:** PEM (mit `-----BEGIN CERTIFICATE-----`)

**Beispiel:**
```bash
# CA-Zertifikat angeben (für Client-Validierung)
--tls-ca certs/ca.crt

# CA-Zertifikat erstellen (für Tests):
openssl req -x509 -new -nodes \
  -key ca.key \
  -sha256 \
  -days 1024 \
  -out ca.crt \
  -subj "/CN=My Test CA"
```

---

### Zusammenfassung: TLS vs. mTLS

| Feature | HTTP (kein TLS) | TLS | mTLS |
|---------|----------------|-----|------|
| **Verschlüsselung** | ❌ Klartext | ✅ AES-256 | ✅ AES-256 |
| **Server-Auth** | ❌ Nein | ✅ Ja (via Zertifikat) | ✅ Ja |
| **Client-Auth** | ❌ Nein | ❌ Nein | ✅ Ja (via Zertifikat) |
| **Verwendung** | Nur localhost | Production Standard | High-Security |
| **Komplexität** | Einfach | Mittel | Hoch |
| **Benötigte Parameter** | `--bind` | `--tls`, `--tls-cert`, `--tls-key` | + `--mtls`, `--tls-ca` |

**Empfehlung:**
- **Entwicklung (localhost):** HTTP ohne TLS (`--bind 127.0.0.1:8080`)
- **Production (Internet):** TLS (`--tls --tls-cert --tls-key`)
- **High-Security (B2B/M2M):** mTLS (`--mtls --tls-ca`)

---

**Endpoints:**
- `GET /healthz` - Health Check (öffentlich) - "Ist der Server erreichbar?"
- `GET /readyz` - Readiness Check (öffentlich) - "Kann der Server Anfragen verarbeiten?"
- `POST /verify` - Proof verifizieren (authentifiziert)
- `POST /policy/v2/compile` - Policy kompilieren (authentifiziert)
- `GET /policy/:id` - Policy abrufen (authentifiziert)

**Beenden:**
- `Ctrl+C` im Terminal

**Tipps:**
- Für lokale Entwicklung: Kein TLS nötig, einfach `cargo run --bin cap-verifier-api`
- Für Production: Immer TLS nutzen!
- Token notieren - brauchst du später in der WebUI

---

## 📦 Commitment-Commands

### `prepare` - CSV-Daten in Commitments umwandeln

**Wofür brauche ich das?**
Du hast sensible Geschäftsdaten (Namen deiner Zulieferer, UBOs) als CSV-Dateien und willst daraus **kryptografische Fingerprints** machen, die beweisen "Ich habe diese Daten", **ohne die Daten zu zeigen**.

**Warum ist das wichtig?**
Das ist der **erste Schritt** im gesamten Proof-Workflow. Ohne Commitments kannst du keine Proofs erstellen. Ein "Commitment" ist wie ein versiegelter Briefumschlag: Du kannst beweisen, dass du einen Brief hast, ohne ihn zu öffnen.

**Wann nutze ich das?**
- Zu Beginn jedes Compliance-Nachweises
- Nachdem du neue CSV-Dateien mit Supplier/UBO-Daten erstellt hast
- Wenn du deine internen Daten aktualisiert hast

**Was macht das?**
Liest Supplier- und UBO-Daten aus CSV-Dateien, berechnet kryptografische Hashes (BLAKE3-basierte Merkle-Roots) und speichert diese als "Commitments". Gleichzeitig wird ein Audit-Log angelegt, das alle Schritte nachvollziehbar dokumentiert.

**Command:**
```bash
cargo run --bin cap-agent -- prepare \
  --suppliers <CSV-DATEI> \
  --ubos <CSV-DATEI>
```

**Beispiel:**
```bash
cargo run --bin cap-agent -- prepare \
  --suppliers ../examples/suppliers.csv \
  --ubos ../examples/ubos.csv
```

**Was wird erstellt:**
- `build/commitments.json` - Merkle-Roots und Counts (die kryptografischen Fingerprints)
- `build/agent.audit.jsonl` - Audit-Log mit Hash-Chain (Nachweis, dass nichts manipuliert wurde)

**CSV-Format Suppliers:**
```csv
name,jurisdiction,tier
Acme GmbH,DE,1
Globex AG,PL,2
Umbrella Corp,US,1
```

**Was bedeuten die Spalten?**
- `name` - Firmenname des Zulieferers
- `jurisdiction` - Land (ISO 3166-1 alpha-2 Code, z.B. DE, PL, US)
- `tier` - Lieferanten-Stufe (1 = direkt, 2 = Sublieferant, etc.)

**CSV-Format UBOs:**
```csv
name,birthdate,citizenship
Alice Example,1980-01-01,DE
Bob Muster,1975-02-02,AT
```

**Was bedeuten die Spalten?**
- `name` - Name des wirtschaftlich Berechtigten (Ultimate Beneficial Owner)
- `birthdate` - Geburtsdatum (YYYY-MM-DD)
- `citizenship` - Staatsbürgerschaft (ISO 3166-1 alpha-2)

**Output-Datei (`build/commitments.json`):**
```json
{
  "supplier_root": "0xdde3f2c96c5ffc46eef6af7fe449ba6c575b71eff26d0829ce6d48872b2f1610",
  "ubo_root": "0xf89ea642046c73faa32494ed30672c7a7a7f764e399d1fb6d1c342ff3e7bf846",
  "company_commitment_root": "0x83a8779d0d7e3a7590133318265569f2651a4f8090afcae880741efcfc898ae5",
  "supplier_count": 2,
  "ubo_count": 2
}
```

**Was bedeuten die Werte?**
- `supplier_root` - Kryptografischer Hash **aller** Supplier-Daten zusammen
- `ubo_root` - Kryptografischer Hash **aller** UBO-Daten zusammen
- `company_commitment_root` - Kombinierter Hash (Gesamtbild)
- `supplier_count` / `ubo_count` - Anzahl der Einträge

**Wichtig:** Diese Hashes ändern sich bei **jeder kleinsten Änderung** der CSV-Dateien. Dadurch kannst du später beweisen, dass nichts manipuliert wurde.

---

### `inspect` - Commitments anzeigen

**Wofür brauche ich das?**
Du willst **schnell prüfen**, welche Commitments erstellt wurden, ohne die JSON-Datei manuell zu öffnen.

**Warum ist das wichtig?**
Debugging und Kontrolle: Du kannst sofort sehen, ob die Commitment-Berechnung funktioniert hat und wie viele Supplier/UBOs erfasst wurden.

**Wann nutze ich das?**
- Nach `prepare`, um zu prüfen ob alles geklappt hat
- Beim Debugging (z.B. "Warum sind es nur 2 Supplier statt 3?")
- Für schnelle Überprüfung ohne `cat | jq`

**Was macht das?**
Liest eine Commitments-JSON-Datei und zeigt die Inhalte formatiert und lesbar im Terminal an.

**Command:**
```bash
cargo run --bin cap-agent -- inspect <DATEI>
```

**Beispiel:**
```bash
cargo run --bin cap-agent -- inspect build/commitments.json
```

**Output:**
```
📊 Commitments-Datei: build/commitments.json

Merkle-Roots:
  Supplier Root:           0xdde3f2c96c5ffc46eef6af7fe449ba6c575b71eff26d0829ce6d48872b2f1610
  UBO Root:                0xf89ea642046c73faa32494ed30672c7a7a7f764e399d1fb6d1c342ff3e7bf846
  Company Commitment Root: 0x83a8779d0d7e3a7590133318265569f2651a4f8090afcae880741efcfc898ae5

Counts:
  Suppliers: 2
  UBOs:      2
```

**Tipps:**
- Wenn `Suppliers: 0` → CSV-Datei leer oder falsch formatiert
- Wenn Root = `0x000...` → Fehler beim Hashing, CSV nochmal prüfen

---

## 📋 Policy-Commands

### Policies: Was sind das überhaupt?

**Einfach erklärt:** Eine Policy ist eine **Regel-Datei**, die festlegt, welche Compliance-Anforderungen du erfüllen musst.

**Beispiel:** "Mindestens 1 UBO muss dokumentiert sein" oder "Maximal 10 Supplier erlaubt"

**Warum brauche ich das?** Damit der Proof-Verifizierer weiß, **was geprüft werden soll**. Ohne Policy kann der Verifier nicht entscheiden, ob deine Daten den Anforderungen entsprechen.

---

### `policy validate` - Policy-Datei prüfen (Legacy)

**Wofür brauche ich das?**
Du hast eine Policy-Datei (v1 Format, YAML) geschrieben und willst **vor dem Einsatz prüfen**, ob sie syntaktisch korrekt ist.

**Warum ist das wichtig?**
Eine fehlerhafte Policy führt dazu, dass der gesamte Proof-Workflow scheitert. Besser jetzt prüfen als später beim Manifest-Build eine kryptische Fehlermeldung bekommen!

**Wann nutze ich das?**
- Nachdem du eine neue Policy geschrieben hast
- Vor dem `manifest build` (Fehler früh erkennen!)
- Beim Debugging von Policy-Problemen

**Was macht das?**
Validiert eine Policy-Datei (YAML oder JSON) und berechnet den Policy-Hash (eindeutiger Fingerprint der Policy).

**Command:**
```bash
cargo run --bin cap-agent -- policy validate \
  --file <POLICY-DATEI>
```

**Beispiel:**
```bash
cargo run --bin cap-agent -- policy validate \
  --file ../examples/policy.lksg.v1.yml
```

**Policy-Format (YAML):**
```yaml
version: lksg.v1
name: LkSG Demo Policy
created_at: "2025-11-20T10:00:00Z"
constraints:
  require_at_least_one_ubo: true
  supplier_count_max: 10
notes: "Beispiel-Policy für LkSG-Compliance"
```

**Was bedeuten die Felder?**
- `version` - Policy-Format-Version (lksg.v1 = alte Version)
- `name` - Beschreibender Name
- `created_at` - Erstellungszeitpunkt (RFC3339 Format)
- `constraints` - Die eigentlichen Regeln (siehe unten)
- `notes` - Optionale Notizen

**Constraints (Beispiele):**
- `require_at_least_one_ubo: true` - Mindestens 1 UBO muss vorhanden sein
- `supplier_count_max: 10` - Maximal 10 Supplier erlaubt

**Output bei Erfolg:**
```
✅ Policy ist gültig!
Policy Hash: 0x0afcb40286c59c2b6ec47e7e3a5f8a9b...
```

**Tipps:**
- Policy Hash merken - den brauchst du später zum Verifizieren
- Constraints dürfen nicht widersprüchlich sein (z.B. min=5, max=3)

---

### `policy lint` - PolicyV2 linting

**Wofür brauche ich das?**
Du nutzt das **neue Policy-Format (v2)** und willst nicht nur Syntax-Fehler finden, sondern auch **Best Practices** überprüfen (z.B. "Solltest du nicht auch einen Sanktionslisten-Check haben?").

**Warum ist das wichtig?**
PolicyV2 ist flexibler als v1, aber auch komplexer. Der Linter hilft dir, **häufige Fehler zu vermeiden** und sicherzustellen, dass deine Policy sinnvoll ist.

**Wann nutze ich das?**
- Immer wenn du eine PolicyV2 schreibst
- Vor dem Compile-Schritt
- Im Strict Mode: Wenn du sichergehen willst, dass **keine Warnungen** mehr da sind

**Was macht das?**
Prüft eine PolicyV2-Datei auf Fehler und Warnungen. Im Relaxed Mode (Standard) sind Warnungen ok, im Strict Mode führen Warnungen zum Abbruch.

**Command:**
```bash
cargo run --bin cap-agent -- policy lint <DATEI> [--strict]
```

**Beispiel:**
```bash
# Relaxed Mode (Warnungen erlaubt)
cargo run --bin cap-agent -- policy lint ../examples/policy_v2.yml

# Strict Mode (Warnungen = Fehler)
cargo run --bin cap-agent -- policy lint ../examples/policy_v2.yml --strict
```

**PolicyV2-Format:**
```yaml
id: lksg.demo.v1
version: 1.0.0
legal_basis:
  - directive: LkSG
    article: §3
description: Demo policy für Lieferketten-Compliance
inputs:
  ubo_count:
    type: integer
  supplier_count:
    type: integer
rules:
  - id: rule_ubo_exists
    op: range_min
    lhs:
      var: ubo_count
    rhs: 1
  - id: rule_supplier_limit
    op: range_min
    lhs:
      var: supplier_count
    rhs: 1
```

**Was bedeuten die Felder?**
- `id` - Eindeutige Policy-ID (z.B. für Registry-Lookups)
- `version` - Versionsnummer der Policy (SemVer)
- `legal_basis` - Welches Gesetz/Regulierung (z.B. LkSG §3, GDPR Art. 5)
- `description` - Was prüft diese Policy?
- `inputs` - Welche Variablen braucht die Policy? (z.B. ubo_count, supplier_count)
- `rules` - Die eigentlichen Prüfregeln

**Regeln-Syntax:**
```yaml
- id: rule_ubo_exists       # Eindeutige Regel-ID
  op: range_min             # Operator (siehe unten)
  lhs:                      # Linke Seite (Left Hand Side)
    var: ubo_count          # Variable aus inputs
  rhs: 1                    # Rechte Seite: Mindestwert 1
```

**Bedeutung:** "ubo_count muss >= 1 sein"

**Erlaubte Operatoren:**
- `range_min` - Minimum-Check (≥) - "Mindestens X"
- `range_max` - Maximum-Check (≤) - "Maximal X"
- `eq` - Equality (=) - "Exakt X"
- `non_membership` - Blacklist-Check - "Darf NICHT in Liste sein"

**Output (Relaxed Mode):**
```
⚠️  1 Warnung gefunden:
  - Regel 'rule_ubo_exists': Solltest du nicht auch ein Maximum setzen?

✅ Keine Fehler, Policy ist verwendbar
```

**Output (Strict Mode):**
```
❌ 1 Warnung gefunden, strict mode aktiviert:
  - Regel 'rule_ubo_exists': Solltest du nicht auch ein Maximum setzen?

Policy-Lint fehlgeschlagen
```

**Tipps:**
- Nutze `--strict` vor Production-Deployment
- Warnungen ernst nehmen - oft weisen sie auf Lücken in der Compliance hin

---

### `policy compile` - PolicyV2 kompilieren

**Wofür brauche ich das?**
Du willst eine PolicyV2 **in maschinenlesbares Format** umwandeln (Intermediate Representation = IR), damit sie vom Proof-System genutzt werden kann.

**Warum ist das wichtig?**
Die YAML-Policy ist für Menschen lesbar, aber das Proof-System braucht ein optimiertes JSON-Format (IR). Der Compile-Schritt übersetzt und **optimiert** die Policy.

**Wann nutze ich das?**
- Nach dem Lint-Schritt (wenn keine Fehler mehr da sind)
- Vor dem Upload zur Registry/API
- Wenn du Policies zur Laufzeit laden willst (z.B. für dynamische Verifikation)

**Was macht das?**
Kompiliert eine PolicyV2 in Intermediate Representation (IR) - ein JSON-Format, das vom ZK-System ausgeführt werden kann.

**Command:**
```bash
cargo run --bin cap-agent -- policy compile \
  <INPUT-DATEI> \
  --output <OUTPUT-JSON>
```

**Beispiel:**
```bash
cargo run --bin cap-agent -- policy compile \
  ../examples/policy_v2.yml \
  --output build/policy_ir.json
```

**Output:** JSON-Datei mit kompiliertem IR

**Beispiel IR-Output:**
```json
{
  "schema": "cap-policy-ir.v2",
  "id": "lksg.demo.v1",
  "version": "1.0.0",
  "bytecode": [
    {"op": "load_var", "var": "ubo_count"},
    {"op": "push_const", "value": 1},
    {"op": "cmp_gte"},
    {"op": "assert", "rule_id": "rule_ubo_exists"}
  ]
}
```

**Was ist "Bytecode"?**
Eine Folge von Anweisungen, die der ZK-Verifier ausführt, um die Compliance zu prüfen. Ähnlich wie Assembler-Code für CPUs.

**Tipps:**
- IR-Datei kann sehr groß werden bei komplexen Policies
- IR ist **deterministisch**: Gleiche Policy → gleicher IR (wichtig für Reproduzierbarkeit)

---

### `policy show` - IR anzeigen

**Wofür brauche ich das?**
Du hast eine kompilierte Policy (IR) und willst **verstehen, was der Bytecode macht**.

**Warum ist das wichtig?**
Debugging: Wenn eine Policy nicht das tut, was du erwartest, hilft der `show` Befehl zu verstehen, was tatsächlich ausgeführt wird.

**Wann nutze ich das?**
- Beim Debugging von Policy-Fehlern
- Um zu verstehen, wie deine YAML-Policy in Bytecode übersetzt wurde
- Für Audits (Auditor will verstehen, was geprüft wird)

**Was macht das?**
Zeigt kompilierten Policy-IR lesbar und strukturiert an.

**Command:**
```bash
cargo run --bin cap-agent -- policy show <IR-JSON>
```

**Beispiel:**
```bash
cargo run --bin cap-agent -- policy show build/policy_ir.json
```

**Output:**
```
📋 Policy IR: lksg.demo.v1
Version: 1.0.0
Schema: cap-policy-ir.v2

Bytecode (4 Instruktionen):
  0: load_var(ubo_count)
  1: push_const(1)
  2: cmp_gte
  3: assert(rule_id=rule_ubo_exists)

Bedeutung:
  Lade Wert von 'ubo_count' → Pushe 1 auf Stack → Vergleiche (>=) → Assert (sonst Fehler)
```

---

## 📄 Manifest-Commands

### Manifests: Was ist das?

**Einfach erklärt:** Ein Manifest ist ein **Datenblatt**, das alle wichtigen Informationen zusammenfasst:
- Welche Commitments (verschlüsselte Daten) liegen vor?
- Welche Policy soll geprüft werden?
- Wann wurde das Manifest erstellt?
- Audit-Trail: Was ist passiert?

**Metapher:** Wie ein Inhaltsverzeichnis eines Buchs - zeigt, was drin ist, ohne die Details preiszugeben.

---

### `manifest build` - Manifest erstellen

**Wofür brauche ich das?**
Du hast Commitments und eine Policy - jetzt willst du beides **zusammenführen** zu einem Manifest, das später für Proofs genutzt wird.

**Warum ist das wichtig?**
Das Manifest ist die **zentrale Datenstruktur** für den gesamten Proof-Workflow. Ohne Manifest kein Proof! Es bindet Commitments + Policy + Audit-Log zusammen.

**Wann nutze ich das?**
- Nach `prepare` (Commitments müssen existieren!)
- Nachdem du eine passende Policy ausgewählt hast
- Als Schritt 2 im Standard-Workflow (prepare → manifest build → proof build)

**Was macht das?**
Erstellt ein Compliance-Manifest aus Commitments und Policy. Liest `build/commitments.json` und die angegebene Policy, kombiniert alles und schreibt `build/manifest.json`.

**Command:**
```bash
cargo run --bin cap-agent -- manifest build \
  --policy <POLICY-DATEI> \
  [--out <OUTPUT-DATEI>]
```

**Beispiel:**
```bash
# Standard (Output: build/manifest.json)
cargo run --bin cap-agent -- manifest build \
  --policy ../examples/policy.lksg.v1.yml

# Custom Output
cargo run --bin cap-agent -- manifest build \
  --policy ../examples/policy.lksg.v1.yml \
  --out custom/manifest.json
```

**Voraussetzung:**
- `build/commitments.json` muss existieren (vorher `prepare` ausführen!)

**Output-Datei (`build/manifest.json`):**
```json
{
  "version": "manifest.v1.0",
  "created_at": "2025-11-20T15:30:00Z",
  "supplier_root": "0xdde3f2c...",
  "ubo_root": "0xf89ea64...",
  "company_commitment_root": "0x83a8779...",
  "policy": {
    "name": "LkSG Demo Policy",
    "version": "lksg.v1",
    "hash": "0x0afcb40..."
  },
  "audit": {
    "tail_digest": "0xdb0507c...",
    "events_count": 20
  }
}
```

**Was bedeuten die Felder?**
- `version` - Manifest-Format-Version
- `created_at` - Zeitstempel (wichtig für Registry)
- `supplier_root` / `ubo_root` - Von Commitments übernommen
- `policy.hash` - Eindeutiger Fingerprint der Policy (damit kann man später prüfen: "Wurde die richtige Policy verwendet?")
- `audit.tail_digest` - Letzter Hash der Audit-Chain (Manipulationsschutz)
- `audit.events_count` - Wie viele Events wurden geloggt?

**Tipps:**
- Manifest Hash wird später wichtig! (Für Registry und Signatur)
- Wenn `created_at` fehlt → Fehler im System-Zeitstempel
- Wenn `policy.hash` = null → Policy-Datei konnte nicht gelesen werden

---

### `manifest validate` - Manifest gegen Schema prüfen

**Wofür brauche ich das?**
Du hast ein Manifest erstellt (oder von jemand anderem bekommen) und willst **prüfen, ob es dem offiziellen Standard entspricht**.

**Warum ist das wichtig?**
Manifests müssen einem strikten Schema folgen (JSON Schema Draft 2020-12). Wenn ein Manifest Schema-Fehler hat, wird es von Verifizierern **abgelehnt**.

**Wann nutze ich das?**
- Bevor du ein Manifest an Auditoren schickst
- Nach manueller Bearbeitung von Manifests (solltest du eigentlich nie machen!)
- Beim Debugging von Verifikationsfehlern ("Warum wird mein Manifest abgelehnt?")

**Was macht das?**
Validiert ein Manifest gegen das JSON Schema (Draft 2020-12) und prüft alle Felder auf Korrektheit.

**Command:**
```bash
cargo run --bin cap-agent -- manifest validate \
  --file <MANIFEST-DATEI> \
  [--schema <SCHEMA-DATEI>]
```

**Beispiel:**
```bash
# Mit Standard-Schema (docs/manifest.schema.json)
cargo run --bin cap-agent -- manifest validate \
  --file build/manifest.json

# Mit Custom-Schema (z.B. für v2)
cargo run --bin cap-agent -- manifest validate \
  --file build/manifest.json \
  --schema custom/manifest.v2.schema.json
```

**Output bei Erfolg:**
```
✅ Manifest-Validierung erfolgreich!
Manifest: build/manifest.json
Schema: docs/manifest.schema.json

Geprüfte Felder: 12
  ✅ version: manifest.v1.0
  ✅ created_at: 2025-11-20T15:30:00Z (gültiges RFC3339)
  ✅ policy.hash: 0x0afcb... (64 hex chars)
  ✅ supplier_root: 0xdde3f... (64 hex chars)
  ... alle Prüfungen bestanden
```

**Output bei Fehler:**
```
❌ Manifest-Validierung fehlgeschlagen!
Fehler:
  - "created_at" is not a valid RFC3339 timestamp
    Gefunden: "2025-11-20 15:30:00"
    Erwartet: "2025-11-20T15:30:00Z"

  - "policy.hash" must match pattern "^0x[a-f0-9]{64}$"
    Gefunden: "0xabc123" (zu kurz)
    Erwartet: 66 Zeichen (0x + 64 hex)

  - "audit.events_count" must be >= 0
    Gefunden: -5
```

**Tipps:**
- **Immer vor dem Versand validieren!**
- Häufigster Fehler: Zeitstempel falsch formatiert (muss ISO 8601 / RFC3339 sein)
- Hashes müssen **genau** 66 Zeichen haben (0x + 64 hex)

---

### `manifest verify` - Offline-Verifikation

**Wofür brauche ich das?**
Du hast ein **komplettes Proof-Package** (Manifest + Proof + optional Registry/Timestamp) und willst alles **lokal verifizieren**, ohne Server.

**Warum ist das wichtig?**
Vertraue nicht blind! Du solltest Proofs **selbst prüfen können**, bevor du sie weitergibst. Dieser Befehl macht genau das - ohne Cloud, ohne API, nur lokal auf deinem Rechner.

**Wann nutze ich das?**
- Bevor du ein Proof-Package an Auditoren schickst
- Wenn du ein Proof-Package von jemand anderem bekommen hast (Vertraue, aber prüfe!)
- Beim Debugging ("Warum schlägt die Verifikation fehl?")
- Als Teil eines CI/CD-Prozesses (Automatisierte Qualitätsprüfung)

**Was macht das?**
Verifiziert ein vollständiges Proof-Paket offline (ohne Server). Prüft Hashes, Signaturen, Timestamp und Registry-Einträge.

**Command:**
```bash
cargo run --bin cap-agent -- manifest verify \
  --manifest <MANIFEST-DATEI> \
  --proof <PROOF-DATEI> \
  --registry <REGISTRY-DATEI> \
  [--timestamp <TSR-DATEI>] \
  [--out <REPORT-DATEI>]
```

**Beispiel:**
```bash
cargo run --bin cap-agent -- manifest verify \
  --manifest build/manifest.json \
  --proof build/proof.capz \
  --registry build/registry.json \
  --timestamp build/timestamp.tsr \
  --out build/verification.report.json
```

**Verifikationsschritte:**
1. **Hash-Berechnung:** Manifest + Proof → Hashes berechnen
2. **Signatur-Verifikation:** Wenn vorhanden, Ed25519-Signatur prüfen
3. **Timestamp-Verifikation:** Wenn vorhanden, RFC3161 Timestamp prüfen
4. **Registry-Match:** Proof in Registry vorhanden?
5. **Policy-Check:** Wurde die richtige Policy verwendet?

**Output:**
```
🔍 Starte Offline-Verifikation...

📄 Manifest:
  Hash: 0xd490be94abc12345...
  Created: 2025-11-20T15:30:00Z
  Policy: LkSG Demo Policy (0x0afcb40...)

🔬 Proof:
  Hash: 0x83a8779ddef45678...
  Type: mock
  Status: ok

✅ Hash-Verifikation
  Manifest Hash: 0xd490be94... ✅
  Proof Hash:    0x83a8779d... ✅

✅ Signatur-Verifikation
  Signer PubKey: 0x9a1b2c3d...
  Signature:     ✅ Gültig

✅ Timestamp-Verifikation
  Timestamp:     2025-11-20T15:35:00Z
  TSA:           DFN-PKI
  Status:        ✅ Gültig

✅ Registry-Verifikation
  Entry ID:      proof_001
  Status:        ✅ Gefunden

🎉 Gesamtstatus: OK

Verification Report gespeichert: build/verification.report.json
```

**Output bei Fehler:**
```
❌ Verifikation fehlgeschlagen!

🔍 Probleme gefunden:
  ❌ Hash-Verifikation
     Manifest Hash stimmt nicht überein
     Berechnet: 0xabc123...
     Im Proof:  0xdef456...
     → Manifest wurde nach Proof-Erstellung verändert!

  ❌ Registry-Verifikation
     Proof nicht in Registry gefunden
     → Proof wurde nicht registriert oder falsche Registry

Status: FAIL
```

**Verification Report (`build/verification.report.json`):**
```json
{
  "timestamp": "2025-11-20T16:00:00Z",
  "manifest_hash": "0xd490be94...",
  "proof_hash": "0x83a8779d...",
  "checks": {
    "hash_verification": "pass",
    "signature_verification": "pass",
    "timestamp_verification": "pass",
    "registry_verification": "pass"
  },
  "overall_status": "ok"
}
```

**Tipps:**
- **Immer einen Verification Report erstellen** (mit `--out`) für Audit-Trail
- Wenn Hash-Verifikation fehlschlägt → Manifest oder Proof manipuliert!
- Wenn Signatur-Verifikation fehlschlägt → Falscher Public Key oder Signatur ungültig

---

## 🔬 Proof-Commands

### Proofs: Was ist das?

**Einfach erklärt:** Ein Proof ist der **mathematische Beweis**, dass deine Daten die Policy erfüllen, **ohne die Daten selbst preiszugeben**.

**Beispiel:** Du beweist "Ich habe mindestens 1 UBO dokumentiert" ohne zu sagen, wer es ist.

**Technisch:** Zero-Knowledge Proof - du zeigst, dass du ein Geheimnis kennst, ohne das Geheimnis zu verraten.

---

### `proof build` - Proof erstellen

**Wofür brauche ich das?**
Du hast ein Manifest und willst jetzt den **eigentlichen kryptografischen Beweis** erzeugen, dass deine Daten die Policy erfüllen.

**Warum ist das wichtig?**
Das ist der **Kern des gesamten Systems**! Der Proof ist das, was du Auditoren gibst. Er beweist Compliance ohne sensible Daten preiszugeben.

**Wann nutze ich das?**
- Nach `manifest build`
- Als Schritt 3 im Standard-Workflow (prepare → manifest build → **proof build**)
- Jedes Mal wenn sich deine Daten ändern (neue CSV → neue Commitments → neues Manifest → neuer Proof)

**Was macht das?**
Erstellt einen kryptografischen Proof aus Manifest und Policy. Nutzt aktuell ein Mock-Backend (für echte ZK-Proofs wird später WASM/Risc0 genutzt).

**Command:**
```bash
cargo run --bin cap-agent -- proof build
```

**Hinweis:** Der Befehl braucht keine Parameter mehr! Er liest automatisch:
- `build/manifest.json` - Manifest
- `build/commitments.json` - Commitments
- Policy aus dem Manifest

**Beispiel:**
```bash
# Einfach
cargo run --bin cap-agent -- proof build

# Falls Dateien woanders liegen (alt, wird nicht mehr empfohlen)
cargo run --bin cap-agent -- proof build \
  --policy ../examples/policy.lksg.v1.yml \
  --manifest build/manifest.json
```

**Voraussetzung:**
- `build/manifest.json` muss existieren
- `build/commitments.json` muss existieren

**Output-Dateien:**
- `build/proof.capz` - Base64-kodierter Proof (das gibst du weiter!)
- `build/proof.json` - Lesbare JSON-Version (für Debugging)

**Proof-Struktur (`build/proof.json`):**
```json
{
  "version": "proof.v0",
  "type": "mock",
  "statement": "policy:lksg.v1",
  "manifest_hash": "0xd490be94...",
  "policy_hash": "0x0afcb402...",
  "proof_data": {
    "checked_constraints": [
      {"name": "require_at_least_one_ubo", "ok": true},
      {"name": "supplier_count_max_10", "ok": true}
    ]
  },
  "status": "ok"
}
```

**Was bedeuten die Felder?**
- `type: "mock"` - Mock-Backend (kein echter ZK-Proof, nur für Demos)
- `statement` - Was wurde bewiesen? (z.B. "policy:lksg.v1")
- `manifest_hash` - Welches Manifest wurde verwendet?
- `policy_hash` - Welche Policy wurde geprüft?
- `checked_constraints` - Welche Regeln wurden geprüft? Alle ok?
- `status` - Gesamtergebnis: "ok", "warn" oder "fail"

**Status-Bedeutungen:**
- `ok` - Alle Constraints erfüllt ✅
- `warn` - Einige Constraints erfüllt, aber Warnungen ⚠️
- `fail` - Mindestens ein Constraint nicht erfüllt ❌

**Output im Terminal:**
```
🔨 Erstelle Proof...
📄 Lade Manifest: build/manifest.json
📋 Lade Policy: lksg.v1
🧮 Berechne Proof...

Constraint-Checks:
  ✅ require_at_least_one_ubo: OK (UBO count = 2)
  ✅ supplier_count_max_10: OK (Supplier count = 2)

✅ Proof erfolgreich erstellt!
  Status: ok
  Proof gespeichert: build/proof.capz
  JSON gespeichert: build/proof.json
```

**Tipps:**
- Wenn Status = "fail" → Deine Daten erfüllen die Policy nicht! CSV-Dateien prüfen.
- Proof.capz ist Base64 - nicht lesbar, aber kompakt zum Versenden
- Proof.json ist zum Debuggen - kannst du mit `jq` lesbar machen

---

### `proof verify` - Proof verifizieren

**Wofür brauche ich das?**
Du hast einen Proof erstellt (oder bekommen) und willst **lokal prüfen**, ob er gültig ist.

**Warum ist das wichtig?**
Selbstkontrolle! Bevor du einen Proof an Auditoren schickst, solltest du ihn selbst verifizieren. Verhindert peinliche Fehler wie "Proof ungültig" beim Auditor.

**Wann nutze ich das?**
- Direkt nach `proof build` (Qualitätskontrolle)
- Bevor du Proof-Packages versendest
- Wenn du einen Proof von jemand anderem bekommen hast

**Was macht das?**
Verifiziert einen Proof gegen das Manifest. Prüft ob Hashes stimmen und ob der Proof das Manifest korrekt nachweist.

**Command:**
```bash
cargo run --bin cap-agent -- proof verify
```

**Hinweis:** Keine Parameter nötig! Liest automatisch:
- `build/proof.capz` - Der Proof
- `build/manifest.json` - Das Manifest

**Beispiel (mit Parametern, alt):**
```bash
cargo run --bin cap-agent -- proof verify \
  --proof build/proof.capz \
  --manifest build/manifest.json
```

**Output:**
```
🔍 Verifiziere Proof...

📄 Manifest:
  Hash: 0xd490be94...
  Policy: LkSG Demo Policy (0x0afcb402...)

🔬 Proof:
  Hash: 0x83a8779d...
  Type: mock
  Status: ok

✅ Hash-Verifikation
  Manifest Hash im Proof: 0xd490be94... ✅ stimmt überein
  Policy Hash im Proof:   0x0afcb402... ✅ stimmt überein

✅ Constraint-Checks
  require_at_least_one_ubo ✅
  supplier_count_max_10    ✅

🎉 Proof-Verifikation erfolgreich!
Gesamtstatus: ok
```

**Output bei Fehler:**
```
❌ Proof-Verifikation fehlgeschlagen!

🔍 Probleme:
  ❌ Manifest Hash stimmt nicht
     Im Proof:     0xd490be94...
     Berechnet:    0xabc12345...
     → Manifest wurde nach Proof-Erstellung geändert!

  ❌ Constraint 'require_at_least_one_ubo' fehlgeschlagen
     UBO count = 0
     Erwartet: >= 1

Status: FAIL
```

**Tipps:**
- Immer direkt nach `proof build` ausführen!
- Wenn Hash-Check fehlschlägt → Dateien manipuliert oder falsche Dateien verwendet

---

### `proof export` - Standardisiertes Proof-Paket erstellen

**Wofür brauche ich das?**
Du willst ein **vollständiges Paket** erstellen, das alle nötigen Dateien enthält und **ready für Auditoren** ist.

**Warum ist das wichtig?**
Auditoren brauchen nicht nur den Proof, sondern auch:
- Manifest (um zu wissen, was geprüft wurde)
- Optional: Timestamp (um zu wissen, wann)
- Optional: Registry-Entry (um zu wissen, ob registriert)
- README (um zu wissen, wie man es verifiziert)

Das `proof export` Kommando packt alles zusammen in ein standardisiertes Format.

**Wann nutze ich das?**
- Nach erfolgreicher `proof verify`
- Bevor du das Package als ZIP verschickst
- Als letzter Schritt im Compliance-Workflow

**Was macht das?**
Erstellt ein auditor-fertiges CAP Proof-Paket (v1.0) mit allen Dateien in standardisierter Struktur.

**Command:**
```bash
cargo run --bin cap-agent -- proof export \
  --manifest <MANIFEST-DATEI> \
  --proof <PROOF-DATEI> \
  [--timestamp <TSR-DATEI>] \
  [--registry <REGISTRY-DATEI>] \
  [--report <REPORT-DATEI>] \
  [--out <OUTPUT-DIR>] \
  [--force]
```

**Beispiel:**
```bash
# Minimal (nur Manifest + Proof)
cargo run --bin cap-agent -- proof export \
  --manifest build/manifest.json \
  --proof build/proof.capz

# Vollständig (mit allem)
cargo run --bin cap-agent -- proof export \
  --manifest build/manifest.json \
  --proof build/proof.capz \
  --timestamp build/timestamp.tsr \
  --registry build/registry.json \
  --out build/cap-proof-v2 \
  --force
```

**Output-Struktur:**
```
build/cap-proof/
├─ manifest.json              # Manifest mit Commitments
├─ proof.capz                 # ZK-Proof (Base64, der eigentliche Beweis!)
├─ timestamp.tsr              # Timestamp (optional, RFC3161)
├─ registry.json              # Registry (optional, Liste aller Proofs)
├─ verification.report.json   # Verification Report (automatisch erstellt)
├─ README.txt                 # Anleitung für Auditoren
└─ _meta.json                 # SHA3-256 Hashes aller Dateien (Integritätsprüfung)
```

**Was steht im README.txt?**
```
CAP Proof Package v1.0
======================

Dieses Paket enthält einen Zero-Knowledge Proof für Lieferketten-Compliance.

Dateien:
- manifest.json: Compliance-Manifest
- proof.capz: Kryptografischer Proof
- timestamp.tsr: RFC3161 Timestamp
- _meta.json: SHA3-256 Hashes aller Dateien

Verifikation:
1. Hashes prüfen: cat _meta.json
2. Proof verifizieren: cap-agent manifest verify --manifest manifest.json --proof proof.capz
3. Timestamp prüfen: openssl ts -verify -in timestamp.tsr -data manifest.json

Kontakt: compliance@example.com
```

**Was steht in _meta.json?**
```json
{
  "schema": "cap-bundle.v1",
  "bundle_id": "550e8400-e29b-41d4-a716-446655440000",
  "created_at": "2025-11-20T16:00:00Z",
  "files": {
    "manifest.json": {
      "role": "manifest",
      "hash": "0x1da941f7026bae3cf8b1bcdc3a8e01e76ea678c32ec6bc2c374fb67b3744571f",
      "size": 1234,
      "content_type": "application/json",
      "optional": false
    },
    "proof.capz": {
      "role": "proof",
      "hash": "0x83a8779ddef4567890123456789012345678901234567890123456789012345678",
      "size": 5678,
      "content_type": "application/octet-stream",
      "optional": false
    },
    "policy.yml": {
      "role": "policy",
      "hash": "0x0afcb40286c59c2b6ec47e7e3a5f8a9b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7",
      "size": 456,
      "content_type": "application/x-yaml",
      "optional": true
    }
  },
  "proof_units": [
    {
      "manifest_file": "manifest.json",
      "proof_file": "proof.capz",
      "policy_info": {
        "name": "LkSG Demo Policy",
        "version": "lksg.v1",
        "hash": "0x0afcb40286c59c2b6ec47e7e3a5f8a9b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7"
      },
      "backend": "mock"
    }
  ]
}
```

**BundleMeta Struktur (cap-bundle.v1):**
- `schema`: Format-Version ("cap-bundle.v1")
- `bundle_id`: Eindeutige Bundle-UUID (v4)
- `created_at`: Erstellungszeitpunkt (RFC3339)
- `files`: HashMap mit BundleFileMeta für jede Datei
  - `role`: Datei-Rolle ("manifest" | "proof" | "policy" | "other")
  - `hash`: SHA3-256 Hash (0x-präfixiert, 64 hex chars)
  - `size`: Dateigröße in Bytes
  - `content_type`: MIME-Type (z.B. "application/json")
  - `optional`: Ob die Datei optional ist (true/false)
- `proof_units`: Array von ProofUnit-Objekten
  - `manifest_file`: Referenz zur Manifest-Datei
  - `proof_file`: Referenz zur Proof-Datei
  - `policy_info`: Auto-extrahierte Policy-Informationen
    - `name`: Policy-Name
    - `version`: Policy-Version
    - `hash`: Policy-Hash (SHA3-256)
  - `backend`: ZK-Backend ("mock" | "zkvm" | "halo2")

**Backward-Kompatibilität:**
Der `verifier run` Befehl unterstützt automatisch das ältere `cap-proof.v1.0` Format als Fallback. Bundles im alten Format können weiterhin verifiziert werden.

**Security Features:**
- Path Traversal Prevention (sanitize_filename)
- Dependency Cycle Detection (DFS-Algorithmus)
- TOCTOU Mitigation (Load-Once-Pattern)
- Bundle Type Detection (Modern vs Legacy)

**Flags:**
- `--force` - Überschreibt existierendes Output-Verzeichnis (sonst Fehler bei Duplikaten)
- `--out` - Custom Output-Pfad (Standard: `build/cap-proof`)

**Dann ZIP erstellen:**
```bash
cd build
zip -r proof-package.zip cap-proof/
```

**Tipps:**
- Immer `_meta.json` mit verschicken - Auditoren können damit Integrität prüfen
- README.txt anpassen mit deinen Kontaktdaten
- ZIP komprimiert gut (Proof ist Base64, komprimiert auf ~70% der Größe)

---

## 🔍 Verifier-Commands

### Verifier: Wofür?

**Einfach erklärt:** Der Verifier ist das Werkzeug für **Auditoren** (oder dich selbst), um Proof-Packages zu **prüfen und analysieren**.

**Du bist der Ersteller** → Nutze `proof build`
**Du bist der Prüfer** → Nutze `verifier run`

---

### `verifier run` - Proof-Paket verifizieren

**Wofür brauche ich das?**
Du hast ein Proof-Package bekommen (oder selbst erstellt) und willst es **komplett durchprüfen** - alle Dateien, alle Hashes, alle Signaturen.

**Warum ist das wichtig?**
Das ist der **offizielle Verifizierungs-Workflow** für Auditoren! Stell dir vor, ein Auditor bekommt dein ZIP - er entpackt es und führt `verifier run` aus. Wenn das OK ist, bist du durch.

**Wann nutze ich das?**
- Als Auditor: Immer wenn du ein Proof-Package bekommst
- Als Ersteller: Als finale Qualitätskontrolle vor Versand
- In CI/CD: Automatisierte Verifikation

**Was macht das?**
Offline-Verifikation eines vollständigen Proof-Pakets. Prüft alle Dateien, Hashes, Signaturen und Proof-Logik.

**Command:**
```bash
cargo run --bin cap-agent -- verifier run \
  --package <PACKAGE-DIR>
```

**Beispiel:**
```bash
# Nach Entpacken des ZIPs
unzip proof-package.zip -d cap-proof
cargo run --bin cap-agent -- verifier run \
  --package cap-proof
```

**Output:**
```
📦 Proof-Paket-Verifikation
Package: cap-proof/

🔍 Schritt 1: Integritätsprüfung
  Lade _meta.json...
  Prüfe Dateien:
    ✅ manifest.json (Hash: 0x1da941f... ✅)
    ✅ proof.capz    (Hash: 0x83a8779... ✅)
    ✅ timestamp.tsr (Hash: 0xabc1234... ✅)

  → Alle Hashes stimmen überein

🔬 Schritt 2: Proof-Verifikation
  Manifest Hash: 0xd490be94abc12345...
  Policy Hash:   0x0afcb40286c59c2b...

  Constraint-Checks:
    ✅ require_at_least_one_ubo (UBO count = 2)
    ✅ supplier_count_max_10 (Supplier count = 2)

  → 2/2 Constraints erfüllt

🕐 Schritt 3: Timestamp-Verifikation
  Timestamp: 2025-11-20T15:35:00Z
  TSA:       DFN-PKI
  Status:    ✅ Gültig

📋 Schritt 4: Registry-Verifikation
  Entry ID:  proof_001
  Status:    ✅ Gefunden

🎉 Gesamtstatus: OK

Ergebnis gespeichert: cap-proof/verification.report.json
```

**Output bei Fehler:**
```
❌ Proof-Paket-Verifikation fehlgeschlagen!

🔍 Schritt 1: Integritätsprüfung
  ❌ manifest.json
     Hash-Mismatch!
     Erwartet (_meta.json): 0x1da941f...
     Berechnet:             0xabc1234...
     → Datei wurde verändert nach Package-Erstellung!

  → Paket kompromittiert, Verifikation abgebrochen

Status: FAIL
```

**Verifikationsschritte im Detail:**

1. **Integritätsprüfung** - Wurden Dateien verändert?
   - Lese `_meta.json`
   - Berechne SHA3-256 Hash jeder Datei
   - Vergleiche mit Hashes in `_meta.json`
   - Wenn Mismatch → STOP!

2. **Proof-Verifikation** - Ist der Proof gültig?
   - Lade Manifest + Proof
   - Prüfe Manifest Hash
   - Prüfe Policy Hash
   - Führe alle Constraint-Checks aus
   - Wenn fail → STOP!

3. **Timestamp-Verifikation** (falls vorhanden)
   - Prüfe RFC3161 Timestamp
   - Prüfe TSA-Signatur
   - Prüfe Zeitstempel-Validität
   - Wenn ungültig → WARN (nicht STOP)

4. **Registry-Verifikation** (falls vorhanden)
   - Suche Proof in Registry
   - Prüfe KID (Key ID)
   - Prüfe Entry-Signatur
   - Wenn nicht gefunden → WARN (nicht STOP)

**Tipps:**
- **Immer vor dem Versand selbst ausführen!**
- Wenn Schritt 1 fehlschlägt → Dateien manipuliert oder ZIP beschädigt
- Wenn Schritt 2 fehlschlägt → Proof ungültig oder Policy nicht erfüllt

---

### `verifier extract` - Manifest extrahieren

**Wofür brauche ich das?**
Du willst **schnell verstehen**, was in einem Proof-Package drin ist, ohne alle Dateien manuell zu öffnen.

**Warum ist das wichtig?**
Als Auditor bekommst du vielleicht 10 Proof-Packages - du willst schnell sehen: "Welche Policy? Wie viele Supplier? Wann erstellt?"

**Wann nutze ich das?**
- Erste Analyse eines neuen Proof-Packages
- Überblick verschaffen vor detaillierter Prüfung
- Quick Check: "Ist das überhaupt das richtige Package?"

**Was macht das?**
Zeigt formatierte Zusammenfassung eines Proof-Pakets - Manifest-Infos, Proof-Infos, Constraints.

**Command:**
```bash
cargo run --bin cap-agent -- verifier extract \
  --package <PACKAGE-DIR>
```

**Beispiel:**
```bash
cargo run --bin cap-agent -- verifier extract \
  --package cap-proof
```

**Output:**
```
📄 Manifest-Informationen:
  Version:              manifest.v1.0
  Created At:           2025-11-20T15:30:00Z
  Company Root:         0x83a8779d0d7e3a7590133318265569f2651a4f8090afcae880741efcfc898ae5
  Supplier Root:        0xdde3f2c96c5ffc46eef6af7fe449ba6c575b71eff26d0829ce6d48872b2f1610
  UBO Root:             0xf89ea642046c73faa32494ed30672c7a7a7f764e399d1fb6d1c342ff3e7bf846

  Counts:
    Suppliers: 2
    UBOs:      2

📋 Policy-Informationen:
  Name:                 LkSG Demo Policy
  Version:              lksg.v1
  Hash:                 0x0afcb40286c59c2b6ec47e7e3a5f8a9b...

🔬 Proof-Informationen:
  Version:              proof.v0
  Type:                 mock
  Statement:            policy:lksg.v1
  Status:               ok

✅ Constraints (2/2 erfüllt):
  ✅ require_at_least_one_ubo
  ✅ supplier_count_max_10

📊 Audit-Trail:
  Events:               20
  Tail Digest:          0xdb0507c678598f504c6adbae471c14fe...
```

**Tipps:**
- Nutze das für schnelles Screening von Packages
- Wenn Status = "fail" → Package sofort ablehnen
- Wenn Created At > heute → Zeitstempel-Fehler oder Betrugsversuch

---

### `verifier audit` - Audit-Trail anzeigen

**Wofür brauche ich das?**
Du willst **nachvollziehen**, welche Schritte bei der Proof-Erstellung durchgeführt wurden - für forensische Analyse oder Compliance-Audits.

**Warum ist das wichtig?**
Der Audit-Trail ist eine **manipulationssichere Event-Kette** (Hash-Chain). Du kannst sehen: "Wann wurden die Daten geladen? Wann wurde das Manifest erstellt?" - und es ist beweisbar, dass nichts gelöscht wurde.

**Wann nutze ich das?**
- Forensische Analyse: "Was ist genau passiert?"
- Compliance-Audits: "Wie wurde der Proof erstellt?"
- Debugging: "Warum ist der Proof fehlgeschlagen?"

**Was macht das?**
Zeigt Audit-Event-Kette aus einem Proof-Paket. Listet alle Events chronologisch mit Timestamps und Hashes.

**Command:**
```bash
cargo run --bin cap-agent -- verifier audit \
  --package <PACKAGE-DIR>
```

**Beispiel:**
```bash
cargo run --bin cap-agent -- verifier audit \
  --package cap-proof
```

**Output:**
```
📜 Audit-Trail: cap-proof/

Events:           20
Tail Digest:      0xdb0507c678598f504c6adbae471c14fe9fff680ef85c6c9b83421a22d16df214

Event-Kette (letzte 10):
  11. 2025-11-20T15:25:00Z | prepare_started
      Digest: 0xa1b2c3d4...

  12. 2025-11-20T15:25:01Z | csv_loaded
      Details: {"file": "suppliers.csv", "rows": 2}
      Digest: 0xb2c3d4e5...

  13. 2025-11-20T15:25:01Z | csv_loaded
      Details: {"file": "ubos.csv", "rows": 2}
      Digest: 0xc3d4e5f6...

  14. 2025-11-20T15:25:02Z | merkle_root_computed
      Details: {"type": "supplier", "root": "0xdde3f2c..."}
      Digest: 0xd4e5f6a1...

  15. 2025-11-20T15:25:02Z | merkle_root_computed
      Details: {"type": "ubo", "root": "0xf89ea64..."}
      Digest: 0xe5f6a1b2...

  16. 2025-11-20T15:25:03Z | commitments_saved
      Details: {"file": "build/commitments.json"}
      Digest: 0xf6a1b2c3...

  17. 2025-11-20T15:30:00Z | manifest_build_started
      Digest: 0xa2b3c4d5...

  18. 2025-11-20T15:30:01Z | policy_loaded
      Details: {"policy": "lksg.v1", "hash": "0x0afcb40..."}
      Digest: 0xb3c4d5e6...

  19. 2025-11-20T15:30:02Z | manifest_created
      Details: {"file": "build/manifest.json", "hash": "0xd490be9..."}
      Digest: 0xc4d5e6f7...

  20. 2025-11-20T15:35:00Z | proof_created
      Details: {"status": "ok", "constraints_passed": 2}
      Digest: 0xdb0507c6... (Tail)

Hash-Chain verifiziert: ✅
Keine Lücken gefunden: ✅
```

**Was ist ein "Digest"?**
Jedes Event hat einen Hash, der vom vorherigen Event abhängt:
```
Event 1 → Hash A
Event 2 + Hash A → Hash B
Event 3 + Hash B → Hash C
...
Event N + Hash(N-1) → Tail Digest
```

**Warum ist das wichtig?**
Wenn jemand Event 5 löscht, stimmt der Tail Digest nicht mehr! Du kannst **Manipulationen erkennen**.

**Tipps:**
- Wenn "Hash-Chain verifiziert: ❌" → Events wurden manipuliert!
- Wenn Lücken in Timestamps → Verdacht auf gelöschte Events
- Event-Details geben Kontext (z.B. welche Datei wurde geladen)

---

## 🔐 Signatur-Commands

### Signaturen: Wofür?

**Einfach erklärt:** Eine digitale Signatur ist wie eine **handschriftliche Unterschrift**, nur fälschungssicher.

**Warum wichtig?** Du willst beweisen:
1. **Authentizität:** "Dieses Manifest kommt wirklich von Firma XYZ"
2. **Integrität:** "Das Manifest wurde nicht verändert seit der Signatur"

**Technisch:** Ed25519 Public-Key Kryptographie - schnell, sicher, modern.

---

### `sign keygen` - Schlüsselpaar erzeugen

**Wofür brauche ich das?**
Du willst **Manifests signieren** - dafür brauchst du ein Schlüsselpaar (Private Key + Public Key).

**Warum ist das wichtig?**
- **Private Key:** Damit signierst du (wie dein Stempel)
- **Public Key:** Damit verifizieren andere deine Signatur (öffentlich bekannt)

**Wann nutze ich das?**
- Beim Setup: Einmalig Schlüssel erzeugen
- Alle 1-2 Jahre: Schlüssel-Rotation (neues Paar)

**Was macht das?**
Erzeugt ein Ed25519-Schlüsselpaar für digitale Signaturen.

**Command:**
```bash
cargo run --bin cap-agent -- sign keygen \
  --dir <KEYS-DIR>
```

**Beispiel:**
```bash
cargo run --bin cap-agent -- sign keygen \
  --dir keys
```

**Output-Dateien:**
- `keys/company.ed25519` - Private Key (32 bytes, geheim!)
- `keys/company.pub` - Public Key (32 bytes, öffentlich)

**Output im Terminal:**
```
🔑 Generiere Ed25519-Schlüsselpaar...

✅ Schlüssel erfolgreich erstellt:
  Private Key: keys/company.ed25519 (32 bytes)
  Public Key:  keys/company.pub (32 bytes)

⚠️  WICHTIG:
  - Private Key NIEMALS teilen!
  - Private Key sicher verwahren (z.B. Hardware Security Module)
  - Public Key kann öffentlich sein (z.B. auf Website)

Public Key (Hex):
9a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b
```

**⚠️ WICHTIG:**
- **Private Key NIEMALS committen** (nicht in Git!)
- **Private Key NIEMALS per E-Mail schicken**
- Am besten: Hardware Security Module (HSM) oder verschlüsselter USB-Stick

**Tipps:**
- Backup des Private Keys anlegen (verschlüsselt!)
- Public Key auf Website veröffentlichen (damit andere verifizieren können)
- Keys alle 1-2 Jahre rotieren (siehe `keys rotate`)

---

### `sign manifest` - Manifest signieren

**Wofür brauche ich das?**
Du hast ein Manifest und willst **beweisen**, dass es von dir kommt (Authentizität + Integrität).

**Warum ist das wichtig?**
Ohne Signatur kann jeder behaupten, dein Manifest zu haben. Mit Signatur kannst du beweisen: "Ja, das ist wirklich von mir und wurde nicht verändert."

**Wann nutze ich das?**
- Bevor du Manifests an Auditoren schickst
- In Production: Immer signieren!
- Optional für interne Tests

**Was macht das?**
Signiert ein Manifest mit Ed25519 Private Key. Erstellt eine Signature-Datei mit Manifest-Hash, Signatur und Public Key.

**Command:**
```bash
cargo run --bin cap-agent -- sign manifest \
  --manifest-in <MANIFEST-DATEI> \
  --key <PRIVATE-KEY> \
  --out <SIGNATURE-DATEI>
```

**Beispiel:**
```bash
cargo run --bin cap-agent -- sign manifest \
  --manifest-in build/manifest.json \
  --key keys/company.ed25519 \
  --out build/signature.json
```

**Output-Datei (`build/signature.json`):**
```json
{
  "manifest_hash": "0xd490be94abc12345678901234567890123456789012345678901234567890123",
  "signature": "0x4f2a8b3c1d9e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b",
  "signer_pubkey": "0x9a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b",
  "signed_at": "2025-11-20T15:35:00Z"
}
```

**Was bedeuten die Felder?**
- `manifest_hash` - SHA3-256 Hash des Manifests (das wurde signiert)
- `signature` - Ed25519 Signatur (64 bytes als Hex)
- `signer_pubkey` - Public Key des Signers (32 bytes als Hex)
- `signed_at` - Zeitstempel der Signatur (RFC3339)

**Output im Terminal:**
```
🔏 Signiere Manifest...

📄 Lade Manifest: build/manifest.json
  Hash: 0xd490be94abc123...

🔑 Lade Private Key: keys/company.ed25519
  Public Key: 0x9a1b2c3d4e5f6a7b...

✍️  Erstelle Signatur...
✅ Signatur erfolgreich erstellt!
  Signatur gespeichert: build/signature.json
```

**Tipps:**
- Signatur-Datei mit Manifest zusammen verschicken
- Public Key separat veröffentlichen (damit andere verifizieren können)
- Timestamp in Signatur beachten (für zeitliche Nachvollziehbarkeit)

---

### `sign verify` - Signatur verifizieren

**Wofür brauche ich das?**
Du hast ein signiertes Manifest bekommen und willst **prüfen**, ob die Signatur wirklich von der angegebenen Person/Firma kommt.

**Warum ist das wichtig?**
Trust, but verify! Nur weil jemand sagt "Hier, signiertes Manifest", heißt das nicht, dass es stimmt. Verifiziere!

**Wann nutze ich das?**
- Immer wenn du signierte Manifests bekommst
- Vor dem Akzeptieren von Proof-Packages
- Als Auditor: Pflicht!

**Was macht das?**
Verifiziert eine Ed25519-Signatur mit Public Key. Prüft ob Manifest-Hash stimmt und Signatur gültig ist.

**Command:**
```bash
cargo run --bin cap-agent -- sign verify \
  --signature <SIGNATURE-DATEI> \
  --key <PUBLIC-KEY>
```

**Beispiel:**
```bash
cargo run --bin cap-agent -- sign verify \
  --signature build/signature.json \
  --key keys/company.pub
```

**Output bei Erfolg:**
```
✅ Signatur gültig!

📄 Manifest:
  Hash: 0xd490be94abc123...

🔑 Signer:
  Public Key: 0x9a1b2c3d4e5f6a7b...
  Signed At:  2025-11-20T15:35:00Z

✅ Signatur-Verifikation erfolgreich!
  Manifest wurde signiert von: 0x9a1b2c3d...
  Manifest wurde nicht verändert seit Signatur
```

**Output bei Fehler:**
```
❌ Signatur ungültig!

🔍 Problem:
  Signatur-Verifikation fehlgeschlagen

  Mögliche Ursachen:
  1. Manifest wurde nach Signatur verändert
  2. Falscher Public Key verwendet
  3. Signatur manipuliert

Status: FAIL
```

**Was wird geprüft?**
1. Manifest-Hash neu berechnen
2. Mit Hash in Signature-Datei vergleichen → Wenn ungleich: Manifest verändert!
3. Ed25519-Signatur mit Public Key verifizieren → Wenn ungültig: Falsche Signatur!

**Tipps:**
- Wenn Verifikation fehlschlägt → Nicht vertrauen!
- Public Key sollte aus vertrauenswürdiger Quelle kommen (z.B. offizielle Website)
- Signed At Timestamp prüfen (zu alt → verdächtig)

---

## 🔑 Schlüssel-Commands (Key Management)

### Key Management: Warum komplex?

**Problem mit einfachen Keys:**
- Woher weiß ich, ob ein Key noch aktiv ist?
- Wie rotiere ich Keys sicher?
- Wie beweise ich Ownership bei Key-Rotation?

**Lösung: Key-Metadaten + Chain of Trust**

---

### `keys keygen` - Schlüssel mit Metadaten erzeugen

**Wofür brauche ich das?**
Du willst nicht nur einen Schlüssel, sondern auch **Metadaten** dazu (Wer? Wann? Wie lange gültig?).

**Warum ist das wichtig?**
Einfache Keys (ohne Metadaten) führen zu Problemen:
- "Ist dieser Key noch gültig?"
- "Wem gehört dieser Key?"
- "Wofür darf dieser Key genutzt werden?"

Mit Metadaten hast du alle Infos in einer Datei!

**Wann nutze ich das?**
- Statt `sign keygen` für Production-Umgebungen
- Wenn du Keys mit Rotation, Archivierung, etc. verwalten willst
- Für Multi-Tenant-Szenarien (mehrere Firmen)

**Was macht das?**
Erzeugt Ed25519-Schlüssel mit vollständigen Metadaten (KID, Valid-Dates, Owner, Usage, etc.).

**Command:**
```bash
cargo run --bin cap-agent -- keys keygen \
  --owner <OWNER-NAME> \
  --out <OUTPUT-DATEI> \
  [--algo ed25519] \
  [--valid-days <TAGE>] \
  [--comment <TEXT>]
```

**Beispiel:**
```bash
cargo run --bin cap-agent -- keys keygen \
  --owner "ACME Corporation" \
  --out keys/acme-key.v1.json \
  --valid-days 730 \
  --comment "Production signing key for LkSG compliance"
```

**Output-Dateien:**
- `keys/acme-key.v1.json` - Key-Metadaten (JSON, öffentlich)
- `keys/acme-key.v1.ed25519` - Private Key (geheim!)
- `keys/acme-key.v1.pub` - Public Key (öffentlich)

**Metadaten-Format (`acme-key.v1.json`):**
```json
{
  "schema": "cap-key.v1",
  "kid": "a010ac65166984697b93b867c36e9c94",
  "owner": "ACME Corporation",
  "created_at": "2025-11-20T10:00:00Z",
  "valid_from": "2025-11-20T10:00:00Z",
  "valid_to": "2027-11-20T10:00:00Z",
  "algorithm": "ed25519",
  "status": "active",
  "usage": ["signing", "registry"],
  "public_key": "LS0tLS1CRUdJTiBQVUJMSUMgS0VZL... (Base64)",
  "fingerprint": "sha256:a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6...",
  "comment": "Production signing key for LkSG compliance"
}
```

**Was bedeuten die Felder?**
- `schema` - Metadaten-Format-Version
- `kid` - Key Identifier (eindeutige ID, 32 hex chars)
- `owner` - Wem gehört der Schlüssel?
- `created_at` - Wann erstellt?
- `valid_from` / `valid_to` - Gültigkeitszeitraum
- `algorithm` - Krypto-Algorithmus (ed25519)
- `status` - `active`, `retired` oder `revoked`
- `usage` - Wofür darf der Key genutzt werden?
- `public_key` - Base64-kodierter Public Key
- `fingerprint` - SHA256-Fingerprint (für Key-Vergleich)
- `comment` - Optionale Beschreibung

**KID (Key Identifier) - Wie wird das berechnet?**
```
KID = blake3(base64(public_key))[0:16]
```
- Blake3 Hash des Base64-kodierten Public Keys
- Erste 16 Bytes → 32 hex characters
- Eindeutig für jeden Public Key (Kollisionsfrei)

**Parameter:**
- `--owner` - Schlüsselinhaber (Pflicht) - z.B. "ACME Corporation"
- `--out` - Output-Pfad (Pflicht, muss auf `.v1.json` enden)
- `--algo` - Algorithmus (Standard: `ed25519`, aktuell einziger)
- `--valid-days` - Gültigkeitsdauer in Tagen (Standard: 730 = 2 Jahre)
- `--comment` - Optionale Beschreibung

**Output im Terminal:**
```
🔑 Generiere Ed25519-Schlüssel mit Metadaten...

✅ Schlüssel erfolgreich erstellt:
  Metadaten:   keys/acme-key.v1.json
  Private Key: keys/acme-key.v1.ed25519
  Public Key:  keys/acme-key.v1.pub

📋 Key-Details:
  KID:         a010ac65166984697b93b867c36e9c94
  Owner:       ACME Corporation
  Gültig von:  2025-11-20T10:00:00Z
  Gültig bis:  2027-11-20T10:00:00Z
  Status:      active
  Usage:       signing, registry

⚠️  Wichtig:
  - Private Key sicher verwahren!
  - Metadaten-Datei kann öffentlich sein
  - KID für Registry-Einträge nutzen
```

**Tipps:**
- KID ist wichtig - damit kannst du Keys in Registry-Einträgen referenzieren
- `valid_days` sinnvoll wählen: 365 Tage = 1 Jahr, 730 = 2 Jahre
- Comment nutzen für Zweck (z.B. "Production", "Test", "Backup")

---

### `keys list` - Schlüssel auflisten

**Wofür brauche ich das?**
Du hast mehrere Keys (aktiv, archiviert, widerrufen) und willst **Überblick**.

**Warum ist das wichtig?**
Key-Management wird schnell unübersichtlich:
- "Welcher Key ist aktuell aktiv?"
- "Welche Keys sind abgelaufen?"
- "Habe ich Keys, die ich widerrufen sollte?"

**Wann nutze ich das?**
- Regelmäßig zur Kontrolle (z.B. monatlich)
- Vor Schlüssel-Rotation
- Bei Sicherheitsaudits

**Was macht das?**
Listet alle Schlüssel im Verzeichnis auf (inkl. Archiv). Filtert nach Status und Owner.

**Command:**
```bash
cargo run --bin cap-agent -- keys list \
  --dir <KEYS-DIR> \
  [--status <STATUS>] \
  [--owner <OWNER>]
```

**Beispiel:**
```bash
# Alle Schlüssel
cargo run --bin cap-agent -- keys list \
  --dir keys

# Nur aktive Schlüssel
cargo run --bin cap-agent -- keys list \
  --dir keys \
  --status active

# Nur Schlüssel von ACME
cargo run --bin cap-agent -- keys list \
  --dir keys \
  --owner "ACME Corporation"

# Aktive Keys von ACME
cargo run --bin cap-agent -- keys list \
  --dir keys \
  --status active \
  --owner "ACME Corporation"
```

**Output:**
```
📋 Schlüssel im Verzeichnis: keys/

KID                              Owner              Status    Valid Until
--------------------------------------------------------------------------------
a010ac65166984697b93b867c36e9c94 ACME Corporation   active    2027-11-20
b123cd45ef678901234567890abcdef0 Partner GmbH       active    2026-05-15
c234de56fg789012345678901bcdef01 Old Key            retired   2024-12-31 (abgelaufen)
d345ef67gh890123456789012cdef012 Revoked Key        revoked   2025-01-15

Gesamt: 4 Schlüssel
  ✅ Active:  2
  📦 Retired: 1
  ❌ Revoked: 1

⚠️  Warnung: 1 Key ist abgelaufen und sollte archiviert werden!
```

**Status-Bedeutungen:**
- `active` ✅ - Aktive Schlüssel, können für Signaturen verwendet werden
- `retired` 📦 - Archivierte Schlüssel, nur für Verifikation (nicht für neue Signaturen!)
- `revoked` ❌ - Widerrufene Schlüssel, NICHT verwenden! (kompromittiert oder ungültig)

**Filter:**
- `--status active` - Nur aktive Keys
- `--status retired` - Nur archivierte Keys
- `--status revoked` - Nur widerrufene Keys
- `--owner "ACME"` - Nur Keys von bestimmtem Owner

**Tipps:**
- Regelmäßig `keys list` ausführen und abgelaufene Keys archivieren
- Nie mehr als 1-2 aktive Keys pro Owner (sonst: welchen nutzen?)
- Retired Keys behalten (für Verifikation alter Signaturen!)

---

### `keys show` - Schlüssel-Details anzeigen

**Wofür brauche ich das?**
Du willst **alle Details** zu einem bestimmten Schlüssel sehen.

**Warum ist das wichtig?**
Manchmal brauchst du mehr als nur die Liste - z.B.:
- "Was ist der Fingerprint dieses Keys?"
- "Wofür darf dieser Key genutzt werden (Usage)?"
- "Wann wurde er erstellt?"

**Wann nutze ich das?**
- Debugging: "Warum funktioniert dieser Key nicht?"
- Audits: "Zeig mir alle Infos zu Key X"
- Vor Key-Nutzung: "Ist dieser Key wirklich für 'signing' freigegeben?"

**Was macht das?**
Zeigt vollständige Metadaten eines Schlüssels formatiert an.

**Command:**
```bash
cargo run --bin cap-agent -- keys show \
  --dir <KEYS-DIR> \
  --kid <KEY-ID>
```

**Beispiel:**
```bash
cargo run --bin cap-agent -- keys show \
  --dir keys \
  --kid a010ac65166984697b93b867c36e9c94
```

**Output:**
```
🔑 Schlüssel-Details:

KID:        a010ac65166984697b93b867c36e9c94
Owner:      ACME Corporation
Status:     active ✅
Algorithm:  ed25519

Gültigkeit:
  Created:    2025-11-20T10:00:00Z
  Valid From: 2025-11-20T10:00:00Z
  Valid To:   2027-11-20T10:00:00Z
  Days Left:  730 Tage

Usage:      signing, registry
Comment:    Production signing key for LkSG compliance

Public Key (Base64):
LS0tLS1CRUdJTiBQVUJMSUMgS0VZLS0tLS0KTUZrd0V3WUhLb1pJemowQ0FRWUlLb1pJemowREFRY0RRZ0FFVHh...

Fingerprint:
sha256:a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2

Dateien:
  Metadaten:   keys/acme-key.v1.json
  Private Key: keys/acme-key.v1.ed25519
  Public Key:  keys/acme-key.v1.pub
```

**Was bedeutet "Usage"?**
- `signing` - Darf für Manifest-Signaturen verwendet werden
- `registry` - Darf für Registry-Einträge verwendet werden
- `attestation` - Darf für Key-Attestierungen verwendet werden

**Tipps:**
- Fingerprint nutzen um Keys zu vergleichen (eindeutig!)
- "Days Left" beachten - bei < 30 Tagen: Rotation planen
- Public Key Base64 kann direkt in anderen Systemen genutzt werden

---

### `keys rotate` - Schlüssel-Rotation

**Wofür brauche ich das?**
Dein aktueller Key wird bald ablaufen (oder ist kompromittiert) - du brauchst einen **neuen Key**, willst aber den alten für Verifikation behalten.

**Warum ist das wichtig?**
Schlüssel sollten **regelmäßig rotiert werden** (alle 1-2 Jahre) - aus Sicherheitsgründen. Aber: Alte Signaturen müssen weiterhin verifizierbar bleiben!

**Lösung:** Alter Key → `retired` (nur Verifikation), Neuer Key → `active` (Signaturen)

**Wann nutze ich das?**
- Alle 1-2 Jahre (planmäßige Rotation)
- Bei Verdacht auf Kompromittierung
- Bei organisatorischen Änderungen (z.B. neue Firma)

**Was macht das?**
Rotiert Schlüssel: Alter Key → retired + archiviert, Neuer Key → active.

**Command:**
```bash
cargo run --bin cap-agent -- keys rotate \
  --dir <KEYS-DIR> \
  --current <CURRENT-KEY> \
  --new <NEW-KEY>
```

**Beispiel:**
```bash
# Zuerst neuen Key erstellen
cargo run --bin cap-agent -- keys keygen \
  --owner "ACME Corporation" \
  --out keys/acme-key-2025.v1.json \
  --valid-days 730

# Dann rotieren
cargo run --bin cap-agent -- keys rotate \
  --dir keys \
  --current keys/acme-key-2023.v1.json \
  --new keys/acme-key-2025.v1.json
```

**Was passiert:**

1. **Alter Schlüssel:**
   - Status → `retired` (in Metadaten)
   - Verschoben nach `keys/archive/acme-key-2023.v1.json`
   - Private Key bleibt (für Notfall-Verifikation)
   - Kann NICHT mehr für neue Signaturen verwendet werden
   - Kann weiterhin alte Signaturen verifizieren

2. **Neuer Schlüssel:**
   - Status bleibt `active`
   - Wird ab jetzt für neue Signaturen verwendet

3. **Audit-Log:**
   - Event "key_rotated" wird geloggt
   - KID des alten + neuen Keys dokumentiert

**Output:**
```
🔄 Starte Schlüssel-Rotation...

📋 Alter Key:
  KID:    a010ac65166984697b93b867c36e9c94
  Owner:  ACME Corporation
  Status: active → retired

📋 Neuer Key:
  KID:    b123cd45ef678901234567890abcdef0
  Owner:  ACME Corporation
  Status: active

✅ Rotation durchgeführt:
  ✅ Alter Key archiviert: keys/archive/acme-key-2023.v1.json
  ✅ Neuer Key aktiv: keys/acme-key-2025.v1.json
  ✅ Audit-Event erstellt

💡 Nächster Schritt (empfohlen):
   Erstelle Attestierung: keys attest --signer <alter> --subject <neuer>
```

**Best Practice: Rotation + Attestation**

Nach Rotation solltest du eine **Attestierung** erstellen:
```bash
cargo run --bin cap-agent -- keys attest \
  --signer keys/archive/acme-key-2023.v1.json \
  --subject keys/acme-key-2025.v1.json \
  --out keys/rotation-2023-to-2025.json
```

**Warum Attestierung?**
Beweist: "Ja, dieser neue Key gehört wirklich zu ACME - signiert vom alten Key".

**Tipps:**
- Rotation alle 1-2 Jahre (nicht zu oft, nicht zu selten)
- Alten Key NICHT löschen! (Brauchst du für Verifikation alter Signaturen)
- Attestierung erstellen (für Chain-of-Trust)

---

### `keys attest` - Schlüssel attestieren (Chain of Trust)

**Wofür brauche ich das?**
Nach einer Key-Rotation willst du **beweisen**, dass der neue Key wirklich von dir ist.

**Warum ist das wichtig?**
Stell dir vor:
- Alter Key: KID `a010ac...` (bekannt, vertraut)
- Neuer Key: KID `b123cd...` (unbekannt)

**Problem:** Woher weiß ein Auditor, dass `b123cd...` wirklich zu ACME gehört?

**Lösung:** Alter Key signiert neuen Key = **Attestierung**

**Wann nutze ich das?**
- Immer nach Schlüssel-Rotation
- Bei Multi-Generationen-Keys (Key1 → Key2 → Key3)
- Für Audits (Nachweis der Ownership-Kontinuität)

**Was macht das?**
Erstellt signierte Attestierung: Alter Schlüssel bestätigt neuen Schlüssel.

**Command:**
```bash
cargo run --bin cap-agent -- keys attest \
  --signer <SIGNER-KEY> \
  --subject <SUBJECT-KEY> \
  --out <ATTESTATION-DATEI>
```

**Beispiel:**
```bash
cargo run --bin cap-agent -- keys attest \
  --signer keys/archive/acme-key-2023.v1.json \
  --subject keys/acme-key-2025.v1.json \
  --out keys/attestation-2023-to-2025.json
```

**Output-Datei (`attestation-2023-to-2025.json`):**
```json
{
  "attestation": {
    "schema": "cap-attestation.v1",
    "signer_kid": "a010ac65166984697b93b867c36e9c94",
    "signer_owner": "ACME Corporation",
    "subject_kid": "b123cd45ef678901234567890abcdef0",
    "subject_owner": "ACME Corporation",
    "subject_public_key": "LS0tLS1CRUdJTiBQVUJMSUMgS0VZL... (Base64)",
    "attested_at": "2025-11-20T16:00:00Z"
  },
  "signature": "0x4f2a8b3c1d9e5f6a7b8c... (Ed25519 Signature)",
  "signer_public_key": "LS0tLS1CRUdJTiBQVUJMSUMgS0VZL... (Base64)"
}
```

**Was bedeuten die Felder?**
- `signer_kid` - KID des alten Keys (der attestiert)
- `subject_kid` - KID des neuen Keys (der bestätigt wird)
- `subject_public_key` - Public Key des neuen Keys (Base64)
- `attested_at` - Zeitpunkt der Attestierung
- `signature` - Ed25519-Signatur des alten Keys über die Attestierung

**Use Case:**

```
Auditor erhält Manifest mit Signatur von Key b123cd...
Auditor denkt: "Kenne ich nicht, ist das wirklich ACME?"

Auditor lädt Attestierung herunter:
  "Key a010ac... (alter bekannter ACME-Key) bestätigt: Key b123cd... gehört auch zu ACME"

Auditor prüft Signatur der Attestierung:
  ✅ Signatur gültig mit Public Key von a010ac...

Auditor: "OK, b123cd... ist vertrauenswürdig"
```

**Output im Terminal:**
```
🔏 Erstelle Attestierung...

📋 Signer (Alter Key):
  KID:   a010ac65166984697b93b867c36e9c94
  Owner: ACME Corporation

📋 Subject (Neuer Key):
  KID:   b123cd45ef678901234567890abcdef0
  Owner: ACME Corporation

✅ Attestierung erstellt:
  Datei: keys/attestation-2023-to-2025.json
  Signiert am: 2025-11-20T16:00:00Z

💡 Nächster Schritt:
   Verifizieren: keys verify-chain --dir keys --attestations keys/attestation-2023-to-2025.json
```

**Tipps:**
- Attestierung mit Manifest zusammen verschicken (für Auditoren)
- Bei mehreren Rotationen: Chain of Attestations (Key1 → Key2 → Key3)
- Attestierung ist selbst signiert → fälschungssicher

---

### `keys archive` - Schlüssel archivieren

**Wofür brauche ich das?**
Du hast einen Key, den du **nicht mehr für neue Signaturen nutzen willst**, aber für **Verifikation alter Signaturen behalten** musst.

**Warum ist das wichtig?**
Keys sollten nicht gelöscht werden! Alte Signaturen müssen auch in 5 Jahren noch verifizierbar sein.

**Unterschied zu `rotate`:**
- `rotate` - Automatisch alter → retired + neuer → active
- `archive` - Manuell ein Key → retired (ohne neuen Key)

**Wann nutze ich das?**
- Key ist abgelaufen (Valid To überschritten)
- Key wurde kompromittiert (→ später auf revoked setzen)
- Key wird nicht mehr gebraucht (z.B. Projekt eingestellt)

**Was macht das?**
Markiert Schlüssel als `retired` und verschiebt ins Archiv-Verzeichnis.

**Command:**
```bash
cargo run --bin cap-agent -- keys archive \
  --dir <KEYS-DIR> \
  --kid <KEY-ID>
```

**Beispiel:**
```bash
cargo run --bin cap-agent -- keys archive \
  --dir keys \
  --kid a010ac65166984697b93b867c36e9c94
```

**Was passiert:**

1. **Metadaten:**
   - `status: "active"` → `status: "retired"`
   - `archived_at: "2025-11-20T17:00:00Z"` hinzugefügt

2. **Dateien:**
   - Alle 3 Dateien (`.json`, `.ed25519`, `.pub`) verschoben nach `keys/archive/`
   - Original-Pfad wird geleert

3. **Nutzung:**
   - Kann NICHT mehr für neue Signaturen verwendet werden
   - Kann weiterhin für Verifikation verwendet werden
   - Registry-Einträge bleiben gültig (KID bleibt gleich)

**Output:**
```
📦 Archiviere Schlüssel...

🔑 Key:
  KID:    a010ac65166984697b93b867c36e9c94
  Owner:  ACME Corporation
  Status: active → retired

✅ Archivierung durchgeführt:
  Dateien verschoben nach: keys/archive/
    - acme-key.v1.json
    - acme-key.v1.ed25519
    - acme-key.v1.pub

⚠️  Wichtig:
  - Key kann NICHT mehr für neue Signaturen verwendet werden
  - Key kann weiterhin für Verifikation verwendet werden
  - Alte Registry-Einträge bleiben gültig
```

**Tipps:**
- Archivierte Keys NICHT löschen!
- Backup des archive/-Verzeichnisses anlegen
- Bei Kompromittierung: Erst archivieren, dann auf `revoked` setzen

---

### `keys verify-chain` - Chain-of-Trust verifizieren

**Wofür brauche ich das?**
Du hast mehrere Attestierungen (Key1 → Key2 → Key3) und willst **die gesamte Kette prüfen**.

**Warum ist das wichtig?**
Bei mehreren Rotationen entsteht eine Kette:
```
Key 2023 → Key 2024 → Key 2025
```

Auditor kennt nur Key 2023, aber aktuelles Manifest ist mit Key 2025 signiert.

**Frage:** Ist Key 2025 vertrauenswürdig?

**Antwort:** Ja, wenn die Chain-of-Trust lückenlos ist!

**Wann nutze ich das?**
- Als Auditor: Bei Manifest mit unbekanntem Key
- Als Ersteller: Vor Versand (Qualitätskontrolle der Chain)
- Bei komplexen Multi-Generation-Setups

**Was macht das?**
Verifiziert eine vollständige Attestation-Kette. Prüft jede Attestierung und Chain-Kontinuität.

**Command:**
```bash
cargo run --bin cap-agent -- keys verify-chain \
  --dir <KEYS-DIR> \
  --attestations <ATT1>,<ATT2>,<ATT3>
```

**Beispiel:**
```bash
cargo run --bin cap-agent -- keys verify-chain \
  --dir keys \
  --attestations keys/att-2023-to-2024.json,keys/att-2024-to-2025.json
```

**Verifikationsschritte:**

1. **Jede Attestation einzeln verifizieren:**
   - Lade Signer Public Key
   - Prüfe Ed25519-Signatur
   - Prüfe KID-Übereinstimmung

2. **Chain-Kontinuität prüfen:**
   - Subject(Attestation 1) = Signer(Attestation 2)?
   - Subject(Attestation 2) = Signer(Attestation 3)?
   - Keine Lücken?

3. **Key-Status prüfen:**
   - Alle Signer-Keys im KeyStore?
   - Keine revoked Keys in Chain?

4. **Owner-Konsistenz:**
   - Alle Keys vom gleichen Owner? (sonst: Ownership-Wechsel!)

**Output bei Erfolg:**
```
✅ Chain-of-Trust verifiziert!

Chain: a010ac65... → b123cd45... → c234de56...

Details:
  📋 Attestation 1: keys/att-2023-to-2024.json
    Signer:  a010ac65... (ACME Corporation, retired)
    Subject: b123cd45... (ACME Corporation, active)
    Date:    2024-01-15T10:00:00Z
    ✅ Signatur gültig

  📋 Attestation 2: keys/att-2024-to-2025.json
    Signer:  b123cd45... (ACME Corporation, retired)
    Subject: c234de56... (ACME Corporation, active)
    Date:    2025-11-20T16:00:00Z
    ✅ Signatur gültig

Chain-Kontinuität: ✅
  Subject(Att1) = Signer(Att2) → b123cd45... ✅

Alle Signer verfügbar: ✅
  a010ac65... → keys/archive/acme-key-2023.v1.json
  b123cd45... → keys/archive/acme-key-2024.v1.json

Keine widerrufenen Keys: ✅

Owner-Konsistenz: ✅ (alle ACME Corporation)

🎉 Chain ist vertrauenswürdig!
```

**Output bei Fehler:**
```
❌ Chain-of-Trust-Verifikation fehlgeschlagen!

Chain: a010ac65... → b123cd45... → c234de56...

🔍 Probleme:

  ❌ Attestation 2: Signatur ungültig
     Signer KID: b123cd45...
     Public Key: keys/archive/acme-key-2024.v1.pub
     → Signatur-Verifikation fehlgeschlagen!

  ❌ Chain-Kontinuität verletzt
     Subject(Att1) = b123cd45...
     Signer(Att2)  = c234de56...
     → Lücke in der Kette! (b123cd45 ≠ c234de56)

  ⚠️  Key c234de56... ist revoked
     Status: revoked
     → Chain enthält widerrufenen Key!

Status: FAIL - Chain NICHT vertrauenswürdig
```

**Häufige Fehler:**

- **Chain-Lücke:** Subject(N) ≠ Signer(N+1) → Attestierungen in falscher Reihenfolge oder Attestierung fehlt
- **Signatur ungültig:** Key manipuliert oder falsche Attestierung
- **Revoked Key in Chain:** Kompromittierter Key wurde genutzt
- **Owner-Mismatch:** Ownership-Wechsel ohne Dokumentation

**Tipps:**
- Attestierungen in chronologischer Reihenfolge angeben
- Alle Keys (auch archivierte) im KeyStore behalten
- Bei Chain-Fehler: Attestierungen einzeln prüfen (`sign verify`)

---

## 📚 Registry-Commands

### Registry: Was ist das?

**Einfach erklärt:** Eine Registry ist ein **Verzeichnis aller erstellten Proofs** - wie ein Logbuch.

**Warum wichtig?**
1. **Audit-Trail:** Welche Proofs wurden erstellt? Wann?
2. **Duplikat-Erkennung:** Wurde dieser Proof schon registriert?
3. **Verifikation:** Ist dieser Proof offiziell? (Registry-Check)

**Zwei Backends:**
- **JSON** - Einfach, textbasiert, gut für kleine Mengen (<100 Einträge)
- **SQLite** - Datenbankbasiert, performant für große Mengen (>1000 Einträge)

---

### `registry add` - Proof zur Registry hinzufügen

**Wofür brauche ich das?**
Du hast einen Proof erstellt und willst ihn **offiziell registrieren** - damit er später nachweisbar ist.

**Warum ist das wichtig?**
Ohne Registry-Eintrag:
- Kann jeder behaupten "Ich habe 100 Proofs erstellt"
- Keine Nachweisbarkeit

Mit Registry-Eintrag:
- Beweisbar: "Dieser Proof wurde am 2025-11-20 um 16:30 Uhr registriert"
- Signiert mit KID → Nachvollziehbar, wer registriert hat

**Wann nutze ich das?**
- Nach erfolgreicher `proof verify`
- Vor dem Export (Proof-Package mit Registry-Eintrag ist vollständiger)
- In Production: Immer registrieren!

**Was macht das?**
Fügt einen Proof-Eintrag zur Registry hinzu (mit optionaler Signatur + KID).

**Command:**
```bash
cargo run --bin cap-agent -- registry add \
  --manifest <MANIFEST-DATEI> \
  --proof <PROOF-DATEI> \
  [--timestamp <TSR-DATEI>] \
  [--registry <REGISTRY-DATEI>] \
  [--backend json|sqlite] \
  [--signing-key <PRIVATE-KEY>] \
  [--validate-key] \
  [--keys-dir <KEYS-DIR>]
```

**Beispiel:**
```bash
# Minimal (ohne Signatur, JSON-Backend)
cargo run --bin cap-agent -- registry add \
  --manifest build/manifest.json \
  --proof build/proof.capz \
  --registry build/registry.json

# Mit Signatur und KID (empfohlen!)
cargo run --bin cap-agent -- registry add \
  --manifest build/manifest.json \
  --proof build/proof.capz \
  --signing-key keys/company.ed25519 \
  --registry build/registry.sqlite \
  --backend sqlite

# Mit Key-Validierung (nur active Keys akzeptieren)
cargo run --bin cap-agent -- registry add \
  --manifest build/manifest.json \
  --proof build/proof.capz \
  --signing-key keys/company.ed25519 \
  --validate-key \
  --keys-dir keys
```

**Parameter:**
- `--backend` - `json` (Standard, einfach) oder `sqlite` (performant)
- `--signing-key` - Private Key für Signatur (optional, aber empfohlen!)
- `--validate-key` - Prüft ob Key Status `active` hat (optional, sicher!)
- `--keys-dir` - Verzeichnis mit Key-Metadaten (für `--validate-key`)
- `--timestamp` - Optional: RFC3161 Timestamp

**Registry-Eintrag MIT KID:**
```json
{
  "entries": [
    {
      "id": "proof_001",
      "manifest_hash": "0xd490be94abc123...",
      "proof_hash": "0x83a8779ddef456...",
      "timestamp_file": "build/timestamp.tsr",
      "added_at": "2025-11-20T16:30:00Z",
      "signature": "0x4f2a8b3c1d9e5f6a7b8c9d0e1f2a3b... (Base64)",
      "public_key": "0x9a1b2c3d4e5f6a7b8c9d0e1f2a3b4c... (Base64)",
      "kid": "a010ac65166984697b93b867c36e9c94",
      "signature_scheme": "ed25519"
    }
  ]
}
```

**Was bedeuten die Felder?**
- `id` - Eindeutige Entry-ID (z.B. "proof_001", UUID)
- `manifest_hash` - SHA3-256 Hash des Manifests
- `proof_hash` - SHA3-256 Hash des Proofs
- `added_at` - Registrierungszeitpunkt
- `signature` - Ed25519-Signatur über (Manifest Hash + Proof Hash)
- `kid` - Key Identifier (welcher Key hat signiert?)

**Output:**
```
📝 Füge Proof zur Registry hinzu...

📄 Manifest:
  Hash: 0xd490be94abc123...

🔬 Proof:
  Hash: 0x83a8779ddef456...

🔑 Signiere mit Key:
  KID: a010ac65166984697b93b867c36e9c94
  Status: active ✅

✅ Entry erfolgreich zur Registry hinzugefügt:
  Entry ID: proof_001
  Registry: build/registry.sqlite
  Backend:  sqlite

📊 Registry Stats:
  Gesamt Einträge: 1
```

**Mit Key-Validierung (`--validate-key`):**
```
🔍 Validiere Signing Key...
  KID:    a010ac65166984697b93b867c36e9c94
  Status: active ✅
  Owner:  ACME Corporation
  Valid:  2025-11-20 bis 2027-11-20 ✅

✅ Key-Validierung erfolgreich
```

**Fehler bei inaktivem Key:**
```
❌ Key-Validierung fehlgeschlagen!
  KID:    c234de56fg789012345678901bcdef01
  Status: retired ❌

  Fehler: Key ist nicht aktiv (Status: retired)
  Lösung: Nutze einen aktiven Key oder deaktiviere --validate-key

Entry wurde NICHT zur Registry hinzugefügt.
```

**Tipps:**
- **Immer `--signing-key` nutzen** (sonst: unsigniert = jeder könnte Entry faken)
- **Immer `--validate-key` nutzen** (verhindert versehentliche Nutzung von retired Keys)
- SQLite für Production (besser für viele Einträge)

---

### `registry list` - Registry-Einträge auflisten

**Wofür brauche ich das?**
Du willst **sehen, welche Proofs registriert wurden**.

**Warum ist das wichtig?**
Überblick und Kontrolle:
- "Wie viele Proofs haben wir bisher erstellt?"
- "Wann wurde Proof X registriert?"
- "Welche KIDs wurden genutzt?"

**Wann nutze ich das?**
- Regelmäßige Kontrolle
- Vor Audits ("Zeig mir alle Proofs der letzten 6 Monate")
- Debugging ("Wurde dieser Proof überhaupt registriert?")

**Was macht das?**
Listet alle Registry-Einträge tabellarisch auf.

**Command:**
```bash
cargo run --bin cap-agent -- registry list \
  [--registry <REGISTRY-DATEI>] \
  [--backend json|sqlite]
```

**Beispiel:**
```bash
# JSON Backend
cargo run --bin cap-agent -- registry list \
  --registry build/registry.json

# SQLite Backend
cargo run --bin cap-agent -- registry list \
  --registry build/registry.sqlite \
  --backend sqlite
```

**Output:**
```
📋 Registry-Einträge: build/registry.json

ID          Manifest Hash            Proof Hash               KID           Added At
---------------------------------------------------------------------------------------------
proof_001   0xd490be94abc123...      0x83a8779ddef456...      a010ac65...   2025-11-20 16:30
proof_002   0x32f0a7411827ac...      0xad7fa85ee8a542...      b123cd45...   2025-11-20 17:00
proof_003   0x1da941f7026bae...      0x58dad4f88d9853...      a010ac65...   2025-11-21 09:15

Gesamt: 3 Einträge

📊 Statistiken:
  Keys verwendet:
    a010ac65... (ACME Corporation): 2 Einträge
    b123cd45... (Partner GmbH):     1 Eintrag

  Zeitraum: 2025-11-20 bis 2025-11-21
```

**Tipps:**
- Bei vielen Einträgen: SQLite nutzen (schneller!)
- KID-Spalte prüfen: Alle Einträge vom gleichen Owner?

---

### `registry verify` - Proof gegen Registry verifizieren

**Wofür brauche ich das?**
Du hast einen Proof bekommen und willst **prüfen, ob er offiziell registriert ist**.

**Warum ist das wichtig?**
Jeder kann einen Proof erstellen - aber nur registrierte Proofs sind "offiziell".

**Analogie:** Jeder kann ein Dokument schreiben, aber nur notariell beglaubigte Dokumente sind rechtlich bindend.

**Wann nutze ich das?**
- Als Auditor: Immer! ("Ist dieser Proof echt?")
- Bei verdächtigen Proofs ("Wieso hat dieser Proof keinen Registry-Eintrag?")
- Vor Akzeptanz eines Proofs

**Was macht das?**
Verifiziert, ob ein Proof in der Registry registriert ist. Prüft Hashes und optional Signatur.

**Command:**
```bash
cargo run --bin cap-agent -- registry verify \
  --manifest <MANIFEST-DATEI> \
  --proof <PROOF-DATEI> \
  [--registry <REGISTRY-DATEI>] \
  [--backend json|sqlite]
```

**Beispiel:**
```bash
cargo run --bin cap-agent -- registry verify \
  --manifest build/manifest.json \
  --proof build/proof.capz \
  --registry build/registry.json
```

**Output bei Fund:**
```
🔍 Suche Proof in Registry...

📄 Manifest Hash: 0xd490be94abc123...
🔬 Proof Hash:    0x83a8779ddef456...

✅ Proof in Registry gefunden!

Entry ID:      proof_001
Manifest Hash: 0xd490be94... ✅ stimmt überein
Proof Hash:    0x83a8779d... ✅ stimmt überein
Added At:      2025-11-20T16:30:00Z

Signatur:
  KID:           a010ac65166984697b93b867c36e9c94
  Owner:         ACME Corporation
  Signature:     ✅ Gültig

Status: VERIFIED ✅
```

**Output bei Nicht-Fund:**
```
❌ Proof NICHT in Registry gefunden!

📄 Manifest Hash: 0xd490be94abc123...
🔬 Proof Hash:    0x83a8779ddef456...

🔍 Suche in Registry: build/registry.json
  Durchsuchte Einträge: 3
  Keine Übereinstimmung gefunden

⚠️  Mögliche Ursachen:
  1. Proof wurde nicht registriert
  2. Falsche Registry-Datei
  3. Manifest oder Proof wurde nach Registrierung verändert

Status: NOT FOUND ❌
```

**Tipps:**
- Bei "NOT FOUND" → Proof ablehnen!
- Bei gültigem Entry aber ohne Signatur → Vorsicht (könnte gefälscht sein)
- KID prüfen: Ist der Key vertrauenswürdig?

---

### `registry migrate` - Registry zwischen Backends migrieren

**Wofür brauche ich das?**
Du hast eine JSON-Registry mit 100+ Einträgen - wird langsam. Du willst auf **SQLite migrieren** für bessere Performance.

Oder: Du willst ein Backup als JSON (menschenlesbar).

**Warum ist das wichtig?**
Backend-Wahl hat Konsequenzen:
- JSON: Einfach, lesbar, aber langsam bei >100 Einträgen
- SQLite: Schnell, robust, aber binär (nicht direkt lesbar)

**Wann nutze ich das?**
- JSON → SQLite: Wenn Registry zu groß wird
- SQLite → JSON: Für Backup oder Audit-Export
- Bei Backend-Wechsel (z.B. Migration zu neuer Infrastruktur)

**Was macht das?**
Migriert Registry von einem Backend zum anderen. Kopiert alle Einträge 1:1.

**Command:**
```bash
cargo run --bin cap-agent -- registry migrate \
  --from json|sqlite \
  --input <INPUT-DATEI> \
  --to json|sqlite \
  --output <OUTPUT-DATEI>
```

**Beispiel:**
```bash
# JSON → SQLite (Production-Migration)
cargo run --bin cap-agent -- registry migrate \
  --from json \
  --input build/registry.json \
  --to sqlite \
  --output build/registry.sqlite

# SQLite → JSON (Backup/Audit)
cargo run --bin cap-agent -- registry migrate \
  --from sqlite \
  --input build/registry.sqlite \
  --to json \
  --output build/registry_backup.json
```

**Output:**
```
🔄 Migriere Registry: json → sqlite

Quell-Backend:  json (build/registry.json)
Ziel-Backend:   sqlite (build/registry.sqlite)

🔍 Lade Einträge aus Quelle...
  Gefunden: 123 Einträge

📝 Schreibe Einträge in Ziel-Backend...
  ✅ Entry 1/123 migriert (proof_001)
  ✅ Entry 2/123 migriert (proof_002)
  ...
  ✅ Entry 123/123 migriert (proof_123)

✅ Migration abgeschlossen!
  Anzahl Einträge: 123
  Alle Signaturen erhalten: ✅
  Alle KIDs erhalten: ✅

Neue Registry: build/registry.sqlite
```

**Was wird migriert?**
- Alle Entry-IDs
- Alle Hashes (Manifest + Proof)
- Alle Timestamps
- Alle Signaturen + KIDs
- Alle optionalen Felder (Timestamp-Files, etc.)

**Wichtig:**
- Migration ist **verlustfrei** (1:1 Kopie)
- Original-Registry bleibt unverändert
- Bei Fehler: Migration bricht ab (keine Partial-Migration)

**Tipps:**
- Vor Migration: Backup anlegen!
- Nach Migration: Beide Registries mit `registry list` vergleichen
- SQLite-Datei ist binär → nicht direkt editierbar (gut für Integrität!)

---

## 💾 BLOB-Store-Commands

### BLOB Store: Was ist das?

**Einfach erklärt:** Ein Content-Addressable Storage - Dateien werden nach ihrem **Hash** gespeichert, nicht nach Namen.

**Beispiel:**
- Normale Speicherung: `manifest_v1.json`, `manifest_v2.json`, ...
- Content-Addressable: `0x1da941f7026bae3cf8b1bcdc3a8e01e76ea678c32ec6bc2c374fb67b3744571f`

**Vorteil:**
1. **Automatische Deduplizierung:** Gleicher Inhalt → gleicher Hash → nur 1x gespeichert
2. **Integrität:** Hash ist gleichzeitig Prüfsumme
3. **Unveränderbarkeit:** Hash ändert sich bei jeder Änderung → Manipulation erkennbar

**Warum wichtig?**
Wenn du viele Manifests/Proofs speicherst, willst du keine Duplikate. BLOB Store macht das automatisch.

---

### `blob put` - BLOB einfügen

**Wofür brauche ich das?**
Du willst eine Datei **dedupliziert speichern** - wenn die gleiche Datei schon existiert, wird sie nicht nochmal gespeichert.

**Warum ist das wichtig?**
Speicherplatz sparen:
- Ohne Deduplizierung: 100x das gleiche Manifest = 100x Speicher
- Mit Deduplizierung: 100x das gleiche Manifest = 1x Speicher

**Wann nutze ich das?**
- Für Langzeit-Archivierung von Manifests/Proofs
- In Production mit vielen Registry-Einträgen
- Wenn Storage-Effizienz wichtig ist

**Was macht das?**
Fügt eine Datei in den Content-Addressable BLOB Store ein. Berechnet BLAKE3-Hash, prüft Duplikate, speichert wenn nötig.

**Command:**
```bash
cargo run --bin cap-agent -- blob put \
  --file <DATEI> \
  --type manifest|proof|wasm|abi|other \
  [--registry <REGISTRY-DATEI>] \
  [--link-entry-id <UUID>] \
  [--stdin] \
  [--out <OUTPUT-DATEI>] \
  [--no-dedup]
```

**Beispiel:**
```bash
# Einfaches Einfügen
cargo run --bin cap-agent -- blob put \
  --file build/manifest.json \
  --type manifest

# Mit Registry-Verknüpfung (erhöht refcount)
cargo run --bin cap-agent -- blob put \
  --file build/proof.capz \
  --type proof \
  --registry build/registry.sqlite \
  --link-entry-id 550e8400-e29b-41d4-a716-446655440000

# Von stdin (z.B. für Pipes)
echo "test data" | cargo run --bin cap-agent -- blob put \
  --stdin \
  --type other

# BLOB ID in Datei speichern
cargo run --bin cap-agent -- blob put \
  --file build/manifest.json \
  --type manifest \
  --out blob_id.txt
```

**Output:**
```
📥 Lese Datei: build/manifest.json
📊 Größe: 1234 bytes, Medientyp: manifest

🧮 Berechne BLAKE3-Hash...
  Hash: 0x1da941f7026bae3cf8b1bcdc3a8e01e76ea678c32ec6bc2c374fb67b3744571f

🔍 Prüfe auf Duplikat...
  ✅ Neuer BLOB (noch nicht vorhanden)

💾 Speichere BLOB...
✅ BLOB gespeichert: 0x1da941f7026bae3cf8b1bcdc3a8e01e76ea678c32ec6bc2c374fb67b3744571f

📊 Metadaten:
  Type:     manifest
  Size:     1234 bytes
  Refcount: 1

BLOB ID: 0x1da941f7026bae3cf8b1bcdc3a8e01e76ea678c32ec6bc2c374fb67b3744571f
```

**Output bei Duplikat:**
```
📥 Lese Datei: build/manifest.json
🧮 Berechne BLAKE3-Hash...
  Hash: 0x1da941f7026bae3cf8b1bcdc3a8e01e76ea678c32ec6bc2c374fb67b3744571f

🔍 Prüfe auf Duplikat...
  ℹ️  BLOB bereits vorhanden (Refcount: 1 → 2)

✅ BLOB ID: 0x1da941f7026bae3cf8b1bcdc3a8e01e76ea678c32ec6bc2c374fb67b3744571f
```

**BLOB ID Format:**
- 0x-Präfix (hex)
- 64 hex characters (256 bit BLAKE3 Hash)
- Gesamt: 66 Zeichen

**Parameter:**
- `--type` - Medientyp (manifest, proof, wasm, abi, other) - für Kategorisierung
- `--link-entry-id` - UUID des Registry-Eintrags (erhöht refcount)
- `--stdin` - Liest von stdin statt Datei (für Pipes)
- `--out` - Schreibt BLOB ID in Datei (für Scripting)
- `--no-dedup` - Erzwingt Re-Insert (nur für Tests, ignoriert Deduplizierung)

**Was ist "Refcount"?**
Referenzzähler: Wie oft wird dieser BLOB verwendet?
- Refcount = 0 → Unreferenziert, kann gelöscht werden (GC)
- Refcount > 0 → In Benutzung, NICHT löschen!

**Tipps:**
- BLOB ID speichern (brauchst du für `blob get`)
- Bei `--link-entry-id`: Refcount steigt → BLOB wird nicht von GC gelöscht
- Medientyp korrekt setzen (hilft bei Filterung mit `blob list`)

---

### `blob get` - BLOB abrufen

**Wofür brauche ich das?**
Du hast eine BLOB ID und willst den **Inhalt wiederherstellen**.

**Warum ist das wichtig?**
Content-Addressable Storage speichert nach Hash - ohne `blob get` kannst du die Daten nicht zurückholen.

**Wann nutze ich das?**
- Archiv-Zugriff: "Gib mir Manifest von vor 6 Monaten"
- Disaster Recovery: Registry kaputt, aber BLOB Store noch da
- Export: BLOB → Datei für Weitergabe

**Was macht das?**
Extrahiert BLOB-Inhalt anhand ID. Schreibt in Datei oder stdout.

**Command:**
```bash
cargo run --bin cap-agent -- blob get \
  --id <BLOB-ID> \
  [--out <OUTPUT-DATEI>] \
  [--stdout]
```

**Beispiel:**
```bash
# In Datei speichern
cargo run --bin cap-agent -- blob get \
  --id 0x1da941f7026bae3cf8b1bcdc3a8e01e76ea678c32ec6bc2c374fb67b3744571f \
  --out retrieved.json

# Nach stdout (für Piping)
cargo run --bin cap-agent -- blob get \
  --id 0x1da941f7026bae3cf8b1bcdc3a8e01e76ea678c32ec6bc2c374fb67b3744571f \
  --stdout | jq
```

**Output:**
```
🔍 Suche BLOB: 0x1da941f7026bae3cf8b1bcdc3a8e01e76ea678c32ec6bc2c374fb67b3744571f

✅ BLOB gefunden!
📊 Metadaten:
  Type:     manifest
  Size:     1234 bytes
  Refcount: 2

📄 BLOB geschrieben nach: retrieved.json
```

**Output bei Nicht-Fund:**
```
❌ BLOB nicht gefunden!
  ID: 0x1da941f7026bae3cf8b1bcdc3a8e01e76ea678c32ec6bc2c374fb67b3744571f

🔍 Mögliche Ursachen:
  1. BLOB ID falsch
  2. BLOB wurde gelöscht (GC)
  3. BLOB Store korrupt

Tipp: blob list --unused-only um gelöschte BLOBs zu finden
```

**Tipps:**
- `--stdout` nutzen für Piping (z.B. `blob get --id X --stdout | jq`)
- Bei wichtigen BLOBs: Regelmäßiges Backup (BLOB Store Directory kopieren)

---

### `blob list` - BLOBs auflisten

**Wofür brauche ich das?**
Du willst **sehen, welche BLOBs gespeichert sind** - mit Filtern und Sortierung.

**Warum ist das wichtig?**
Storage-Management:
- "Wie viel Speicher nutzen wir?"
- "Welche BLOBs sind unreferenziert? (können gelöscht werden)"
- "Welche BLOBs sind am größten?"

**Wann nutze ich das?**
- Vor Garbage Collection (unreferenzierte BLOBs finden)
- Storage-Analyse ("Was frisst den meisten Platz?")
- Debugging ("Ist BLOB X überhaupt gespeichert?")

**Was macht das?**
Listet BLOBs mit Filtern (Typ, Größe, Refcount) und Sortierung.

**Command:**
```bash
cargo run --bin cap-agent -- blob list \
  [--type manifest|proof|wasm|abi|other] \
  [--min-size <BYTES>] \
  [--max-size <BYTES>] \
  [--unused-only] \
  [--limit <ANZAHL>] \
  [--order size|refcount|blob_id]
```

**Beispiel:**
```bash
# Alle BLOBs
cargo run --bin cap-agent -- blob list

# Nur unreferenzierte BLOBs (Refcount = 0)
cargo run --bin cap-agent -- blob list \
  --unused-only

# Größte BLOBs zuerst (top 10)
cargo run --bin cap-agent -- blob list \
  --order size \
  --limit 10

# Nur Manifests zwischen 1-10 KB
cargo run --bin cap-agent -- blob list \
  --type manifest \
  --min-size 1024 \
  --max-size 10240

# Am meisten referenzierte BLOBs
cargo run --bin cap-agent -- blob list \
  --order refcount \
  --limit 5
```

**Output:**
```
📋 BLOB Store Übersicht

Gesamt BLOBs: 15
Gesamt Größe: 123456 bytes (120.56 KB)

Gefilterte BLOBs: 3

BLOB ID                                                            Type      Size      Refcount
---------------------------------------------------------------------------------------------------
0x1da941f7026bae3cf8b1bcdc3a8e01e76ea678c32ec6bc2c374fb67b3744571f manifest  1234      2
0x83a8779ddef4567890123456789012345678901234567890123456789012345678 proof     5678      1
0xabc123def456789012345678901234567890123456789012345678901234567890 wasm      9012      0

💾 Speicher-Statistik:
  Verwendet (Refcount > 0): 6912 bytes (2 BLOBs)
  Unreferenziert (Refcount = 0): 9012 bytes (1 BLOB)

⚠️  1 BLOB unreferenziert (kann mit 'blob gc' gelöscht werden)
```

**Filter-Optionen:**
- `--type <TYP>` - Nur bestimmter Medientyp
- `--min-size <N>` - Mindestgröße in Bytes
- `--max-size <N>` - Maximalgröße in Bytes
- `--unused-only` - Nur Refcount = 0
- `--limit <N>` - Maximal N Ergebnisse
- `--order <FELD>` - Sortierung (size, refcount, blob_id)

**Tipps:**
- `--unused-only` zeigt Lösch-Kandidaten für GC
- `--order size --limit 10` findet Speicherfresser
- `--order refcount` zeigt meist-genutzte BLOBs

---

### `blob gc` - Garbage Collection

**Wofür brauche ich das?**
Du willst **unreferenzierte BLOBs löschen** (Refcount = 0) um Speicherplatz freizugeben.

**Warum ist das wichtig?**
Im Laufe der Zeit entstehen unreferenzierte BLOBs:
- Test-Manifests
- Alte Proofs, die durch neue ersetzt wurden
- BLOBs von gelöschten Registry-Einträgen

Ohne GC → Speicher wächst unbegrenzt!

**Wann nutze ich das?**
- Regelmäßig (z.B. monatlich) in Production
- Wenn Speicherplatz knapp wird
- Nach großen Aufräum-Aktionen (z.B. alte Registry-Einträge gelöscht)

**Was macht das?**
Löscht unreferenzierte BLOBs (refcount=0). Optional mit Dry-Run.

**Command:**
```bash
cargo run --bin cap-agent -- blob gc \
  [--dry-run] \
  [--force] \
  [--print-ids]
```

**Beispiel:**
```bash
# Dry-Run (zeigt was gelöscht würde, ohne zu löschen)
cargo run --bin cap-agent -- blob gc \
  --dry-run \
  --print-ids

# Echte Löschung (mit Bestätigung)
cargo run --bin cap-agent -- blob gc

# Echte Löschung ohne Bestätigung
cargo run --bin cap-agent -- blob gc \
  --force

# Mit BLOB IDs ausgeben
cargo run --bin cap-agent -- blob gc \
  --force \
  --print-ids
```

**Output (Dry-Run):**
```
🗑️  Starte Garbage Collection (DRY RUN)...

🔍 Suche unreferenzierte BLOBs...
📊 Gefunden: 3 BLOBs (Refcount = 0)

🗑️  Zu löschende BLOB IDs:
  - 0xabc123def456789012345678901234567890123456789012345678901234567890
  - 0xdef456abc123789012345678901234567890123456789012345678901234567890
  - 0x123456def789abc012345678901234567890123456789012345678901234567890

💾 Freizugebender Speicher:
  Anzahl BLOBs: 3
  Größe:        27036 bytes (26.40 KB)

🔍 DRY RUN - Keine Löschung durchgeführt
💡 Führen Sie den Befehl mit --force aus, um zu löschen
```

**Output (Real GC):**
```
🗑️  Starte Garbage Collection...

🔍 Suche unreferenzierte BLOBs...
📊 Gefunden: 3 BLOBs (Refcount = 0)

⚠️  WARNUNG: 3 BLOBs werden unwiderruflich gelöscht!
   Freizugebender Speicher: 27036 bytes (26.40 KB)

Fortfahren? [y/N]: y

🗑️  Lösche unreferenzierte BLOBs...
  ✅ 0xabc123... gelöscht (9012 bytes)
  ✅ 0xdef456... gelöscht (9012 bytes)
  ✅ 0x123456... gelöscht (9012 bytes)

✅ Garbage Collection abgeschlossen!
  Gelöschte BLOBs: 3
  Freigegebener Speicher: 27036 bytes (26.40 KB)

📊 BLOB Store nach GC:
  Verbleibende BLOBs: 12
  Gesamt Größe: 96420 bytes (94.16 KB)
```

**Flags:**
- `--dry-run` - Simulation, keine Löschung (sicher zum Testen!)
- `--force` - Bestätigung nicht nötig (für Automatisierung)
- `--print-ids` - Gibt gelöschte BLOB IDs aus (für Logging)

**Was passiert mit referenzierten BLOBs?**
Nichts! GC löscht **nur** Refcount = 0. Referenzierte BLOBs bleiben unberührt.

**Wichtig:**
- **Immer erst --dry-run!** (sicherheitshalber prüfen was gelöscht wird)
- GC ist **irreversibel** - gelöschte BLOBs sind weg!
- Backup vor GC anlegen (bei wichtigen Daten)

**Tipps:**
- GC regelmäßig laufen lassen (z.B. cron job)
- `blob list --unused-only` vorher ausführen (Kontrolle)
- Bei Production: Erst Dry-Run, dann bei gutem Gefühl --force

---

## 🌐 WebUI Anleitung (v0.11.0)

### Was ist die WebUI?

**Einfach erklärt:** Eine **grafische Benutzeroberfläche im Browser** (wie eine Website), mit der du Proof-Packages hochladen und verifizieren kannst - **ohne Terminal-Befehle**.

**Wofür brauche ich das?**
- Wenn du **nicht gerne mit dem Terminal arbeitest**
- Wenn du **Demos** für nicht-technische Kollegen machen willst
- Wenn du **schnell mal ein Proof-Package testen** willst ohne lange CLI-Befehle

**Warum ist das wichtig?**
Nicht jeder ist mit der Kommandozeile vertraut. Die WebUI macht den CAP Agent **zugänglich für alle**:
- Management kann Proofs hochladen und Status sehen
- Auditoren können Packages selbst verifizieren
- QA kann ohne Entwickler-Kenntnisse testen

**Wann nutze ich das?**
- **Für Demos und Präsentationen** (sieht professioneller aus als Terminal)
- **Für nicht-technische User** (z.B. Compliance-Team)
- **Für schnelle Tests** (einfacher als CLI)

**Wann nutze ich das NICHT?**
- **Für Produktions-Workflows** (CLI ist stabiler und scriptbar)
- **Für Automatisierung** (CLI lässt sich besser in CI/CD integrieren)
- **Für große Batches** (CLI kann Scripts nutzen, WebUI ist manuell)

### Architektur: Wie funktioniert das?

```
Browser (localhost:5173)                    Server (localhost:8080)
┌────────────────────────┐                 ┌────────────────────────┐
│  WebUI (React/TypeScript) │  ←─HTTP/JSON─→  │  REST API (Rust/Axum)  │
│                        │                 │                        │
│  1. User wählt ZIP     │                 │  1. Empfängt ZIP       │
│  2. Upload-Button      │  ── POST /upload ──→ │  2. Extrahiert Files   │
│  3. Zeigt Manifest     │                 │  3. Parsed Manifest    │
│  4. "Verifizieren"     │  ── POST /verify ──→ │  4. Prüft Proof        │
│  5. Zeigt Ergebnis     │                 │  5. Sendet Result      │
└────────────────────────┘                 └────────────────────────┘
```

**3 Teile:**
1. **WebUI (Frontend)** - React-App im Browser
2. **REST API (Backend)** - Rust-Server, verarbeitet Anfragen
3. **Datenbank/Files (Storage)** - Wo Proofs gespeichert werden

---

### Setup: WebUI starten

**Du brauchst 2 Terminal-Fenster:**

**Terminal 1 - Backend API:**
```bash
cd agent
cargo run --bin cap-verifier-api
```

**Was passiert:**
- Server startet auf `http://127.0.0.1:8080`
- Endpoints werden aktiviert: `/proof/upload`, `/verify`, `/policy/v2/compile`
- Terminal zeigt: `🚀 Starting CAP Verifier API v0.1.0`

**Terminal 2 - WebUI Dev Server:**
```bash
cd webui
npm install  # Nur beim ersten Mal!
npm run dev
```

**Was passiert:**
- Vite Dev Server startet auf `http://localhost:5173`
- WebUI lädt im Browser
- Terminal zeigt: `➜  Local:   http://localhost:5173/`

**Browser öffnen:**
```
http://localhost:5173
```

**Hinweis für macOS:** Wenn du beim ersten Start eine Firewall-Warnung bekommst → "Erlauben" klicken

---

### Workflow: Proof-Package hochladen & verifizieren

#### Schritt 1: Proof-Package erstellen (CLI)

**Warum?** Die WebUI erstellt KEINE Proofs, sie verifiziert nur! Du musst zuerst ein Proof-Package mit dem CLI erstellen.

```bash
cd agent

# 1. Daten vorbereiten
cargo run --bin cap-agent -- prepare \
  --suppliers ../examples/suppliers.csv \
  --ubos ../examples/ubos.csv

# 2. Manifest erstellen
cargo run --bin cap-agent -- manifest build \
  --policy ../examples/policy.lksg.v1.yml

# 3. Proof erstellen
cargo run --bin cap-agent -- proof build

# 4. Package exportieren
cargo run --bin cap-agent -- proof export \
  --manifest build/manifest.json \
  --proof build/proof.capz \
  --output build/package

# 5. ZIP erstellen (für Upload)
cd build/package && zip -r ../proof-package.zip . && cd ../..

# Fertig! Datei liegt in: agent/build/proof-package.zip
```

---

#### Schritt 2: Policy kompilieren (Backend)

**Warum?** Bevor du verifizieren kannst, muss die Policy im Backend-Cache gespeichert sein.

**Terminal 3 (oder separates Fenster):**
```bash
TOKEN="admin-tom"

curl -X POST http://localhost:8080/policy/v2/compile \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "policy": {
      "id": "lksg.demo.v1",
      "version": "1.0.0",
      "legal_basis": [
        {"directive": "LkSG", "article": "§3"}
      ],
      "description": "Demo policy for WebUI testing",
      "inputs": {
        "ubo_count": {"type": "integer"},
        "supplier_count": {"type": "integer"}
      },
      "rules": [
        {
          "id": "rule_ubo_exists",
          "op": "range_min",
          "lhs": {"var": "ubo_count"},
          "rhs": 1
        }
      ]
    },
    "persist": true,
    "lint_mode": "relaxed"
  }'
```

**Erwartete Antwort:**
```json
{
  "policy_id": "lksg.demo.v1",
  "policy_hash": "0x...",
  "stored": true
}
```

**Backend-Logs prüfen:**
```
✅ Policy stored in cache: lksg.demo.v1
```

---

#### Schritt 3: ZIP in WebUI hochladen

**Im Browser (http://localhost:5173):**

1. **Drag & Drop Zone finden:**
   - Großer Bereich mit Text "Drag & drop your proof package here"
   - Oder: "Browse Files" Button klicken

2. **ZIP-Datei auswählen:**
   - Entweder: Datei reinziehen (Drag & Drop)
   - Oder: Datei-Browser öffnen und `agent/build/proof-package.zip` auswählen

3. **Upload startet automatisch:**
   - Progress-Bar erscheint (falls Datei groß)
   - Nach wenigen Sekunden: Manifest wird angezeigt

**Was du siehst (ManifestViewer):**
```
📄 Manifest Details

Company Commitment Root:
0x83a8779d0d7e3a7590133318265569f2651a4f8090afcae880741efcfc898ae5

Policy:
  Name:    LkSG Demo Policy
  Version: lksg.v1
  Hash:    0x0afcb402e74ff6a11601863fc4ae2f2d756124db71bc703f5889ecefcd371ff4

Audit:
  Events: 20
  Created: 2025-11-20T15:30:00Z
```

---

#### Schritt 4: Proof verifizieren

**Im Browser:**

1. **"Proof Verifizieren" Button klicken**
   - Großer grüner Button unter dem Manifest
   - Text: "Verify Proof"

2. **Verification läuft:**
   - Spinner erscheint
   - Backend prüft: Manifest Hash, Policy, Constraints
   - Dauert ca. 1-2 Sekunden

3. **Ergebnis wird angezeigt (VerificationView):**

**Bei Erfolg:**
```
✅ Verification Successful

Status: OK
Manifest Hash: 0x32f0a7411827...
Proof Hash: 0xad7fa85ee8a542...
Signature: ✅ Valid

Details:
  ✅ Manifest Hash verifiziert
  ✅ Policy Hash stimmt überein
  ✅ Alle Constraints erfüllt
```

**Bei Fehler (z.B. Demo-Daten haben keine echten UBOs):**
```
❌ Verification Failed

Status: FAIL
Manifest Hash: 0x32f0a7411827...
Proof Hash: 0xad7fa85ee8a542...
Signature: ❌ Not Present

Details:
  ❌ Constraint "rule_ubo_exists" verletzt
  → UBO count = 0, erwartet >= 1
```

**Wichtig:** "FAIL" ist KEIN Bug! Die Demo-Proof-Packages enthalten absichtlich keine echten Daten. Für echte Verifikation musst du CSV-Dateien mit Daten verwenden.

---

### Troubleshooting WebUI

#### Problem 1: CORS Preflight 401 Error

**Symptom:**
```
Error: Preflight response is not successful. Status code: 401
XMLHttpRequest cannot load http://localhost:8080/proof/upload
```

**Ursache:** Backend lehnt OPTIONS-Preflight-Request ab (Browser sendet diesen automatisch vor jedem POST mit Authorization Header)

**Lösung:** CORS-Layer muss NACH Auth-Middleware angewendet werden

**Check Backend Logs:**
```bash
# Terminal 1 (Backend):
# Sollte NICHT erscheinen:
OPTIONS /proof/upload → 401 Unauthorized
```

**Fix (bereits in v0.11.0 implementiert):**
```rust
// In agent/src/bin/verifier_api.rs
let public_routes = Router::new()
    .route("/healthz", get(handle_healthz));

let protected_routes = Router::new()
    .route("/verify", post(handle_verify))
    .layer(auth_middleware);

// CORS zuletzt anwenden (nicht vor Auth!)
Router::new()
    .merge(public_routes)
    .merge(protected_routes)
    .layer(cors)  // ← Wichtig: Erst hier!
```

**Test:**
```bash
# CORS Preflight testen
curl -X OPTIONS http://localhost:8080/proof/upload \
  -H "Access-Control-Request-Method: POST" \
  -H "Access-Control-Request-Headers: authorization" \
  -v

# Erwartete Response: 200 OK (nicht 401!)
# Mit Headers:
#   Access-Control-Allow-Origin: *
#   Access-Control-Allow-Methods: GET, POST, OPTIONS
```

---

#### Problem 2: 400 Bad Request - Policy not found

**Symptom:**
```json
{
  "error": "Policy not found: lksg.demo.v1. Did you compile and persist it?"
}
```

**Ursache:** Policy wurde noch nicht kompiliert und im Backend-Cache gespeichert

**Lösung:** Policy kompilieren (siehe Schritt 2 oben)

**Check:**
```bash
# Backend-Logs prüfen
# Sollte erscheinen:
✅ Policy stored in cache: lksg.demo.v1
```

**Wenn nicht da:**
```bash
# Policy nochmal kompilieren
TOKEN="admin-tom"
curl -X POST http://localhost:8080/policy/v2/compile \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d @/tmp/policy_v2_request.json
```

---

#### Problem 3: Network Error / Connection Refused

**Symptom:**
```
Error: connect ECONNREFUSED 127.0.0.1:8080
```

**Ursache:** Backend-Server läuft nicht

**Lösung:**
```bash
# Terminal 1 (Backend) prüfen
# Sollte laufen mit:
🚀 Starting CAP Verifier API v0.1.0
🎧 Listening on http://127.0.0.1:8080

# Wenn nicht:
cd agent
cargo run --bin cap-verifier-api
```

**Check:**
```bash
# Ist Port 8080 offen?
lsof -ti:8080

# Wenn leer: Server läuft nicht
# Wenn Prozess-ID: Server läuft

# Manueller Test:
curl http://localhost:8080/healthz
# Sollte: {"status":"OK","version":"0.1.0"}
```

---

#### Problem 4: Token "admin-tom" funktioniert nicht

**Symptom:**
```
401 Unauthorized: Invalid token
```

**Ursache:** Backend wurde mit anderem Token gestartet oder ohne Development-Mode

**Lösung:** Backend mit Standard-Token starten

```bash
# Backend stoppen (Ctrl+C)

# Neu starten (admin-tom ist default in Development)
cd agent
cargo run --bin cap-verifier-api

# WebUI prüfen (src/App.tsx):
const [bearerToken, setBearerToken] = useState('admin-tom');
```

**⚠️ Wichtig für Production:** Token "admin-tom" ist NUR für Development! In Production MUSS dieser Token entfernt und OAuth2 konfiguriert werden.

---

### Performance & Tipps

**Upload-Limits:**
- Max File Size: 100 MB (Standard)
- Timeout: 30 Sekunden
- Concurrent Uploads: 1 (nacheinander hochladen!)

**Browser-Kompatibilität:**
- ✅ Chrome 90+
- ✅ Firefox 88+
- ✅ Safari 14+ (macOS)
- ✅ Edge 90+
- ❌ IE 11 (nicht unterstützt)

**Keyboard Shortcuts:**
- `Esc` - Schließt Dialoge
- `Ctrl+V` / `Cmd+V` - Datei aus Clipboard pasten (falls Browser unterstützt)

**Tipps für beste Performance:**
- WebUI auf localhost nutzen (nicht über Netzwerk)
- Backend und WebUI auf gleicher Maschine laufen lassen
- Bei großen ZIP-Dateien (> 10 MB): Geduld, Upload dauert länger

---

## 📊 Monitoring & Observability (v0.11.0)

### Was ist Monitoring?

**Einfach erklärt:** Monitoring ist wie ein **"Dashboard im Auto"** - es zeigt dir in Echtzeit:
- **Wie schnell läuft das System?** (Requests pro Sekunde)
- **Gibt es Probleme?** (Errors, Timeouts)
- **Sind alle Teile gesund?** (Server up/down)

**Wofür brauche ich das?**
- **Production-Systeme überwachen** (sind alle Services erreichbar?)
- **Probleme früh erkennen** (bevor User sich beschweren)
- **Performance tracken** (wird das System langsamer?)
- **Incidents debuggen** (was ist genau passiert?)

**Warum ist das wichtig?**
Ohne Monitoring bist du **blind**:
- User beschweren sich: "API ist langsam!" - Du: "Wie langsam? Wann? Welcher Endpoint?"
- Server crashed - Du: "Warum? Was war die letzte Action?"
- Audit-Trail gefordert - Du: "Welche Requests gab es in den letzten 30 Tagen?"

Mit Monitoring kannst du **sofort antworten**:
- "API Response Time ist von 200ms auf 800ms gestiegen - seit gestern 15:00 Uhr"
- "5 Requests pro Sekunde sind fehlgeschlagen - Error: Policy not found"
- "Letzte 1000 Requests sind alle in der Grafana-Dashboard sichtbar"

---

### Monitoring Stack Übersicht

**CAP Agent nutzt 4 Tools (die "4 Säulen der Observability"):**

| Tool | Zweck | Was siehst du? |
|------|-------|----------------|
| **Prometheus** | Metrics (Zahlen) | Request Count, Error Rate, Cache Hit Ratio |
| **Grafana** | Visualisierung (Dashboards) | Graphen, Alerts, Trends |
| **Loki** | Logs (Text) | "Request received", "Policy compiled", "Error: ..." |
| **Jaeger** | Traces (Pfade) | Request-Flow: Upload → Parse → Verify → Response |

**Zusätzlich:**
- **Promtail** - Sammelt Logs und schickt sie an Loki
- **Node Exporter** - Sammelt Host-Metriken (CPU, RAM, Disk)
- **cAdvisor** - Sammelt Container-Metriken (Docker)

---

### Setup: Monitoring starten (Docker Compose)

**Voraussetzung:** Docker installiert (`docker --version`)

**Terminal (neues Fenster):**
```bash
cd monitoring
docker compose up -d
```

**Was passiert:**
- 8 Container werden gestartet:
  1. `cap-verifier-api` - Die REST API (mit Metrics-Endpoint)
  2. `prometheus` - Metrics Collection
  3. `grafana` - Dashboards
  4. `loki` - Log Aggregation
  5. `promtail` - Log Collection
  6. `jaeger` - Distributed Tracing
  7. `node-exporter` - Host Metrics
  8. `cadvisor` - Container Metrics

**Check ob alles läuft:**
```bash
docker compose ps

# Sollte zeigen: 8/8 running, 5/5 healthy
```

**Services aufrufen:**
- Grafana: http://localhost:3000 (User: `admin`, Password: `admin`)
- Prometheus: http://localhost:9090
- Jaeger UI: http://localhost:16686
- API: http://localhost:8080

---

### Grafana Dashboard nutzen

**Im Browser:** http://localhost:3000

**Login:**
- Username: `admin`
- Password: `admin`
- Bei erstem Login: Neues Password setzen (oder Skip)

---

#### Dashboard 1: CAP Verifier API - Production Monitoring

**Dashboard öffnen:**
- Menü (☰) links → "Dashboards"
- → "CAP Verifier API - Production Monitoring"

**Was siehst du? (13 Panels)**

**Overview (oben):**
```
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│ Total Requests  │  │ Request Rate    │  │ Error Rate      │  │ Cache Hit Ratio │
│ 1,234           │  │ 15.3 req/s      │  │ 0.8% ⚠️         │  │ 85% ✅          │
└─────────────────┘  └─────────────────┘  └─────────────────┘  └─────────────────┘
```

**Request Metrics (Mitte):**
- **Request Rate by Result** - Graph: Requests über Zeit (ok, warn, fail)
- **Request Distribution** - Pie Chart: Anteil ok vs. fail

**Authentication & Security (unten):**
- **Auth Failures Timeline** - Graph: Auth-Fehler über Zeit
- **Total Auth Failures** - Zähler: Wie viele gescheiterte Logins?

**Cache Performance (rechts):**
- **Cache Hit Ratio (Timeline)** - Graph: Wie gut ist der Cache?
- **Cache Misses** - Zähler: Wie oft Cache-Miss?

---

**Beispiel: Request Rate interpretieren**

```
Request Rate: 15.3 req/s

Graph zeigt:
  - Blaue Linie (ok): 14 req/s → gut!
  - Rote Linie (fail): 1.3 req/s → ⚠️ 8% Error Rate
```

**Was tun?**
1. **Logs prüfen** (siehe unten: Loki/Logs)
2. **Welche Endpoints?** (Query: `rate(cap_verifier_requests_total{result="fail"}[5m]) by (endpoint)`)
3. **Error-Messages** (Loki Query: `{app="cap-verifier-api"} |= "ERROR"`)

---

#### Dashboard 2: SLO Monitoring

**Was sind SLOs?**
- **Service Level Objective** = "Wie gut muss unser Service sein?"
- Beispiel: "99.9% Availability" = "Maximal 43 Minuten Downtime pro Monat"

**Dashboard öffnen:**
- Menü → "Dashboards" → "SLO Monitoring"

**Was siehst du? (17 Panels)**

**SLO Compliance (oben):**
```
┌──────────────────────┐  ┌──────────────────────┐  ┌──────────────────────┐
│ Availability SLO     │  │ Error Rate SLO       │  │ Auth Success SLO     │
│ 99.95% ✅            │  │ 0.05% ✅             │  │ 99.98% ✅            │
│ Target: 99.9%        │  │ Target: < 0.1%       │  │ Target: 99.95%       │
└──────────────────────┘  └──────────────────────┘  └──────────────────────┘
```

**Error Budget Status:**
```
┌──────────────────────────┐
│ Availability Error Budget│
│ Remaining: 85% ✅        │
│ ┌──────────────────────┐ │
│ │████████████████░░░░░░│ │ (85% von 100%)
│ └──────────────────────┘ │
└──────────────────────────┘
```

**Was bedeutet Error Budget?**
- **100% = voller Budget** → Alles super, noch viel Spielraum
- **50% = halber Budget** → ⚠️ Langsam problematisch, Deployment-Freeze erwägen
- **0% = Budget aufgebraucht** → 🔴 Freeze! Keine Deployments mehr, nur Bug-Fixes

**Burn Rate:**
- **Wie schnell verbrauchen wir das Budget?**
- < 1.0x = Gut (wir sind im Plan)
- 5.0x = ⚠️ 5x zu schnell! (Budget ist in 6 Tagen weg statt 30 Tagen)
- 10.0x = 🔴 Incident! (Budget ist in 3 Tagen weg)

---

### Prometheus nutzen (Metrics abfragen)

**Im Browser:** http://localhost:9090

**Was kannst du hier machen?**
- **Metrics durchsuchen** (alle verfügbaren Metriken)
- **Queries schreiben** (PromQL = Prometheus Query Language)
- **Graphen erstellen** (zur Visualisierung)

---

**Beispiel-Queries:**

**1. Request Rate (letzte 5 Minuten):**
```promql
rate(cap_verifier_requests_total[5m])
```

**Was siehst du:**
```
{endpoint="/verify", result="ok"}  14.2
{endpoint="/verify", result="fail"} 1.3
{endpoint="/policy/v2/compile", result="ok"} 0.8
```

**Bedeutung:** 14.2 Requests/Sekunde auf `/verify` waren erfolgreich, 1.3 gescheitert.

---

**2. Error Rate (%):**
```promql
100 * rate(cap_verifier_requests_total{result="fail"}[5m])
  / rate(cap_verifier_requests_total[5m])
```

**Was siehst du:**
```
8.4%
```

**Bedeutung:** 8.4% aller Requests schlagen fehl → ⚠️ Problem!

---

**3. Cache Hit Ratio:**
```promql
cap_cache_hit_ratio
```

**Was siehst du:**
```
0.85
```

**Bedeutung:** 85% Cache Hits → 15% Cache Misses (ok, aber könnte besser sein)

---

**4. Request Latency (95th Percentile):**
```promql
histogram_quantile(0.95,
  sum(rate(cap_verifier_request_duration_seconds_bucket[5m])) by (le)
)
```

**Was siehst du:**
```
0.89
```

**Bedeutung:** 95% aller Requests dauern < 890ms (P95 Latency)

---

### Loki nutzen (Logs durchsuchen)

**Im Browser:** Grafana → "Explore" → "Loki" auswählen

**Was kannst du hier machen?**
- **Logs filtern** (nach Zeit, Level, App)
- **Log-Zeilen durchsuchen** (nach Keyword)
- **Traces korrelieren** (von Log zu Trace springen)

---

**Beispiel-Queries (LogQL):**

**1. Alle Logs der letzten 5 Minuten:**
```logql
{app="cap-verifier-api"}
```

**Was siehst du:**
```
2025-11-20 15:32:10 [INFO] Request received: POST /verify
2025-11-20 15:32:10 [DEBUG] Loading policy: lksg.demo.v1
2025-11-20 15:32:10 [INFO] Verification successful
```

---

**2. Nur ERROR-Logs:**
```logql
{app="cap-verifier-api"} |= "ERROR"
```

**Was siehst du:**
```
2025-11-20 15:35:42 [ERROR] Policy not found: lksg.demo.v1
2025-11-20 15:36:10 [ERROR] Auth token validation failed
```

---

**3. Logs für bestimmten Endpoint:**
```logql
{app="cap-verifier-api"} |= "POST /verify"
```

---

**4. Logs mit Trace ID (für Korrelation):**
```logql
{app="cap-verifier-api"} | json | trace_id!=""
```

**Was siehst du:**
```
2025-11-20 15:32:10 [INFO] Request received
  trace_id: abc123def456
  span_id: xyz789
```

**Dann:** Trace ID kopieren → Jaeger öffnen → Trace suchen → Kompletten Request-Flow sehen!

---

### Jaeger nutzen (Distributed Tracing)

**Im Browser:** http://localhost:16686

**Was sind Traces?**
Ein **Trace** ist der komplette Pfad eines Requests durch das System:

```
Trace: Request abc123def456
  Span 1: POST /verify (800ms total)
    ├─ Span 2: Load Policy (50ms)
    ├─ Span 3: Parse Manifest (100ms)
    ├─ Span 4: Verify Proof (600ms)
    │   ├─ Span 5: Check Constraint 1 (300ms)
    │   └─ Span 6: Check Constraint 2 (300ms)
    └─ Span 7: Build Response (50ms)
```

**Warum ist das nützlich?**
- **Performance-Debugging:** "Warum ist Request X so langsam?" → Span 4 (Verify Proof) dauert 600ms → Constraint 1+2 jeweils 300ms → Optimierungspotential!
- **Error-Debugging:** "Request failed bei Schritt 4" → Span 5 (Check Constraint 1) hat Error geworfen
- **Dependency-Tracking:** Welche Services rufen welche auf?

---

**Jaeger UI nutzen:**

1. **Service auswählen:** `cap-verifier-api`
2. **Operation auswählen:** `POST /verify`
3. **Zeitraum:** Last 1 Hour
4. **"Find Traces" klicken**

**Was siehst du:**
- Liste aller Traces (neueste zuerst)
- Dauer, Anzahl Spans, Status (ok/error)

**Trace öffnen (klicken):**
- Timeline mit allen Spans
- Welcher Span dauerte wie lange?
- Welcher Span hatte Errors?

**Von Trace zu Logs springen:**
- "View Logs" Button (rechts oben)
- → Öffnet Loki mit gefilterten Logs für diese Trace ID

---

### Monitoring Cheat Sheet

**Quick-Check (ist alles gesund?):**
```bash
# 1. Container-Status
docker compose ps
# Sollte: 8/8 running, 5/5 healthy

# 2. API Health
curl http://localhost:8080/healthz
# Sollte: {"status":"OK"}

# 3. Prometheus Targets
curl http://localhost:9090/api/v1/targets | jq '.data.activeTargets[] | {job, health}'
# Sollte: Alle "health": "up"
```

**Häufigste Queries:**

```promql
# Request Rate
rate(cap_verifier_requests_total[5m])

# Error Rate
sum(rate(cap_verifier_requests_total{result="fail"}[5m]))
  / sum(rate(cap_verifier_requests_total[5m]))

# P95 Latency
histogram_quantile(0.95,
  rate(cap_verifier_request_duration_seconds_bucket[5m])
)

# Active Connections
cap_active_connections
```

---

## ⚡ Performance & Metrics (v0.11.0)

### Performance-Übersicht

**Wie schnell ist der CAP Agent? (Production-Ready Metriken)**

**Load Testing Results (Week 5):**
```
Sustained Throughput: 22-27 RPS (Requests per Second)
Total Requests:       10,000 requests over 6 minutes
Success Rate:         100% (keine Timeouts, keine 500 Errors)
```

**Request Latency:**
```
P50 (Median):   380ms  ← 50% aller Requests schneller als 380ms
P95:            890ms  ← 95% aller Requests schneller als 890ms
P99:            1200ms ← 99% aller Requests schneller als 1.2 Sekunden
Max:            1850ms ← Langsamster Request
```

**Was bedeutet das?**
- **Median 380ms** = Typischer Request dauert < 0.5 Sekunden (sehr gut!)
- **P95 890ms** = Nur 5% der Requests dauern länger als 0.9 Sekunden
- **P99 1.2s** = Nur 1% der Requests dauern länger als 1.2 Sekunden (acceptable für Batch-Jobs)

---

### Code Coverage (Week 6)

**Test-Abdeckung: 100% Success Rate**

```
Total Tests:         556 Tests
Status:              556/556 passed ✅ (100% Success Rate, 0 Failures)
Test Breakdown:      385 Library + 164 Binary + 42 Integration Suites + 7 Doc Tests
Security Features:   Path Traversal Prevention, Cycle Detection, TOCTOU Mitigation
```

**Was bedeutet 100% Success Rate?**
- ✅ **Exzellent** - Alle 556 Tests bestehen ohne Fehler
- ✅ Package Flow Refactoring vollständig implementiert
- ✅ cap-bundle.v1 Format mit Security Features (Bundle Type Detection, Hash Validation)

**Coverage nach Modul:**
```
crypto::*               95% ✅  (Hashing, Signing sehr gut getestet)
verifier::core::*       91% ✅  (Verifikationslogik gut getestet)
api::*                  82% ✅  (REST Endpoints gut getestet)
policy::*               79% ✅  (Policy-Logik gut getestet)
registry::*             75% ✅  (Registry CRUD gut getestet)
blob_store::*           72% ⚠️  (GC + Edge-Cases fehlen)
```

---

### Rate Limiting (Production Ready)

**Was ist Rate Limiting?**
Rate Limiting verhindert, dass ein User **zu viele Requests** in kurzer Zeit sendet (Protection vor Abuse + DoS).

**Beispiel:**
```
User X sendet 200 Requests in 1 Minute
→ Rate Limit: 100 Requests/Minute
→ Request 101-200 werden mit "429 Too Many Requests" abgelehnt
```

---

**CAP Agent Rate Limits (Default):**

| Endpoint | Limit | Burst | Bedeutung |
|----------|-------|-------|-----------|
| **Global (Default)** | 100 req/min | 120 | Alle Endpoints zusammen |
| **POST /verify** | 20 req/min | 25 | Proof-Verifikation (moderate) |
| **POST /policy/v2/compile** | 10 req/min | 15 | Policy-Compilation (teuer) |
| **POST /proof/upload** | 20 req/min | 25 | Upload (I/O-intensiv) |

**Was bedeutet "Burst"?**
- **Limit 100** = Durchschnittlich 100 Requests pro Minute erlaubt
- **Burst 120** = Kurzzeitig bis zu 120 Requests möglich (Puffer für Traffic-Spikes)

**Metapher:** Wassertank mit Loch
- Wasser fließt mit konstanter Rate rein (Limit)
- Tank hat Kapazität für Burst (Puffer)
- Wenn Tank voll → Overflow → 429 Error

---

**Rate Limit konfigurieren (beim Start):**

```bash
# Standard (100 req/min, Burst 120)
cargo run --bin cap-verifier-api

# Custom (höhere Limits für Production)
cargo run --bin cap-verifier-api \
  --rate-limit 1000 \
  --rate-limit-burst 1200
```

**Production-Empfehlung:**
- **Kleine Teams (<10 User):** 100 req/min (Standard)
- **Mittlere Teams (10-50 User):** 500 req/min
- **Große Teams (>50 User):** 1000 req/min oder höher

---

**429 Too Many Requests - Was tun?**

**Symptom (Client):**
```bash
curl -X POST http://localhost:8080/verify \
  -H "Authorization: Bearer admin-tom" \
  -d @request.json

# Response:
HTTP/1.1 429 Too Many Requests
Retry-After: 42
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 0
```

**Response Headers:**
- `Retry-After: 42` → Warte 42 Sekunden, dann nochmal probieren
- `X-RateLimit-Remaining: 0` → Limit aufgebraucht, warte bis Reset

**Lösung für User:**
1. **Warte** die angegebene Zeit (`Retry-After`)
2. **Reduziere Request-Rate** (weniger Requests pro Sekunde)
3. **Nutze Batching** (mehrere Operations in einem Request, falls möglich)

**Lösung für Admins:**
1. **Erhöhe Rate Limit** (siehe oben: `--rate-limit`)
2. **Monitoring prüfen** (ist das legitimer Traffic oder Abuse?)
3. **IP-basierte Limits** (nur bestimmte IPs blocken, nicht alle)

---

### Performance-Tipps

**Für beste Performance:**

**1. Hardware:**
- ✅ NVMe SSD (für BLOB Store + SQLite)
- ✅ 4+ CPU Cores (für parallele Requests)
- ✅ 8+ GB RAM (für Cache + OS)

**2. Configuration:**
- ✅ SQLite Backend für Registry (schneller als JSON)
- ✅ WAL Mode für SQLite (Concurrent Writes)
- ✅ InMemory Policy Store für Dev (SQLite für Production)

**3. Caching:**
- ✅ Policy Cache aktiviert (Standard)
- ✅ BLOB Store Deduplication nutzen
- ✅ Client-Side Caching (Cache-Control Headers beachten)

**4. Monitoring:**
- ✅ Prometheus Metrics aktivieren (`/metrics` Endpoint)
- ✅ Grafana Dashboards nutzen (Performance-Trends sehen)
- ✅ Loki Logs prüfen (Error-Patterns finden)

---

**Bottleneck-Analyse:**

**Langsame Requests debuggen:**
```bash
# 1. Prometheus: Welcher Endpoint ist langsam?
rate(cap_verifier_request_duration_seconds_sum[5m])
  / rate(cap_verifier_request_duration_seconds_count[5m])

# 2. Jaeger: Wo im Request-Flow ist das Bottleneck?
# → Trace öffnen → Längste Spans finden

# 3. Loki: Gibt es Errors in den Logs?
{app="cap-verifier-api"} |= "ERROR" | json
```

**Häufigste Bottlenecks:**
1. **Policy Compilation** (10-50ms) → Cache nutzen!
2. **SQLite Writes** (5-20ms) → WAL Mode aktivieren
3. **File I/O** (10-100ms) → NVMe SSD nutzen
4. **Proof Verification** (100-500ms) → Normal für ZK-Proofs

---

## 🛠️ Hilfreiche Tipps

### Build-Verzeichnis aufräumen

**Problem:** Alte Dateien von vorherigen Runs liegen noch rum.

**Lösung:**
```bash
# Alles löschen
cd agent
rm -rf build
mkdir build

# Nur bestimmte Dateien behalten (z.B. Commitments)
rm build/proof.* build/manifest.json
# Jetzt nur noch commitments.json übrig
```

**Wann nötig?**
- Fehler wie "trailing characters" → Alte korrupte Dateien
- Verwirrung: "Welche Version ist das?" → Aufräumen!
- Vor neuem Run: Frischer Start

---

### Alle Server gleichzeitig stoppen

**Problem:** 2-3 Terminal-Fenster mit Servern - mühsam alle mit Ctrl+C zu stoppen.

**Lösung:**
```bash
# macOS/Linux - tötet alle cap-verifier-api Prozesse
pkill -f "cap-verifier-api"

# Tötet Vite (WebUI)
pkill -f "vite"

# Oder: Port direkt freigeben
lsof -ti:8080 | xargs kill -9  # API Server
lsof -ti:5173 | xargs kill -9  # WebUI
```

**Alternativ:** Ctrl+C in jedem Terminal (sauberer, aber aufwändiger)

---

### Logs anschauen

**Audit-Log live verfolgen:**
```bash
# Letzten 20 Events
tail -20 build/agent.audit.jsonl

# Live-Verfolgung (wie tail -f)
tail -f build/agent.audit.jsonl

# Mit jq formatiert
tail -10 build/agent.audit.jsonl | jq
```

**Registry-Logs (bei SQLite):**
```bash
# SQLite direkt abfragen
sqlite3 build/registry.sqlite "SELECT * FROM entries ORDER BY added_at DESC LIMIT 10"
```

---

### Dateien prüfen

**Commitments:**
```bash
cat build/commitments.json | jq
# Oder: cap-agent inspect build/commitments.json
```

**Manifest:**
```bash
cat build/manifest.json | jq

# Schnell-Check: Nur wichtige Felder
cat build/manifest.json | jq '{policy: .policy, created_at, supplier_count: .supplier_root | length}'
```

**Proof (JSON):**
```bash
cat build/proof.json | jq

# Nur Status
cat build/proof.json | jq '.status'
```

**Proof (CAPZ - Base64):**
```bash
# Dekodieren (für Debugging)
cat build/proof.capz | base64 -d > proof.bin
hexdump -C proof.bin | head -20
```

---

### Schneller Neustart (alles neu)

**Problem:** Du willst komplett von vorne starten.

**Lösung (3 Terminals):**

**Terminal 1 - API Server:**
```bash
cd agent
rm -rf build && mkdir build
cargo run --bin cap-verifier-api
```

**Terminal 2 - WebUI:**
```bash
cd webui
npm run dev
```

**Terminal 3 - Proof erstellen (komplett):**
```bash
cd agent && \
cargo run --bin cap-agent -- prepare --suppliers ../examples/suppliers.csv --ubos ../examples/ubos.csv && \
cargo run --bin cap-agent -- manifest build --policy ../examples/policy.lksg.v1.yml && \
cargo run --bin cap-agent -- proof build && \
cargo run --bin cap-agent -- proof verify && \
cargo run --bin cap-agent -- proof export --manifest build/manifest.json --proof build/proof.capz --output build/package && \
cd build/package && zip -r ../proof-package.zip . && cd ../.. && \
echo "✅ Fertig! Package: agent/build/proof-package.zip"
```

**Dann:** Browser → http://localhost:5173 → Package hochladen

---

### Fehlersuche

**Problem: Command nicht gefunden**
```bash
# Stelle sicher, dass du im richtigen Verzeichnis bist
pwd
# Sollte sein: /Users/tomwesselmann/Desktop/LsKG-Agent/agent

cd /Users/tomwesselmann/Desktop/LsKG-Agent/agent
```

**Problem: Datei existiert nicht**
```bash
# Prüfe, ob vorherige Schritte erfolgreich waren
ls -la build/

# Erwartete Dateien:
# - commitments.json (nach prepare)
# - manifest.json (nach manifest build)
# - proof.capz (nach proof build)
```

**Problem: Port schon belegt**
```bash
# Finde Prozess auf Port 8080
lsof -ti:8080

# Beende Prozess
lsof -ti:8080 | xargs kill -9

# Oder alle cap-verifier-api Prozesse
pkill -f cap-verifier-api
```

**Problem: "Trailing characters" Fehler**
```bash
# Build-Verzeichnis ist korrupt
rm -rf build
mkdir build

# Dann nochmal von vorne (prepare → ...)
```

**Problem: CSV-Parsing-Fehler**
```bash
# CSV-Datei prüfen
cat ../examples/suppliers.csv

# Auf Encoding-Probleme prüfen
file ../examples/suppliers.csv
# Sollte: "ASCII text" oder "UTF-8 Unicode text"

# Auf versteckte Zeichen prüfen
od -c ../examples/suppliers.csv | head -20
```

---

### Version anzeigen

```bash
cargo run --bin cap-agent -- --version
# Oder in der Binary:
./target/release/cap-agent --version
```

---

## 📞 Support

**Hilfe benötigt?**

1. **Prüfe Dateien:**
   ```bash
   ls -la build/
   # Sind alle erwarteten Dateien da?
   ```

2. **Schaue in die Logs:**
   ```bash
   tail build/agent.audit.jsonl
   # Was war der letzte Event?
   ```

3. **Starte alle Server neu:**
   ```bash
   pkill -f cap-verifier-api
   pkill -f vite
   # Dann neu starten
   ```

4. **Checke die Dokumentation:**
   - QUICK_START.md - Für schnellen Einstieg
   - CLAUDE.md - Für technische Details
   - test-coverage-report.md - Für Test-Infos

5. **Security-Probleme:**
   - [Security Troubleshooting](./06-troubleshooting.md#-enterprise-security-troubleshooting-neu---dezember-2025) - Sicherheitsprobleme lösen
   - [Security Audit Report](../../security/SECURITY_AUDIT_REPORT.md) - Bekannte Security Issues

**GitHub Issues:** https://github.com/anthropics/claude-code/issues

---

## 🔐 Enterprise Security Hinweise

Wenn Sie CAP Agent in einer Unternehmensumgebung einsetzen, beachten Sie:

### Vor Production-Deployment

```
□ CORS auf explizite Origins konfigurieren (nicht Allow-All)
□ Security Headers aktivieren (HSTS, CSP, X-Frame-Options)
□ Dev-Token "admin-tom" deaktivieren
□ TLS/mTLS für API-Server aktivieren
□ OAuth2 mit echtem Identity Provider konfigurieren
□ Rate Limiting überprüfen
□ Logging und Monitoring einrichten
```

### Sichere Desktop App Nutzung

Die Desktop App ist für lokale Nutzung sicher:
- Alle Daten bleiben auf Ihrem Rechner
- Keine Netzwerkverbindung erforderlich
- Audit-Trail mit Hash-Chain für Integrität

### API-Server Security Checkliste

Für Server-Deployment in Production:
1. TLS aktivieren (`--tls` Flag)
2. CORS-Origins explizit setzen
3. Security Headers konfigurieren
4. Rate Limiting aktivieren
5. Monitoring einrichten (Prometheus/Grafana)

**Detaillierte Anleitung:** [Enterprise Deployment](./05-deployment.md#-enterprise-security-requirements-neu---dezember-2025)

---

**🔐 Enterprise Security Status:**
- Aktuell: 57% Enterprise Readiness
- Ziel: 95% nach 14 Wochen Hardening
- [Vollständiger Security Audit Report](../../security/SECURITY_AUDIT_REPORT.md)
- [Enterprise Hardening Roadmap](../../ROADMAP_ENTERPRISE.md)

---

*Dokument-Version: 2.1 (aktualisiert mit Enterprise Security)*
*Letzte Aktualisierung: 4. Dezember 2025*
*Projekt: LsKG-Agent v0.12.0*
*Autor: Claude Code*
*Für: Laien und Fortgeschrittene*
