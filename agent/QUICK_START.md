# Quick Start: Proof Package Upload - Checklist

**Ziel**: Einen Proof erstellen und in der WebUI hochladen
**Geschätzte Zeit**: 5-10 Minuten

---

## Vorbereitung

### ☐ Terminal 1: Build-Verzeichnis bereinigen

```bash
cd /Users/tomwesselmann/Desktop/LsKG-Agent/agent
rm -rf build
mkdir build
```

**Warum?** Sauberer Start ohne alte Dateien

---

## Schritt 1: Proof Package erstellen

### ☐ 1.1 Commitments erstellen

```bash
cargo run --bin cap-agent -- prepare \
  --suppliers ../examples/suppliers.csv \
  --ubos ../examples/ubos.csv
```

**Erwartetes Ergebnis:**
```
✅ Commitments gespeichert: build/commitments.json
```

### ☐ 1.2 Manifest erstellen

```bash
cargo run --bin cap-agent -- manifest build \
  --policy ../examples/policy.lksg.v1.yml
```

**Erwartetes Ergebnis:**
```
✅ Manifest erstellt: build/manifest.json
```

### ☐ 1.3 Proof erstellen

```bash
cargo run --bin cap-agent -- proof build
```

**Erwartetes Ergebnis:**
```
✅ Proof erstellt: build/proof.capz
```

### ☐ 1.4 Proof Package exportieren

```bash
cargo run --bin cap-agent -- proof export \
  --manifest build/manifest.json \
  --proof build/proof.capz \
  --output build/package
```

**Erwartetes Ergebnis:**
```
✅ Package exportiert nach: build/package/
```

### ☐ 1.5 ZIP erstellen

```bash
cd build/package
zip -r ../proof-package.zip .
cd ../..
```

**Prüfen:**
```bash
ls -lh build/proof-package.zip
```

Du solltest sehen: `proof-package.zip` (~1-5 KB)

---

## Schritt 2: API Server starten

### ☐ 2.1 Server starten (Terminal 1)

```bash
cargo run --bin cap-verifier-api -- \
  --bind 127.0.0.1:8080 \
  --token admin-tom
```

**Erwartetes Ergebnis:**
```
🚀 Server läuft auf: http://127.0.0.1:8080
🔑 Token: admin-tom
```

**Server läuft weiter - Terminal offen lassen!**

---

## Schritt 3: WebUI starten

### ☐ 3.1 WebUI bauen und starten (Terminal 2 - NEUES TERMINAL)

```bash
cd /Users/tomwesselmann/Desktop/LsKG-Agent/webui
npm run dev
```

**Erwartetes Ergebnis:**
```
  VITE ready in XXX ms

  ➜  Local:   http://localhost:5173/
  ➜  Network: use --host to expose
```

**WebUI läuft weiter - Terminal offen lassen!**

---

## Schritt 4: Proof in WebUI hochladen

### ☐ 4.1 Browser öffnen

```
http://localhost:5173
```

### ☐ 4.2 API-Einstellungen konfigurieren

1. Klicke auf **"Settings"** (Zahnrad-Symbol oben rechts)
2. Trage ein:
   - **API URL**: `http://127.0.0.1:8080`
   - **Bearer Token**: `admin-tom`
3. Klicke **"Save Settings"**

### ☐ 4.3 Proof hochladen

1. Gehe zur Hauptseite
2. Klicke **"Upload Proof Package"**
3. Wähle: `/Users/tomwesselmann/Desktop/LsKG-Agent/agent/build/proof-package.zip`
4. Klicke **"Upload"**

### ☐ 4.4 Ergebnis prüfen

**Erfolg sieht so aus:**
- ✅ Grünes Häkchen oder "Verification Successful"
- Anzeige von Proof Details (Policy ID, Merkle Roots, etc.)

**Bei Fehler:**
- Rotes X oder Fehlermeldung
- Prüfe Server-Logs in Terminal 1

---

## Troubleshooting während Upload

### Problem: "Network Error" oder "Cannot connect"

**Lösung:**
```bash
# Terminal 3 (neues Terminal)
curl -H "Authorization: Bearer admin-tom" http://127.0.0.1:8080/healthz
```

Sollte zurückgeben: `{"status":"healthy"}`

Falls nicht → Server neu starten (Terminal 1)

### Problem: "Unauthorized" oder 401

**Lösung:**
- Prüfe Token in Settings: `admin-tom` (ohne Anführungszeichen)
- Prüfe API URL: `http://127.0.0.1:8080` (ohne trailing slash)

### Problem: "Invalid proof package"

**Lösung:**
```bash
# ZIP-Inhalt prüfen
unzip -l build/proof-package.zip

# Sollte enthalten:
# manifest.json
# proof.capz
```

Falls Dateien fehlen → Schritt 1.4 wiederholen

### Problem: "Policy not found"

**Lösung:**
Die WebUI erwartet eine compilierte Policy. Entweder:

**Option A: Policy hochladen**
```bash
# Terminal 3
curl -X POST http://127.0.0.1:8080/policy/v2/compile \
  -H "Authorization: Bearer admin-tom" \
  -H "Content-Type: application/json" \
  -d @../examples/policy-v2-payload.json
```

**Option B: Mock-Backend nutzen**
Das proof-package sollte mit Mock-Backend funktionieren (keine Policy nötig)

---

## Fertig! 🎉

Wenn du den Proof erfolgreich hochgeladen und das Ergebnis gesehen hast, ist der Test abgeschlossen.

### Nächste Schritte (optional):

- ☐ Andere CSV-Daten testen (eigene suppliers.csv / ubos.csv)
- ☐ Verschiedene Policies testen
- ☐ Registry-Integration testen (siehe BENUTZERHANDBUCH.md)
- ☐ Signature-Workflow testen (siehe BENUTZERHANDBUCH.md)

---

## Server stoppen

Wenn du fertig bist:

1. **Terminal 2 (WebUI)**: `Ctrl+C`
2. **Terminal 1 (API Server)**: `Ctrl+C`
3. **Build aufräumen**: `rm -rf build` (optional)

---

**Erstellt**: 2025-11-20
**Für detaillierte Kommando-Referenz siehe**: `BENUTZERHANDBUCH.md`
