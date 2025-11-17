# 06 - Troubleshooting & FAQ

## 📖 Über dieses Kapitel

Sie haben jetzt das komplette System kennengelernt: **was** es macht, **wie** es aufgebaut ist, **welche Teile** es hat, **wie man es bedient** und **wie man es installiert**. Dieses letzte Kapitel zeigt **was zu tun ist, wenn etwas nicht funktioniert**.

**Für wen ist dieses Kapitel?**
- **Management:** Die FAQ-Sektion mit häufigen Fragen
- **IT-Support:** Die Problemlösungen für Support-Tickets
- **IT-Administratoren:** Die Debug-Techniken und Log-Analyse
- **Alle Nutzer:** Die Fehlermeldungen und ihre Bedeutung

**Was Sie lernen werden:**
1. Die 10 häufigsten Probleme und ihre Lösungen
2. Wie man Fehler systematisch analysiert
3. Wo man weitere Hilfe bekommt
4. Antworten auf häufige Fragen (FAQ)

**Analogie:** Dies ist die **Fehlerbehebungs-Anleitung** - wie bei einem Auto das Kapitel "Was tun bei Panne?"

---

## 👔 Für Management: Wann IT-Support kontaktieren?

### Probleme, die Sie selbst lösen können: ✅
- Vergessenes Passwort/Token → Neu anfordern
- Alte Dateien können nicht gelesen werden → Dateien aktualisieren
- "Service nicht erreichbar" → Prüfen Sie Ihre Internetverbindung

### Probleme für IT-Support: ⚠️
- "Authentication failed" trotz korrektem Token → Support kontaktieren
- "Database locked" → Datenbank-Problem, Support nötig
- Langsame Performance → Möglicherweise Ressourcen-Problem

### Kritische Probleme (sofort eskalieren): 🚨
- "Security violation detected" → Sicherheitsteam informieren
- Datenverlust nach Backup-Restore → Sofort IT-Leitung informieren
- System komplett nicht erreichbar → Kritischer Ausfall

### Eskalationsstufen:

1. **Stufe 1 - First Level Support:** Einfache Probleme (Passwort, Zugriff)
2. **Stufe 2 - IT-Administrator:** Technische Probleme (Server, Konfiguration)
3. **Stufe 3 - Entwickler/Vendor:** Komplexe Bugs, Systemfehler
4. **Stufe 4 - Sicherheitsteam:** Sicherheitsvorfälle

**Tipp:** Bevor Sie Support kontaktieren, notieren Sie:
- Fehlermeldung (kompletter Text oder Screenshot)
- Was haben Sie gerade gemacht? (Reproduktionsschritte)
- Wann trat das Problem auf? (Datum, Uhrzeit)

---

## Häufige Probleme & Lösungen

**Legende:**
- 🟢 **Einfach:** Ohne IT-Kenntnisse lösbar
- 🟡 **Mittel:** Basis IT-Kenntnisse nötig
- 🔴 **Schwer:** IT-Administrator nötig

### 1. API Server startet nicht 🔴

**Symptom:**
```
Error: Failed to bind to address 0.0.0.0:8443
```

**Ursachen & Lösungen:**

**A) Port bereits belegt**
```bash
# Prüfe welcher Prozess Port 8443 verwendet
sudo lsof -i :8443
# oder
sudo netstat -tulpn | grep 8443

# Prozess beenden
sudo kill <PID>
```

**B) Keine Berechtigung für Port < 1024**
```bash
# Lösung 1: Verwende Port >= 1024
--bind 0.0.0.0:8080

# Lösung 2: Gebe Binary CAP_NET_BIND_SERVICE capability
sudo setcap 'cap_net_bind_service=+ep' /usr/local/bin/cap-verifier-api
```

**C) Firewall blockiert Port**
```bash
# UFW
sudo ufw allow 8443/tcp

# iptables
sudo iptables -A INPUT -p tcp --dport 8443 -j ACCEPT
```

---

### 2. TLS Certificate Errors

**Symptom:**
```
Error: Failed to load TLS certificate
```

**Ursachen & Lösungen:**

**A) Certificate-Datei nicht gefunden**
```bash
# Prüfe Pfad
ls -la /certs/server.crt
ls -la /certs/server.key

# Korrekter Pfad in Konfiguration
tls:
  cert_path: "/certs/server.crt"
  key_path: "/certs/server.key"
```

**B) Falsche Permissions**
```bash
# Setze korrekte Permissions
sudo chown cap-verifier:cap-verifier /certs/server.crt
sudo chown cap-verifier:cap-verifier /certs/server.key
sudo chmod 600 /certs/server.key
sudo chmod 644 /certs/server.crt
```

**C) Certificate-Format falsch**
```bash
# Prüfe Format (muss PEM sein)
openssl x509 -in /certs/server.crt -text -noout

# Key-Format prüfen
openssl rsa -in /certs/server.key -check

# Falls PKCS#12 → PEM konvertieren
openssl pkcs12 -in cert.p12 -out server.crt -clcerts -nokeys
openssl pkcs12 -in cert.p12 -out server.key -nocerts -nodes
```

**D) Certificate abgelaufen**
```bash
# Ablaufdatum prüfen
openssl x509 -in /certs/server.crt -noout -enddate

# Neues Certificate generieren
openssl req -x509 -newkey rsa:4096 -keyout server.key -out server.crt -days 365 -nodes
```

---

### 3. OAuth2 Authentication Failures

**Symptom:**
```
401 Unauthorized: Invalid or expired token
```

**Ursachen & Lösungen:**

**A) Token abgelaufen**
```bash
# Token dekodieren (auf jwt.io)
# Prüfe "exp" claim

# Neuen Token anfordern
curl -X POST https://auth.example.com/oauth/token \
  -d "grant_type=client_credentials" \
  -d "client_id=YOUR_CLIENT_ID" \
  -d "client_secret=YOUR_CLIENT_SECRET"
```

**B) Falscher Public Key**
```bash
# Public Key vom Auth Server abrufen
curl https://auth.example.com/.well-known/jwks.json

# In PEM konvertieren und in /config/public.pem speichern

# Server neu starten
systemctl restart cap-verifier
```

**C) Falscher Issuer oder Audience**
```bash
# Token-Claims prüfen
# "iss" muss matchen: config/auth.yaml → issuer
# "aud" muss matchen: config/auth.yaml → audience

# Beispiel:
oauth2:
  issuer: "https://auth.example.com"
  audience: "cap-verifier"
```

**D) Fehlende Scopes**
```bash
# Token-Claims prüfen
# "scope" muss enthalten: "verify:read"

# Bei Token-Anforderung Scope angeben
curl -X POST https://auth.example.com/oauth/token \
  -d "grant_type=client_credentials" \
  -d "client_id=YOUR_CLIENT_ID" \
  -d "client_secret=YOUR_CLIENT_SECRET" \
  -d "scope=verify:read"
```

---

### 4. Proof Verification Failures

**Symptom:**
```
Verification: FAIL
Status: Invalid manifest hash
```

**Ursachen & Lösungen:**

**A) Manifest wurde modifiziert**
```bash
# Hash neu berechnen
cat build/manifest.json | sha3sum -a 256

# Mit erwarteten Hash vergleichen
# Falls unterschiedlich: Manifest ist beschädigt

# Lösung: Manifest neu erstellen
cap manifest build \
  --commitments build/commitments.json \
  --policy policy.lksg.v1.yml \
  --output build/manifest.json
```

**B) Falscher Proof Backend**
```bash
# Backend in Proof prüfen
cat build/proof.json | jq '.type'

# Mit verwendetem Backend vergleichen
cap proof verify --proof build/proof.dat --manifest build/manifest.json --backend mock
```

**C) Policy Hash stimmt nicht überein**
```bash
# Policy Hash neu berechnen
cap policy validate --policy policy.lksg.v1.yml

# Mit Manifest vergleichen
cat build/manifest.json | jq '.policy.hash'

# Falls unterschiedlich: Manifest neu erstellen mit korrekter Policy
```

---

### 5. CSV Import Errors

**Symptom:**
```
Error: Failed to parse CSV: InvalidRecord
```

**Ursachen & Lösungen:**

**A) Falsche Encoding**
```bash
# Prüfe Encoding
file -i suppliers.csv

# Falls nicht UTF-8: Konvertieren
iconv -f ISO-8859-1 -t UTF-8 suppliers.csv > suppliers_utf8.csv
```

**B) Falsches Delimiter**
```bash
# Delimiter prüfen (muss Komma sein)
head -1 suppliers.csv

# Falls Tab oder Semikolon: Ersetzen
sed 's/\t/,/g' suppliers.csv > suppliers_fixed.csv
sed 's/;/,/g' suppliers.csv > suppliers_fixed.csv
```

**C) Fehlende Header**
```bash
# Header prüfen
head -1 suppliers.csv
# Erwartete Header: name,jurisdiction,tier

head -1 ubos.csv
# Erwartete Header: name,birthdate,citizenship
```

**D) Fehlende Pflichtfelder**
```bash
# Prüfe ob alle Felder befüllt sind
awk -F',' 'NF!=3 {print NR": " $0}' suppliers.csv

# Zeilen mit fehlenden Feldern korrigieren
```

---

### 6. Registry Database Errors

**Symptom:**
```
Error: database is locked
```

**Ursachen & Lösungen:**

**A) Mehrere Prozesse greifen auf Registry zu**
```bash
# Laufende Prozesse prüfen
ps aux | grep cap

# Nur ein Prozess sollte schreibend zugreifen

# Falls mehrere: Andere Prozesse beenden
```

**B) WAL-Mode nicht aktiviert**
```bash
# WAL-Mode prüfen
sqlite3 registry.db "PRAGMA journal_mode;"

# Falls nicht WAL: Aktivieren
sqlite3 registry.db "PRAGMA journal_mode=WAL;"
```

**C) Corruption**
```bash
# Integrität prüfen
sqlite3 registry.db "PRAGMA integrity_check;"

# Falls corruption: Von Backup wiederherstellen
cp /backups/registry.db.bak registry.db
```

---

### 7. Key Management Issues

**Symptom:**
```
Error: Invalid KID format
```

**Ursachen & Lösungen:**

**A) KID-Format falsch**
```bash
# KID muss 32 hex chars sein (128 bits)
# Beispiel: a1b2c3d4e5f67890a1b2c3d4e5f67890

# KID aus Public Key ableiten
cap keys derive-kid --public-key keys/company.pub
```

**B) Key-Metadaten beschädigt**
```bash
# Metadaten prüfen
cat keys/company.json | jq '.'

# Schema validieren (muss "cap-key.v1" sein)
cat keys/company.json | jq '.schema'

# Falls beschädigt: Metadaten neu erstellen
cap keys keygen --owner "Company" --output keys/company
```

**C) Key-Status falsch**
```bash
# Status prüfen (muss "active" sein für Signing)
cat keys/company.json | jq '.status'

# Status ändern
cap keys update-status --key keys/company.json --status active
```

---

### 8. Memory Issues

**Symptom:**
```
Out of memory
```

**Ursachen & Lösungen:**

**A) Zu große CSV-Dateien**
```bash
# Dateigröße prüfen
du -h suppliers.csv

# Falls > 1 GB: Datei aufteilen
split -l 10000 suppliers.csv suppliers_part_

# Einzeln verarbeiten
for file in suppliers_part_*; do
  cap prepare --suppliers $file --ubos ubos.csv --output build_$file/
done
```

**B) Zu viele BLOBs im Store**
```bash
# BLOB Store Größe prüfen
du -sh /data/blobs/

# Garbage Collection ausführen
cap blob-store gc --store /data/blobs/

# Alte BLOBs manuell löschen (nur unreferenzierte!)
```

**C) Container Memory Limits zu niedrig**
```yaml
# Kubernetes: Limits erhöhen
resources:
  limits:
    memory: 4Gi  # Statt 2Gi
```

---

### 9. Performance Issues

**Symptom:**
```
Request taking > 10 seconds
```

**Ursachen & Lösungen:**

**A) SQLite nicht optimiert**
```bash
# WAL-Mode aktivieren
sqlite3 registry.db "PRAGMA journal_mode=WAL;"

# Cache Size erhöhen
sqlite3 registry.db "PRAGMA cache_size=-64000;"  # 64 MB
```

**B) Zu viele Log-Events**
```bash
# Log-Level reduzieren
export RUST_LOG=info  # Statt debug/trace

# Oder in Systemd:
Environment="RUST_LOG=info"
```

**C) Langsame Proof-Backends**
```bash
# Mock-Backend verwenden für Tests
--backend mock

# Oder Backend-Timeout erhöhen
verification:
  timeout_seconds: 60  # Statt 30
```

**D) Keine Indexes**
```bash
# Prüfe ob Indexes existieren
sqlite3 registry.db ".indexes"

# Falls nicht: Erstellen
sqlite3 registry.db "CREATE INDEX idx_manifest_proof ON registry_entries(manifest_hash, proof_hash);"
```

---

### 10. Docker Issues

**Symptom:**
```
Container exits immediately
```

**Ursachen & Lösungen:**

**A) Fehlende Volumes**
```bash
# Prüfe Volumes
docker inspect cap-verifier | jq '.[0].Mounts'

# Volumes mounten
docker run -v $(pwd)/config:/config:ro -v $(pwd)/data:/data ...
```

**B) Permissions**
```bash
# Container läuft als User 1000
# Host-Verzeichnis muss lesbar sein
sudo chown -R 1000:1000 /path/to/data
```

**C) Port-Mapping falsch**
```bash
# Host:Container Port richtig mappen
-p 8443:8443  # Host 8443 → Container 8443
```

**D) Health Check schlägt fehl**
```bash
# Health Check manuell prüfen
docker exec cap-verifier curl -f http://localhost:8080/healthz

# Logs ansehen
docker logs cap-verifier
```

---

## Debug-Techniken

### 1. Verbose Logging

```bash
# Maximale Log-Details
export RUST_LOG=trace

# Module-spezifisch
export RUST_LOG=cap_agent::verifier=debug,cap_agent::api=trace
```

### 2. Request Tracing

```bash
# Request ID in Logs suchen
grep "request_id=abc123" /var/log/cap-verifier.log

# Alle Events für einen Request
grep "request_id=abc123" /var/log/cap-verifier.log | jq '.'
```

### 3. Profiling

```bash
# CPU Profiling (Linux)
perf record -F 99 -g -- ./cap-verifier-api
perf report

# Memory Profiling
valgrind --tool=massif ./cap-verifier-api
```

### 4. Database Inspection

```bash
# SQLite Registry analysieren
sqlite3 registry.db <<EOF
.schema
SELECT COUNT(*) FROM registry_entries;
SELECT * FROM registry_entries LIMIT 10;
.exit
EOF

# Größte Tabellen
sqlite3 registry.db "SELECT name, SUM(pgsize) as size FROM dbstat GROUP BY name ORDER BY size DESC;"
```

### 5. Network Debugging

```bash
# TLS Handshake prüfen
openssl s_client -connect localhost:8443 -showcerts

# HTTP Request manuell
curl -v -k https://localhost:8443/healthz

# Mit OAuth2 Token
curl -v -k -H "Authorization: Bearer $TOKEN" https://localhost:8443/verify
```

---

## FAQ

### Q1: Wie kann ich den API-Server ohne TLS starten?

**A:** Setze `TLS_MODE=disabled` oder starte ohne `--tls` Flag:
```bash
cap-verifier-api --config app.yaml
# Bindet an Port 8080 (HTTP)
```

### Q2: Kann ich mehrere Policies gleichzeitig verwenden?

**A:** Ja, jede Policy hat einen eindeutigen Hash. Du kannst mehrere Policies kompilieren und beim Proof-Build die gewünschte Policy referenzieren.

### Q3: Wie rotiere ich Signing Keys?

**A:** Siehe Key Rotation Guide in [04-api-reference.md](./04-api-reference.md#cap-keys-rotate):
```bash
# 1. Neuen Key generieren
cap keys keygen --owner "Company" --output keys/new-key

# 2. Attestierung erstellen
cap keys attest --signer keys/old-key.json --subject keys/new-key.json --output keys/attestation.json

# 3. Rotation durchführen
cap keys rotate --current keys/old-key.json --new keys/new-key.json
```

### Q4: Wie migriere ich von JSON zu SQLite Registry?

**A:**
```bash
cap registry migrate \
  --from registry.json \
  --to registry.db \
  --backend sqlite
```

### Q5: Unterstützt der API-Server Horizontal Scaling?

**A:** Ja, aber beachte:
- API-Server ist **stateless** (keine Shared Memory)
- Registry/BLOB Store müssen geteilt werden (z.B. PostgreSQL, S3)
- Verwende Load Balancer für Traffic-Verteilung

### Q6: Wie groß können CSV-Dateien sein?

**A:** Theoretisch unbegrenzt, praktisch:
- **Empfohlen:** < 100 MB pro Datei
- **Maximum getestet:** 1 GB
- Für größere Dateien: Aufteilen und batch-verarbeiten

### Q7: Welche Proof-Backends sind produktionsreif?

**A:** Aktuell (v0.11.0):
- **Mock** - Produktionsreif für Testing
- **ZK-VM** - Phase 3 (geplant)
- **Halo2** - Phase 3 (geplant)

### Q8: Kann ich eigene Proof-Backends entwickeln?

**A:** Ja, implementiere das `ProofSystem` Trait:
```rust
trait ProofSystem {
    fn backend_name(&self) -> &str;
    fn verify(&self, proof_data: &ProofData, statement: &str) -> Result<bool>;
}
```

### Q9: Wie überwache ich die API-Performance?

**A:** Verwende Prometheus + Grafana:
1. Scrape `/metrics` Endpoint
2. Erstelle Dashboard mit wichtigen Metriken
3. Alerts für Latenz/Fehlerrate einrichten

### Q10: Was passiert bei einem Hash-Collision?

**A:** Extrem unwahrscheinlich (BLAKE3/SHA3-256):
- **BLAKE3:** 2^256 Möglichkeiten (≈ 10^77)
- **SHA3-256:** 2^256 Möglichkeiten
- Wahrscheinlichkeit: < 10^-60

---

## Kontakt & Support

### Community

- **GitHub Issues:** https://github.com/your-org/LsKG-Agent/issues
- **Discussions:** https://github.com/your-org/LsKG-Agent/discussions
- **Slack:** #lksg-agent

### Enterprise Support

Für Enterprise-Support kontaktieren Sie: support@example.com

### Weitere Dokumentation

- [01-overview.md](./01-overview.md) - Systemüberblick
- [02-architecture.md](./02-architecture.md) - Architektur
- [03-components.md](./03-components.md) - Komponenten
- [04-api-reference.md](./04-api-reference.md) - API-Referenz
- [05-deployment.md](./05-deployment.md) - Deployment

### Changelog

Siehe [ROADMAP_2025.md](../ROADMAP_2025.md) für geplante Features und bekannte Issues.
