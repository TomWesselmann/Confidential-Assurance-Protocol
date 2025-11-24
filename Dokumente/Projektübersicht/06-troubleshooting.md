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

### 11. Policy Store Errors

**Symptom:**
```
Failed to save policy: DatabaseError
Policy not found: 404
Deduplication not working
```

**Ursachen & Lösungen:**

**A) SQLite Database nicht erreichbar**
```bash
# Fehler:
# Error: unable to open database file

# Prüfe Dateipfad
ls -la /var/lib/cap-verifier/policies.sqlite

# Prüfe Permissions
# Datei muss für Prozess-User beschreibbar sein
sudo chown cap-verifier:cap-verifier /var/lib/cap-verifier/policies.sqlite
sudo chmod 664 /var/lib/cap-verifier/policies.sqlite

# Prüfe Verzeichnis-Permissions (wichtig für WAL mode!)
sudo chown cap-verifier:cap-verifier /var/lib/cap-verifier
sudo chmod 775 /var/lib/cap-verifier

# Lösung: Environment Variable korrigieren
export POLICY_DB_PATH=/var/lib/cap-verifier/policies.sqlite
```

**B) Database Corruption (WAL Fehler)**
```bash
# Fehler:
# Error: database disk image is malformed
# Error: WAL file corrupted

# Symptom: Datenbank-Datei existiert, aber nicht lesbar

# Prüfe Integrität
sqlite3 policies.sqlite "PRAGMA integrity_check;"

# Lösung 1: WAL Checkpoint erzwingen
sqlite3 policies.sqlite "PRAGMA wal_checkpoint(TRUNCATE);"

# Lösung 2: Backup wiederherstellen
cp policies.sqlite.backup policies.sqlite

# Lösung 3: Database neu erstellen (VORSICHT: Datenverlust!)
rm policies.sqlite policies.sqlite-wal policies.sqlite-shm
# Beim nächsten Start wird DB automatisch neu erstellt
```

**C) Deduplication funktioniert nicht**
```bash
# Symptom: Gleiche Policy wird mehrfach gespeichert

# Ursache: Hash-Berechnung nicht konsistent
# Lösung: Prüfe Policy JSON Format

# Test mit curl:
curl -X POST http://localhost:8080/policy/compile \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d @policy.json

# Wenn unterschiedliche policy_hash zurückkommt:
# → JSON Whitespace/Reihenfolge unterschiedlich
# → Canonical JSON verwenden

# Prüfe Hash manuell:
cargo run -- policy validate --file policy.json
# Ausgabe: policy_hash sollte identisch sein
```

**D) Concurrent Access Fehler**
```bash
# Fehler:
# Error: database is locked
# Error: mutex lock timeout

# Ursache 1: SQLite WAL mode nicht aktiv
# Prüfe WAL mode:
sqlite3 policies.sqlite "PRAGMA journal_mode;"
# Sollte "wal" zurückgeben

# Lösung 1: WAL mode aktivieren
sqlite3 policies.sqlite "PRAGMA journal_mode=WAL;"

# Ursache 2: Zu viele gleichzeitige Writes
# Lösung 2: Connection Pool vergrößern (zukünftig)
# Aktuell: SQLite hat nur eine Write-Connection

# Ursache 3: Mutex Lock Timeout (InMemory Backend)
# Lösung 3: Reduce concurrency oder Backend wechseln
```

**E) Policy nicht gefunden (404)**
```bash
# Fehler:
# GET /policy/:id → 404 Not Found

# Ursache 1: UUID falsch
# UUIDs sind case-sensitive
# Korrekt: a010ac65-1669-8469-7b93-b867c36e9c94
# Falsch:  A010AC65-1669-8469-7B93-B867C36E9C94

# Lösung 1: UUID lowercase verwenden

# Ursache 2: Policy existiert nicht in diesem Backend
# Prüfe Backend:
curl http://localhost:8080/healthz
# Zeigt POLICY_STORE_BACKEND

# Wenn Backend gewechselt wurde:
# InMemory → SQLite: Policies gehen verloren
# Lösung: Migration durchführen

# Prüfe direkt in DB:
sqlite3 policies.sqlite "SELECT id, name, hash FROM policies;"
```

**F) Status Transition Fehler**
```bash
# Fehler:
# PUT /policy/:id/status → 400 Bad Request
# Invalid status transition

# Erlaubte Übergänge:
# Draft → Active
# Active → Deprecated
# Draft → Deprecated

# Nicht erlaubt:
# Deprecated → Active (Use new policy instead)
# Active → Draft (Cannot revert to draft)

# Lösung: Neue Policy-Version erstellen
curl -X POST http://localhost:8080/policy/compile \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"policy": {...}, "status": "active"}'
```

**G) Backend Migration Fehler**
```bash
# Symptom: Nach Backend-Wechsel sind Policies weg

# InMemory → SQLite Migration:
# 1. InMemory Policies exportieren (vor Wechsel!)
curl -X GET http://localhost:8080/policy/list \
  -H "Authorization: Bearer $TOKEN" > policies_backup.json

# 2. Backend wechseln
export POLICY_STORE_BACKEND=sqlite
export POLICY_DB_PATH=/data/policies.sqlite

# 3. Server neu starten
systemctl restart cap-verifier

# 4. Policies re-importieren
# Für jede Policy in policies_backup.json:
curl -X POST http://localhost:8080/policy/compile \
  -H "Authorization: Bearer $TOKEN" \
  -d @policy.json

# WICHTIG: Deduplication verhindert Duplikate (gleicher Hash)
```

**H) Disk Full Errors**
```bash
# Fehler:
# Error: no space left on device
# Error: disk quota exceeded

# Prüfe Disk Space:
df -h /var/lib/cap-verifier

# SQLite WAL Dateien können groß werden
ls -lh /var/lib/cap-verifier/policies.sqlite*
# policies.sqlite      - Hauptdatei
# policies.sqlite-wal  - Write-Ahead Log (kann mehrere MB sein)
# policies.sqlite-shm  - Shared Memory

# Lösung 1: WAL Checkpoint
sqlite3 policies.sqlite "PRAGMA wal_checkpoint(TRUNCATE);"

# Lösung 2: Alte Policies löschen
# VORSICHT: Nur deprecated Policies löschen
sqlite3 policies.sqlite "DELETE FROM policies WHERE status='deprecated' AND updated_at < date('now', '-90 days');"

# Lösung 3: Disk erweitern
# Für Docker:
docker volume inspect policy-data

# Für Kubernetes:
kubectl describe pvc cap-verifier-policy-pvc
```

**Debug-Befehle:**

```bash
# 1. Policy Store Status prüfen
curl http://localhost:8080/healthz

# 2. Alle Policies auflisten
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/policy/list

# 3. SQLite Datei untersuchen
sqlite3 policies.sqlite ".schema"
sqlite3 policies.sqlite "SELECT COUNT(*) FROM policies;"
sqlite3 policies.sqlite "SELECT status, COUNT(*) FROM policies GROUP BY status;"

# 4. Logs durchsuchen
journalctl -u cap-verifier | grep -i "policy"
docker logs cap-verifier 2>&1 | grep -i "policy"

# 5. Datenbank-Größe prüfen
du -h /var/lib/cap-verifier/policies.sqlite*
```

---

### 12. WebUI Errors (v0.11.0)

**Symptom:**
```
CORS Error, 401 Unauthorized, Upload failed
WebUI kann nicht mit Backend kommunizieren
```

**Ursachen & Lösungen:**

**A) CORS Errors - Preflight 401**
```bash
# Fehler im Browser Console:
# Access to XMLHttpRequest blocked by CORS policy
# Preflight response is not successful. Status code: 401

# Ursache: OPTIONS Preflight-Request hat keinen Authorization Header
# Backend lehnt Preflight-Request ab

# Lösung: Backend CORS Middleware NACH Auth-Middleware anwenden
# In agent/src/bin/verifier_api.rs:

# ❌ Falsch (CORS VOR Auth):
let app = Router::new()
    .layer(cors)
    .layer(auth_middleware);

# ✅ Richtig (CORS NACH Auth):
let public_routes = Router::new()
    .route("/healthz", get(handle_healthz));

let protected_routes = Router::new()
    .route("/verify", post(handle_verify))
    .layer(auth_middleware);

Router::new()
    .merge(public_routes)
    .merge(protected_routes)
    .layer(cors);  # CORS zuletzt anwenden!
```

**B) 401 Unauthorized bei Upload**
```bash
# Fehler: 401 Unauthorized
# POST /proof/upload failed

# Ursache 1: Bearer Token nicht gesetzt
# In webui/src/App.tsx prüfen:
const [bearerToken, setBearerToken] = useState('admin-tom'); # Development

# Ursache 2: Token abgelaufen (Production)
# Lösung: JWT Token Refresh implementieren

# Ursache 3: Backend CORS erlaubt localhost:5173 nicht
# In agent/src/bin/verifier_api.rs:
let cors = CorsLayer::new()
    .allow_origin("http://localhost:5173".parse().unwrap())
    .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
    .allow_headers([AUTHORIZATION, CONTENT_TYPE]);
```

**C) Upload schlägt fehl mit 400 Bad Request**
```bash
# Fehler: 400 Bad Request - File field missing

# Ursache: FormData field name falsch
# Lösung: Field muss "file" heißen
const formData = new FormData();
formData.append('file', selectedFile);  # ✅ Korrekt
formData.append('bundle', selectedFile); # ❌ Falsch

# Backend erwartet:
# Field name: "file"
# Content-Type: multipart/form-data
```

**D) Verification schlägt fehl - Policy not found**
```bash
# Fehler: Policy not found: lksg.demo.v1

# Ursache: Policy wurde noch nicht kompiliert
# Lösung: Policy kompilieren BEVOR Verifikation

# 1. Policy kompilieren:
curl -X POST http://localhost:8080/policy/v2/compile \
  -H "Authorization: Bearer admin-tom" \
  -H "Content-Type: application/json" \
  -d @examples/policy_v2_request.json

# 2. Backend Logs prüfen:
# "Policy stored in cache: lksg.demo.v1"

# 3. Dann WebUI Upload & Verification
```

**E) WebUI zeigt "Network Error"**
```bash
# Fehler: Network Error / Connection Refused

# Ursache 1: Backend API nicht erreichbar
curl http://localhost:8080/healthz
# Lösung: Backend starten
cd agent && cargo run --bin cap-verifier-api

# Ursache 2: Falsche API URL in WebUI
# In webui/src/App.tsx:
const [apiUrl, setApiUrl] = useState('http://localhost:8080');

# Ursache 3: Firewall blockiert Port 8080
sudo ufw allow 8080/tcp
```

**F) Build Fehler - "Module not found"**
```bash
# Fehler: Error: Cannot find module 'axios'

# Lösung: Dependencies neu installieren
cd webui
rm -rf node_modules package-lock.json
npm install
npm run build
```

**G) Vite Dev Server CORS Issues (Production Build)**
```bash
# Problem: Production Build hat andere CORS Requirements

# Lösung: Nginx als Reverse Proxy
# nginx.conf:
location /api/ {
    proxy_pass http://backend:8080/;
    proxy_set_header Host $host;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
}

location / {
    root /var/www/webui;
    try_files $uri $uri/ /index.html;
}
```

**Debug-Befehle:**

```bash
# 1. Browser Console öffnen (F12)
# Check für CORS Errors, 401, Network Errors

# 2. Network Tab prüfen
# - Preflight OPTIONS Request (sollte 200 OK sein)
# - POST Request mit Authorization Header

# 3. Backend Logs prüfen
cargo run --bin cap-verifier-api
# Output: "Listening on http://127.0.0.1:8080"

# 4. WebUI Dev Server prüfen
cd webui && npm run dev
# Output: "Local: http://localhost:5173/"

# 5. API Health Check
curl http://localhost:8080/healthz

# 6. Test Upload mit curl (bypass WebUI)
curl -X POST http://localhost:8080/proof/upload \
  -H "Authorization: Bearer admin-tom" \
  -F "file=@build/proof_package.zip"
```

---

### 13. Monitoring Stack Issues (Week 2)

**Symptom:**
```
Prometheus: Target Down
Grafana: No Data
Loki: Connection Refused
Jaeger: Traces not showing
```

**Ursachen & Lösungen:**

**A) Prometheus kann CAP API nicht scrapen**
```bash
# Fehler: Target Down (auf http://localhost:9090/targets)

# Ursache 1: CAP API nicht erreichbar
curl http://cap-verifier-api:8080/metrics
# Lösung: API muss laufen
docker compose up -d cap-verifier-api

# Ursache 2: Falscher Scrape Target in prometheus.yml
# monitoring/prometheus/prometheus.yml:
scrape_configs:
  - job_name: 'cap-verifier'
    static_configs:
      - targets: ['cap-verifier-api:8080']  # ✅ Docker Service Name
      - targets: ['localhost:8080']          # ❌ Falsch in Docker

# Ursache 3: Network Isolation
# Prometheus und CAP API müssen im gleichen Docker Network sein
docker network ls
docker network inspect cap-monitoring
# Lösung: In docker-compose.yml:
networks:
  - cap-monitoring
```

**B) Grafana: "No Data" in Dashboards**
```bash
# Fehler: Panels zeigen "No Data"

# Ursache 1: Prometheus Datasource nicht konfiguriert
# Lösung: Grafana → Configuration → Data Sources → Add Prometheus
# URL: http://prometheus:9090

# Ursache 2: Dashboards nicht auto-provisioned
ls -la monitoring/grafana/provisioning/dashboards/
# Muss enthalten:
# - dashboards.yml
# - cap-verifier-api.json
# - slo-monitoring.json

# Ursache 3: Keine Metriken vorhanden (kein Traffic)
# Lösung: Test-Request senden
curl -X POST http://localhost:8080/verify \
  -H "Authorization: Bearer admin-tom" \
  -H "Content-Type: application/json" \
  -d @examples/verify_request.json

# Dann Prometheus prüfen:
# rate(cap_verifier_requests_total[5m])
```

**C) Loki: "Connection Refused"**
```bash
# Fehler: Loki not reachable at http://loki:3100

# Ursache 1: Loki Container nicht gestartet
docker compose ps loki
# Status sollte "healthy" sein

# Ursache 2: Loki Config Fehler
# Prüfe Logs:
docker compose logs loki | grep -i error

# Häufiger Fehler: v11 schema incompatibility
# Lösung: In monitoring/loki/loki-config.yml:
schema_config:
  configs:
    - from: 2024-01-01
      store: tsdb
      object_store: filesystem
      schema: v13
      index:
        prefix: loki_index_
        period: 24h

# v11 Features deaktivieren:
limits_config:
  allow_structured_metadata: false
```

**D) Promtail: Keine Logs in Loki**
```bash
# Fehler: Loki zeigt keine Logs

# Ursache 1: Promtail kann Docker Socket nicht lesen
ls -la /var/run/docker.sock
# Lösung: Promtail Container braucht Volume Mount:
volumes:
  - /var/run/docker.sock:/var/run/docker.sock:ro

# Ursache 2: Falsche Container Labels
# Promtail filtert nach app=cap-verifier-api Label
docker inspect cap-verifier-api | grep -i label

# Ursache 3: Promtail Scrape Config falsch
# monitoring/promtail/promtail-config.yml:
scrape_configs:
  - job_name: cap-verifier-api
    docker_sd_configs:
      - host: unix:///var/run/docker.sock
    relabel_configs:
      - source_labels: ['__meta_docker_container_label_app']
        regex: 'cap-verifier-api'
        action: keep
```

**E) Jaeger: Keine Traces sichtbar**
```bash
# Fehler: Jaeger UI zeigt keine Traces

# Ursache 1: Jaeger nicht richtig deployed
docker compose ps jaeger
curl http://localhost:14269/  # Health Check

# Ursache 2: CAP API exportiert keine Traces
# Tracing muss im Code aktiviert werden (zukünftige Feature)
# Aktuell: Logs → Traces Correlation via trace_id

# Ursache 3: Sampling Rate zu niedrig
# monitoring/jaeger/jaeger-config.yml:
sampling:
  strategies:
    - type: probabilistic
      param: 1.0  # 100% für Development, 0.01 für Production
```

**F) Container Health Checks schlagen fehl**
```bash
# Fehler: 5/8 healthy statt 8/8

# Prüfe welche Container unhealthy sind
docker compose ps

# Logs des unhealthy Containers:
docker compose logs <service-name>

# Häufige Ursachen:
# - Port nicht erreichbar (Health Check URL falsch)
# - Startup dauert zu lange (start_period zu kurz)
# - Dependencies noch nicht ready

# Lösung: Health Check manuell testen
docker compose exec prometheus wget -O- http://localhost:9090/-/healthy
docker compose exec loki wget -O- http://localhost:3100/ready
docker compose exec grafana wget -O- http://localhost:3000/api/health
```

**Debug-Befehle:**

```bash
# 1. Alle Container-Status prüfen
cd monitoring
docker compose ps

# 2. Test-Script ausführen
./test-monitoring.sh

# 3. Container Logs prüfen
docker compose logs -f <service>

# 4. Prometheus Targets prüfen
open http://localhost:9090/targets

# 5. Grafana Datasources prüfen
open http://localhost:3000/datasources

# 6. Loki Query testen
curl -G 'http://localhost:3100/loki/api/v1/query' \
  --data-urlencode 'query={app="cap-verifier-api"}'

# 7. Jaeger UI öffnen
open http://localhost:16686

# 8. Node Exporter Metrics
curl http://localhost:9100/metrics

# 9. Stack neu starten (Clean Slate)
docker compose down -v
docker compose up -d
```

---

### 14. Rate Limiting Errors

**Symptom:**
```
429 Too Many Requests
X-RateLimit-Remaining: 0
Retry-After: 36
```

**Ursachen & Lösungen:**

**A) 429 Too Many Requests Error**
```bash
# Fehler: HTTP 429 - Too many requests. Please retry after 36 seconds.

# Ursache: Rate Limit überschritten (100 req/min default)

# Lösung 1: Retry-After Header beachten
# Client sollte 36 Sekunden warten (aus Header)

# Lösung 2: Rate Limit erhöhen (für Production)
cargo run --bin cap-verifier-api \
  --rate-limit 1000 \
  --rate-limit-burst 1200

# Lösung 3: Requests batchen/throttlen
# Client-seitig Rate Limiting implementieren
```

**B) Rate Limit zu restriktiv für Production**
```bash
# Problem: 10 req/min für /policy/v2/compile zu niedrig

# Lösung: Per-Endpoint Limits anpassen
# In agent/src/bin/verifier_api.rs:

use crate::api::rate_limit::{rate_limiter_layer, RateLimitConfig};

// Custom Config für Policy Compile
let policy_config = RateLimitConfig {
    requests_per_minute: 50,  # Statt 10
    burst_size: 60,           # Statt 15
};

let policy_limiter = rate_limiter_layer(policy_config);

Router::new()
    .route("/policy/v2/compile", post(handle_policy_compile))
    .layer(policy_limiter);
```

**C) Falsche IP-Adresse extrahiert (hinter Reverse Proxy)**
```bash
# Problem: Rate Limiting basiert auf Proxy IP statt Client IP

# Ursache: X-Forwarded-For Header nicht gesetzt
# Lösung: Nginx/Apache Proxy muss Header setzen

# nginx.conf:
location / {
    proxy_pass http://backend:8080;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Real-IP $remote_addr;
}

# Backend nutzt automatisch X-Forwarded-For
# via SmartIpKeyExtractor (tower_governor)
```

**D) Rate Limit Headers fehlen in Response**
```bash
# Problem: Client kann Remaining Requests nicht sehen

# Ursache: StateInformationMiddleware nicht aktiviert
# Lösung: In agent/src/api/rate_limit.rs bereits implementiert

# Response sollte Header haben:
# X-RateLimit-Limit: 100
# X-RateLimit-Remaining: 95

# Check mit curl:
curl -I http://localhost:8080/healthz
```

**E) Distributed Setup: Rate Limiting nicht synchronisiert**
```bash
# Problem: Multi-Instance Deployment, jede Instance hat eigenen Counter

# Aktuell: In-Memory Token Bucket (nicht distributed)
# Lösung (zukünftig): Redis-Backend für shared state

# Workaround: Load Balancer Sticky Sessions
# nginx.conf:
upstream cap-backend {
    ip_hash;  # Gleiches Client IP → gleicher Backend
    server backend1:8080;
    server backend2:8080;
}
```

**F) Monitoring: Rate Limit Hits nicht sichtbar**
```bash
# Problem: Keine Metrik für Rate Limit Exceeded Events

# Lösung: Custom Prometheus Metrik (zukünftige Feature)
# Geplant: rate_limit_exceeded_total counter

# Workaround: 429 Status in Logs suchen
docker logs cap-verifier-api 2>&1 | grep "429"
journalctl -u cap-verifier | grep "429"

# Oder in Loki:
{app="cap-verifier-api"} |= "429"
```

**Debug-Befehle:**

```bash
# 1. Rate Limit Status prüfen
curl -I http://localhost:8080/verify \
  -H "Authorization: Bearer admin-tom"

# Output:
# X-RateLimit-Limit: 20
# X-RateLimit-Remaining: 19

# 2. Rate Limit auslösen (Test)
for i in {1..25}; do
  curl -X POST http://localhost:8080/verify \
    -H "Authorization: Bearer admin-tom" \
    -H "Content-Type: application/json" \
    -d '{}' -I
done
# Ab Request 21 sollte 429 kommen

# 3. Retry-After Header prüfen
curl -I http://localhost:8080/verify \
  -H "Authorization: Bearer admin-tom" | grep "Retry-After"

# 4. Backend Logs für Rate Limit Events
docker logs cap-verifier-api 2>&1 | grep -i "rate"

# 5. Current Rate Limit Config prüfen
# In Startup Logs:
# "Rate Limiting: 100 req/min (burst: 120)"
```

---

### 15. cap-bundle.v1 Package Errors ⭐

> ⭐ **NEU in v0.11.0:** Troubleshooting für das standardisierte cap-bundle.v1 Package Format

**Symptom:**
```
_meta.json missing
Hash mismatch in bundle
Bundle verification failed
File not found in bundle
```

**Ursachen & Lösungen:**

**A) _meta.json fehlt in exportiertem Bundle**
```bash
# Fehler:
# Error: Bundle metadata file '_meta.json' not found in package directory

# Ursache 1: Bundle wurde mit altem Proof Export erstellt
# Lösung: Bundle neu exportieren mit v0.11.0+
cap-agent proof export \
  --manifest build/manifest.json \
  --proof build/proof.dat \
  --out build/cap-proof \
  --force

# Ursache 2: _meta.json wurde manuell gelöscht
# Lösung: Bundle ist ungültig, neu erstellen

# Ursache 3: Bundle ist Legacy Format (cap-proof.v1.0)
# Lösung: Für Legacy-Bundles ist _meta.json optional
# Verifier hat Fallback-Mechanismus
```

**B) Hash Mismatch in _meta.json**
```bash
# Fehler:
# Error: File hash mismatch for 'manifest.json'
# Expected: 0x1da941f7026bae3cf8b1bcdc3a8e01e76ea678c32ec6bc2c374fb67b3744571f
# Got:      0x2eb052e8390c47c59b3d1a84f56e8f9a5c7a3d2b1f6e4c8a9b0d5e7f3a2c1b0d

# Ursache: Datei wurde nach Bundle-Erstellung modifiziert

# Diagnose:
# 1. Hash aus _meta.json extrahieren
jq -r '.files."manifest.json".hash' build/cap-proof/_meta.json

# 2. Hash der Datei neu berechnen
sha3sum -a 256 build/cap-proof/manifest.json

# 3. Vergleichen
# Falls unterschiedlich → Datei wurde modifiziert

# Lösung 1: Bundle neu erstellen
rm -rf build/cap-proof
cap-agent proof export \
  --manifest build/manifest.json \
  --proof build/proof.dat \
  --out build/cap-proof \
  --force

# Lösung 2: Original-Dateien wiederherstellen (falls Backup vorhanden)
cp build/manifest.json build/cap-proof/manifest.json
```

**C) Bundle-Verifikation schlägt fehl (verifier run)**
```bash
# Fehler:
# Bundle-Verifikation fehlgeschlagen
# Status: fail

# Ursache 1: Proof/Manifest Hash-Mismatch (siehe B)
# Lösung: Siehe oben

# Ursache 2: Policy nicht gefunden im Backend
# Lösung: Policy kompilieren
cap-agent policy compile --file policy.lksg.v1.yml

# Ursache 3: Proof-Backend inkompatibel
# _meta.json zeigt backend: "mock"
# Manifest zeigt proof_backend: "zkvm"
# Lösung: Backend muss übereinstimmen

# Ursache 4: Mandatory file fehlt im Bundle
# _meta.json listet Datei mit "optional": false
# Aber Datei existiert nicht
# Lösung: Bundle neu erstellen mit allen Dateien
```

**D) proof.dat Format Errors**
```bash
# Fehler:
# Error: Failed to parse proof.dat
# Invalid CAPZ format

# Ursache 1: proof.dat ist kein gültiges Base64
# Diagnose:
base64 -d < build/cap-proof/proof.dat > /tmp/proof_decoded.json
cat /tmp/proof_decoded.json | jq '.'

# Falls JSON-Parse-Error:
# → proof.dat ist beschädigt

# Lösung: Proof neu erstellen
cap-agent proof build \
  --manifest build/manifest.json \
  --policy policy.lksg.v1.yml

# Ursache 2: Proof wurde mit älterem Backend erstellt
# Lösung: Backend-Kompatibilität prüfen
jq -r '.type' build/cap-proof/proof.json
# Sollte matchen mit _meta.json proof_units[].backend
```

**E) Bundle Schema Version Mismatch**
```bash
# Fehler:
# Error: Unsupported bundle schema version

# Ursache: Verifier Version zu alt für Bundle
# _meta.json zeigt "schema": "cap-bundle.v2"
# Verifier unterstützt nur "cap-bundle.v1"

# Diagnose:
jq -r '.schema' build/cap-proof/_meta.json

# Lösung 1: Verifier aktualisieren
cargo build --release
./target/release/cap-agent verifier run --package build/cap-proof

# Lösung 2: Bundle mit älterer Schema-Version erstellen
# (Nicht empfohlen, nur für Legacy-Systeme)
```

**F) Files HashMap Inkonsistenz**
```bash
# Fehler:
# Error: File listed in _meta.json but not found on disk

# Symptom: _meta.json listet "registry.json", aber Datei fehlt

# Diagnose:
# 1. Files aus _meta.json auflisten
jq -r '.files | keys[]' build/cap-proof/_meta.json

# 2. Tatsächliche Dateien auflisten
ls -1 build/cap-proof/

# 3. Diff prüfen
comm -3 <(jq -r '.files | keys[]' _meta.json | sort) <(ls -1 | sort)

# Lösung 1: Fehlende Datei hinzufügen
# Falls optional=false → Bundle ist ungültig, neu erstellen

# Lösung 2: _meta.json aktualisieren (NICHT EMPFOHLEN!)
# Hashes werden ungültig
```

**G) ProofUnit Metadata Fehler**
```bash
# Fehler:
# Error: ProofUnit missing required field 'policy_id'

# Ursache: _meta.json proof_units array fehlerhaft

# Diagnose:
jq '.proof_units[]' build/cap-proof/_meta.json

# Erwartete Felder:
# - id: "default"
# - manifest_file: "manifest.json"
# - proof_file: "proof.dat"
# - policy_id: "LkSG Demo Policy"
# - policy_hash: "0xabc123..."
# - backend: "mock"

# Lösung: Bundle neu erstellen
# Policy-Informationen werden automatisch aus Manifest extrahiert
cap-agent proof export \
  --manifest build/manifest.json \
  --proof build/proof.dat \
  --out build/cap-proof \
  --force
```

**H) Bundle-Transport Fehler (Download/Upload)**
```bash
# Fehler:
# Error: Checksum mismatch after download

# Ursache: Bundle wurde während Transport beschädigt

# Diagnose: Gesamthash des Bundles prüfen
find build/cap-proof -type f -exec sha3sum {} \; | sort | sha3sum

# Lösung 1: Bundle als TAR/ZIP archivieren
tar -czf cap-proof.tar.gz build/cap-proof/
# Vor Upload Checksum erstellen
sha3sum cap-proof.tar.gz > cap-proof.tar.gz.sha3

# Nach Download Checksum prüfen
sha3sum -c cap-proof.tar.gz.sha3

# Lösung 2: SFTP/SCP mit Integrity Check verwenden
scp -C cap-proof.tar.gz user@server:/path/
# -C = Compression während Transfer
```

**I) Legacy Bundle Migration**
```bash
# Problem: Altes cap-proof.v1.0 Bundle zu cap-bundle.v1 migrieren

# Symptom: Bundle hat keine _meta.json

# Migration-Workflow:
# 1. Manifest extrahieren
cp old-bundle/manifest.json /tmp/manifest.json

# 2. Proof extrahieren
cp old-bundle/proof.dat /tmp/proof.dat

# 3. Neues Bundle erstellen
cap-agent proof export \
  --manifest /tmp/manifest.json \
  --proof /tmp/proof.dat \
  --out new-bundle/ \
  --force

# 4. Alte optionale Dateien kopieren (falls vorhanden)
if [ -f old-bundle/timestamp.tsr ]; then
  cp old-bundle/timestamp.tsr new-bundle/
fi

# 5. _meta.json wird automatisch mit allen Dateien erstellt
ls -la new-bundle/_meta.json

# Verifikation:
cap-agent verifier run --package new-bundle/
```

**J) Bundle Size Explosion**
```bash
# Problem: Bundle-Größe unerwartet groß (> 100 MB)

# Diagnose: Dateigrößen prüfen
jq -r '.files | to_entries[] | "\(.key): \(.value.size) bytes"' \
  build/cap-proof/_meta.json | sort -t: -k2 -n

# Häufige Ursachen:
# 1. Große proof.dat (ZK-Proofs können mehrere MB sein)
# 2. Große registry.json (viele Einträge)
# 3. Große Audit-Logs

# Lösung 1: Proof-Kompression (zukünftige Feature)
# Aktuell: Keine Kompression in proof.dat

# Lösung 2: Registry extern referenzieren
# Statt gesamte Registry: Nur relevante Einträge
cap-agent proof export \
  --manifest build/manifest.json \
  --proof build/proof.dat \
  --out build/cap-proof \
  # Registry nicht inkludieren

# Lösung 3: Bundle als .tar.gz archivieren (20-40% Kompression)
tar -czf cap-proof.tar.gz build/cap-proof/
ls -lh cap-proof.tar.gz
```

**Debug-Befehle:**

```bash
# 1. Bundle-Struktur validieren
cap-agent verifier run --package build/cap-proof/

# 2. _meta.json Schema prüfen
jq '.' build/cap-proof/_meta.json

# Muss enthalten:
# - "schema": "cap-bundle.v1"
# - "bundle_id": "bundle-..."
# - "created_at": "2025-..."
# - "files": { ... }
# - "proof_units": [ ... ]

# 3. Alle Datei-Hashes neu berechnen (zur Verifikation)
cd build/cap-proof
for file in $(jq -r '.files | keys[]' _meta.json); do
  echo "$file:"
  echo "  Expected: $(jq -r ".files.\"$file\".hash" _meta.json)"
  echo "  Got:      0x$(sha3sum -a 256 "$file" | cut -d' ' -f1)"
done

# 4. ProofUnit Konfiguration prüfen
jq '.proof_units[]' build/cap-proof/_meta.json

# 5. Bundle-Größe analysieren
du -sh build/cap-proof/
du -h build/cap-proof/* | sort -h

# 6. Legacy-Bundle erkennen
if [ ! -f build/cap-proof/_meta.json ]; then
  echo "Legacy Bundle (cap-proof.v1.0)"
else
  echo "Modern Bundle ($(jq -r '.schema' build/cap-proof/_meta.json))"
fi

# 7. Bundle-Integrität mit tar prüfen
tar -tzf cap-proof.tar.gz | grep _meta.json
# Falls _meta.json fehlt → Legacy oder beschädigt
```

**Best Practices zur Fehlervermeidung:**

1. **Immer mit --force exportieren:**
   ```bash
   cap-agent proof export ... --force
   # Überschreibt existierendes Verzeichnis, verhindert partielle Bundles
   ```

2. **Bundle sofort nach Export verifizieren:**
   ```bash
   cap-agent proof export ... && \
   cap-agent verifier run --package build/cap-proof/
   ```

3. **Bundles als Tarball archivieren:**
   ```bash
   tar -czf cap-proof-$(date +%Y%m%d).tar.gz build/cap-proof/
   sha3sum cap-proof-$(date +%Y%m%d).tar.gz > cap-proof-$(date +%Y%m%d).tar.gz.sha3
   ```

4. **Niemals _meta.json manuell editieren:**
   - Hashes werden ungültig
   - Verifikation schlägt fehl
   - Bundle neu erstellen statt editieren

5. **Bundle-Versioning:**
   ```bash
   # Bundle mit Timestamp versionen
   BUNDLE_DIR="cap-proof-$(date +%Y%m%d-%H%M%S)"
   cap-agent proof export --out "$BUNDLE_DIR"
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
