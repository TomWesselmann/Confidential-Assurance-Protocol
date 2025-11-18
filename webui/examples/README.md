# CAP WebUI Test Examples

Dieses Verzeichnis enthält Beispieldateien zum Testen der CAP Verifier WebUI.

## Verfügbare Dateien

### manifest.json
- **Typ**: CAP Manifest (standalone)
- **Verwendung**: Zum schnellen Testen der Manifest-Anzeige
- **Upload**: Direkt in der WebUI hochladen
- **Struktur**:
  - Version: manifest.v0
  - Proof Type: none
  - Policy: LkSG Demo Policy
  - Audit Events: 45

### manifest-extended.json
- **Typ**: CAP Manifest mit erweiterten Audit-Feldern (CAP Manifest v0.1+)
- **Verwendung**: Zum Testen der erweiterten Audit Trail Anzeige
- **Upload**: Direkt in der WebUI hochladen
- **Erweiterte Felder**:
  - time_range: Zeitspanne der Events (start/end)
  - event_categories: Kategorisierung (data_changes, compliance, system)
  - last_event_type: Typ des letzten Events
  - hash_function: SHA3-256
  - chain_type: linear_hash_chain
  - integrity: verified
  - audit_chain_version: 1

### proof-bundle.zip
- **Typ**: Vollständiges CAP Proof Package
- **Verwendung**: Zum Testen der Bundle-Extraktion
- **Inhalt**:
  - manifest.json - CAP Manifest
  - proof.dat - Mock Proof (Base64)

## Verwendung

### Option 1: Manifest direkt hochladen
```
1. WebUI öffnen: http://localhost:5173/
2. Datei auswählen: manifest.json
3. Manifest wird angezeigt
```

### Option 2: Vollständiges Bundle hochladen
```
1. WebUI öffnen: http://localhost:5173/
2. Datei auswählen: proof-bundle.zip
3. Bundle wird extrahiert
4. Manifest und Proof werden angezeigt
```

## Erwartetes Verhalten

### Manifest-Anzeige (manifest.json)
- Header: "CAP Manifest"
- Version: manifest.v0
- Erstellt: 25.10.2025, 15:43:32
- Supplier Root: 0xdde3f2c...72b2f1610
- UBO Root: 0xf89ea64...3e7bf846
- Company Commitment Root: 0x83a8779...cfc898ae5
- Policy Name: LkSG Demo Policy
- Policy Version: lksg.v1
- Policy Hash: 0xd490be9...64f9b52b3
- Proof Type: None
- Proof Status: none
- Audit Events: 45
- Tail Digest: 0xb93b80c...5c93ff2e
- Signaturen: 0

### Erweiterte Manifest-Anzeige (manifest-extended.json)
Zusätzlich zu den Standard-Feldern wird eine erweiterte Audit Trail Sektion angezeigt:
- **Erweiterte Audit-Informationen** (nur wenn Felder vorhanden):
  - Zeitspanne: 10.12.2024, 10:12:03 → 25.10.2025, 15:43:32
  - Event-Kategorien: Data: 28 • Compliance: 12 • System: 5
  - Letztes Event: manifest_generated
  - Hash-Funktion: SHA3-256
  - Chain Type: linear_hash_chain
  - Integrität: verified (grünes Badge)
  - Audit-Chain-Version: 1
- Audit-Beschreibung passt sich dynamisch an (zeigt Hash-Funktion, Chain Type, Integrität)

### Hash-Validierung
Alle angezeigten Hashes sollten mit einem grünen Häkchen validiert werden:
- Format: 0x + 64 Hex-Zeichen
- BLAKE3 (Supplier/UBO Roots)
- SHA3-256 (Policy, Audit)

## Troubleshooting

### "Invalid manifest structure"
- Prüfen Sie, dass die Datei ein gültiges JSON ist
- Prüfen Sie, dass alle erforderlichen Felder vorhanden sind

### "manifest.json not found in bundle"
- Prüfen Sie, dass das ZIP-Archiv korrekt erstellt wurde
- Prüfen Sie, dass die Datei im root des ZIPs liegt (nicht in Unterordner)

### "Unsupported file type"
- Nur .zip und .json Dateien werden unterstützt
- Prüfen Sie die Dateiendung

## Entwicklung

### Neue Beispieldateien erstellen
```bash
# Im agent-Verzeichnis
cd /Users/tomwesselmann/Desktop/LsKG-Agent/agent

# Commitments erstellen
cargo run --bin cap-agent -- prepare --suppliers examples/suppliers.csv --ubos examples/ubos.csv

# Manifest erstellen
cargo run --bin cap-agent -- manifest build --policy examples/policy.lksg.v1.yml

# Proof erstellen
cargo run --bin cap-agent -- proof build --manifest build/manifest.json --policy examples/policy.lksg.v1.yml

# Package exportieren
cargo run --bin cap-agent -- proof export --manifest build/manifest.json --proof build/proof.dat --out build/proof_package

# In WebUI kopieren
cp build/manifest.json ../webui/examples/
cd build && zip -r ../../webui/examples/proof-bundle.zip proof_package/
```
