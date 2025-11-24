# Testing Plan – Deployment-Dokumentations-Konsolidierung

**Datum:** 2025-11-24
**Phase:** Schritt 3 von CAP_CLAUDE_WORKFLOW_V2
**Voraussetzung:** Schritt 2 Strategie abgeschlossen (_strategy_consolidation.md)

---

## Hinweis: Dokumentations-Testing

**Wichtig:** Da es sich um eine Dokumentations-Konsolidierung handelt, gibt es **keine automatischen Tests** (kein cargo test, kein Linting für Docs). Alle Tests sind **manuell** und basieren auf **Review-Checklisten**.

---

## Test-Kategorien

### 1. Strukturelle Integrität
- ✅ Table of Contents ist vollständig
- ✅ Alle Kapitel sind vorhanden (1-7)
- ✅ Markdown-Formatierung ist korrekt
- ✅ Überschriften-Hierarchie ist konsistent (H1 → H2 → H3)

### 2. Content-Vollständigkeit
- ✅ Alle Unique-Inhalte aus 3 Quell-Dokumenten sind vorhanden
- ✅ Keine Informationsverluste
- ✅ Content-Mapping aus Schritt 2 ist vollständig implementiert

### 3. Security & Correctness
- ✅ TLS/mTLS-Konfigurationen sind unverändert (Copy-Paste)
- ✅ Kubectl/Helm-Commands sind syntaktisch korrekt
- ✅ Docker-Commands sind sicher (keine Privilege Escalation)
- ✅ Keine Hardcoded Credentials

### 4. Links & Cross-References
- ✅ Alle internen Links funktionieren
- ✅ Cross-References zwischen Kapiteln sind korrekt
- ✅ Externe Links sind valide (HTTP 200 Check)
- ✅ Code-Block-Referenzen sind vorhanden

### 5. Code-Blöcke & Syntax
- ✅ Alle Code-Blöcke haben Syntax-Highlighting (```bash, ```yaml, ```json)
- ✅ Keine Syntax-Fehler in YAML/JSON-Beispielen
- ✅ Docker/Kubectl-Commands sind vollständig (keine abgeschnittenen Zeilen)

### 6. Referenz-Updates
- ✅ docs/CLAUDE.md: Deployment-Links aktualisiert
- ✅ README.md (falls vorhanden): Deployment-Links aktualisiert
- ✅ Keine toten Links in anderen Dokumenten

---

## Review-Checkliste (Schritt 6)

### Phase 1: Strukturprüfung (5 min)

**Checklist:**
- [ ] Table of Contents generiert und vollständig
- [ ] 7 Hauptkapitel vorhanden (Quick Start, Docker, Kubernetes, TLS, Monitoring, Config, Troubleshooting)
- [ ] Markdown-Linting bestanden (manuell, oder mit markdownlint-cli)
- [ ] Überschriften-Hierarchie korrekt (H1 für Titel, H2 für Kapitel, H3 für Unterkapitel)
- [ ] Keine doppelten Überschriften-IDs

**Tool:**
```bash
# Markdown-Linting (optional, wenn markdownlint installiert)
npx markdownlint-cli docs/deployment/DEPLOYMENT.md

# Überschriften-IDs prüfen (manuell)
grep -E '^#{1,6} ' docs/deployment/DEPLOYMENT.md | sort | uniq -d
```

---

### Phase 2: Content-Vollständigkeitsprüfung (15 min)

**Checklist:**
- [ ] **Kapitel 1 (Quick Start):**
  - [ ] Prerequisites aus README_DEPLOYMENT.md vorhanden
  - [ ] Docker Quickstart aus README_DEPLOYMENT.md vorhanden
  - [ ] Docker Compose Quickstart aus DOCKER_DEPLOYMENT.md vorhanden
  - [ ] Kubernetes Quickstart aus README_DEPLOYMENT.md vorhanden

- [ ] **Kapitel 2 (Docker Deployment):**
  - [ ] Basic Docker Build aus allen 3 Dokumenten (beste Version)
  - [ ] Multi-Stage Build (Alpine) aus DOCKER_DEPLOYMENT.md vorhanden
  - [ ] Multi-Stage Build (Distroless) aus DEPLOYMENT.md vorhanden
  - [ ] Docker Compose Setup aus DOCKER_DEPLOYMENT.md vorhanden
  - [ ] Multi-Platform Builds aus README_DEPLOYMENT.md vorhanden
  - [ ] Mac-Specific Instructions aus DOCKER_DEPLOYMENT.md vorhanden

- [ ] **Kapitel 3 (Kubernetes Deployment):**
  - [ ] Basic Deployment aus README_DEPLOYMENT.md vorhanden
  - [ ] Production Deployment aus DEPLOYMENT.md vorhanden
  - [ ] Helm Charts aus DEPLOYMENT.md vorhanden
  - [ ] Kyma Service Mesh (Future) aus DEPLOYMENT.md vorhanden
  - [ ] Image Signing (Future) aus DEPLOYMENT.md vorhanden

- [ ] **Kapitel 4 (TLS/mTLS Configuration):**
  - [ ] TLS-only Setup aus README_DEPLOYMENT.md vorhanden
  - [ ] Mutual TLS aus DEPLOYMENT.md + README_DEPLOYMENT.md vorhanden
  - [ ] Certificate Management aus README_DEPLOYMENT.md vorhanden
  - [ ] Testing TLS aus README_DEPLOYMENT.md vorhanden

- [ ] **Kapitel 5 (Monitoring & Observability):**
  - [ ] Prometheus Setup aus DOCKER_DEPLOYMENT.md vorhanden
  - [ ] Grafana Dashboards aus DOCKER_DEPLOYMENT.md vorhanden
  - [ ] Loki aus monitoring/loki/ vorhanden
  - [ ] Jaeger aus monitoring/jaeger/ vorhanden
  - [ ] SLO/SLI aus monitoring/slo/ vorhanden

- [ ] **Kapitel 6 (Configuration Reference):**
  - [ ] Environment Variables aus README_DEPLOYMENT.md vorhanden
  - [ ] Runtime Flags aus README_DEPLOYMENT.md vorhanden
  - [ ] Policy Store Backend aus README_DEPLOYMENT.md vorhanden

- [ ] **Kapitel 7 (Troubleshooting):**
  - [ ] Common Issues aus README_DEPLOYMENT.md vorhanden
  - [ ] Debugging TLS aus README_DEPLOYMENT.md vorhanden
  - [ ] Container Logs aus README_DEPLOYMENT.md vorhanden
  - [ ] Health Checks aus README_DEPLOYMENT.md vorhanden

**Tool:**
```bash
# Keyword-Check: Stichproben-Check für wichtige Keywords
grep -i "helm" docs/deployment/DEPLOYMENT.md | wc -l  # Sollte >0 sein
grep -i "distroless" docs/deployment/DEPLOYMENT.md | wc -l  # Sollte >0 sein
grep -i "prometheus" docs/deployment/DEPLOYMENT.md | wc -l  # Sollte >0 sein
grep -i "alpine" docs/deployment/DEPLOYMENT.md | wc -l  # Sollte >0 sein
```

---

### Phase 3: Security Review (20 min)

**Checklist:**
- [ ] **TLS/mTLS-Konfigurationen:**
  - [ ] openssl-Commands sind unverändert (Copy-Paste aus Quell-Dokumenten)
  - [ ] Certificate Paths sind korrekt (`certs/server.crt`, `certs/server.key`)
  - [ ] mTLS CA-Certificate Path ist korrekt (`certs/ca.crt`)
  - [ ] Keine Hardcoded Passwords/Secrets

- [ ] **Kubectl/Helm-Commands:**
  - [ ] kubectl apply-Commands sind syntaktisch korrekt
  - [ ] Helm install-Commands sind syntaktisch korrekt
  - [ ] Namespace-Angaben sind korrekt
  - [ ] Resource Limits sind sinnvoll (keine 0-Werte)

- [ ] **Docker-Commands:**
  - [ ] Keine `--privileged` Flags (außer explizit dokumentiert)
  - [ ] Port-Mappings sind sicher (keine Host-Network-Mode)
  - [ ] Volume-Mounts sind read-only wo möglich
  - [ ] User-IDs sind nicht root (wo möglich)

- [ ] **Code-Injection-Prävention:**
  - [ ] Keine unescaped Variables in Shell-Commands
  - [ ] Keine `eval` oder `exec` in Beispielen (außer notwendig)

**Tool:**
```bash
# Security-Pattern-Check (manuell)
grep -E "(--privileged|eval|exec|rm -rf /)" docs/deployment/DEPLOYMENT.md
# Erwartung: Keine gefährlichen Patterns (oder explizit dokumentiert)

# Hardcoded Secrets-Check
grep -E "(password|secret|token).*=.*['\"]" docs/deployment/DEPLOYMENT.md
# Erwartung: Keine Hardcoded Credentials
```

---

### Phase 4: Links & Cross-References (10 min)

**Checklist:**
- [ ] **Interne Links (innerhalb DEPLOYMENT.md):**
  - [ ] Table of Contents-Links funktionieren (Anchor-IDs korrekt)
  - [ ] Cross-References zwischen Kapiteln funktionieren
  - [ ] Code-Block-Referenzen vorhanden (z.B. "siehe Kapitel 4.2")

- [ ] **Externe Links:**
  - [ ] Links zu kubernetes/ Dateien funktionieren
  - [ ] Links zu monitoring/ Dateien funktionieren
  - [ ] Links zu docs/CLAUDE.md funktionieren (aktualisiert in Schritt 5)
  - [ ] Links zu README.md funktionieren (falls vorhanden)

- [ ] **Broken Links Check:**
  - [ ] Keine Links zu gelöschten Dateien (README_DEPLOYMENT.md, DOCKER_DEPLOYMENT.md)

**Tool:**
```bash
# Anchor-ID-Check (manuell)
# Extrahiere alle [link](#anchor) und prüfe ob #anchor existiert
grep -oE '\[.*\]\(#[^)]+\)' docs/deployment/DEPLOYMENT.md

# Externe Link-Check (manuell oder mit linkchecker)
# Prüfe ob ../kubernetes/, ../monitoring/ Pfade korrekt sind
```

---

### Phase 5: Code-Blöcke & Syntax (10 min)

**Checklist:**
- [ ] **Syntax-Highlighting:**
  - [ ] Alle Bash-Blöcke haben ```bash
  - [ ] Alle YAML-Blöcke haben ```yaml
  - [ ] Alle JSON-Blöcke haben ```json
  - [ ] Alle Dockerfile-Blöcke haben ```dockerfile

- [ ] **YAML-Syntax:**
  - [ ] Alle YAML-Beispiele sind valide (keine Tabs, korrekte Indentation)
  - [ ] kubectl apply -f YAML sollte theoretisch funktionieren

- [ ] **JSON-Syntax:**
  - [ ] Alle JSON-Beispiele sind valide (keine Trailing Commas, korrekte Quotes)
  - [ ] jq-Commands sollten theoretisch funktionieren

- [ ] **Vollständigkeit:**
  - [ ] Keine abgeschnittenen Code-Blöcke (letzte Zeile sichtbar)
  - [ ] Alle Placeholder-Werte klar markiert (z.B. `<your-value>`)

**Tool:**
```bash
# YAML-Validierung (extrahiere YAML-Blöcke manuell und validiere)
# Beispiel:
cat docs/deployment/DEPLOYMENT.md | awk '/```yaml/,/```/' | yamllint -

# JSON-Validierung (extrahiere JSON-Blöcke manuell und validiere)
cat docs/deployment/DEPLOYMENT.md | awk '/```json/,/```/' | jq .
```

---

### Phase 6: Referenz-Updates (5 min)

**Checklist:**
- [ ] **docs/CLAUDE.md:**
  - [ ] Deployment-Links aktualisiert (keine Links zu README_DEPLOYMENT.md, DOCKER_DEPLOYMENT.md)
  - [ ] Link zu DEPLOYMENT.md mit erweiterter Beschreibung

- [ ] **README.md (Projekt-Root):**
  - [ ] Falls Deployment-Links vorhanden → aktualisiert

- [ ] **Andere Docs:**
  - [ ] Suche nach Links zu README_DEPLOYMENT.md / DOCKER_DEPLOYMENT.md in allen .md-Dateien
  - [ ] Aktualisiere alle gefundenen Links

**Tool:**
```bash
# Suche nach alten Links in allen .md-Dateien
grep -r "README_DEPLOYMENT.md" docs/
grep -r "DOCKER_DEPLOYMENT.md" docs/

# Erwartung: Keine Treffer (außer in _analysis_consolidation.md, _strategy_consolidation.md)
```

---

## Acceptance Criteria (Quality Gates)

### Must-Have (Blocking)
- ✅ Alle 7 Kapitel sind vollständig implementiert
- ✅ Keine Security-Issues (TLS, Kubectl, Docker)
- ✅ Keine Hardcoded Credentials
- ✅ Alle Unique-Inhalte aus 3 Quell-Dokumenten sind vorhanden
- ✅ Table of Contents ist vollständig und funktioniert
- ✅ Keine Broken Links

### Should-Have (Non-Blocking)
- ✅ Markdown-Linting bestanden (optional)
- ✅ YAML/JSON-Syntax-Validierung (optional)
- ✅ External Links funktionieren (HTTP 200 Check)
- ✅ Cross-References zwischen Kapiteln sind vorhanden

### Nice-to-Have
- ✅ Screenshots/Diagramme (falls vorhanden) migriert
- ✅ Appendix mit Migration Notes (Hinweis auf alte Dateinamen)
- ✅ Glossar (optional)

---

## Fallback-Strategien

### Fallback 1: Security-Issue gefunden
**Aktion:**
1. Rollback zur letzten sicheren Version (git revert)
2. Security-Issue dokumentieren (_security_issues.md)
3. Manuelle Korrektur in Schritt 5
4. Re-run Security Review

### Fallback 2: Content-Verlust festgestellt
**Aktion:**
1. Content-Mapping aus Schritt 2 überprüfen
2. Fehlenden Content aus Quell-Dokumenten nachholen
3. Re-run Content-Vollständigkeitsprüfung

### Fallback 3: Broken Links
**Aktion:**
1. Liste aller Broken Links erstellen
2. Links aktualisieren (Pfade korrigieren)
3. Re-run Link-Check

---

## Test-Tools (Optional)

### Markdown-Linting
```bash
# Installation (npm)
npm install -g markdownlint-cli

# Run
markdownlint docs/deployment/DEPLOYMENT.md
```

### YAML-Validierung
```bash
# Installation (pip)
pip install yamllint

# Run
yamllint -s docs/deployment/DEPLOYMENT.md
# (manuell YAML-Blöcke extrahieren)
```

### JSON-Validierung
```bash
# Eingebaut (jq)
jq . < example.json
```

### Link-Checking
```bash
# Installation (npm)
npm install -g markdown-link-check

# Run
markdown-link-check docs/deployment/DEPLOYMENT.md
```

---

## Timeline

| Phase | Dauer | Verantwortlich |
|-------|-------|----------------|
| Strukturprüfung | 5 min | Manual Review |
| Content-Vollständigkeit | 15 min | Manual Review + Keyword Check |
| Security Review | 20 min | Manual Review + Pattern Check |
| Links & Cross-References | 10 min | Manual Review + Link Checker |
| Code-Blöcke & Syntax | 10 min | Manual Review + YAML/JSON Linting |
| Referenz-Updates | 5 min | Manual Review + grep |

**Gesamt:** ~65 Minuten (1 Stunde)

---

## Testing-Report-Template

### Report für Schritt 6

```markdown
# Testing Report – Deployment-Dokumentations-Konsolidierung

**Datum:** 2025-11-24
**Phase:** Schritt 6 von CAP_CLAUDE_WORKFLOW_V2
**Tester:** Claude Code

---

## Test-Ergebnisse

### Phase 1: Strukturprüfung
- ✅ Table of Contents vollständig
- ✅ 7 Hauptkapitel vorhanden
- ✅ Markdown-Linting bestanden
- ✅ Überschriften-Hierarchie korrekt

### Phase 2: Content-Vollständigkeit
- ✅ Kapitel 1: Quick Start (4/4 Abschnitte vollständig)
- ✅ Kapitel 2: Docker Deployment (6/6 Abschnitte vollständig)
- ✅ Kapitel 3: Kubernetes Deployment (5/5 Abschnitte vollständig)
- ✅ Kapitel 4: TLS/mTLS (4/4 Abschnitte vollständig)
- ✅ Kapitel 5: Monitoring (5/5 Abschnitte vollständig)
- ✅ Kapitel 6: Configuration (3/3 Abschnitte vollständig)
- ✅ Kapitel 7: Troubleshooting (4/4 Abschnitte vollständig)

### Phase 3: Security Review
- ✅ TLS/mTLS-Konfigurationen unverändert
- ✅ Kubectl/Helm-Commands syntaktisch korrekt
- ✅ Docker-Commands sicher (keine Privilege Escalation)
- ✅ Keine Hardcoded Credentials

### Phase 4: Links & Cross-References
- ✅ Interne Links funktionieren (100%)
- ✅ Externe Links funktionieren (100%)
- ✅ Keine Broken Links

### Phase 5: Code-Blöcke & Syntax
- ✅ Syntax-Highlighting korrekt (100%)
- ✅ YAML-Syntax valide (100%)
- ✅ JSON-Syntax valide (100%)
- ✅ Keine abgeschnittenen Code-Blöcke

### Phase 6: Referenz-Updates
- ✅ docs/CLAUDE.md aktualisiert
- ✅ README.md aktualisiert (falls vorhanden)
- ✅ Keine alten Links gefunden

---

## Zusammenfassung

**Test-Status:** ✅ PASSED
**Gefundene Issues:** 0
**Blocked by:** Keine

**Recommendation:** Proceed to Schritt 7 (Git Commit)

---

**Report erstellt:** 2025-11-24
**Tester:** Claude Code
```

---

## Approval

**Testing-Plan genehmigt:** ⏳ Warte auf User-Bestätigung

**Nächster Schritt:** Schritt 4 – Security Review (Pre-Implementation)

---

**Dokument erstellt:** 2025-11-24
**Autor:** Claude Code (CAP_CLAUDE_WORKFLOW_V2)
**Version:** 1.0
