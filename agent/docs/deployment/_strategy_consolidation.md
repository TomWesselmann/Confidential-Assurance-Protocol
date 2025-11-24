# Konsolidierungsstrategie – Deployment-Dokumentation

**Datum:** 2025-11-24
**Phase:** Schritt 2 von CAP_CLAUDE_WORKFLOW_V2
**Voraussetzung:** Schritt 1 Analyse abgeschlossen (_analysis_consolidation.md)

---

## Zieldefinition

### Primärziel
Konsolidierung der 3 Deployment-Dokumentationen in **ein einziges, strukturiertes Master-Dokument**, das:
1. **Alle Unique-Inhalte** aus den 3 Quell-Dokumenten erhält
2. **Redundanzen eliminiert** (60-70% Überschneidung)
3. **Klare Struktur** mit logischem Aufbau (Quick Start → Advanced)
4. **Wartbarkeit verbessert** (1 Datei statt 3)
5. **User Experience optimiert** (Table of Contents, Cross-References)

### Sekundärziele
- **Security:** Keine Verfälschung von TLS/mTLS/Kubectl-Commands
- **Vollständigkeit:** Kein Informationsverlust
- **Lesbarkeit:** Markdown Best Practices
- **Zukunftssicherheit:** Erweiterbar für neue Deployment-Methoden

---

## Entscheidung: Master-Dokument

### Gewähltes Master-Dokument: `DEPLOYMENT.md`

**Begründung:**
1. **Beste Ausgangsstruktur:**
   - Fokus auf Production-Ready Kubernetes-Deployment
   - Enthält Helm Charts, Multi-Stage Builds, Distroless Images
   - Security-First-Ansatz (mTLS, Image Signing)

2. **Erweiterbarkeit:**
   - Kann einfach um Quick Start (aus README_DEPLOYMENT.md) erweitert werden
   - Monitoring-Sektion (aus DOCKER_DEPLOYMENT.md) passt gut als eigenes Kapitel

3. **Namenskonvention:**
   - `DEPLOYMENT.md` ist der generischste Name
   - README_DEPLOYMENT.md ist redundant (README sollte Projekt-Übersicht sein, nicht Deployment-Details)
   - DOCKER_DEPLOYMENT.md ist zu spezifisch (auch Kubernetes wird behandelt)

4. **Dateigröße:**
   - 479 Zeilen (mittelgroß, Platz für Erweiterungen)
   - Nicht zu überladen (wie README_DEPLOYMENT.md mit 690 Zeilen)

---

## Ziel-Struktur: Neues DEPLOYMENT.md

### Haupt-Gliederung (7 Kapitel)

```markdown
# CAP Verifier - Deployment Guide

> Production-Ready Deployment-Anleitung für CAP Verifier API mit Docker, Kubernetes und Monitoring

## Table of Contents
1. [Quick Start](#1-quick-start)
2. [Docker Deployment](#2-docker-deployment)
3. [Kubernetes Deployment](#3-kubernetes-deployment)
4. [TLS/mTLS Configuration](#4-tlsmtls-configuration)
5. [Monitoring & Observability](#5-monitoring--observability)
6. [Configuration Reference](#6-configuration-reference)
7. [Troubleshooting](#7-troubleshooting)

---

## 1. Quick Start

**Quelle:** README_DEPLOYMENT.md (Schnellstart-Sektion)

### 1.1 Prerequisites
- Docker 20.10+
- Rust 1.70+ (für lokale Builds)
- kubectl (für Kubernetes)
- Helm 3+ (optional)

### 1.2 Quick Start: Docker (HTTP)
```bash
# Build Image
docker build -t cap-agent:latest .

# Run Container (HTTP-only, Development)
docker run -d -p 8080:8080 \
  --name cap-verifier \
  cap-agent:latest \
  cap-verifier-api --bind 0.0.0.0:8080
```

### 1.3 Quick Start: Docker Compose
**Quelle:** DOCKER_DEPLOYMENT.md
```bash
cd monitoring
docker compose up -d
```

### 1.4 Quick Start: Kubernetes
**Quelle:** README_DEPLOYMENT.md
```bash
kubectl apply -f kubernetes/deployment.yml
kubectl apply -f kubernetes/service.yml
```

---

## 2. Docker Deployment

**Quellen:** DOCKER_DEPLOYMENT.md + README_DEPLOYMENT.md + DEPLOYMENT.md

### 2.1 Basic Docker Build
**Quelle:** Alle 3 Dokumente (Duplikat entfernen, beste Version wählen)

### 2.2 Multi-Stage Build (Alpine + Distroless)
**Quelle:** DEPLOYMENT.md (enthält vollständige Multi-Stage-Strategie)

#### Dockerfile.alpine (Lightweight)
**Quelle:** DOCKER_DEPLOYMENT.md (Dockerfile.optimized)

#### Dockerfile.distroless (Security-Hardened)
**Quelle:** DEPLOYMENT.md (distroless-spezifisch)

### 2.3 Docker Compose Setup
**Quelle:** DOCKER_DEPLOYMENT.md + monitoring/docker-compose.yml

### 2.4 Multi-Platform Builds
**Quelle:** README_DEPLOYMENT.md (Docker Buildx)

### 2.5 Mac-Specific Instructions (M1/M2)
**Quelle:** DOCKER_DEPLOYMENT.md

---

## 3. Kubernetes Deployment

**Quellen:** DEPLOYMENT.md + README_DEPLOYMENT.md

### 3.1 Basic Deployment
**Quelle:** README_DEPLOYMENT.md (kubernetes/deployment.yml)

### 3.2 Production Deployment
**Quelle:** DEPLOYMENT.md (mit Resource Limits, Health Checks, Liveness Probes)

### 3.3 Helm Charts
**Quelle:** DEPLOYMENT.md (Helm-Struktur)

### 3.4 Kyma Service Mesh (Future)
**Quelle:** DEPLOYMENT.md (Hinweis auf geplante Integration)

### 3.5 Image Signing with Cosign (Future)
**Quelle:** DEPLOYMENT.md (Security-Feature)

---

## 4. TLS/mTLS Configuration

**Quellen:** Alle 3 Dokumente (mergen, beste Beispiele auswählen)

### 4.1 TLS-only Setup (Server Certificate)
**Quelle:** README_DEPLOYMENT.md (detaillierte Anleitung)

### 4.2 Mutual TLS (mTLS)
**Quelle:** DEPLOYMENT.md + README_DEPLOYMENT.md

### 4.3 Certificate Management
**Quelle:** README_DEPLOYMENT.md (Let's Encrypt, Self-Signed)

### 4.4 Testing TLS Connections
**Quelle:** README_DEPLOYMENT.md (curl-Beispiele)

---

## 5. Monitoring & Observability

**Quelle:** DOCKER_DEPLOYMENT.md (vollständiger Stack) + monitoring/

### 5.1 Prometheus Setup
**Quelle:** DOCKER_DEPLOYMENT.md (prometheus.yml, Scrape-Config)

### 5.2 Grafana Dashboards
**Quelle:** DOCKER_DEPLOYMENT.md (Dashboard-Import, Datasources)

### 5.3 Loki (Logging)
**Quelle:** monitoring/loki/loki-config.yml

### 5.4 Jaeger (Distributed Tracing)
**Quelle:** monitoring/jaeger/jaeger-config.yml

### 5.5 SLO/SLI Monitoring
**Quelle:** monitoring/slo/slo-config.yml

---

## 6. Configuration Reference

**Quelle:** README_DEPLOYMENT.md (Environment Variables)

### 6.1 Environment Variables
- `BIND_ADDRESS`
- `TLS_MODE`
- `POLICY_STORE_BACKEND`
- `POLICY_DB_PATH`
- `RATE_LIMIT`
- `RATE_LIMIT_BURST`

### 6.2 Runtime Flags
- `--bind`
- `--tls / --tls-cert / --tls-key`
- `--mtls / --tls-ca`
- `--rate-limit`

### 6.3 Policy Store Backend Selection
**Quelle:** README_DEPLOYMENT.md (InMemory vs. SQLite)

---

## 7. Troubleshooting

**Quelle:** README_DEPLOYMENT.md (vollständige Troubleshooting-Sektion)

### 7.1 Common Issues
- TLS Certificate Errors
- Auth Token Validation Failures
- Database Connection Issues

### 7.2 Debugging TLS
**Quelle:** README_DEPLOYMENT.md (openssl s_client)

### 7.3 Container Logs
```bash
docker logs cap-verifier
kubectl logs -f deployment/cap-verifier
```

### 7.4 Health Checks
```bash
curl http://localhost:8080/healthz
curl http://localhost:8080/readyz
```

---

## Appendix: Migration Notes

**Hinweis für bestehende User:**
- README_DEPLOYMENT.md → DEPLOYMENT.md (Kapitel 1, 4, 6, 7)
- DOCKER_DEPLOYMENT.md → DEPLOYMENT.md (Kapitel 2.2, 2.3, 5)
- Alte Dateien werden nach Konsolidierung gelöscht
```

---

## Content-Mapping (Quell-Dokument → Ziel-Kapitel)

### DEPLOYMENT.md → Neues DEPLOYMENT.md

| Original-Abschnitt | Ziel-Kapitel | Aktion |
|--------------------|--------------|--------|
| Multi-Stage Dockerfile | 2.2 | Übernehmen (vollständig) |
| Kubernetes Deployment | 3.2 | Übernehmen + Erweitern |
| Helm Charts | 3.3 | Übernehmen (vollständig) |
| Distroless Images | 2.2 | Übernehmen (vollständig) |
| TLS/mTLS | 4.1, 4.2 | Mergen mit README_DEPLOYMENT.md |
| Kyma Service Mesh | 3.4 | Übernehmen (Future-Hinweis) |
| Image Signing | 3.5 | Übernehmen (Future-Hinweis) |

**Beibehaltungsrate:** 100% (nichts wird gelöscht)

---

### README_DEPLOYMENT.md → Neues DEPLOYMENT.md

| Original-Abschnitt | Ziel-Kapitel | Aktion |
|--------------------|--------------|--------|
| Schnellstart | 1.2, 1.4 | Übernehmen (vollständig) |
| TLS/mTLS Setup | 4.1, 4.2, 4.3, 4.4 | Mergen mit DEPLOYMENT.md |
| Multi-Platform Builds | 2.4 | Übernehmen (vollständig) |
| Configuration | 6.1, 6.2, 6.3 | Übernehmen (vollständig) |
| Troubleshooting | 7.1, 7.2, 7.3, 7.4 | Übernehmen (vollständig) |
| Kubernetes Deployment | 3.1 | Mergen mit DEPLOYMENT.md |

**Beibehaltungsrate:** 100% (Unique Content wird erhalten)

---

### DOCKER_DEPLOYMENT.md → Neues DEPLOYMENT.md

| Original-Abschnitt | Ziel-Kapitel | Aktion |
|--------------------|--------------|--------|
| Dockerfile.optimized | 2.2 | Übernehmen (vollständig) |
| docker-compose.yml | 2.3, 5.1-5.5 | Übernehmen (vollständig) |
| Prometheus Setup | 5.1 | Übernehmen (vollständig) |
| Grafana Dashboards | 5.2 | Übernehmen (vollständig) |
| Loki/Jaeger | 5.3, 5.4 | Übernehmen (vollständig) |
| Mac Instructions | 2.5 | Übernehmen (vollständig) |
| Alpine Optimization | 2.2 | Mergen mit Multi-Stage Build |

**Beibehaltungsrate:** 100% (Monitoring-Stack vollständig erhalten)

---

## Duplikate-Eliminierungsstrategie

### Kategorie 1: Identische Abschnitte (löschen)
- **Docker Build Basics:** 3x identisch → 1x in Kapitel 2.1
- **TLS Certificate Generation:** 2x identisch → 1x in Kapitel 4.3
- **Kubernetes Service YAML:** 2x identisch → 1x in Kapitel 3.1

**Regel:** Bei identischen Abschnitten wird die **best formatted** Version übernommen.

### Kategorie 2: Ähnliche Abschnitte (mergen)
- **TLS/mTLS Setup:** 3x unterschiedliche Detailtiefe → Mergen in Kapitel 4
- **Multi-Stage Builds:** 2x unterschiedliche Ansätze (Alpine vs. Distroless) → Beide in Kapitel 2.2

**Regel:** Bei ähnlichen Abschnitten werden **komplementäre Details kombiniert**.

### Kategorie 3: Unique Content (vollständig übernehmen)
- **Monitoring Stack:** Nur in DOCKER_DEPLOYMENT.md → Kapitel 5 (komplett)
- **Troubleshooting:** Nur in README_DEPLOYMENT.md → Kapitel 7 (komplett)
- **Helm Charts:** Nur in DEPLOYMENT.md → Kapitel 3.3 (komplett)

**Regel:** Unique Content wird **immer vollständig übernommen**.

---

## Datei-Operationen

### Zu bearbeitende Datei
- **`docs/deployment/DEPLOYMENT.md`**
  - Aktion: Erweitern (von 479 auf ~1200 Zeilen)
  - Neue Kapitel: 1, 5, 6, 7
  - Erweiterte Kapitel: 2, 3, 4

### Zu löschende Dateien (nach Merge)
- **`docs/deployment/README_DEPLOYMENT.md`** (690 Zeilen)
  - Grund: Content vollständig in DEPLOYMENT.md übernommen
- **`docs/deployment/DOCKER_DEPLOYMENT.md`** (457 Zeilen)
  - Grund: Content vollständig in DEPLOYMENT.md übernommen

### Zu aktualisierende Referenzen
- **`docs/CLAUDE.md`:**
  - Zeile 1286: Link zu README_DEPLOYMENT.md entfernen
  - Zeile 1287: Link zu DOCKER_DEPLOYMENT.md entfernen
  - Zeile 1285: Link zu DEPLOYMENT.md aktualisieren (Beschreibung erweitern)

- **`README.md` (Projekt-Root):**
  - Falls Deployment-Links vorhanden → aktualisieren

---

## Quality Gates (Acceptance Criteria)

### Schritt 3: Tests planen
- ✅ Manuelle Review-Checkliste definiert
- ✅ Security-Check-Punkte identifiziert

### Schritt 4: Security Review
- ✅ TLS/mTLS-Konfigurationen überprüft (keine Verfälschung)
- ✅ Kubectl/Helm-Commands verifiziert (keine Security-Issues)
- ✅ Docker-Commands auf Privilege Escalation geprüft

### Schritt 5: Implementation
- ✅ Alle 7 Kapitel vollständig implementiert
- ✅ Table of Contents generiert
- ✅ Cross-References eingefügt
- ✅ Markdown-Formatierung korrekt (Linting)

### Schritt 6: Tests ausführen
- ✅ Manuelle Review-Checkliste abgearbeitet
- ✅ Keine toten Links
- ✅ Alle Code-Blöcke syntaktisch korrekt
- ✅ Screenshots/Diagramme (falls vorhanden) migriert

### Schritt 7: Git Commit
- ✅ Commit-Message: "docs: Consolidate 3 deployment docs into unified DEPLOYMENT.md"
- ✅ Co-Authored-By: Claude <noreply@anthropic.com>
- ✅ Audit-Log-Eintrag für Dokumentations-Änderung

---

## Risiko-Mitigation

### Risiko 1: Informationsverlust
**Mitigation:**
- Vollständige Keyword-Frequenzanalyse durchgeführt (Schritt 1)
- Content-Mapping erstellt (siehe oben)
- Beibehaltungsrate: 100% (kein Unique Content wird gelöscht)

### Risiko 2: Security-Fehler
**Mitigation:**
- Manuelle Security-Review (Schritt 4)
- TLS/mTLS-Konfigurationen werden NICHT modifiziert (Copy-Paste)
- Kubectl/Helm-Commands werden verifiziert

### Risiko 3: Broken Links
**Mitigation:**
- Alle internen Links werden aktualisiert (docs/CLAUDE.md)
- Externe Links werden validiert (Linting)

### Risiko 4: User-Verwirrung
**Mitigation:**
- Migration Notes im Appendix (Hinweis auf alte Dateinamen)
- Clear Table of Contents
- Cross-References zu verwandten Kapiteln

---

## Timeline (geschätzt)

| Phase | Dauer | Status |
|-------|-------|--------|
| Schritt 1: Analyse | 30 min | ✅ Abgeschlossen |
| Schritt 2: Ziele definieren | 20 min | ✅ Abgeschlossen |
| Schritt 3: Tests planen | 10 min | ⏳ Nächster Schritt |
| Schritt 4: Security Review | 20 min | ⏳ Pending |
| Schritt 5: Implementation | 60 min | ⏳ Pending |
| Schritt 6: Tests ausführen | 15 min | ⏳ Pending |
| Schritt 7: Git Commit | 5 min | ⏳ Pending |

**Gesamt:** ~2.5 Stunden

---

## Approval

**Strategie genehmigt:** ⏳ Warte auf User-Bestätigung

**Nächster Schritt:** Schritt 3 – Tests planen (Review-Checkliste erstellen)

---

**Dokument erstellt:** 2025-11-24
**Autor:** Claude Code (CAP_CLAUDE_WORKFLOW_V2)
**Version:** 1.0
