# E2E Tests - CAP Desktop Prover

End-to-end tests für die CAP Desktop Prover Tauri Anwendung.

## Voraussetzungen

1. **tauri-driver** installieren:
   ```bash
   cargo install tauri-driver
   ```

2. **Tauri App** bauen (Release Build):
   ```bash
   cd ../src-tauri
   cargo build --release
   ```

3. **Dependencies** installieren:
   ```bash
   npm install
   ```

## Tests ausführen

### Alle E2E Tests
```bash
npm run e2e
```

### Einzelne Test-Suiten
```bash
# Prover Workflow Tests
npm run e2e:prover

# Verifier Workflow Tests
npm run e2e:verifier

# Audit Trail Tests
npm run e2e:audit
```

### Headless Mode (CI)
```bash
npm run e2e:headless
```

## Verzeichnisstruktur

```
e2e/
├── fixtures/           # Test-Dateien (CSV, Policy, Bundles)
│   ├── suppliers.csv
│   ├── ubos.csv
│   └── test-policy.json
├── pageobjects/        # Page Object Pattern
│   ├── BasePage.ts
│   ├── ProverPage.ts
│   ├── VerifierPage.ts
│   └── AuditPage.ts
├── specs/              # Test-Specs
│   ├── prover-workflow.e2e.ts
│   ├── verifier-workflow.e2e.ts
│   └── audit-trail.e2e.ts
├── screenshots/        # Screenshot bei Fehlern
├── wdio.conf.ts        # WebdriverIO Konfiguration
└── tsconfig.json       # TypeScript Config für E2E
```

## Test-Szenarien

### Prover Workflow (6 Schritte)
1. CSV Import (Suppliers + UBOs)
2. Commitments erstellen
3. Policy laden
4. Manifest bauen
5. Proof generieren
6. Bundle exportieren

### Verifier Workflow
1. Bundle laden (Drag & Drop oder File Dialog)
2. Verification starten
3. Ergebnisse anzeigen (Constraints, Hashes, Signatur)
4. Reset

### Audit Trail
1. Timeline anzeigen
2. Events filtern/suchen
3. Hash-Chain verifizieren
4. Event Details expandieren

## Page Object Pattern

Jede Seite hat ein eigenes Page Object mit:
- **Selectors**: Getter für UI-Elemente (data-testid)
- **Actions**: Methoden für Benutzerinteraktionen
- **Assertions**: Hilfsmethoden für Verifizierung

Beispiel:
```typescript
// ProverPage.ts
get buildProofButton() {
  return $('button*=Proof generieren');
}

async buildProof(): Promise<void> {
  const btn = await this.buildProofButton;
  await btn.click();
  // ...
}
```

## data-testid Konventionen

| Element | Format | Beispiel |
|---------|--------|----------|
| View Container | `{name}-view` | `prover-view`, `verifier-view` |
| Buttons | `{action}-btn` | `next-step-btn`, `verify-btn` |
| Status | `{name}-status` | `import-status`, `chain-status` |
| Hash Display | `{name}-hash` | `manifest-hash`, `proof-hash` |
| Progress | `{name}-progress` | `proof-progress` |

## Debugging

Screenshots werden automatisch bei Testfehlern erstellt:
```
e2e/screenshots/test-name-2024-01-15T10-30-00.png
```

Logs vom tauri-driver:
```bash
RUST_LOG=debug npm run e2e
```
