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

## LsKG-Agent (CAP v0.11.0)

### Was ist der LsKG-Agent?

Der **LsKG-Agent** ist ein produktionsreifer, kryptographischer Compliance-Proof-System für das deutsche **Lieferkettensorgfaltspflichtengesetz (LkSG)**.

**In einfachen Worten:** Eine Software, die Compliance-Nachweise erstellt und überprüft, ohne sensible Daten preiszugeben.

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
- **REST API** - Für Systemintegration und Entwickler
- **Web UI** (geplant) - Für nicht-technische Nutzer

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
- Erstellen neuer "Unterschriften-Schlüssel"
- Alte Schlüssel in Rente schicken (wie abgelaufene Ausweise)
- Neue Schlüssel von alten bestätigen lassen (Vertrauenskette)

*Analogie:* Wie bei Firmen-Stempeln - alte werden archiviert, neue werden vom Geschäftsführer beglaubigt.

#### 5. Audit Trail (Prüfpfad)
**Was es macht:** Dokumentiert jede Aktion unveränderlich

**Eigenschaften:**
- Jede Aktion bekommt eine Nummer und einen Zeitstempel
- Neue Aktionen bauen auf alten auf (wie Blockchain)
- Änderungen sind unmöglich (würde sofort auffallen)

*Analogie:* Wie ein Fahrtenbuch mit nummerierten Seiten - man kann keine Seite entfernen oder austauschen, ohne dass es auffällt.

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
├── agent/                    # Hauptprojekt (Rust)
│   ├── src/                  # Quellcode (65+ Module)
│   ├── tests/                # Integration Tests (24 Tests)
│   ├── benches/              # Performance Benchmarks
│   └── Cargo.toml            # Dependencies
├── sap-adapter/              # SAP Integration (geplant)
├── docs/                     # Dokumentation (diese Dateien)
├── examples/                 # Beispieldaten
└── MD/                       # Design-Dokumente (PRDs)
```

### Entwicklungsstatus

**Version:** 0.11.0
**Veröffentlicht:** November 2025
**Status:** Produktionsbereit für Phase 1+2, MVP v1.0 bis 31. Dezember 2025

#### Schnell-Übersicht

| Was? | Status |
|------|--------|
| **CLI & Core Features** | ✅ Produktionsreif |
| **REST API & Security** | ✅ Produktionsreif |
| **Monitoring & Observability** | ✅ **Produktionsreif** (Week 2) |
| **Echte ZK-Proofs (Halo2)** | 🔄 In Entwicklung (Woche 1-2) |
| **SAP-Integration** | 🔄 In Entwicklung (Woche 3) |
| **Web UI** | 📅 Geplant (Woche 4) |
| **Security Audit** | 📅 Geplant (Woche 5) |
| **Production Deployment** | 📅 Geplant (Woche 6) |

**📊 Für Details siehe:** [07-status-und-roadmap.md](./07-status-und-roadmap.md)

#### Test-Abdeckung

```
✅ 146/146 Tests bestanden (100%)
✅ 0 Clippy-Warnings
✅ Security: cargo audit in CI/CD
```

#### Zeitplan

- **Jetzt:** v0.11.0 (CLI + REST API produktionsreif)
- **31. Dezember 2025:** MVP v1.0 (Halo2 + SAP + Web UI)
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
