# 05 - Deployment & Betrieb

## 📖 Über dieses Kapitel

Nachdem Sie nun wissen, **was** das System macht, **wie** es aufgebaut ist, **welche Teile** es hat und **wie man es bedient**, erklärt dieses Kapitel **wie man es in Betrieb nimmt**.

**Für wen ist dieses Kapitel?**
- **Management:** Die Deployment-Optionen-Übersicht mit Kosten/Nutzen
- **IT-Leiter:** Entscheidungshilfe für die richtige Deployment-Methode
- **IT-Administratoren:** Detaillierte Installations- und Betriebsanleitungen
- **DevOps-Engineers:** Kubernetes, Docker, CI/CD-Konfigurationen

**Was Sie lernen werden:**
1. Welche 4 Deployment-Optionen es gibt (und welche für Sie passt)
2. Wie man das System installiert
3. Wie man es überwacht und wartet
4. Wie man Backups erstellt

**Analogie:** Dies ist die **Installationsanleitung** - wie bei einem Gerät die Anleitung zum Aufbauen und Anschließen.

---

## 👔 Für Management: Welche Installations-Methode?

Das System kann auf **5 verschiedene Arten** installiert werden. Hier eine Entscheidungshilfe:

| Methode | Für wen? | Kosten | Komplexität | Skalierbarkeit |
|---------|----------|--------|-------------|----------------|
| **1. Desktop App** | Einzelnutzer, Offline | € | ⭐ Sehr Einfach | ⭐ Single User |
| **2. Binary** | Kleine Firmen, Tests | € | ⭐ Einfach | ⭐ Limitiert |
| **3. Docker** | Mittlere Firmen | €€ | ⭐⭐ Mittel | ⭐⭐ Gut |
| **4. Kubernetes** | Konzerne, Cloud | €€€€ | ⭐⭐⭐⭐ Komplex | ⭐⭐⭐⭐ Exzellent |
| **5. Systemd** | Linux-Server | € | ⭐⭐ Mittel | ⭐⭐ Gut |

### Empfehlungen nach Unternehmensgröße:

**Einzelperson / Freelancer:**
- ✅ **Desktop App** (Tauri)
- Keine Installation nötig, komplett offline
- Alle Daten bleiben lokal
- *Analogie:* Wie ein Textverarbeitungsprogramm - einfach starten und nutzen

**Kleine Unternehmen (< 50 Mitarbeiter):**
- ✅ **Desktop App** oder **Binary/Systemd** auf einem Linux-Server
- Einfach, kostengünstig, ausreichend
- *Analogie:* Wie ein Desktop-PC statt Server-Rack

**Mittlere Unternehmen (50-500 Mitarbeiter):**
- ✅ **Docker** mit Docker Compose
- Balance zwischen Einfachheit und Professionalität
- Einfaches Update-Management
- *Analogie:* Wie ein kleines Server-Rack mit wenigen Geräten

**Große Unternehmen / Konzerne (> 500 Mitarbeiter):**
- ✅ **Kubernetes** in der Cloud oder On-Premise
- Hochverfügbarkeit, automatische Skalierung
- Integration in bestehende K8s-Infrastruktur
- *Analogie:* Wie ein Rechenzentrum mit automatischer Verwaltung

### Kosten-Vergleich (geschätzte Jahreskosten):

| Methode | Hardware | Software | Wartung | Gesamt/Jahr |
|---------|----------|----------|---------|-------------|
| Binary | 1 Server (~2.000€) | Kostenlos | 5 PT (~10.000€) | ~12.000€ |
| Docker | 1-2 Server (~4.000€) | Kostenlos | 8 PT (~16.000€) | ~20.000€ |
| Kubernetes | Cloud oder 3+ Server (~15.000€) | Kostenlos | 20 PT (~40.000€) | ~55.000€ |

**Hinweis:** PT = Personentage (angenommener Tagessatz: 2.000€)

---

## 🔐 Enterprise Security Requirements (NEU - Dezember 2025)

Vor dem Production-Deployment sollten folgende Security-Maßnahmen beachtet werden:

### Enterprise Readiness Checklist

| Anforderung | Status | Aktion |
|-------------|--------|--------|
| CORS explizit konfigurieren | 🔴 Offen | `allow_origin(Any)` → Explizite Origins |
| Security Headers aktivieren | 🔴 Offen | HSTS, CSP, X-Frame-Options hinzufügen |
| Dev-Token entfernen | 🔴 Offen | `admin-tom` aus auth.rs entfernen |
| Production-Keys generieren | 🔴 Offen | Mock-Keys durch echte RSA-Keys ersetzen |
| TLS aktivieren | ✅ Bereit | `--tls --tls-cert --tls-key` Flags nutzen |
| Rate Limiting konfigurieren | ⚠️ Teilweise | `--rate-limit` Flag nutzen |
| Graceful Shutdown | 🔴 Offen | In Kubernetes: terminationGracePeriodSeconds |

### Mindest-Anforderungen für Production

```bash
# Production-Start mit Security-Optionen
cargo run --release --bin cap-verifier-api -- \
  --bind 0.0.0.0:8443 \
  --tls \
  --tls-cert /certs/server.crt \
  --tls-key /certs/server.key \
  --rate-limit 100 \
  --rate-limit-burst 120
```

### Aktueller Enterprise Readiness Score

**57%** - Nach Abschluss der Hardening-Roadmap: **95%**

**📋 Details:** [SECURITY_AUDIT_REPORT.md](../../security/SECURITY_AUDIT_REPORT.md) | [ROADMAP_ENTERPRISE.md](../../ROADMAP_ENTERPRISE.md)

---

## Deployment-Optionen

Der LsKG-Agent kann auf verschiedene Arten deployed werden:

1. **Desktop App (Tauri)** - Native App für Windows/macOS/Linux (offline, v0.12.0)
2. **Binary Deployment** - Direktes Ausführen des Rust-Binaries (einfachste Methode)
3. **Docker Container** - Containerisierte Anwendung (empfohlen für Produktion)
4. **Kubernetes** - Orchestrierte Container in einem Cluster (für Enterprise)
5. **Systemd Service** - Systemd-managed Service auf Linux (klassischer Ansatz)

---

## 1. Desktop App Deployment (Tauri 2.0) - NEU in v0.12.0

> ⭐ **NEU in v0.12.0:** Native Desktop-Anwendung für komplett offline Compliance-Workflow

Die Desktop App ist die einfachste Deployment-Option und erfordert keine Server-Infrastruktur.

### Voraussetzungen

**Für Endbenutzer:**
- Windows 10/11, macOS 10.15+, oder Linux (Ubuntu 20.04+)
- 4 GB RAM minimum
- 100 MB Festplatte für die App
- Keine Internetverbindung erforderlich

**Für Build (Entwickler):**
- Rust 1.75+
- Node.js 18+
- Tauri CLI (`cargo install tauri-cli`)

### Installation (Endbenutzer)

**macOS:**
```bash
# Download der .dmg Datei
# Doppelklick auf .dmg
# App in Applications-Ordner ziehen
# Starten via Launchpad oder Spotlight
```

**Windows:**
```bash
# Download der .msi oder .exe Installer
# Doppelklick zum Installieren
# Starten via Startmenü
```

**Linux:**
```bash
# Download der .AppImage oder .deb
# AppImage: chmod +x ./desktop-proofer.AppImage && ./desktop-proofer.AppImage
# Debian: sudo dpkg -i ./desktop-proofer.deb
```

### Build from Source

```bash
# Repository klonen
git clone https://github.com/your-org/LsKG-Agent.git
cd LsKG-Agent

# Frontend Dependencies installieren
cd webui
npm install

# Tauri App bauen
cd ../src-tauri
cargo build --release

# Binary ist verfügbar unter:
# macOS: target/release/desktop-proofer
# Windows: target/release/desktop-proofer.exe
# Linux: target/release/desktop-proofer
```

### App Distribution erstellen

```bash
# Für alle Plattformen (auf jeweiliger Plattform ausführen)
cd src-tauri
cargo tauri build

# Output:
# macOS: src-tauri/target/release/bundle/dmg/
# Windows: src-tauri/target/release/bundle/msi/
# Linux: src-tauri/target/release/bundle/appimage/
```

### Konfiguration

Die Desktop App ist **zero-config** - keine Konfigurationsdateien nötig.

**Daten werden gespeichert in:**
- Workspace: Vom Benutzer gewählter Ordner (z.B. `~/cap-workspace/`)
- Pro Projekt: `{workspace}/{project-name}/`
  - `input/` - CSV-Dateien, Policy
  - `build/` - Commitments, Manifest, Proof, Audit Log
  - `export/` - Exportierte Bundles

### Projekt-Struktur (automatisch erstellt)

```
~/cap-workspace/
└── cap-proof-2025-11-27-xyz123/
    ├── input/
    │   ├── suppliers.csv
    │   ├── ubos.csv
    │   └── policy.yml
    ├── build/
    │   ├── commitments.json
    │   ├── manifest.json
    │   ├── proof.capz
    │   └── audit.jsonl         # V1.0 Audit Trail
    └── export/
        └── cap-bundle-2025-11-27_120000.zip
```

### Audit Trail

Jedes Projekt enthält einen manipulationssicheren Audit Trail in `build/audit.jsonl`:

```json
{"seq":1,"ts":"2025-11-27T10:00:00Z","event":"project_created","details":{"project_name":"cap-proof-xyz"},"prev_digest":"0x0","digest":"0x1a2b3c..."}
{"seq":2,"ts":"2025-11-27T10:01:00Z","event":"csv_imported","details":{"file_type":"suppliers","row_count":150},"prev_digest":"0x1a2b3c...","digest":"0x4d5e6f..."}
```

**Hash-Chain:** Jeder Eintrag referenziert den Hash des vorherigen Eintrags → Manipulationen sofort erkennbar.

### Desktop App vs. Server Deployment

| Aspekt | Desktop App | Server (API) |
|--------|-------------|--------------|
| **Installation** | App Download | Server Setup |
| **Netzwerk** | Offline | Netzwerk erforderlich |
| **Daten** | Lokal auf Rechner | Server-seitig |
| **Multi-User** | Single User | Multi-Tenant |
| **Audit Trail** | Lokal (audit.jsonl) | Zentral |
| **Updates** | Manuell/Auto-Update | Rolling Deployment |
| **Kosten** | 0€ (kein Server) | Server + Wartung |

### Troubleshooting Desktop App

**Problem: App startet nicht auf macOS**
```
"desktop-proofer" kann nicht geöffnet werden, da es von einem nicht verifizierten Entwickler stammt
```
**Lösung:**
```bash
# Rechtsklick auf App → Öffnen → Bestätigen
# Oder in Systemeinstellungen → Sicherheit → "Trotzdem öffnen"
```

**Problem: CSV-Import schlägt fehl**
```
Fehler beim Importieren: Invalid UTF-8
```
**Lösung:** CSV-Datei in UTF-8 kodieren (nicht ANSI/Windows-1252)

**Problem: Audit Trail zeigt Hash-Chain-Fehler**
```
Error: Hash chain broken at seq 3
```
**Lösung:** audit.jsonl wurde manuell bearbeitet → Neues Projekt erstellen

---

## 2. Binary Deployment

### Voraussetzungen

**System:**
- Linux (x86_64) oder macOS (x86_64/ARM64)
- 4 GB RAM minimum, 8 GB empfohlen
- 2 CPU Cores minimum, 4 Cores empfohlen
- 10 GB Festplatte

**Software:**
- Rust 1.75+ (für Build)
- OpenSSL 1.1+ (für TLS)
- SQLite 3.35+ (optional, für Registry)

### Build from Source

```bash
# Repository klonen
git clone https://github.com/your-org/LsKG-Agent.git
cd LsKG-Agent/agent

# Dependencies installieren
cargo fetch

# Release Build erstellen
cargo build --release

# Binary ist verfügbar unter:
# target/release/cap (CLI)
# target/release/cap-verifier-api (API Server)
```

### Installation

```bash
# Binary in System-Pfad installieren
sudo cp target/release/cap /usr/local/bin/
sudo cp target/release/cap-verifier-api /usr/local/bin/

# Executable permissions setzen
sudo chmod +x /usr/local/bin/cap
sudo chmod +x /usr/local/bin/cap-verifier-api

# Verifizieren
cap --version
cap-verifier-api --version
```

### Konfiguration

```bash
# Konfigurationsverzeichnis erstellen
sudo mkdir -p /etc/cap-verifier
sudo mkdir -p /var/lib/cap-verifier
sudo mkdir -p /var/log/cap-verifier

# Konfigurationsdateien kopieren
sudo cp config/app.yaml /etc/cap-verifier/
sudo cp config/auth.yaml /etc/cap-verifier/
sudo cp config/tls.yaml /etc/cap-verifier/

# Permissions setzen
sudo chown -R cap-verifier:cap-verifier /etc/cap-verifier
sudo chown -R cap-verifier:cap-verifier /var/lib/cap-verifier
sudo chown -R cap-verifier:cap-verifier /var/log/cap-verifier
```

#### Environment Variables

**Policy Store Configuration:**

```bash
# POLICY_STORE_BACKEND - Backend selection
# Options: memory (default) | sqlite
export POLICY_STORE_BACKEND=sqlite

# POLICY_DB_PATH - SQLite database file path
# Default: build/policies.sqlite
# Production: /var/lib/cap-verifier/policies.sqlite
export POLICY_DB_PATH=/var/lib/cap-verifier/policies.sqlite
```

**Anwendungsfälle:**

| Umgebung | Backend | DB Path | Beschreibung |
|----------|---------|---------|-------------|
| **Development** | `memory` | - | Schnell, keine Persistenz, ideal für Tests |
| **Production** | `sqlite` | `/var/lib/cap-verifier/policies.sqlite` | Persistent, ACID-konform, für echte Workloads |
| **CI/CD** | `memory` | - | Schnelle Integration-Tests ohne Setup |
| **Docker** | `sqlite` | `/data/policies.sqlite` (Volume) | Persistent mit Volume-Mount |
| **Kubernetes** | `sqlite` | `/data/policies.sqlite` (PVC) | Persistent mit PersistentVolumeClaim |

**Beispiel: Development (InMemory)**
```bash
# Keine Konfiguration nötig - InMemory ist Standard
cargo run --bin cap-verifier-api --bind 127.0.0.1:8080
```

**Beispiel: Production (SQLite)**
```bash
# Mit Environment Variables
POLICY_STORE_BACKEND=sqlite \
POLICY_DB_PATH=/var/lib/cap-verifier/policies.sqlite \
cargo run --bin cap-verifier-api \
  --bind 0.0.0.0:8443 \
  --tls \
  --tls-cert /etc/cap-verifier/certs/server.crt \
  --tls-key /etc/cap-verifier/certs/server.key
```

**Systemd Service mit Environment Variables:**

Fügen Sie folgende Zeilen in der `[Service]` Sektion hinzu:

```ini
[Service]
# ... (andere Optionen) ...

# Policy Store Configuration
Environment="POLICY_STORE_BACKEND=sqlite"
Environment="POLICY_DB_PATH=/var/lib/cap-verifier/policies.sqlite"

# ... (ExecStart, etc.) ...
```

**Docker Environment Variables:**

```dockerfile
# In Dockerfile
ENV POLICY_STORE_BACKEND=sqlite
ENV POLICY_DB_PATH=/data/policies.sqlite
```

Oder bei `docker run`:
```bash
docker run -d \
  -e POLICY_STORE_BACKEND=sqlite \
  -e POLICY_DB_PATH=/data/policies.sqlite \
  -v policy-data:/data \
  cap-verifier-api:latest
```

**Kubernetes ConfigMap/Environment:**

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: cap-verifier-config
data:
  POLICY_STORE_BACKEND: "sqlite"
  POLICY_DB_PATH: "/data/policies.sqlite"
```

Verwendung in Deployment:
```yaml
spec:
  containers:
  - name: cap-verifier-api
    envFrom:
    - configMapRef:
        name: cap-verifier-config
```

**Wichtige Hinweise:**

- ⚠️ **Backup:** SQLite-Datenbank sollte regelmäßig gesichert werden
- ⚠️ **Permissions:** Datenbank-Datei muss für Prozess-User beschreibbar sein
- ⚠️ **Volume:** Bei Docker/Kubernetes immer Volume/PVC für SQLite verwenden
- ⚠️ **WAL Mode:** SQLite nutzt Write-Ahead Logging für concurrent access
- ⚠️ **Migration:** Bei Backend-Wechsel müssen Policies manuell migriert werden

### Systemd Service

**Service-Datei erstellen:**
```bash
sudo nano /etc/systemd/system/cap-verifier.service
```

**Inhalt:**
```ini
[Unit]
Description=CAP Verifier API Server
After=network.target

[Service]
Type=simple
User=cap-verifier
Group=cap-verifier
WorkingDirectory=/var/lib/cap-verifier
ExecStart=/usr/local/bin/cap-verifier-api \
    --config /etc/cap-verifier/app.yaml \
    --tls \
    --tls-cert /etc/cap-verifier/certs/server.crt \
    --tls-key /etc/cap-verifier/certs/server.key
Restart=on-failure
RestartSec=10
StandardOutput=journal
StandardError=journal

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/cap-verifier /var/log/cap-verifier

[Install]
WantedBy=multi-user.target
```

**Service aktivieren:**
```bash
sudo systemctl daemon-reload
sudo systemctl enable cap-verifier
sudo systemctl start cap-verifier
sudo systemctl status cap-verifier
```

**Logs ansehen:**
```bash
sudo journalctl -u cap-verifier -f
```

---

## 2. Docker Deployment

### Dockerfile

Das Projekt enthält bereits ein Multi-Stage Dockerfile:

**agent/Dockerfile:**
```dockerfile
# Stage 1: Build
FROM rust:1.75-bullseye AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    musl-tools \
    && rm -rf /var/lib/apt/lists/*

# Add musl target
RUN rustup target add x86_64-unknown-linux-musl

# Create app directory
WORKDIR /app

# Copy source code
COPY . .

# Build release binary
RUN cargo build --release --target x86_64-unknown-linux-musl

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 cap-verifier

# Copy binary from builder
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/cap-verifier-api /usr/local/bin/

# Create directories
RUN mkdir -p /config /certs /data && \
    chown -R cap-verifier:cap-verifier /config /certs /data

# Switch to non-root user
USER cap-verifier

# Expose ports
EXPOSE 8080 8443

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/healthz || exit 1

# Start server
CMD ["cap-verifier-api", "--config", "/config/app.yaml"]
```

### Docker Build

```bash
# Im agent/ Verzeichnis
docker build -t cap-agent:0.11.0 .

# Tag für Registry
docker tag cap-agent:0.11.0 registry.example.com/cap-agent:0.11.0

# Push zu Registry
docker push registry.example.com/cap-agent:0.11.0
```

### Docker Run

**HTTP Mode (Development):**
```bash
docker run -d \
  --name cap-verifier \
  -p 8080:8080 \
  -v $(pwd)/config:/config:ro \
  -v $(pwd)/data:/data \
  cap-agent:0.11.0
```

**HTTPS Mode (Production):**
```bash
docker run -d \
  --name cap-verifier \
  -p 8443:8443 \
  -v $(pwd)/config:/config:ro \
  -v $(pwd)/certs:/certs:ro \
  -v $(pwd)/data:/data \
  -e TLS_MODE=tls \
  cap-agent:0.11.0
```

### Docker Compose

**docker-compose.yml:**
```yaml
version: '3.8'

services:
  cap-verifier:
    image: cap-agent:0.11.0
    container_name: cap-verifier
    restart: unless-stopped
    ports:
      - "8443:8443"
    volumes:
      - ./config:/config:ro
      - ./certs:/certs:ro
      - cap-data:/data
    environment:
      - TLS_MODE=tls
      - RUST_LOG=info
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/healthz"]
      interval: 30s
      timeout: 3s
      retries: 3
      start_period: 5s
    networks:
      - cap-network

  prometheus:
    image: prom/prometheus:latest
    container_name: prometheus
    restart: unless-stopped
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
    networks:
      - cap-network

  grafana:
    image: grafana/grafana:latest
    container_name: grafana
    restart: unless-stopped
    ports:
      - "3000:3000"
    volumes:
      - grafana-data:/var/lib/grafana
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    networks:
      - cap-network

volumes:
  cap-data:
  prometheus-data:
  grafana-data:

networks:
  cap-network:
    driver: bridge
```

**Starten:**
```bash
docker-compose up -d
```

**Logs:**
```bash
docker-compose logs -f cap-verifier
```

**Stoppen:**
```bash
docker-compose down
```

---

## 2.5 WebUI Deployment (v0.11.0)

> ⭐ **NEU in v0.11.0:** React-basierte Web-Oberfläche für Proof-Upload und -Verifikation

Die WebUI bietet eine benutzerfreundliche grafische Oberfläche für nicht-technische Benutzer. Sie kommuniziert mit dem REST API Backend über HTTP/HTTPS.

**Technology Stack:**
- React 18.x + TypeScript 5.x
- Vite 5.x (Build Tool)
- TailwindCSS 3.x (Styling)
- Axios 1.x (HTTP Client)

### Voraussetzungen

**System:**
- Node.js 18+ (LTS empfohlen)
- npm 9+ oder yarn 1.22+

### Development Setup

**Installation:**
```bash
# Im webui/ Verzeichnis
cd webui
npm install
```

**Development Server starten:**
```bash
# Terminal 1: Backend API
cd agent
cargo run --bin cap-verifier-api

# Terminal 2: WebUI Dev Server
cd webui
npm run dev

# Browser öffnen: http://localhost:5173
```

**Wichtige Konfiguration (Development):**
- API URL: `http://localhost:8080` (in `src/App.tsx`)
- Bearer Token: `admin-tom` (Development-only, hardcoded)
- CORS: Backend erlaubt localhost:5173

### Production Build

**Build erstellen:**
```bash
cd webui
npm run build

# Output: webui/dist/
# Enthält: index.html, assets/*.js, assets/*.css
```

**Build-Optionen in `vite.config.ts`:**
```typescript
export default defineConfig({
  build: {
    outDir: 'dist',
    sourcemap: true,
    minify: 'terser',
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['react', 'react-dom'],
          axios: ['axios'],
        }
      }
    }
  }
})
```

### Deployment-Optionen

#### Option 1: Nginx (Empfohlen für Production)

**nginx.conf:**
```nginx
server {
    listen 80;
    server_name verifier-ui.example.com;

    # Redirect HTTP to HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name verifier-ui.example.com;

    # SSL Configuration
    ssl_certificate /etc/letsencrypt/live/verifier-ui.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/verifier-ui.example.com/privkey.pem;

    # Root directory (WebUI dist/)
    root /var/www/cap-webui;
    index index.html;

    # SPA routing (alle Requests zu index.html)
    location / {
        try_files $uri $uri/ /index.html;
    }

    # API Proxy (Backend API)
    location /api/ {
        proxy_pass https://verifier-api.example.com/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Static assets caching
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf|eot)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    # Security Headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
}
```

**Deployment:**
```bash
# Build erstellen
cd webui && npm run build

# Zu Server kopieren
scp -r dist/* user@server:/var/www/cap-webui/

# Nginx neu laden
sudo systemctl reload nginx
```

#### Option 2: Docker (Static File Server)

**webui/Dockerfile:**
```dockerfile
# Stage 1: Build
FROM node:18-alpine AS builder

WORKDIR /app

# Install dependencies
COPY package*.json ./
RUN npm ci

# Copy source
COPY . .

# Build production bundle
RUN npm run build

# Stage 2: Serve with nginx
FROM nginx:alpine

# Copy built files
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy custom nginx config
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Expose port
EXPOSE 80

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget --quiet --tries=1 --spider http://localhost/ || exit 1

CMD ["nginx", "-g", "daemon off;"]
```

**Build und Run:**
```bash
# Build Image
docker build -t cap-webui:0.11.0 -f webui/Dockerfile webui/

# Run Container
docker run -d \
  --name cap-webui \
  -p 80:80 \
  cap-webui:0.11.0
```

#### Option 3: Docker Compose (WebUI + Backend)

**docker-compose.webui.yml:**
```yaml
version: '3.8'

services:
  cap-verifier-api:
    image: cap-agent:0.11.0
    container_name: cap-verifier-api
    restart: unless-stopped
    ports:
      - "8443:8443"
    volumes:
      - ./config:/config:ro
      - ./certs:/certs:ro
      - cap-data:/data
    environment:
      - TLS_MODE=tls
      - RUST_LOG=info
      - POLICY_STORE_BACKEND=sqlite
      - POLICY_DB_PATH=/data/policies.sqlite
    networks:
      - cap-network

  cap-webui:
    image: cap-webui:0.11.0
    container_name: cap-webui
    restart: unless-stopped
    ports:
      - "80:80"
    depends_on:
      - cap-verifier-api
    networks:
      - cap-network

volumes:
  cap-data:

networks:
  cap-network:
    driver: bridge
```

**Starten:**
```bash
docker-compose -f docker-compose.webui.yml up -d
```

#### Option 4: Kubernetes Deployment

**k8s/webui-deployment.yaml:**
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cap-webui
  namespace: cap-system
  labels:
    app: cap-webui
spec:
  replicas: 2
  selector:
    matchLabels:
      app: cap-webui
  template:
    metadata:
      labels:
        app: cap-webui
    spec:
      containers:
      - name: cap-webui
        image: registry.example.com/cap-webui:0.11.0
        imagePullPolicy: IfNotPresent
        ports:
        - containerPort: 80
          name: http
          protocol: TCP
        resources:
          requests:
            cpu: 100m
            memory: 128Mi
          limits:
            cpu: 500m
            memory: 512Mi
        livenessProbe:
          httpGet:
            path: /
            port: 80
            scheme: HTTP
          initialDelaySeconds: 5
          periodSeconds: 30
          timeoutSeconds: 3
          successThreshold: 1
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /
            port: 80
            scheme: HTTP
          initialDelaySeconds: 3
          periodSeconds: 10
          timeoutSeconds: 3
          successThreshold: 1
          failureThreshold: 3
---
apiVersion: v1
kind: Service
metadata:
  name: cap-webui-svc
  namespace: cap-system
  labels:
    app: cap-webui
spec:
  type: ClusterIP
  selector:
    app: cap-webui
  ports:
  - name: http
    port: 80
    targetPort: 80
    protocol: TCP
---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: cap-webui-ingress
  namespace: cap-system
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
spec:
  ingressClassName: nginx
  tls:
  - hosts:
    - ui.verifier.example.com
    secretName: cap-webui-tls
  rules:
  - host: ui.verifier.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: cap-webui-svc
            port:
              number: 80
```

**Anwenden:**
```bash
kubectl apply -f k8s/webui-deployment.yaml
```

### Konfiguration (Production)

**Environment-spezifische Builds:**

Erstellen Sie `.env.production` im `webui/` Verzeichnis:

```bash
# API Endpoint (Production)
VITE_API_URL=https://api.verifier.example.com

# OAuth2 Settings (Production)
VITE_OAUTH_ENABLED=true
VITE_OAUTH_ISSUER=https://auth.example.com
```

Dann Build mit Production-Environment:
```bash
npm run build
```

**Wichtige Sicherheitshinweise:**

⚠️ **Development Token entfernen:**
- Der hardcoded Token `admin-tom` in `src/App.tsx` MUSS für Production entfernt werden
- OAuth2 Client Credentials Flow implementieren
- JWT Token Management mit Refresh Logic

⚠️ **CORS Configuration:**
- Backend CORS auf spezifische Origins beschränken (nicht `allow_origin(Any)`)
- Nur https:// Origins in Production erlauben

⚠️ **CSP Headers:**
- Content Security Policy in nginx/Apache konfigurieren
- Restrict script/style sources

### Troubleshooting

**Problem: CORS Errors**
```
Access to XMLHttpRequest at 'http://localhost:8080/proof/upload' from origin 'http://localhost:5173' has been blocked by CORS policy
```

**Lösung:** Backend CORS Middleware prüfen (`agent/src/bin/verifier_api.rs`)
```rust
let cors = CorsLayer::new()
    .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
    .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
    .allow_headers([AUTHORIZATION, CONTENT_TYPE]);
```

**Problem: 401 Unauthorized bei Upload**

**Lösung:** Bearer Token in WebUI korrekt gesetzt?
```typescript
// In src/App.tsx
capApiClient.setBearerToken('admin-tom'); // Development
// oder
capApiClient.setBearerToken(yourOAuth2Token); // Production
```

**Problem: Build Fehler "Module not found"**

**Lösung:** Dependencies neu installieren
```bash
rm -rf node_modules package-lock.json
npm install
npm run build
```

### Performance Optimizations

**Vite Build Optimizations:**
```typescript
// vite.config.ts
export default defineConfig({
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          'react-vendor': ['react', 'react-dom'],
          'axios-vendor': ['axios'],
        }
      }
    },
    chunkSizeWarningLimit: 1000,
  }
})
```

**Code Splitting:**
```typescript
// Lazy load components
const ManifestViewer = React.lazy(() => import('./components/manifest/ManifestViewer'));
const VerificationView = React.lazy(() => import('./components/verification/VerificationView'));
```

### Monitoring WebUI

**Nginx Access Logs:**
```bash
tail -f /var/log/nginx/access.log | grep "GET /api/"
```

**Bundle Size Analysis:**
```bash
npm run build -- --analyze
# Oder mit rollup-plugin-visualizer:
npx vite-bundle-visualizer
```

---

## 2.6 cap-bundle.v1 Package Format (Deployment Considerations) ⭐

> ⭐ **NEU in v0.11.0:** Standardisiertes Proof-Package-Format für Deployment und Distribution

### Überblick

Das **cap-bundle.v1 Format** ist das standardisierte Package-Format für offline-verifizierbare Compliance-Nachweise. Diese Section erklärt Deployment-spezifische Überlegungen für Systeme, die cap-bundle.v1 Pakete erstellen und verifizieren.

**Problem (vorher):**
- `proof export` erstellte Pakete im alten Format (cap-proof.v1.0)
- `verifier run` erwartete neues Format (cap-bundle.v1)
- **Resultat:** Inkompatibilität, Tests schlugen fehl

**Lösung (jetzt):**
- Beide Tools sprechen die gleiche "Sprache" (cap-bundle.v1)
- Strukturierte Metadaten für jede Datei
- SHA3-256 Hashes für Integritätsprüfung
- Sicherheit: Path Traversal Prevention, Cycle Detection, TOCTOU Mitigation
- Bundle Type Detection (Modern vs Legacy)
- Alle 556 Tests bestehen ✅ (100% Success Rate, 0 Failures)

### Package-Struktur

```
cap-proof/
├─ manifest.json         # Compliance manifest
├─ proof.dat             # Zero-knowledge proof
├─ _meta.json            # Bundle metadata (schema: cap-bundle.v1) ⭐
├─ timestamp.tsr         # Optional: Timestamp
├─ registry.json         # Optional: Registry
├─ verification.report.json  # Verification report
└─ README.txt            # Human-readable instructions
```

**_meta.json Struktur:**
```json
{
  "schema": "cap-bundle.v1",
  "bundle_id": "bundle-1732464123",
  "created_at": "2025-11-24T10:05:30Z",
  "files": {
    "manifest.json": {
      "role": "manifest",
      "hash": "0x1da941f7026bae3cf8b1bcdc3a8e01e76ea678c32ec6bc2c374fb67b3744571f",
      "size": 1234,
      "content_type": "application/json",
      "optional": false
    },
    "proof.dat": {
      "role": "proof",
      "hash": "0x83a8779ddef4567890123456789012345678901234567890123456789012345678",
      "size": 5678,
      "content_type": "application/octet-stream",
      "optional": false
    }
  },
  "proof_units": [
    {
      "id": "default",
      "manifest_file": "manifest.json",
      "proof_file": "proof.dat",
      "policy_id": "LkSG Demo Policy",
      "policy_hash": "0xabc123...",
      "backend": "mock"
    }
  ]
}
```

### Deployment-Überlegungen

#### 1. Proof Export in CI/CD Pipelines

**Automatisierte Bundle-Erstellung:**
```bash
# In CI/CD Pipeline (z.B. GitHub Actions, GitLab CI)
cap-agent proof export \
  --manifest build/manifest.json \
  --proof build/proof.dat \
  --out artifacts/cap-proof \
  --force

# Bundle als Artifact speichern
tar -czf cap-proof-${CI_COMMIT_SHA}.tar.gz artifacts/cap-proof/
```

**GitLab CI Example:**
```yaml
proof_export:
  stage: package
  script:
    - cargo build --release
    - ./target/release/cap-agent proof export --manifest build/manifest.json --proof build/proof.dat --out artifacts/cap-proof --force
    - tar -czf cap-proof-${CI_COMMIT_SHA}.tar.gz artifacts/cap-proof/
  artifacts:
    paths:
      - cap-proof-*.tar.gz
    expire_in: 30 days
```

#### 2. Docker Container mit cap-bundle.v1 Support

**Dockerfile Considerations:**
```dockerfile
FROM rust:1.75-bullseye AS builder

# Build cap-agent with proof export support
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

# Copy binaries
COPY --from=builder /app/target/release/cap-agent /usr/local/bin/

# Volume for bundle output
VOLUME ["/bundles"]

# Default command: Export bundle
CMD ["cap-agent", "proof", "export", \
     "--manifest", "/data/manifest.json", \
     "--proof", "/data/proof.dat", \
     "--out", "/bundles/cap-proof"]
```

**Docker Run Example:**
```bash
docker run -v $(pwd)/build:/data:ro \
           -v $(pwd)/output:/bundles \
           cap-agent:0.11.0
```

#### 3. Kubernetes Job für Bundle-Erstellung

**k8s/proof-export-job.yaml:**
```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: cap-proof-export
  namespace: cap-system
spec:
  template:
    spec:
      containers:
      - name: cap-agent
        image: registry.example.com/cap-agent:0.11.0
        command: ["cap-agent", "proof", "export"]
        args:
          - "--manifest"
          - "/data/manifest.json"
          - "--proof"
          - "/data/proof.dat"
          - "--out"
          - "/output/cap-proof"
          - "--force"
        volumeMounts:
        - name: proof-data
          mountPath: /data
          readOnly: true
        - name: bundle-output
          mountPath: /output
      volumes:
      - name: proof-data
        persistentVolumeClaim:
          claimName: cap-proof-data-pvc
      - name: bundle-output
        persistentVolumeClaim:
          claimName: cap-bundle-output-pvc
      restartPolicy: OnFailure
```

#### 4. Bundle-Verifikation in Production

**Verifier Deployment:**
```bash
# Binary Deployment
cap-agent verifier run --package /path/to/cap-proof/

# Docker Deployment
docker run -v $(pwd)/bundles/cap-proof:/bundle:ro \
           cap-agent:0.11.0 \
           verifier run --package /bundle

# Kubernetes Job
kubectl create -f k8s/verifier-job.yaml
```

**k8s/verifier-job.yaml:**
```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: cap-bundle-verifier
  namespace: cap-system
spec:
  template:
    spec:
      containers:
      - name: cap-verifier
        image: registry.example.com/cap-agent:0.11.0
        command: ["cap-agent", "verifier", "run"]
        args:
          - "--package"
          - "/bundle/cap-proof"
        volumeMounts:
        - name: bundle-input
          mountPath: /bundle
          readOnly: true
      volumes:
      - name: bundle-input
        persistentVolumeClaim:
          claimName: cap-bundle-output-pvc
      restartPolicy: Never
```

#### 5. Bundle-Integrität in Backups

**Backup Script mit _meta.json Validierung:**
```bash
#!/bin/bash

BACKUP_DIR="/backups/cap-bundles"
BUNDLE_DIR="/data/cap-proof"

# Create backup with timestamp
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_PATH="$BACKUP_DIR/bundle_$TIMESTAMP"

mkdir -p "$BACKUP_PATH"

# Copy bundle
echo "Backing up bundle..."
rsync -a "$BUNDLE_DIR/" "$BACKUP_PATH/"

# Verify _meta.json exists
if [ ! -f "$BACKUP_PATH/_meta.json" ]; then
    echo "❌ ERROR: _meta.json missing in bundle!"
    exit 1
fi

# Verify all files listed in _meta.json exist
echo "Verifying bundle integrity..."
FILES_OK=true
while IFS= read -r file; do
    if [ ! -f "$BACKUP_PATH/$file" ]; then
        echo "❌ Missing file: $file"
        FILES_OK=false
    fi
done < <(jq -r '.files | keys[]' "$BACKUP_PATH/_meta.json")

if [ "$FILES_OK" = false ]; then
    echo "❌ Bundle integrity check failed!"
    exit 1
fi

# Create tarball
tar -czf "$BACKUP_DIR/bundle_$TIMESTAMP.tar.gz" -C "$BACKUP_DIR" "bundle_$TIMESTAMP"
rm -rf "$BACKUP_PATH"

echo "✅ Bundle backup complete: $BACKUP_DIR/bundle_$TIMESTAMP.tar.gz"
```

#### 6. Bundle-Distribution

**HTTP Server für Bundle-Distribution:**
```nginx
server {
    listen 443 ssl http2;
    server_name bundles.example.com;

    # SSL Configuration
    ssl_certificate /etc/letsencrypt/live/bundles.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/bundles.example.com/privkey.pem;

    # Bundle Directory
    root /var/www/cap-bundles;
    index _meta.json;

    # Auto-index for browsing
    autoindex on;
    autoindex_format json;

    # Security: Only allow GET
    limit_except GET HEAD {
        deny all;
    }

    # Content-Type Headers
    location ~ \.json$ {
        add_header Content-Type application/json;
    }

    location ~ \.dat$ {
        add_header Content-Type application/octet-stream;
    }

    location ~ \.tsr$ {
        add_header Content-Type application/timestamp-reply;
    }

    # Cache bundles for 1 hour
    expires 1h;
    add_header Cache-Control "public, immutable";
}
```

**Download & Verify Script:**
```bash
#!/bin/bash

BUNDLE_URL="https://bundles.example.com/cap-proof"
LOCAL_DIR="./downloaded-bundle"

# Download bundle
mkdir -p "$LOCAL_DIR"
wget -r -np -nH --cut-dirs=1 -P "$LOCAL_DIR" "$BUNDLE_URL/"

# Verify bundle
cap-agent verifier run --package "$LOCAL_DIR/cap-proof"

if [ $? -eq 0 ]; then
    echo "✅ Bundle downloaded and verified successfully"
else
    echo "❌ Bundle verification failed"
    exit 1
fi
```

#### 7. Monitoring Bundle-Operationen

**Prometheus Metrics für Bundle-Erstellung:**
```promql
# Bundle Export Success Rate
rate(cap_bundle_export_total{result="success"}[5m]) / rate(cap_bundle_export_total[5m])

# Bundle Export Duration
histogram_quantile(0.95, sum(rate(cap_bundle_export_duration_seconds_bucket[5m])) by (le))

# Bundle Size Distribution
cap_bundle_size_bytes{bundle_id="..."}
```

**Grafana Dashboard Panels:**
- Bundle Export Rate (Timeseries)
- Export Success Rate (Stat Panel mit Threshold)
- Average Bundle Size (Gauge)
- Export Duration P95 (Timeseries)

#### 8. Troubleshooting

**Problem: _meta.json missing in exported bundle**

**Symptom:**
```
Error: Bundle metadata file '_meta.json' not found in package directory
```

**Solution:**
```bash
# Check if export command completed successfully
cap-agent proof export --manifest build/manifest.json --proof build/proof.dat --out build/cap-proof --force

# Verify _meta.json was created
ls -la build/cap-proof/_meta.json

# Check _meta.json content
jq . build/cap-proof/_meta.json
```

**Problem: Hash mismatch in _meta.json**

**Symptom:**
```
Error: File hash mismatch for 'manifest.json'
Expected: 0x1da941f7...
Got:      0x2eb052e8...
```

**Solution:**
```bash
# Re-export bundle with fresh files
rm -rf build/cap-proof
cap-agent proof export --manifest build/manifest.json --proof build/proof.dat --out build/cap-proof --force

# Verify hashes
sha3sum build/cap-proof/manifest.json
jq -r '.files."manifest.json".hash' build/cap-proof/_meta.json
```

### Best Practices

1. **Immer _meta.json prüfen:**
   - Vor Backup: Existenz prüfen
   - Nach Download: Hashes validieren
   - Im CI/CD: Verifikation als Test-Step

2. **Bundle-Versionierung:**
   ```bash
   # Bundle mit Version taggen
   BUNDLE_VERSION="v1.2.3"
   tar -czf cap-proof-${BUNDLE_VERSION}.tar.gz build/cap-proof/
   ```

3. **Immutable Bundles:**
   - Bundles niemals modifizieren (SHA3-256 Hashes werden ungültig)
   - Bei Änderungen neues Bundle erstellen

4. **Offline-Verifikation:**
   - Bundles sollten alle benötigten Dateien enthalten
   - Keine externen Dependencies (außer Verifier Binary)

5. **Retention Policy:**
   - Produktions-Bundles: 7 Jahre (LkSG-Anforderung)
   - Test-Bundles: 30 Tage
   - CI/CD-Artifacts: 90 Tage

### Weitere Informationen

Für detaillierte Informationen zum cap-bundle.v1 Format siehe:
- **03-components.md:** "## 13. Proof Format Layer (cap-bundle.v1)"
- **04-api-reference.md:** "### cap-bundle.v1 Package Format"
- **CLI Reference:** `cap-agent proof export --help`

---

## 3. Kubernetes Deployment

### Namespace erstellen

```bash
kubectl create namespace cap-system
```

### ConfigMap

**config/configmap.yaml:**
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: cap-config
  namespace: cap-system
data:
  app.yaml: |
    server:
      bind: "0.0.0.0:8443"
      log_level: "info"
    oauth:
      issuer: "https://auth.example.com"
      audience: "cap-verifier"
      required_scopes:
        - "verify:read"
    verification:
      backend: "mock"
      timeout_seconds: 30
    policy:
      cache_enabled: true
      cache_ttl_seconds: 3600

  auth.yaml: |
    oauth2:
      issuer: "https://auth.example.com"
      audience: "cap-verifier"
      public_key_path: "/config/public.pem"
      required_scopes:
        - "verify:read"
```

**Anwenden:**
```bash
kubectl apply -f config/configmap.yaml
```

### Secret (TLS Certificates)

```bash
kubectl create secret tls cap-tls-certs \
  --cert=certs/server.crt \
  --key=certs/server.key \
  --namespace=cap-system
```

### Deployment

**k8s/deployment.yaml:**
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cap-verifier
  namespace: cap-system
  labels:
    app: cap-verifier
spec:
  replicas: 3
  selector:
    matchLabels:
      app: cap-verifier
  template:
    metadata:
      labels:
        app: cap-verifier
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8080"
        prometheus.io/path: "/metrics"
    spec:
      containers:
      - name: cap-verifier
        image: registry.example.com/cap-agent:0.11.0
        imagePullPolicy: IfNotPresent
        ports:
        - containerPort: 8080
          name: http
          protocol: TCP
        - containerPort: 8443
          name: https
          protocol: TCP
        env:
        - name: TLS_MODE
          value: "tls"
        - name: RUST_LOG
          value: "info"
        volumeMounts:
        - name: config
          mountPath: /config
          readOnly: true
        - name: tls-certs
          mountPath: /certs
          readOnly: true
        - name: data
          mountPath: /data
        resources:
          requests:
            cpu: 500m
            memory: 512Mi
          limits:
            cpu: 2000m
            memory: 2Gi
        livenessProbe:
          httpGet:
            path: /healthz
            port: 8080
            scheme: HTTP
          initialDelaySeconds: 10
          periodSeconds: 30
          timeoutSeconds: 3
          successThreshold: 1
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /readyz
            port: 8080
            scheme: HTTP
          initialDelaySeconds: 5
          periodSeconds: 10
          timeoutSeconds: 3
          successThreshold: 1
          failureThreshold: 3
        securityContext:
          runAsNonRoot: true
          runAsUser: 1000
          allowPrivilegeEscalation: false
          readOnlyRootFilesystem: false
          capabilities:
            drop:
            - ALL
      volumes:
      - name: config
        configMap:
          name: cap-config
      - name: tls-certs
        secret:
          secretName: cap-tls-certs
      - name: data
        persistentVolumeClaim:
          claimName: cap-data-pvc
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 100
            podAffinityTerm:
              labelSelector:
                matchExpressions:
                - key: app
                  operator: In
                  values:
                  - cap-verifier
              topologyKey: kubernetes.io/hostname
```

**Anwenden:**
```bash
kubectl apply -f k8s/deployment.yaml
```

### Service

**k8s/service.yaml:**
```yaml
apiVersion: v1
kind: Service
metadata:
  name: cap-verifier-svc
  namespace: cap-system
  labels:
    app: cap-verifier
spec:
  type: ClusterIP
  selector:
    app: cap-verifier
  ports:
  - name: http
    port: 8080
    targetPort: 8080
    protocol: TCP
  - name: https
    port: 8443
    targetPort: 8443
    protocol: TCP
```

**Anwenden:**
```bash
kubectl apply -f k8s/service.yaml
```

### Ingress

**k8s/ingress.yaml:**
```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: cap-verifier-ingress
  namespace: cap-system
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/backend-protocol: "HTTPS"
spec:
  ingressClassName: nginx
  tls:
  - hosts:
    - verifier.example.com
    secretName: cap-verifier-tls
  rules:
  - host: verifier.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: cap-verifier-svc
            port:
              number: 8443
```

**Anwenden:**
```bash
kubectl apply -f k8s/ingress.yaml
```

### PersistentVolumeClaim

**k8s/pvc.yaml:**
```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: cap-data-pvc
  namespace: cap-system
spec:
  accessModes:
  - ReadWriteOnce
  resources:
    requests:
      storage: 10Gi
  storageClassName: fast-ssd
```

**Anwenden:**
```bash
kubectl apply -f k8s/pvc.yaml
```

### Deployment überprüfen

```bash
# Pods ansehen
kubectl get pods -n cap-system

# Logs eines Pods
kubectl logs -n cap-system cap-verifier-xxx-yyy -f

# Service testen
kubectl port-forward -n cap-system svc/cap-verifier-svc 8443:8443

# Dann in einem anderen Terminal:
curl -k https://localhost:8443/healthz
```

---

## 4. Monitoring & Observability

> ⭐ **NEU (Week 2):** Ein vollständiger Production-Ready Monitoring Stack ist jetzt verfügbar!
>
> **Umfasst:** Prometheus, Grafana (2 Dashboards), Loki, Promtail, Jaeger, Node Exporter, cAdvisor
> **Status:** ✅ 8/8 Container running, 5/5 healthy
> **Dokumentation:** `/Users/tomwesselmann/Desktop/LsKG-Agent/agent/monitoring/README.md`
> **Quick Start:**
> ```bash
> cd /Users/tomwesselmann/Desktop/LsKG-Agent/agent/monitoring
> docker compose up -d
> ./test-monitoring.sh
> ```
>
> **Service URLs:**
> - Grafana: http://localhost:3000 (admin/admin)
> - Prometheus: http://localhost:9090
> - Jaeger UI: http://localhost:16686
>
> **Für Details siehe:** [monitoring/README.md](/Users/tomwesselmann/Desktop/LsKG-Agent/agent/monitoring/README.md) und [07-status-und-roadmap.md](./07-status-und-roadmap.md)

---

### Basic Prometheus Configuration (für Custom Setups)

**prometheus.yml:**
```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'cap-verifier'
    static_configs:
      - targets: ['cap-verifier:8080']
    metrics_path: '/metrics'
```

### Grafana Dashboard

**Dashboard erstellen:**
1. Grafana öffnen (http://localhost:3000)
2. Login: admin / admin
3. Add Data Source → Prometheus → http://prometheus:9090
4. Create Dashboard → Add Panel

**Wichtige Metriken:**

**Request Rate:**
```promql
rate(cap_verifier_requests_total[5m])
```

**Success Rate:**
```promql
sum(rate(cap_verifier_requests_total{result="ok"}[5m])) /
sum(rate(cap_verifier_requests_total[5m]))
```

**Latency (P50, P95, P99):**
```promql
histogram_quantile(0.50, rate(cap_verifier_request_duration_seconds_bucket[5m]))
histogram_quantile(0.95, rate(cap_verifier_request_duration_seconds_bucket[5m]))
histogram_quantile(0.99, rate(cap_verifier_request_duration_seconds_bucket[5m]))
```

**Auth Failures:**
```promql
rate(cap_auth_token_validation_failures_total[5m])
```

**Cache Hit Ratio:**
```promql
cap_cache_hit_ratio
```

---

## 5. Backup & Recovery

### Daten, die gesichert werden müssen

1. **Registry Database** (`registry.db`)
2. **BLOB Store** (`blobs/`)
3. **Keys** (`keys/`)
4. **Konfiguration** (`config/`)
5. **Certificates** (`certs/`)

### Backup Script

**scripts/backup.sh:**
```bash
#!/bin/bash

BACKUP_DIR="/backups/cap-verifier"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_PATH="$BACKUP_DIR/backup_$TIMESTAMP"

# Create backup directory
mkdir -p "$BACKUP_PATH"

# Backup registry
echo "Backing up registry..."
sqlite3 /data/registry.db ".backup '$BACKUP_PATH/registry.db'"

# Backup BLOB store
echo "Backing up BLOB store..."
rsync -a /data/blobs/ "$BACKUP_PATH/blobs/"

# Backup keys
echo "Backing up keys..."
cp -r /data/keys/ "$BACKUP_PATH/keys/"

# Backup config
echo "Backing up config..."
cp -r /etc/cap-verifier/ "$BACKUP_PATH/config/"

# Create tarball
echo "Creating tarball..."
cd "$BACKUP_DIR"
tar -czf "backup_$TIMESTAMP.tar.gz" "backup_$TIMESTAMP"
rm -rf "backup_$TIMESTAMP"

# Remove old backups (keep last 7 days)
find "$BACKUP_DIR" -name "backup_*.tar.gz" -mtime +7 -delete

echo "Backup complete: $BACKUP_DIR/backup_$TIMESTAMP.tar.gz"
```

### Restore Script

**scripts/restore.sh:**
```bash
#!/bin/bash

BACKUP_FILE="$1"

if [ -z "$BACKUP_FILE" ]; then
  echo "Usage: $0 <backup-file.tar.gz>"
  exit 1
fi

# Stop service
systemctl stop cap-verifier

# Extract backup
TEMP_DIR=$(mktemp -d)
tar -xzf "$BACKUP_FILE" -C "$TEMP_DIR"

# Restore registry
echo "Restoring registry..."
cp "$TEMP_DIR/backup_*/registry.db" /data/registry.db

# Restore BLOB store
echo "Restoring BLOB store..."
rsync -a "$TEMP_DIR/backup_*/blobs/" /data/blobs/

# Restore keys
echo "Restoring keys..."
cp -r "$TEMP_DIR/backup_*/keys/" /data/keys/

# Restore config
echo "Restoring config..."
cp -r "$TEMP_DIR/backup_*/config/" /etc/cap-verifier/

# Cleanup
rm -rf "$TEMP_DIR"

# Start service
systemctl start cap-verifier

echo "Restore complete"
```

### Automated Backups (Cron)

```bash
# Edit crontab
crontab -e

# Add daily backup at 2 AM
0 2 * * * /usr/local/bin/cap-backup.sh >> /var/log/cap-backup.log 2>&1
```

---

## 6. Sicherheit

### TLS/mTLS Configuration

**Generiere Self-Signed Certificate (Development):**
```bash
openssl req -x509 -newkey rsa:4096 -keyout server.key -out server.crt -days 365 -nodes \
  -subj "/C=DE/ST=Berlin/L=Berlin/O=Example/CN=localhost"
```

**Production Certificate (Let's Encrypt):**
```bash
# Mit cert-manager in Kubernetes (siehe Ingress Annotation)
# oder mit certbot:
sudo certbot certonly --standalone -d verifier.example.com
```

### OAuth2 Public Key

**Public Key aus JWK extrahieren:**
```bash
# JWK von Auth Server abrufen
curl https://auth.example.com/.well-known/jwks.json | jq '.keys[0]'

# In PEM konvertieren (z.B. mit jwt.io oder online tool)
# Dann in /config/public.pem speichern
```

### Firewall Rules

**UFW (Ubuntu):**
```bash
# Allow SSH
sudo ufw allow 22/tcp

# Allow HTTPS
sudo ufw allow 8443/tcp

# Enable firewall
sudo ufw enable
```

**iptables:**
```bash
# Allow HTTPS
sudo iptables -A INPUT -p tcp --dport 8443 -j ACCEPT

# Save rules
sudo iptables-save > /etc/iptables/rules.v4
```

---

## 7. Performance Tuning

### Rust Runtime

**Environment Variables:**
```bash
# Thread pool size
export TOKIO_WORKER_THREADS=4

# Stack size
export RUST_MIN_STACK=8388608
```

### SQLite Tuning

**In registry.db:**
```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = -64000;  -- 64 MB
PRAGMA temp_store = MEMORY;
```

### Kubernetes Resources

**Adjust based on load:**
```yaml
resources:
  requests:
    cpu: 1000m       # 1 CPU
    memory: 1Gi
  limits:
    cpu: 4000m       # 4 CPU
    memory: 4Gi
```

### Horizontal Pod Autoscaler

**k8s/hpa.yaml:**
```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: cap-verifier-hpa
  namespace: cap-system
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: cap-verifier
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

---

## 8. Logging

### Structured Logging (tracing)

**Log Levels:**
- `error` - Fehler, die sofortiges Handeln erfordern
- `warn` - Warnungen, die überwacht werden sollten
- `info` - Allgemeine Informationen (Standard)
- `debug` - Detaillierte Debug-Informationen
- `trace` - Sehr detaillierte Trace-Informationen

**Environment Variable:**
```bash
export RUST_LOG=info
# Oder für Module-spezifisch:
export RUST_LOG=cap_agent=debug,tower_http=info
```

### Log Aggregation (ELK Stack)

**Filebeat Configuration:**
```yaml
filebeat.inputs:
- type: container
  paths:
    - '/var/lib/docker/containers/*/*.log'
  processors:
    - add_kubernetes_metadata:
        host: ${NODE_NAME}
        matchers:
        - logs_path:
            logs_path: "/var/lib/docker/containers/"

output.elasticsearch:
  hosts: ["elasticsearch:9200"]
```

---

## 9. Health Checks

### Endpoints

- **Liveness:** `GET /healthz` - Prüft ob Service läuft
- **Readiness:** `GET /readyz` - Prüft ob Service bereit ist Requests zu akzeptieren

### Kubernetes Probes

```yaml
livenessProbe:
  httpGet:
    path: /healthz
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 30

readinessProbe:
  httpGet:
    path: /readyz
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 10
```

---

## Zusammenfassung

Der LsKG-Agent kann flexibel deployed werden:
- ✅ **Desktop App** für Offline-Nutzung ohne Server (NEU in v0.12.0)
- ✅ Binary + Systemd für einfache Deployments
- ✅ Docker für Container-basierte Deployments
- ✅ Kubernetes für Enterprise-Skalierung
- ✅ WebUI für Browser-basierte Verifikation
- ✅ Monitoring mit Prometheus + Grafana
- ✅ Automated Backups
- ✅ Security Hardening
- ✅ Performance Tuning

**Empfehlung nach Anwendungsfall:**
- **Einzelperson/Offline:** Desktop App (Tauri)
- **Kleine Teams:** Binary + Systemd
- **Mittlere Unternehmen:** Docker Compose
- **Enterprise:** Kubernetes mit HPA

**🔐 Enterprise Security Status:**
- Aktuell: 57% Enterprise Readiness
- Ziel: 95% nach 14 Wochen Hardening-Roadmap
- [Enterprise Checklist](#-enterprise-security-requirements-neu---dezember-2025) am Anfang dieses Dokuments beachten!

**📋 Details:** [SECURITY_AUDIT_REPORT.md](../../security/SECURITY_AUDIT_REPORT.md) | [ROADMAP_ENTERPRISE.md](../../ROADMAP_ENTERPRISE.md)

---

*Dokument-Version: 2.1 (aktualisiert mit Enterprise Security Requirements)*
*Letzte Aktualisierung: 4. Dezember 2025*
*Projekt: LsKG-Agent v0.12.0*
