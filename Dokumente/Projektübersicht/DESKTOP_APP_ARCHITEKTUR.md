# CAP Desktop Proofer - Systemarchitektur & Inventur

**Version:** 1.0.0
**Datum:** 2024-11-27
**Status:** Implementiert (Phase 2 abgeschlossen)

---

## Inhaltsverzeichnis

1. [Übersicht](#1-übersicht)
2. [Technologie-Stack](#2-technologie-stack)
3. [Verzeichnisstruktur](#3-verzeichnisstruktur)
4. [Backend-Architektur (Rust/Tauri)](#4-backend-architektur-rusttauri)
5. [Frontend-Architektur (React/TypeScript)](#5-frontend-architektur-reacttypescript)
6. [Datenfluss & IPC-Kommunikation](#6-datenfluss--ipc-kommunikation)
7. [Workflow-System](#7-workflow-system)
8. [Audit-Trail-System](#8-audit-trail-system)
9. [Sicherheitsarchitektur](#9-sicherheitsarchitektur)
10. [Dateiformat-Referenz](#10-dateiformat-referenz)
11. [API-Referenz (Tauri Commands)](#11-api-referenz-tauri-commands)
12. [Build & Deployment](#12-build--deployment)

---

## 1. Übersicht

### 1.1 Zweck
CAP Desktop Proofer ist eine **Offline-First** Desktop-Anwendung zur Erstellung und Verifikation von kryptografischen Compliance-Nachweisen (CAP Bundles). Die App ermöglicht:

- **Proofer-Modus:** 6-Schritte-Workflow zur Erstellung von Compliance-Beweisen
- **Verifier-Modus:** Offline-Verifikation von CAP Bundles
- **Audit-Modus:** Visualisierung des tamper-proof Audit-Trails

### 1.2 Architektur-Prinzipien

```
┌─────────────────────────────────────────────────────────────┐
│                    CAP Desktop Proofer                       │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────────┐  │
│  │              React Frontend (WebView)                 │  │
│  │  - App.tauri.tsx (Main Application)                  │  │
│  │  - Zustand Store (State Management)                  │  │
│  │  - Tailwind CSS (Styling)                            │  │
│  └──────────────────────────────────────────────────────┘  │
│                          │ IPC (invoke)                     │
│                          ▼                                  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Tauri Backend (Rust)                     │  │
│  │  - Commands (Tauri IPC Handlers)                     │  │
│  │  - cap-agent Library (Core Crypto)                   │  │
│  │  - Security Module (Validation)                      │  │
│  └──────────────────────────────────────────────────────┘  │
│                          │                                  │
│                          ▼                                  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Local Filesystem                         │  │
│  │  - Workspace/Projects                                │  │
│  │  - Audit Logs (JSONL)                                │  │
│  │  - CAP Bundles (ZIP)                                 │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. Technologie-Stack

### 2.1 Backend (Rust)

| Komponente | Version | Zweck |
|------------|---------|-------|
| **Tauri** | 2.x | Desktop-Framework, IPC |
| **tauri-plugin-dialog** | 2.x | Native Datei-Dialoge |
| **tauri-plugin-fs** | 2.x | Filesystem-Zugriff |
| **serde** | 1.x | Serialisierung (JSON/YAML) |
| **blake3** | 1.x | BLAKE3 Hashing |
| **sha3** | 0.10 | SHA3-256 Hashing |
| **csv** | 1.x | CSV-Parsing |
| **zip** | 2.x | ZIP-Archiv-Erstellung |
| **chrono** | 0.4 | Datums-/Zeithandling |
| **uuid** | 1.x | UUID-Generierung |
| **cap-agent** | local | Core Crypto Library |

### 2.2 Frontend (TypeScript/React)

| Komponente | Version | Zweck |
|------------|---------|-------|
| **React** | 19.x | UI Framework |
| **TypeScript** | 5.9 | Type Safety |
| **Zustand** | 5.x | State Management |
| **Tailwind CSS** | 4.x | Styling |
| **@tauri-apps/api** | 2.x | Tauri IPC Bridge |
| **react-dropzone** | 14.x | Drag & Drop Upload |
| **Vite** | 7.x | Build Tool |
| **Vitest** | 4.x | Testing |

---

## 3. Verzeichnisstruktur

### 3.1 Repository-Struktur

```
LsKG-Agent/
├── src-tauri/                    # Tauri/Rust Backend
│   ├── src/
│   │   ├── main.rs               # Entry Point
│   │   ├── lib.rs                # Library (Command Registration)
│   │   ├── types.rs              # Shared Types
│   │   ├── security.rs           # Security Validation
│   │   ├── audit_logger.rs       # Audit Event Writer
│   │   └── commands/             # Tauri Commands
│   │       ├── mod.rs            # Module Exports
│   │       ├── project.rs        # Project Management
│   │       ├── import.rs         # CSV Import
│   │       ├── commitments.rs    # Merkle Root Creation
│   │       ├── policy.rs         # Policy Loading
│   │       ├── manifest.rs       # Manifest Building
│   │       ├── proof.rs          # Proof Generation
│   │       ├── export.rs         # Bundle Export
│   │       ├── verify.rs         # Bundle Verification
│   │       └── audit.rs          # Audit Log Reading
│   ├── Cargo.toml                # Rust Dependencies
│   └── tauri.conf.json           # Tauri Configuration
│
├── webui/                        # React Frontend
│   ├── src/
│   │   ├── main.tsx              # React Entry Point
│   │   ├── App.tauri.tsx         # Main App Component
│   │   ├── lib/
│   │   │   └── tauri.ts          # Tauri API Client
│   │   ├── store/
│   │   │   ├── workflowStore.ts  # Proofer Workflow State
│   │   │   └── verificationStore.ts
│   │   └── components/
│   │       ├── layout/
│   │       │   └── ProjectSidebar.tsx
│   │       ├── workflow/
│   │       │   ├── ImportView.tsx
│   │       │   ├── CommitmentsView.tsx
│   │       │   ├── PolicyView.tsx
│   │       │   ├── ManifestView.tsx
│   │       │   ├── ProofView.tsx
│   │       │   ├── ExportView.tsx
│   │       │   └── WorkflowStepper.tsx
│   │       ├── audit/
│   │       │   └── AuditTimeline.tsx
│   │       ├── upload/
│   │       │   └── BundleUploader.tauri.tsx
│   │       └── verification/
│   │           └── VerificationView.tsx
│   ├── package.json              # NPM Dependencies
│   └── vite.config.ts            # Vite Configuration
│
└── agent/                        # cap-agent Core Library
    └── src/
        ├── commitment.rs         # Merkle Root Functions
        └── io.rs                 # CSV I/O
```

### 3.2 Projekt-Verzeichnisstruktur (Runtime)

```
workspace/
└── projekt-name/
    ├── taurin.project.json       # Projekt-Metadaten
    ├── input/
    │   ├── suppliers.csv         # Lieferanten-Daten
    │   ├── ubos.csv              # UBO-Daten
    │   └── policy.yml            # Policy-Definition
    ├── build/
    │   ├── commitments.json      # Merkle Roots
    │   ├── manifest.json         # Manifest
    │   ├── proof.capz            # Proof (JSON)
    │   └── proof.dat             # Proof (Binary)
    ├── audit/
    │   └── agent.audit.jsonl     # Audit Trail
    └── export/
        └── cap-bundle-*.zip      # Exportierte Bundles
```

---

## 4. Backend-Architektur (Rust/Tauri)

### 4.1 Modul-Übersicht

```
src-tauri/src/
├── lib.rs                 # 88 Zeilen
│   ├── Module Registration
│   ├── Plugin Initialization
│   └── Command Handler Registration
│
├── types.rs               # 338 Zeilen
│   ├── CsvType (enum)
│   ├── StepStatus (enum)
│   ├── ProjectInfo, ProjectStatus, ProjectMeta
│   ├── ImportResult, CommitmentsResult
│   ├── PolicyInfo, ManifestResult
│   ├── ProofResult, ProofProgress
│   ├── ExportResult
│   ├── VerifyBundleRequest/Response
│   ├── BundleInfo, ProofUnitInfo
│   └── AuditEvent, AuditLog, ChainVerifyResult
│
├── security.rs            # ~200 Zeilen
│   ├── validate_project_name()
│   ├── validate_path_exists()
│   ├── validate_regular_file()
│   ├── validate_file_size()
│   └── sanitize_error_message()
│
├── audit_logger.rs        # 250 Zeilen
│   ├── log_event()         # V1.0 Format Writer
│   ├── compute_digest()    # SHA3-256 Hash
│   └── events module
│       ├── project_created()
│       ├── csv_imported()
│       ├── commitments_created()
│       ├── policy_loaded()
│       ├── manifest_built()
│       ├── proof_built()
│       └── bundle_exported()
│
└── commands/
    ├── mod.rs             # Re-exports
    ├── project.rs         # 290 Zeilen
    │   ├── create_project()
    │   ├── list_projects()
    │   ├── get_project_status()
    │   └── read_file_content()
    │
    ├── import.rs          # 220 Zeilen
    │   ├── import_csv()
    │   └── validate_and_count_csv()
    │
    ├── commitments.rs     # 190 Zeilen
    │   └── create_commitments()
    │
    ├── policy.rs          # 165 Zeilen
    │   └── load_policy()
    │
    ├── manifest.rs        # 250 Zeilen
    │   └── build_manifest()
    │
    ├── proof.rs           # 205 Zeilen
    │   └── build_proof()  # mit Progress Events
    │
    ├── export.rs          # 280 Zeilen
    │   └── export_bundle()
    │
    ├── verify.rs          # ~300 Zeilen
    │   ├── verify_bundle()
    │   └── get_bundle_info()
    │
    └── audit.rs           # 350 Zeilen
        ├── get_audit_log()
        └── verify_audit_chain()
```

### 4.2 Command-Registrierung

```rust
// lib.rs
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            // Project Management
            create_project,
            list_projects,
            get_project_status,
            read_file_content,
            // Proofer Workflow
            import_csv,
            create_commitments,
            load_policy,
            build_manifest,
            build_proof,
            export_bundle,
            // Verifier
            verify_bundle,
            get_bundle_info,
            // Audit
            get_audit_log,
            verify_audit_chain,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## 5. Frontend-Architektur (React/TypeScript)

### 5.1 Komponenten-Hierarchie

```
App.tauri.tsx
├── Header
│   ├── Sidebar Toggle Button
│   ├── App Title ("CAP Desktop Proofer")
│   └── Mode Tabs (Proofer | Verifier | Audit)
│
├── ProjectSidebar (conditional)
│   ├── Workspace Selector
│   ├── Project List
│   └── Create Project Dialog
│
└── Main Content
    ├── [Proofer Mode]
    │   ├── No Project → Create Project Button
    │   └── Project Selected
    │       ├── Project Info Bar
    │       ├── WorkflowStepper (6 Steps)
    │       └── Step Content
    │           ├── ImportView
    │           ├── CommitmentsView
    │           ├── PolicyView
    │           ├── ManifestView
    │           ├── ProofView
    │           └── ExportView
    │
    ├── [Verifier Mode]
    │   ├── No Result → BundleUploader
    │   └── Result → VerificationView
    │
    └── [Audit Mode]
        ├── No Project → Hinweistext
        └── Project Selected → AuditTimeline
```

### 5.2 State Management (Zustand)

#### workflowStore.ts

```typescript
interface WorkflowState {
  // Project
  projectPath: string | null;
  projectName: string | null;

  // Current workflow position
  currentStep: WorkflowStep; // 'import' | 'commitments' | 'policy' | 'manifest' | 'proof' | 'export'

  // Step states
  steps: Record<WorkflowStep, StepState>;

  // Step results
  importResults: { suppliers: ImportResult | null; ubos: ImportResult | null };
  commitmentsResult: CommitmentsResult | null;
  policyInfo: PolicyInfo | null;
  manifestResult: ManifestResult | null;
  proofResult: ProofResult | null;
  exportResult: ExportResult | null;

  // Actions
  setProject(path, name): void;
  initializeFromStatus(path, name, status): void;
  setCurrentStep(step): void;
  goToNextStep(): void;
  goToPreviousStep(): void;
  reset(): void;
}
```

### 5.3 Tauri API Client (lib/tauri.ts)

```typescript
// File Dialog Helpers
selectBundleFile(): Promise<string | null>
selectCsvFile(): Promise<string | null>
selectPolicyFile(): Promise<string | null>
selectExportPath(): Promise<string | null>
selectWorkspace(): Promise<string | null>

// Project Management
createProject(workspace, name): Promise<ProjectInfo>
listProjects(workspace): Promise<ProjectInfo[]>
getProjectStatus(project): Promise<ProjectStatus>

// Workflow Commands
importCsv(project, csvType, filePath): Promise<ImportResult>
createCommitments(project): Promise<CommitmentsResult>
loadPolicy(project, policyPath): Promise<PolicyInfo>
buildManifest(project): Promise<ManifestResult>
buildProof(project, onProgress?): Promise<ProofResult>
exportBundle(project, output): Promise<ExportResult>

// Verifier
verifyBundle(request): Promise<VerifyBundleResponse>
getBundleInfo(bundlePath): Promise<BundleInfo>

// Audit
getAuditLog(project, limit?, offset?): Promise<AuditLog>
verifyAuditChain(project): Promise<ChainVerifyResult>

// File Content
readFileContent(projectPath, relativePath): Promise<string>
```

---

## 6. Datenfluss & IPC-Kommunikation

### 6.1 Frontend → Backend (invoke)

```
┌─────────────┐        invoke('command', {args})        ┌─────────────┐
│   React     │ ──────────────────────────────────────▶ │    Rust     │
│  Component  │                                         │   Command   │
└─────────────┘                                         └─────────────┘
      │                                                        │
      │         Promise<Result>                                │
      │ ◀──────────────────────────────────────────────────── │
      ▼                                                        ▼
 Update State                                            Access FS
 Render UI                                               Compute Hashes
                                                         Write Files
```

### 6.2 Backend → Frontend (Events)

```
┌─────────────┐        emit('proof:progress', payload)   ┌─────────────┐
│    Rust     │ ──────────────────────────────────────▶  │   React     │
│   Command   │                                          │  Listener   │
└─────────────┘                                          └─────────────┘
      │                                                        │
      │              (für Long-Running Operations)             │
      │              z.B. Proof-Generierung                    │
      ▼                                                        ▼
 Periodic Progress                                       Update Progress
 Updates                                                 Bar/Status
```

### 6.3 Sequenzdiagramm: CSV Import

```
┌──────────┐     ┌────────────┐     ┌──────────────┐     ┌────────────┐
│   User   │     │  ImportView│     │  tauri.ts    │     │  import.rs │
└────┬─────┘     └─────┬──────┘     └──────┬───────┘     └─────┬──────┘
     │ Click "Import"  │                   │                   │
     │────────────────▶│                   │                   │
     │                 │ selectCsvFile()   │                   │
     │                 │──────────────────▶│                   │
     │                 │                   │ Native Dialog     │
     │                 │                   │◀──────────────────│
     │                 │ "/path/to/file"   │                   │
     │                 │◀──────────────────│                   │
     │                 │ importCsv(...)    │                   │
     │                 │──────────────────▶│                   │
     │                 │                   │ invoke()          │
     │                 │                   │──────────────────▶│
     │                 │                   │                   │ Validate CSV
     │                 │                   │                   │ Copy to input/
     │                 │                   │                   │ Log to audit
     │                 │                   │                   │ Return hash
     │                 │                   │ ImportResult      │
     │                 │                   │◀──────────────────│
     │                 │ ImportResult      │                   │
     │                 │◀──────────────────│                   │
     │                 │ Update Store      │                   │
     │ Show Success    │                   │                   │
     │◀────────────────│                   │                   │
```

---

## 7. Workflow-System

### 7.1 6-Schritte-Workflow

```
┌─────────┐    ┌─────────────┐    ┌────────┐    ┌──────────┐    ┌───────┐    ┌────────┐
│ Import  │───▶│ Commitments │───▶│ Policy │───▶│ Manifest │───▶│ Proof │───▶│ Export │
└─────────┘    └─────────────┘    └────────┘    └──────────┘    └───────┘    └────────┘
     │               │                │              │              │             │
     ▼               ▼                ▼              ▼              ▼             ▼
 suppliers.csv   commitments.json  policy.yml   manifest.json  proof.capz   bundle.zip
 ubos.csv
```

### 7.2 Step-Abhängigkeiten

| Schritt | Voraussetzung | Erzeugt | Audit Event |
|---------|---------------|---------|-------------|
| **Import** | Projekt existiert | `input/suppliers.csv`, `input/ubos.csv` | `csv_imported` |
| **Commitments** | Beide CSVs importiert | `build/commitments.json` | `commitments_created` |
| **Policy** | - | `input/policy.yml` | `policy_loaded` |
| **Manifest** | Commitments + Policy | `build/manifest.json` | `manifest_built` |
| **Proof** | Manifest existiert | `build/proof.capz` | `proof_built` |
| **Export** | Proof existiert | `export/*.zip` | `bundle_exported` |

### 7.3 Workflow-Persistenz

Der Workflow-Status wird durch Dateisystem-Prüfung wiederhergestellt:

```rust
// get_project_status() in project.rs
let has_suppliers_csv = project_path.join("input/suppliers.csv").exists();
let has_ubos_csv = project_path.join("input/ubos.csv").exists();
let has_policy = project_path.join("input/policy.yml").exists();
let has_commitments = project_path.join("build/commitments.json").exists();
let has_manifest = project_path.join("build/manifest.json").exists();
let has_proof = project_path.join("build/proof.capz").exists();

// Determine current step
let current_step = if has_proof { "export" }
    else if has_manifest { "proof" }
    else if has_commitments && has_policy { "manifest" }
    else if has_suppliers_csv && has_ubos_csv { "commitments" }
    else { "import" };
```

---

## 8. Audit-Trail-System

### 8.1 Audit-Format (V1.0)

```json
{
  "seq": 1,
  "ts": "2024-11-27T10:30:00.000000Z",
  "event": "csv_imported",
  "details": {
    "csv_type": "suppliers",
    "record_count": 42,
    "hash": "0x..."
  },
  "prev_digest": "0x0000...0000",
  "digest": "0x..."
}
```

### 8.2 Hash-Chain Verification

```
┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐
│ Event 1 │───▶│ Event 2 │───▶│ Event 3 │───▶│ Event N │
│ (Genesis)│    │         │    │         │    │         │
│ prev=0x0│    │prev=H1  │    │prev=H2  │    │prev=Hn-1│
│ H1=SHA3 │    │H2=SHA3  │    │H3=SHA3  │    │Hn=SHA3  │
└─────────┘    └─────────┘    └─────────┘    └─────────┘
```

**Hash-Berechnung:**
```rust
fn compute_digest(seq, ts, event, details, prev_digest) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(seq.to_string().as_bytes());
    hasher.update(ts.as_bytes());
    hasher.update(event.as_bytes());
    hasher.update(details.to_string().as_bytes());
    hasher.update(prev_digest.as_bytes());
    format!("0x{}", hex::encode(hasher.finalize()))
}
```

### 8.3 Audit-Timeline UI

```
┌────────────────────────────────────────────────────────┐
│  Hash-Kette: ✓ Verifiziert (7 Events)                 │
├────────────────────────────────────────────────────────┤
│  ● 27.11.2024, 10:30:15 - Projekt erstellt            │
│  │   Details: name="test-project"                     │
│  │                                                     │
│  ● 27.11.2024, 10:31:02 - CSV importiert              │
│  │   Details: csv_type="suppliers", count=42          │
│  │                                                     │
│  ● 27.11.2024, 10:31:15 - CSV importiert              │
│  │   Details: csv_type="ubos", count=15               │
│  │                                                     │
│  ● 27.11.2024, 10:32:00 - Commitments erstellt        │
│  │   Details: supplier_root="0x...", ubo_root="0x..." │
│  ...                                                   │
└────────────────────────────────────────────────────────┘
```

---

## 9. Sicherheitsarchitektur

### 9.1 Input Validation

| Prüfung | Modul | Grenzwerte |
|---------|-------|------------|
| **Projektname** | `security.rs` | Max 128 Zeichen, keine `..`, `/`, `\` |
| **CSV-Dateigröße** | `security.rs` | Max 50 MB |
| **CSV-Feldlänge** | `security.rs` | Max 10.000 Zeichen |
| **Policy-Dateigröße** | `security.rs` | Max 1 MB |
| **Datei-Typ** | `security.rs` | Nur reguläre Dateien (keine Symlinks) |

### 9.2 Path Traversal Prevention

```rust
// security.rs
pub fn validate_project_name(name: &str) -> Result<(), String> {
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err("Invalid project name: path traversal not allowed".into());
    }
    Ok(())
}

// project.rs (read_file_content)
if relative_path.contains("..") || relative_path.starts_with('/') {
    return Err("Invalid path: path traversal not allowed".into());
}
let canonical_file = file_path.canonicalize()?;
if !canonical_file.starts_with(&canonical_project) {
    return Err("Invalid path: file must be within project directory".into());
}
```

### 9.3 Error Message Sanitization

```rust
pub fn sanitize_error_message(msg: &str) -> String {
    // Remove absolute paths from error messages
    let re = Regex::new(r"(/[a-zA-Z0-9_\-./]+)").unwrap();
    re.replace_all(msg, "[path]").to_string()
}
```

---

## 10. Dateiformat-Referenz

### 10.1 taurin.project.json

```json
{
  "schema": "taurin.project.v1",
  "name": "mein-projekt",
  "createdAt": "2024-11-27T10:30:00Z",
  "description": null,
  "capVersion": "0.1.0"
}
```

### 10.2 commitments.json

```json
{
  "supplier_root": "0x...",
  "ubo_root": "0x...",
  "company_root": "0x...",
  "supplier_count": 42,
  "ubo_count": 15,
  "created_at": "2024-11-27T10:35:00Z"
}
```

### 10.3 manifest.json

```json
{
  "version": "manifest.v0",
  "created_at": "2024-11-27T10:40:00Z",
  "supplier_root": "0x...",
  "ubo_root": "0x...",
  "company_commitment_root": "0x...",
  "policy": {
    "name": "LkSG Demo Policy",
    "version": "lksg.v1",
    "hash": "0x..."
  },
  "audit": {
    "tail_digest": "0x...",
    "events_count": 5
  },
  "proof": {
    "type": "simplified_zk",
    "status": "generated"
  },
  "signatures": []
}
```

### 10.4 Bundle-Struktur (ZIP)

```
cap-bundle.zip
├── _meta.json          # Bundle-Metadaten (cap-bundle.v1)
├── manifest.json       # Manifest
├── proof.capz          # Proof-Daten
├── commitments.json    # Optional: Commitments
└── policy.yml          # Optional: Policy
```

---

## 11. API-Referenz (Tauri Commands)

### 11.1 Project Management

| Command | Parameter | Return | Beschreibung |
|---------|-----------|--------|--------------|
| `create_project` | `workspace: String, name: String` | `ProjectInfo` | Erstellt neues Projekt |
| `list_projects` | `workspace: String` | `Vec<ProjectInfo>` | Listet alle Projekte |
| `get_project_status` | `project: String` | `ProjectStatus` | Gibt Workflow-Status zurück |
| `read_file_content` | `project: String, relativePath: String` | `String` | Liest Datei aus Projekt |

### 11.2 Workflow Commands

| Command | Parameter | Return | Beschreibung |
|---------|-----------|--------|--------------|
| `import_csv` | `project, csvType, filePath` | `ImportResult` | Importiert CSV |
| `create_commitments` | `project` | `CommitmentsResult` | Erstellt Merkle Roots |
| `load_policy` | `project, policyPath` | `PolicyInfo` | Lädt Policy |
| `build_manifest` | `project` | `ManifestResult` | Baut Manifest |
| `build_proof` | `project` | `ProofResult` | Generiert Proof |
| `export_bundle` | `project, output` | `ExportResult` | Exportiert Bundle |

### 11.3 Verifier Commands

| Command | Parameter | Return | Beschreibung |
|---------|-----------|--------|--------------|
| `verify_bundle` | `request: VerifyBundleRequest` | `VerifyBundleResponse` | Verifiziert Bundle |
| `get_bundle_info` | `bundlePath: String` | `BundleInfo` | Liest Bundle-Metadaten |

### 11.4 Audit Commands

| Command | Parameter | Return | Beschreibung |
|---------|-----------|--------|--------------|
| `get_audit_log` | `project, limit?, offset?` | `AuditLog` | Liest Audit-Events |
| `verify_audit_chain` | `project` | `ChainVerifyResult` | Prüft Hash-Kette |

---

## 12. Build & Deployment

### 12.1 Development

```bash
# Frontend Development Server
cd webui
npm install
npm run dev

# Tauri Development
cd src-tauri
cargo build
cargo tauri dev
```

### 12.2 Production Build

```bash
# Frontend Build
cd webui
npm run build

# Tauri Release Build
cd src-tauri
cargo build --release

# Full Bundle (macOS .app)
npx @tauri-apps/cli build
```

### 12.3 Binary-Größe (Optimiert)

| Konfiguration | Wert |
|---------------|------|
| `panic` | abort |
| `codegen-units` | 1 |
| `lto` | true |
| `opt-level` | z (size) |
| `strip` | true |

---

## Anhang A: Test-Abdeckung

### Backend Tests (35 Tests)

```
commands::audit::tests::                    6 Tests
commands::project::tests::                  5 Tests
commands::import::tests::                   4 Tests
commands::commitments::tests::              3 Tests
commands::policy::tests::                   3 Tests
commands::manifest::tests::                 2 Tests
commands::export::tests::                   1 Test
commands::verify::tests::                   2 Tests
security::tests::                           5 Tests
audit_logger::tests::                       2 Tests
```

### Frontend Tests (18 Tests)

```
utils/validation.test.ts                    18 Tests
```

---

## Anhang B: Abhängigkeiten (cap-agent)

Die Desktop-App nutzt `cap-agent` als lokale Dependency für:

- `compute_supplier_root()` - Merkle Root für Suppliers
- `compute_ubo_root()` - Merkle Root für UBOs
- `compute_company_root()` - Combined Root
- `read_suppliers_csv()` - CSV Parsing
- `read_ubos_csv()` - CSV Parsing

---

*Dokument erstellt: 2024-11-27*
*Letzte Aktualisierung: 2024-11-27*
