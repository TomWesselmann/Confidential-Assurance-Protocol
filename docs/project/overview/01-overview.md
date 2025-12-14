# 01 - Systemüberblick

## 📖 Über diese Dokumentation

Diese Dokumentation besteht aus 7 zusammenhängenden Kapiteln, die Sie Schritt für Schritt durch das LsKG-Agent System führen:

1. **01-overview.md (DIESE DATEI)** - Was ist das System? Für wen? Warum?
2. **02-architecture.md** - Wie ist das System aufgebaut? (mit Hausplan-Analogie)
3. **03-components.md** - Welche Teile gibt es? (detaillierter Inventar)
4. **04-api-reference.md** - Wie bedient man das System? (Bedienungsanleitung)
5. **05-deployment.md** - Wie installiert man das System? (Installationsanleitung)
6. **06-troubleshooting.md** - Was tun bei Problemen? (Fehlerbehebung)
7. **07-status-und-roadmap.md** - Was ist fertig? Was kommt noch? (Status & Zukunft) ⭐ **NEU**

**Warum diese Struktur?** Stellen Sie sich vor, Sie kaufen ein komplexes Gerät:
- Zuerst lesen Sie den **Überblick** (Was kann es?)
- Dann den **Aufbau** (Wie funktioniert es?)
- Danach die **Teile-Liste** (Was ist drin?)
- Anschließend die **Bedienungsanleitung** (Wie benutze ich es?)
- Dann die **Installation** (Wie baue ich es auf?)
- Und schließlich die **Fehlerbehebung** (Was tun bei Problemen?)
- **Bonus:** Der **Status-Bericht** (Was funktioniert schon? Was kommt noch?)

---

## 👔 Für Management (Nicht-Technische Zusammenfassung)

### In einem Satz
Der LsKG-Agent ist eine Software, die Ihr Unternehmen dabei unterstützt, die gesetzlichen Anforderungen des Lieferkettensorgfaltspflichtengesetzes digital, sicher und prüfbar zu erfüllen.

### Das Problem
- Das deutsche **Lieferkettensorgfaltspflichtengesetz (LkSG)** verpflichtet Unternehmen seit Januar 2023, ihre Lieferketten zu überwachen
- Unternehmen müssen nachweisen, dass sie Menschenrechte und Umweltstandards in der Lieferkette beachten
- Herkömmliche Methoden (Excel, E-Mail, Papier) sind fehleranfällig und schwer prüfbar
- Sensible Daten (z.B. Namen von wirtschaftlich Berechtigten) müssen geschützt werden

### Die Lösung
Der LsKG-Agent ist wie ein **digitaler Tresor mit eingebautem Notar**:
- **Import:** Sie geben Ihre Lieferanten-Daten ein (wie bei einer Excel-Liste)
- **Verschlüsselung:** Das System erstellt kryptographische "Fingerabdrücke" (wie Siegel auf Dokumenten)
- **Nachweis:** Es erstellt einen mathematischen Beweis, dass Ihre Regeln erfüllt wurden
- **Prüfung:** Externe Prüfer können die Richtigkeit bestätigen, **ohne Ihre sensiblen Daten zu sehen**

### Business-Wert
✅ **Rechtskonformität** - Erfüllt LkSG-Anforderungen automatisch
✅ **Datenschutz** - Zero-Knowledge: Rohdaten bleiben bei Ihnen
✅ **Prüfsicherheit** - Manipulationssicher durch Blockchain-ähnliche Technik
✅ **Effizienz** - Automatisierung statt manueller Excel-Arbeit
✅ **Audit-Trail** - Jede Aktion ist dokumentiert und nachvollziehbar

### Risiko-Reduzierung
❌ **Ohne LkSG-Agent:**
- Manueller Prozess (fehleranfällig)
- Daten in Excel/E-Mail (unsicher)
- Schwer prüfbar
- Hoher Zeitaufwand
- Rechtliche Risiken bei Nicht-Compliance

✅ **Mit LkSG-Agent:**
- Automatisiert und standardisiert
- Kryptographisch gesichert
- Sofort prüfbar
- Zeitsparend
- Compliance nachweisbar

---

## 🔤 Grundbegriffe-Glossar (für Nicht-IT-Experten)

Bevor wir ins Detail gehen, hier die wichtigsten Begriffe einfach erklärt:

### Technische Begriffe

**API (Application Programming Interface)**
> Wie eine "Steckdose für Software" - erlaubt anderen Programmen (z.B. SAP), mit unserem System zu kommunizieren
> *Analogie:* Wie ein USB-Anschluss, in den verschiedene Geräte passen

**REST API**
> Eine spezielle Art von Steckdose für Software, die über das Internet funktioniert
> *Analogie:* Wie ein Online-Bestellformular, das andere Programme ausfüllen können

**CLI (Command Line Interface)**
> Bedienung über Textbefehle statt Buttons
> *Analogie:* Wie SMS-Befehle statt Touchscreen

**Container (Docker)**
> Eine "Versandkiste" für Software, die überall gleich funktioniert
> *Analogie:* Wie ein Wohncontainer, der fix und fertig geliefert wird

**Kubernetes (K8s)**
> Ein System, das viele Container verwaltet und bei Bedarf mehr bereitstellt
> *Analogie:* Wie ein Logistik-Center, das Container automatisch verteilt

### Kryptographie-Begriffe

**Hash / Hash-Funktion**
> Ein digitaler "Fingerabdruck" für Daten - jede kleinste Änderung verändert den Hash komplett
> *Analogie:* Wie eine Quersumme, die nur für genau diese Daten passt

**BLAKE3 / SHA3-256**
> Namen von speziellen Hash-Verfahren (wie verschiedene Schlosstypen)
> *Wichtig:* Diese sind "Einweg-Funktionen" - man kann aus dem Hash nicht die Original-Daten rekonstruieren

**Ed25519**
> Ein Verfahren für digitale Signaturen
> *Analogie:* Wie eine handschriftliche Unterschrift, nur fälschungssicher

**Merkle-Baum**
> Eine Struktur, um viele Daten effizient zu "versiegeln"
> *Analogie:* Wie ein Baumdiagramm, wo jedes Blatt ein Dokument ist und der Stamm das Gesamt-Siegel

**Zero-Knowledge Proof**
> Ein Beweis, dass etwas stimmt, OHNE die zugrunde liegenden Daten zu zeigen
> *Analogie:* Wie ein Altersnachweis, der nur "über 18" zeigt, nicht das Geburtsdatum
> **📌 Minimal Local Agent:** Aktuell nur Mock-Proofs verfügbar (keine echten ZK-Proofs)

### Compliance-Begriffe

**LkSG (Lieferkettensorgfaltspflichtengesetz)**
> Deutsches Gesetz seit 2023 - verpflichtet Unternehmen zur Überwachung der Lieferkette

**UBO (Ultimate Beneficial Owner)**
> Die "wahren" Eigentümer eines Unternehmens (wirtschaftlich Berechtigte)
> *Wichtig:* Sensitive persönliche Daten, die geschützt werden müssen

**Compliance**
> Einhaltung von Gesetzen und Regeln

**Audit Trail**
> Lückenlose Dokumentation aller Vorgänge
> *Analogie:* Wie ein Fahrtenbuch, das man nicht nachträglich ändern kann

### Datenbank-Begriffe

**SQLite**
> Eine kleine, eingebettete Datenbank (wie eine digitale Karteikarte)
> *Vorteil:* Keine separate Datenbank-Software nötig

**Registry**
> Ein Verzeichnis/Index aller erstellten Nachweise
> *Analogie:* Wie ein Aktenregister

**JSON**
> Ein Format zum Speichern strukturierter Daten (wie XML oder CSV)
> *Analogie:* Wie eine standardisierte Formular-Vorlage

### Sicherheits-Begriffe

**TLS/mTLS**
> Verschlüsselung für Datenübertragung (wie HTTPS bei Webseiten)
> *mTLS:* Beide Seiten prüfen sich gegenseitig (höhere Sicherheit)

**OAuth2**
> Ein Standard-Verfahren für Zugriffskontrolle
> *Analogie:* Wie ein Ausweis-System, das zeitlich begrenzte Zugangsberechtigungen vergibt

**JWT (JSON Web Token)**
> Ein digitaler "Ausweis" mit Ablaufdatum
> *Analogie:* Wie ein Tages-Pass für ein Museum

---

## LsKG-Agent (CAP v0.12.2 – Minimal Local Agent)

### Was ist der LsKG-Agent?

Der **LsKG-Agent** ist ein produktionsreifer, kryptographischer Compliance-Proof-System für das deutsche **Lieferkettensorgfaltspflichtengesetz (LkSG)**.

**In einfachen Worten:** Eine Software, die Compliance-Nachweise erstellt und überprüft, ohne sensible Daten preiszugeben.

> **📌 Minimal Local Agent (v0.12.2)**
> Diese Version konzentriert sich auf lokale, dateibasierte Schlüsselverwaltung mit Ed25519.
>
> **Verfügbar:**
> - ✅ Software Provider (dateibasierte Ed25519-Schlüssel)
> - ✅ Mock-Proof Backend
> - ✅ Policy Store (InMemory + SQLite)
> - ✅ REST API mit OAuth2 (Verifier API)
> - ✅ Desktop App (Tauri 2.0) mit integrierter WebUI
>
> **Entfernte Enterprise-Features:**
> - ❌ PKCS#11/HSM-Integration
> - ❌ Google CloudKMS
> - ❌ ZK-Backends (zkvm, halo2, spartan)
> - ❌ Sanktionslisten-Integration
> - ❌ Standalone WebUI (ersetzt durch Tauri Desktop App)

### Zweck des Systems (vereinfacht)

**Das Problem:** Unternehmen müssen ihre Lieferketten überwachen (LkSG-Gesetz), aber:
- Die Daten sind sensibel (Namen, Adressen, Eigentümer-Informationen)
- Prüfer müssen die Richtigkeit bestätigen können
- Es darf nichts nachträglich geändert werden können

**Die Lösung des LsKG-Agent:**

1. **Sichere Datenverarbeitung**
   - *Was es macht:* Erstellt "Fingerabdrücke" (Hashes) von Ihren Daten
   - *Analogie:* Wie ein Siegel auf einem Brief - man sieht, dass er nicht geöffnet wurde, ohne den Inhalt zu kennen
   - *Vorteil:* Ihre Rohdaten bleiben bei Ihnen

2. **Überprüfbare Nachweise**
   - *Was es macht:* Erstellt mathematische Beweise, dass Regeln erfüllt wurden
   - *Analogie:* Wie ein TÜV-Siegel - bestätigt "geprüft", ohne Details preiszugeben
   - *Vorteil:* Prüfer sehen nur "erfüllt" oder "nicht erfüllt", nicht Ihre Daten

3. **Manipulationssichere Dokumentation (Audit-Trail)**
   - *Was es macht:* Protokolliert jede Aktion in einer unveränderlichen Kette
   - *Analogie:* Wie ein Fahrtenbuch, wo man Seiten nicht austauschen kann
   - *Vorteil:* Nachträgliche Änderungen sind unmöglich und werden sofort erkannt

4. **Automatisierung**
   - *Was es macht:* Prüft automatisch, ob Ihre Regeln (Policies) erfüllt sind
   - *Analogie:* Wie ein Rechtschreib-Checker für Compliance
   - *Vorteil:* Spart Zeit und reduziert menschliche Fehler

### Zielgruppe

#### Primäre Nutzer
- **Compliance-Beauftragte** - Erstellen und verwalten Compliance-Nachweise
- **Wirtschaftsprüfer** - Verifizieren eingereichte Nachweise
- **IT-Administratoren** - Betreiben und warten das System
- **Entwickler** - Integrieren das System in bestehende Enterprise-Systeme (z.B. SAP)

#### Technisches Level
Das System bietet verschiedene Schnittstellen für unterschiedliche Nutzergruppen:
- **CLI (Command Line Interface)** - Für technische Nutzer und Automatisierung
- **REST API** - Für Systemintegration und Entwickler (Verifier API)
- **Desktop App** - Native App für Windows/macOS/Linux mit Offline-Funktionalität (Proofer + Verifier)

### Hauptfunktionen (mit Alltagsvergleich)

#### 1. Proof Generation (Nachweis-Erstellung)
**Was passiert:** Ihre Daten → Verschlüsselte Fingerabdrücke → Regelprüfung → Nachweis

**Schritt für Schritt:**
1. Sie laden Ihre Lieferanten-Liste hoch (wie eine Excel-Datei)
2. Das System erstellt "Siegel" für jede Zeile (BLAKE3-Hash)
3. Es prüft, ob Ihre Regeln erfüllt sind (z.B. "max. 100 Lieferanten")
4. Es erstellt einen Nachweis, der später überprüfbar ist

*Analogie:* Wie bei einer Notariatssitzung - Dokumente werden geprüft, gesiegelt und dokumentiert.

#### 2. Proof Verification (Nachweis-Prüfung)
**Was passiert:** Nachweis-Paket → Siegel prüfen → Regeln prüfen → Bericht

**Schritt für Schritt:**
1. Ein Prüfer lädt das Nachweis-Paket (ein Ordner mit Dateien)
2. Das System prüft, ob die Siegel echt sind
3. Es prüft, ob die Regeln erfüllt wurden
4. Es erstellt einen Prüfbericht ("bestanden" / "nicht bestanden")

*Analogie:* Wie bei der TÜV-Prüfung - Dokumente werden geprüft, ohne das Auto auseinanderzubauen.

#### 3. Registry Management (Nachweisregister)
**Was es macht:** Speichert eine Liste aller erstellten Nachweise (wie ein Aktenregister)

**Optionen:**
- **JSON-Datei** (einfach, für wenige Nachweise)
- **SQLite-Datenbank** (schnell, für viele Nachweise)

*Analogie:* Wie ein Ordnersystem - entweder ein einfacher Aktenordner (JSON) oder ein computerisiertes Archiv (SQLite).

#### 4. Key Management (Schlüsselverwaltung)
**Was es macht:** Verwaltet digitale Unterschriften

**Funktionen:**
- Erstellen neuer "Unterschriften-Schlüssel" (Ed25519)
- Alte Schlüssel in Rente schicken (wie abgelaufene Ausweise)
- Neue Schlüssel von alten bestätigen lassen (Vertrauenskette)
- KID-Ableitung (Key Identifier via BLAKE3)

> **📌 Minimal Local Agent:** Schlüssel werden ausschließlich lokal im Dateisystem gespeichert (`keys/` Verzeichnis). Hardware-Module (HSM/PKCS#11) und Cloud-Dienste (CloudKMS) sind nicht verfügbar.

**Schlüssel-Dateien:**
- `*.ed25519` – Private Key (32 bytes)
- `*.pub` – Public Key (32 bytes)
- `*.v1.json` – Key-Metadaten (cap-key.v1 Schema)

*Analogie:* Wie bei Firmen-Stempeln - alte werden archiviert, neue werden vom Geschäftsführer beglaubigt.

#### 5. Audit Trail (Prüfpfad)
**Was es macht:** Dokumentiert jede Aktion unveränderlich

**Eigenschaften:**
- Jede Aktion bekommt eine Nummer und einen Zeitstempel
- Neue Aktionen bauen auf alten auf (wie Blockchain)
- Änderungen sind unmöglich (würde sofort auffallen)

*Analogie:* Wie ein Fahrtenbuch mit nummerierten Seiten - man kann keine Seite entfernen oder austauschen, ohne dass es auffällt.

#### 6. Policy Management (Regelverwaltung)
**Was passiert:** Compliance-Regeln werden kompiliert, gespeichert und verwaltet

**Schritt für Schritt:**
1. Sie definieren eine Policy (z.B. "max. 100 Lieferanten")
2. Das System kompiliert die Policy und berechnet einen eindeutigen Hash
3. Die Policy wird im Store gespeichert (automatische Deduplizierung)
4. Sie können die Policy über ID oder Hash abrufen
5. Status-Verwaltung ermöglicht Versionierung (Active/Deprecated/Draft)

**Backend-Optionen:**
- **In-Memory** (schnell, für Development)
- **SQLite** (persistent, für Production)

*Analogie:* Wie ein Bibliothekskatalog - Bücher (Policies) bekommen eine eindeutige ISBN (Hash), werden katalogisiert und können über verschiedene Wege gefunden werden. Veraltete Ausgaben bleiben auffindbar, aber markiert.

#### 7. Desktop App (CAP Desktop Proofer) - v0.12.2
**Was es macht:** Native Desktop-Anwendung für kompletten Offline-Betrieb

**Modi:**
1. **Proofer Modus** - 6-Schritte-Workflow zum Erstellen von Compliance-Nachweisen
2. **Verifier Modus** - Bundle-Upload und Offline-Verifikation
3. **Audit Modus** - Timeline-Ansicht aller Aktionen eines Projekts

**6-Schritte Proofer Workflow:**
1. **Import** - CSV-Dateien (Lieferanten, UBOs) importieren
2. **Commitments** - Kryptographische Commitments berechnen
3. **Policy** - Compliance-Regeln auswählen/hochladen
4. **Manifest** - Manifest erstellen (Metadaten + Commitment-Root)
5. **Proof** - Mock-Proof generieren
6. **Export** - Bundle als ZIP exportieren (cap-bundle.v1)

**Technologie:**
- Tauri 2.0 (Rust-Backend + WebView-Frontend)
- React + TypeScript + Zustand (Frontend)
- Komplett offline - keine Netzwerkverbindung nötig

**Installation:**
```bash
# Build für Release
cd /Users/tomwesselmann/Desktop/LsKG-Agent/src-tauri
cargo build --release

# App starten
./target/release/desktop-proofer
```

**Besonderheiten:**
- **Projekt-basiert:** Jedes Projekt ist ein Ordner mit allen Dateien
- **Audit Trail:** V1.0-Format mit SHA3-256 Hash-Chain
- **Sidebar:** Workspace-Browser für Projektverwaltung
- **State Persistence:** Workflow-Fortschritt bleibt erhalten

*Analogie:* Wie eine Steuersoftware (WISO/Elster) - alle Daten bleiben lokal auf Ihrem Rechner, keine Cloud erforderlich. Der Compliance-Nachweis wird offline erstellt und kann dann verteilt werden.

**Status:** ✅ Production-Ready (v0.12.2)

**Vorteile:**
- ✅ Keine Server-Infrastruktur nötig
- ✅ Sensible Daten verlassen nie den Rechner
- ✅ Funktioniert ohne Internet
- ✅ Native Performance
- ✅ Integrierter Audit-Trail
- ✅ Integrierte WebUI (React + TypeScript)

#### 8. Web UI (in Tauri Desktop App integriert)
**Was es macht:** React-basierte Benutzeroberfläche innerhalb der Desktop App

**Funktionen:**
1. **Proofer Workflow** - 6-Schritte-Prozess für Compliance-Nachweise
2. **Verifier Mode** - Bundle-Upload und Offline-Verifikation
3. **Audit Timeline** - Visualisierung des Audit-Trails
4. **Ed25519 Signierung** - Optionale Manifest-Signierung

**Technologie:**
- React 19.2 + TypeScript 5.9 (moderne Web-Technologie)
- Tauri IPC statt HTTP API (sicher, offline-fähig)
- Zustand für State Management
- TailwindCSS 4.x für Styling

**Nutzung:**
```bash
# Desktop App starten (Dev Mode)
cd src-tauri && cargo tauri dev

# Production Build
cd src-tauri && cargo tauri build
```

*Analogie:* Wie eine Desktop-Anwendung (z.B. WISO Steuer) - alles in einer App, keine Server, keine Konfiguration.

**Status:** ✅ Produktionsreif (v0.12.2)

> **📌 Hinweis:** Die standalone HTTP-basierte WebUI wurde in v0.12.0 entfernt. Alle UI-Funktionen sind jetzt in der Tauri Desktop App integriert.

#### 9. Monitoring & Observability - **NEU in Week 2**
**Was es macht:** Überwacht System-Performance und Gesundheit in Echtzeit

**Komponenten:**
1. **Prometheus** - Sammelt Metriken (wie Statistiken)
   - Request Rate (Anfragen pro Sekunde)
   - Error Rate (Fehlerquote)
   - Latency (Antwortzeit)
   - Cache Hit Rate (Trefferquote)

2. **Grafana** - Visualisiert Metriken (wie Dashboards)
   - Main Dashboard (13 Panels)
   - SLO Dashboard (17 Panels)
   - Real-Time Graphs

3. **Loki** - Log-Aggregation (wie digitales Fahrtenbuch)
   - Strukturierte Logs
   - 31 Tage Retention
   - Suchbar und filterbar

4. **Jaeger** - Distributed Tracing (wie GPS-Tracking für Anfragen)
   - Request-Flow-Visualisierung
   - Performance-Bottlenecks identifizieren
   - Korrelation mit Logs und Metriken

**Deployment:**
```bash
cd monitoring
docker compose up -d
./test-monitoring.sh

# URLs:
# Grafana:    http://localhost:3000 (admin/admin)
# Prometheus: http://localhost:9090
# Jaeger:     http://localhost:16686
```

*Analogie:* Wie das Cockpit in einem Flugzeug - zeigt alle wichtigen Metriken auf einen Blick und warnt bei Problemen. Statt blind zu fliegen, sieht man genau, was im System passiert.

**SLO/SLI Monitoring:**
- **Availability SLO:** 99.9% Uptime (43.2 min Ausfallzeit pro Monat erlaubt)
- **Error Rate SLO:** < 0.1% Fehlerquote
- **Auth Success SLO:** 99.95% erfolgreiche Authentifizierungen
- **Cache Hit Rate SLO:** > 70% Cache-Trefferquote

**Alerting:**
- 11 Alert Rules in 3 Severity-Levels (Critical, Warning, Info)
- Automatische Benachrichtigung bei SLO-Verletzungen
- Error Budget Tracking (wie Kontostand für erlaubte Fehler)

**Status:** ✅ Production-Ready - Alle 8 Container running, 5/5 healthy

#### 10. Policy Store System - **NEU in v0.11.0**
**Was es macht:** Persistente Speicherung von kompilierten Policies mit Versionierung

**Funktionen:**
1. **Content Deduplication** - Gleiche Policy → gleicher Hash → nur 1× gespeichert
2. **Status Lifecycle** - Active/Deprecated/Draft (wie Versionsstände)
3. **Dual Backend** - InMemory (Development) oder SQLite (Production)
4. **UUID Identifiers** - Eindeutige IDs für jede Policy-Version

**Backend-Optionen:**
- **InMemory Store** - Thread-Safe, für Development/Testing
- **SQLite Store** - WAL mode, ACID-Garantien, für Production

**API Integration:**
```bash
# Policy kompilieren und speichern
curl -X POST http://localhost:8080/policy/compile \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d @policy_request.json

# Policy abrufen (per UUID oder Hash)
curl http://localhost:8080/policy/0xabc123...
```

**Environment Configuration:**
```bash
# Development (InMemory)
POLICY_STORE_BACKEND=memory cargo run --bin cap-verifier-api

# Production (SQLite)
POLICY_STORE_BACKEND=sqlite \
POLICY_DB_PATH=/data/policies.sqlite \
cargo run --bin cap-verifier-api
```

*Analogie:* Wie ein Git-Repository für Compliance-Regeln - Policies werden versioniert, dedupliziert und haben einen eindeutigen Hash wie Git-Commits.

**Features:**
- ✅ Automatische Deduplizierung via SHA3-256 Hash
- ✅ Thread-Safe Concurrent Access (Arc<Mutex> + WAL mode)
- ✅ Status Management (Active/Deprecated/Draft)
- ✅ 19/19 Tests passed (7 InMemory + 7 SQLite + 5 API Integration)

**Status:** ✅ Production-Ready - Beide Backends erfolgreich getestet

#### 11. cap-bundle.v1 Format - **NEU in v0.11.0**
**Was es macht:** Standardisiertes Paketformat für offline-verifizierbare Compliance-Nachweise

**Problem (vorher):**
- `proof export` erstellte Pakete im alten Format (cap-proof.v1.0)
- `verifier run` erwartete neues Format (cap-bundle.v1)
- **Ergebnis:** Inkompatibilität, Tests schlugen fehl

**Lösung (jetzt):**
- Beide Tools sprechen die gleiche "Sprache" (cap-bundle.v1)
- Strukturierte Metadaten für jede Datei
- SHA3-256 Hashes für Integritätsprüfung
- Alle 42 Tests bestehen ✅

**Bundle-Struktur:**
```
cap-proof/
├─ manifest.json              # Compliance manifest
├─ proof.dat                  # Zero-knowledge proof
├─ _meta.json                 # Bundle metadata (cap-bundle.v1)
├─ timestamp.tsr              # Optional: Timestamp
├─ registry.json              # Optional: Registry
├─ verification.report.json   # Verification report
└─ README.txt                 # Human-readable instructions
```

**Metadaten-Struktur (_meta.json):**
```json
{
  "schema": "cap-bundle.v1",
  "bundle_id": "bundle-1732464123",
  "created_at": "2025-11-24T...",
  "files": {
    "manifest.json": {
      "role": "manifest",           // Was ist die Datei?
      "hash": "0x1da941f7...",      // SHA3-256 Hash
      "size": 1234,                 // Dateigröße in Bytes
      "content_type": "application/json",  // MIME-Type
      "optional": false              // Pflichtdatei?
    }
  },
  "proof_units": [
    {
      "id": "default",
      "manifest_file": "manifest.json",
      "proof_file": "proof.dat",
      "policy_id": "LkSG Demo Policy",    // Automatisch aus Manifest extrahiert
      "policy_hash": "0xabc123...",       // Automatisch aus Manifest extrahiert
      "backend": "mock"
    }
  ]
}
```

**Vorteile:**
1. **Integrität** - Jede Datei hat SHA3-256 Hash → Manipulationen sofort erkennbar
2. **Metadaten** - Wir wissen mehr über jede Datei (Rolle, Typ, Größe)
3. **Standardisierung** - Alle Tools verstehen das gleiche Format
4. **Auditierbarkeit** - Auditoren können jede Datei einzeln prüfen
5. **Policy-Info** - Policy-Name und Hash automatisch im Paket enthalten

**CLI-Kommandos:**
```bash
# Proof-Paket erstellen (cap-bundle.v1 Format)
cargo run -- proof export \
  --manifest build/manifest.json \
  --proof build/proof.dat \
  --out build/proof_package

# Proof-Paket verifizieren
cargo run -- verifier run --package build/proof_package
```

**Migration:**
- **v1.0 (alt):** Einfache String-Dateinamen in `files` → `{"manifest": "manifest.json"}`
- **cap-bundle.v1 (neu):** Strukturierte Objekte → `{"manifest.json": {role, hash, size, ...}}`

*Analogie:* Wie ein Paket mit Lieferschein - vorher stand nur "1x Dokument" drauf, jetzt steht "Dokument X, Gewicht 1.2kg, Prüfsumme ABC123, Rolle: Vertrag".

**Status:** ✅ Production-Ready - Alle 42 Tests bestehen, End-to-End-Workflow funktional

**Technische Details:**
- Implementiert in `src/main.rs` (Zeilen 921-1555)
- Strukturen: `BundleMeta`, `BundleFileMeta`
- Automatisches Laden der Manifest-Datei für Policy-Extraktion
- SHA3-256 Hash-Berechnung für jede Datei
- Test: `test_cli_complete_workflow` (tests/test_cli_e2e_workflow.rs)

### Architektur-Highlights

#### Technologie-Stack
- **Sprache:** Rust 2021 (Memory-safe, performant)
- **Kryptographie:** BLAKE3 (Merkle-Trees), SHA3-256 (Hashes), Ed25519 (Signaturen)
- **Web Framework:** Axum 0.7 (async, modern)
- **Datenbank:** SQLite (eingebettet, ACID-konform)
- **Container:** Docker + Kubernetes ready
- **Auth:** OAuth2 mit JWT RS256

#### Sicherheitsprinzipien
1. **Defense in Depth** - Mehrere Sicherheitsschichten (Crypto, TLS, OAuth2)
2. **Zero-Knowledge** - Rohdaten bleiben privat, nur Commitments werden geteilt
3. **Audit-First** - Jede Aktion wird protokolliert
4. **Fail-Safe** - Sichere Defaults, explizite Opt-ins für unsichere Modi

### Projektstruktur (Überblick)

```
LsKG-Agent/
├── agent/                    # CLI & API Backend (Rust)
│   ├── src/                  # Quellcode (65+ Module)
│   ├── tests/                # Integration Tests (42 Suites)
│   ├── benches/              # Performance Benchmarks
│   └── Cargo.toml            # Dependencies
├── src-tauri/                # Desktop App Backend (Tauri 2.0)
│   ├── src/                  # Rust Commands + Audit Logger
│   │   ├── commands/         # Tauri IPC Commands
│   │   ├── audit_logger.rs   # V1.0 Audit Trail
│   │   └── lib.rs            # Entry Point
│   └── tauri.conf.json       # Tauri Konfiguration
├── tauri-frontend/           # Desktop Frontend (React + TypeScript)
│   ├── src/
│   │   ├── components/       # UI Komponenten
│   │   ├── store/            # Zustand State Management
│   │   └── lib/tauri.ts      # Tauri IPC Wrapper
│   └── package.json
├── sap-adapter/              # SAP Integration (geplant)
├── docs/project/             # Projektdokumentation
│   └── overview/             # Diese Dokumentation
├── examples/                 # Beispieldaten
└── MD/                       # Design-Dokumente (PRDs)
```

### Entwicklungsstatus

**Version:** 0.12.2
**Veröffentlicht:** Dezember 2025
**Status:** Produktionsbereit für Phase 1+2, MVP v1.0 bis 31. Dezember 2025

#### Schnell-Übersicht

| Was? | Status |
|------|--------|
| **CLI & Core Features** | ✅ Produktionsreif |
| **REST API & Security** | ✅ Produktionsreif (TLS/mTLS, OAuth2) |
| **Desktop App (Tauri)** | ✅ **Produktionsreif** (v0.12.2, Offline Proofer+Verifier+Audit) |
| **Policy Store System** | ✅ **Produktionsreif** (InMemory + SQLite) |
| **Monitoring & Observability** | ✅ **Produktionsreif** (Full Stack) |
| **Web UI (Tauri-integriert)** | ✅ **Produktionsreif** (v0.12.2, React 19.2 + TypeScript 5.9) |
| **Load Testing & Performance** | ✅ **Abgeschlossen** (Week 5, 22-27 RPS) |
| **Code Coverage & Quality** | ✅ **Abgeschlossen** (Week 6, 556 Tests, 0 Failures) |
| **Package Flow Refactoring** | ✅ **Abgeschlossen** (v0.11.0, cap-bundle.v1 mit Security Features) |
| **Echte ZK-Proofs (Halo2)** | 🔄 In Entwicklung (Q1 2026) |
| **SAP-Integration** | 🔄 In Entwicklung (Q1 2026) |
| **External Security Audit** | 📅 Geplant (Q1 2026) |

**📊 Für Details siehe:** [07-status-und-roadmap.md](./07-status-und-roadmap.md)

#### Test-Abdeckung

```
✅ 556/556 Tests bestanden (100% Success Rate, 0 Failures)
   - 385 Library Tests (Unit Tests)
   - 164 Binary Tests (CLI/API Tests)
   - 42 Integration Test Suites (End-to-End)
   - 7 Doc Tests (Dokumentation)

✅ Test-Coverage: Bundle v2, Dual-Anchor, Hash Validation,
   Registry, SQLite, Policy Store, Package Flow Refactoring

✅ 0 Clippy-Warnings (strikte Lint-Regeln)

✅ Security: cargo audit in CI/CD
   - Automatische Vulnerability-Scans
   - Dependency-Update-Tracking
   - Path Traversal Prevention
   - Dependency Cycle Detection
   - TOCTOU Mitigation (Load-Once-Pattern)
```

#### Zeitplan

- **Jetzt:** v0.12.2 (Desktop App + CLI + REST API produktionsreif)
- **31. Dezember 2025:** MVP v1.0 (Halo2 + SAP)
- **2026:** Enterprise v2.0 (Multi-Tenancy + Zertifizierungen)

### Lizenz & Rechtliches

**Lizenz:** [Lizenz-Info in README.md prüfen]
**Compliance:** DSGVO-konform (Privacy by Design)
**Zertifizierungen:** TÜV-Zertifizierung geplant (Phase 4)

### Support & Dokumentation

- **Haupt-README:** `/Users/tomwesselmann/Desktop/LsKG-Agent/README.md`
- **Technische Docs:** `/Users/tomwesselmann/Desktop/LsKG-Agent/agent/CLAUDE.md`
- **Deployment Guide:** `/Users/tomwesselmann/Desktop/LsKG-Agent/DEPLOYMENT.md`
- **Design Docs:** `/Users/tomwesselmann/Desktop/LsKG-Agent/MD/`

### Nächste Schritte

Für detaillierte Informationen zu den einzelnen Komponenten, siehe:
- **[07-status-und-roadmap.md](./07-status-und-roadmap.md)** ⭐ **Was ist fertig? Was kommt?** (NEU)
- [02-architecture.md](./02-architecture.md) - Systemarchitektur
- [03-components.md](./03-components.md) - Alle Module und Dateien
- [04-api-reference.md](./04-api-reference.md) - REST API Dokumentation
- [05-deployment.md](./05-deployment.md) - Installation und Betrieb
- [06-troubleshooting.md](./06-troubleshooting.md) - Fehlersuche und Lösungen
