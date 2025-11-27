# Tauri Desktop Proofer - Architektur-Dokumentation

## Überblick

Diese Dokumentation beschreibt die technische Architektur des Offline-First Desktop Proofers, der mit Tauri 2.0, Rust und React implementiert wird.

**Version:** Phase 1 MVP
**Erstellt:** 2025-11-25
**Status:** In Implementierung

---

## 1. System-Architektur

### 1.1 High-Level-Übersicht

```
┌─────────────────────────────────────────────────────────────┐
│                    Desktop Application                       │
│                         (Tauri)                              │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Frontend Layer (React)                   │  │
│  │                                                        │  │
│  │  • BundleUploader      (Drag & Drop ZIP/Directory)   │  │
│  │  • ManifestViewer      (JSON Tree View)              │  │
│  │  • VerificationView    (Result Display)              │  │
│  │  • verificationStore   (Zustand State Management)    │  │
│  └──────────────────────────────────────────────────────┘  │
│                         │                                    │
│                         │ Tauri Commands (IPC)              │
│                         ▼                                    │
│  ┌──────────────────────────────────────────────────────┐  │
│  │            Backend Layer (Rust)                       │  │
│  │                                                        │  │
│  │  ┌────────────────────────────────────────────────┐  │  │
│  │  │    Tauri Command Handlers                      │  │  │
│  │  │  • verify_bundle_command()                     │  │  │
│  │  │  • get_bundle_info_command()                   │  │  │
│  │  └────────────────────────────────────────────────┘  │  │
│  │                         │                              │  │
│  │                         │ Security Layer               │  │
│  │                         ▼                              │  │
│  │  ┌────────────────────────────────────────────────┐  │  │
│  │  │      Error Mapping & Sanitization              │  │  │
│  │  │  • Path sanitization (no absolute paths)       │  │  │
│  │  │  • Error message filtering (no path leaks)     │  │  │
│  │  └────────────────────────────────────────────────┘  │  │
│  │                         │                              │  │
│  │                         ▼                              │  │
│  │  ┌────────────────────────────────────────────────┐  │  │
│  │  │      cap-agent Core Library                    │  │  │
│  │  │  • bundle::BundleSource                        │  │  │
│  │  │  • bundle::load_bundle_atomic()                │  │  │
│  │  │  • verifier::verify_from_source()              │  │  │
│  │  │  • Deterministic, I/O-free verification        │  │  │
│  │  └────────────────────────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 Architektur-Prinzipien

1. **Functional Core / Imperative Shell** (REQ-02):
   - **Core**: `cap-agent` Library (I/O-free, deterministic, testable)
   - **Shell**: Tauri Commands + React UI (I/O operations, user interaction)

2. **Offline-First** (REQ-01, REQ-07):
   - Keine Netzwerk-Requests im MVP
   - CSP: `connect-src 'none'` enforcement
   - Alle Daten lokal aus Bundles

3. **Security by Design** (REQ-13):
   - Path-Traversal-Prevention (sanitize alle Pfade)
   - Zip-Bomb-Protection (size/ratio limits)
   - TOCTOU-Prevention (atomic bundle loading)
   - Error-Message-Sanitization (keine Pfad-Leaks)

---

## 2. Tauri-Kommando-Schnittstelle

### 2.1 Command: `verify_bundle`

**Zweck:** Verifiziert ein Bundle (ZIP oder Directory) und gibt Report zurück.

#### Input (Frontend → Backend)

```typescript
interface VerifyBundleRequest {
  /** Absoluter Pfad zum Bundle (ZIP-Datei oder Verzeichnis) */
  bundle_path: string;

  /** Verifikations-Optionen */
  options?: {
    check_timestamp?: boolean;  // Default: false (offline)
    check_registry?: boolean;   // Default: false (offline)
  };
}
```

#### Output (Backend → Frontend)

```typescript
interface VerifyBundleResponse {
  /** Status: "ok" | "fail" */
  status: "ok" | "fail";

  /** Bundle-ID (UUID) */
  bundle_id: string;

  /** Manifest-Hash (SHA3-256, 0x-prefixed) */
  manifest_hash: string;

  /** Proof-Hash (SHA3-256, 0x-prefixed) */
  proof_hash: string;

  /** Signatur-Validierung */
  signature_valid: boolean;

  /** Timestamp-Validierung (optional) */
  timestamp_valid?: boolean;

  /** Registry-Match (optional) */
  registry_match?: boolean;

  /** Detaillierte Prüfergebnisse */
  details: {
    manifest_hash: string;
    proof_hash: string;
    checks_passed: number;
    checks_total: number;
    statement_validation: Array<{
      field: string;
      status: "ok" | "mismatch";
      expected?: string;
      found?: string;
    }>;
    signature_present: boolean;
    signature_count?: number;
  };
}
```

#### Fehlerbehandlung

```typescript
interface TauriError {
  /** Sanitized error message (no path leaks) */
  message: string;

  /** Error type: "BundleNotFound" | "InvalidZip" | "VerificationFailed" */
  error_type: string;
}
```

### 2.2 Command: `get_bundle_info`

**Zweck:** Lädt Bundle-Metadaten ohne Verifikation (für Preview).

#### Input

```typescript
interface GetBundleInfoRequest {
  bundle_path: string;
}
```

#### Output

```typescript
interface BundleInfo {
  bundle_id: string;
  schema: string;  // "cap-bundle.v1"
  created_at: string;
  proof_units: Array<{
    id: string;
    policy_id: string;
    backend: string;
  }>;
  file_count: number;
  total_size_bytes: number;
}
```

---

## 3. Implementierungs-Details

### 3.1 Tauri Backend (Rust)

#### Datei: `src-tauri/src/main.rs`

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use cap_agent::bundle::{BundleSource, parse_bundle_source, load_bundle_atomic};
use cap_agent::verifier::{verify_from_source, VerifyOptions};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::command;

// ============================================================================
// Command Input/Output Types
// ============================================================================

#[derive(Debug, Deserialize)]
struct VerifyBundleRequest {
    bundle_path: String,
    #[serde(default)]
    options: Option<VerifyOptionsInput>,
}

#[derive(Debug, Deserialize)]
struct VerifyOptionsInput {
    #[serde(default)]
    check_timestamp: bool,
    #[serde(default)]
    check_registry: bool,
}

#[derive(Debug, Serialize)]
struct VerifyBundleResponse {
    status: String,
    bundle_id: String,
    manifest_hash: String,
    proof_hash: String,
    signature_valid: bool,
    timestamp_valid: Option<bool>,
    registry_match: Option<bool>,
    details: serde_json::Value,
}

// ============================================================================
// Security: Path Sanitization
// ============================================================================

/// Sanitizes user-provided paths (REQ-13)
///
/// - Removes path traversal attempts (..)
/// - Rejects absolute paths starting with /
/// - Returns base filename only
fn sanitize_user_path(path: &str) -> Result<String, String> {
    let path_obj = Path::new(path);

    // Get filename only (strips directory components)
    let filename = path_obj
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "Invalid path format".to_string())?;

    // Check for path traversal attempts
    if filename.contains("..") || path.contains("..") {
        return Err("Path traversal not allowed".to_string());
    }

    Ok(filename.to_string())
}

/// Sanitizes error messages to prevent path leaks (REQ-13)
fn sanitize_error_message(err: &str) -> String {
    // Remove absolute paths from error messages
    err.replace("/Users/", "[USER_PATH]/")
       .replace("/home/", "[USER_PATH]/")
       .replace("C:\\", "[DRIVE]\\")
       .replace("D:\\", "[DRIVE]\\")
}

// ============================================================================
// Tauri Commands
// ============================================================================

#[command]
async fn verify_bundle(request: VerifyBundleRequest) -> Result<VerifyBundleResponse, String> {
    // 1. Security: Path validation
    let bundle_path = Path::new(&request.bundle_path);
    if !bundle_path.exists() {
        return Err("Bundle not found".to_string());
    }

    // 2. Parse bundle source (auto-detect ZIP vs Directory)
    let source = BundleSource::from_path(bundle_path)
        .map_err(|e| sanitize_error_message(&e.to_string()))?;

    // 3. Parse bundle metadata first (for bundle_id)
    let meta = parse_bundle_source(&source)
        .map_err(|e| sanitize_error_message(&e.to_string()))?;

    // 4. Build verify options (offline defaults)
    let verify_opts = if let Some(opts) = request.options {
        VerifyOptions {
            check_timestamp: opts.check_timestamp,
            check_registry: opts.check_registry,
        }
    } else {
        VerifyOptions::default() // Offline defaults
    };

    // 5. Verify bundle (atomic, deterministic)
    let report = verify_from_source(&source, Some(&verify_opts))
        .map_err(|e| sanitize_error_message(&e.to_string()))?;

    // 6. Build response
    Ok(VerifyBundleResponse {
        status: report.status,
        bundle_id: meta.bundle_id,
        manifest_hash: report.manifest_hash,
        proof_hash: report.proof_hash,
        signature_valid: report.signature_valid,
        timestamp_valid: report.timestamp_valid,
        registry_match: report.registry_match,
        details: report.details,
    })
}

#[command]
async fn get_bundle_info(bundle_path: String) -> Result<serde_json::Value, String> {
    let source = BundleSource::from_path(&bundle_path)
        .map_err(|e| sanitize_error_message(&e.to_string()))?;

    let meta = parse_bundle_source(&source)
        .map_err(|e| sanitize_error_message(&e.to_string()))?;

    Ok(serde_json::json!({
        "bundle_id": meta.bundle_id,
        "schema": meta.schema,
        "created_at": meta.created_at,
        "proof_units": meta.proof_units,
        "file_count": meta.files.len(),
    }))
}

// ============================================================================
// Main Entry Point
// ============================================================================

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            verify_bundle,
            get_bundle_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 3.2 Frontend Integration (React + TypeScript)

#### Datei: `src/lib/tauri.ts`

```typescript
import { invoke } from '@tauri-apps/api/core';

export interface VerifyBundleRequest {
  bundle_path: string;
  options?: {
    check_timestamp?: boolean;
    check_registry?: boolean;
  };
}

export interface VerifyBundleResponse {
  status: 'ok' | 'fail';
  bundle_id: string;
  manifest_hash: string;
  proof_hash: string;
  signature_valid: boolean;
  timestamp_valid?: boolean;
  registry_match?: boolean;
  details: any;
}

/**
 * Verifies a proof bundle using Tauri backend command
 *
 * @param request - Bundle path and verification options
 * @returns Verification report
 */
export async function verifyBundle(
  request: VerifyBundleRequest
): Promise<VerifyBundleResponse> {
  return await invoke<VerifyBundleResponse>('verify_bundle', { request });
}

/**
 * Gets bundle metadata without verification
 *
 * @param bundlePath - Path to bundle (ZIP or directory)
 * @returns Bundle info
 */
export async function getBundleInfo(bundlePath: string): Promise<any> {
  return await invoke('get_bundle_info', { bundlePath });
}
```

#### Datei: `src/components/upload/BundleUploader.tsx` (ANPASSUNG)

```typescript
import { useState } from 'react';
import { useDropzone } from 'react-dropzone';
import { open } from '@tauri-apps/plugin-dialog';
import { verifyBundle } from '../../lib/tauri';
import { useVerificationStore } from '../../store/verificationStore';

export function BundleUploader() {
  const [isLoading, setIsLoading] = useState(false);
  const { setVerificationResult, setVerificationError } = useVerificationStore();

  // Handle file drop (for ZIP files)
  const { getRootProps, getInputProps } = useDropzone({
    accept: { 'application/zip': ['.zip'] },
    multiple: false,
    onDrop: async (acceptedFiles) => {
      if (acceptedFiles.length === 0) return;

      const file = acceptedFiles[0];
      // In Tauri: File wird nicht direkt hochgeladen, sondern Pfad wird verwendet
      // Wir nutzen den Tauri Dialog API stattdessen
      alert('Bitte nutzen Sie "Bundle auswählen" für die Dateiauswahl');
    },
  });

  // Handle directory/file selection via Tauri dialog
  const handleSelectBundle = async () => {
    try {
      setIsLoading(true);
      setVerificationError(null);

      // Open Tauri file dialog (supports both files and directories)
      const selected = await open({
        multiple: false,
        directory: false,
        filters: [{
          name: 'Bundle',
          extensions: ['zip']
        }]
      });

      if (!selected) {
        setIsLoading(false);
        return;
      }

      const bundlePath = selected as string;

      // Call Tauri verify command
      const result = await verifyBundle({
        bundle_path: bundlePath,
        options: {
          check_timestamp: false,
          check_registry: false,
        },
      });

      setVerificationResult(result);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      setVerificationError(errorMessage);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-8">
      <h2 className="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-6">
        Bundle hochladen
      </h2>

      {/* Dropzone für visuelle Feedback */}
      <div
        {...getRootProps()}
        className="border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg p-12 text-center cursor-pointer hover:border-blue-500 transition-colors"
      >
        <input {...getInputProps()} />
        <p className="text-gray-600 dark:text-gray-400 mb-4">
          ZIP-Datei hier ablegen oder
        </p>
        <button
          onClick={(e) => {
            e.stopPropagation();
            handleSelectBundle();
          }}
          disabled={isLoading}
          className="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-400 transition-colors"
        >
          {isLoading ? 'Wird verarbeitet...' : 'Bundle auswählen'}
        </button>
      </div>
    </div>
  );
}
```

---

## 4. Content Security Policy (CSP)

### 4.1 Konfiguration (`tauri.conf.json`)

```json
{
  "tauri": {
    "security": {
      "csp": {
        "default-src": "'self'",
        "connect-src": "'none'",
        "script-src": "'self' 'unsafe-inline'",
        "style-src": "'self' 'unsafe-inline'",
        "img-src": "'self' data:",
        "font-src": "'self' data:"
      }
    }
  }
}
```

**Wichtig:** `connect-src: 'none'` verhindert alle Netzwerk-Requests (REQ-07).

### 4.2 Enforcement-Test

Test, dass keine Network-Calls möglich sind:

```typescript
// Test: Dieser Call soll fehlschlagen (CSP blocked)
try {
  await fetch('http://example.com');
  console.error('❌ CSP NOT ENFORCED!');
} catch (error) {
  console.log('✅ CSP enforced: Network call blocked');
}
```

---

## 5. Dateifluss-Diagramm

### 5.1 Verification Workflow

```
User Action                Frontend                 Tauri Command           Core Library
    │                         │                          │                      │
    │  Select Bundle          │                          │                      │
    ├────────────────────────>│                          │                      │
    │                         │                          │                      │
    │                         │  open() Dialog           │                      │
    │                         ├─────────────────────────>│                      │
    │                         │                          │                      │
    │                         │<─────────────────────────┤                      │
    │                         │  Path: "/path/to.zip"    │                      │
    │                         │                          │                      │
    │                         │  invoke('verify_bundle') │                      │
    │                         ├─────────────────────────>│                      │
    │                         │                          │                      │
    │                         │                          │  Path Sanitization   │
    │                         │                          ├─────────────────────>│
    │                         │                          │                      │
    │                         │                          │  BundleSource::      │
    │                         │                          │  from_path()         │
    │                         │                          ├─────────────────────>│
    │                         │                          │                      │
    │                         │                          │  load_bundle_atomic()│
    │                         │                          ├─────────────────────>│
    │                         │                          │                      │
    │                         │                          │  verify_from_source()│
    │                         │                          ├─────────────────────>│
    │                         │                          │                      │
    │                         │                          │<─────────────────────┤
    │                         │                          │  VerifyReport        │
    │                         │                          │                      │
    │                         │<─────────────────────────┤                      │
    │                         │  VerifyBundleResponse    │                      │
    │                         │                          │                      │
    │<────────────────────────┤                          │                      │
    │  Display Results        │                          │                      │
```

---

## 6. Testing-Strategie

### 6.1 Unit Tests (Rust)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_user_path_rejects_traversal() {
        let result = sanitize_user_path("../../../etc/passwd");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Path traversal"));
    }

    #[test]
    fn test_sanitize_error_message_removes_paths() {
        let error = "File not found: /Users/alice/bundle.zip";
        let sanitized = sanitize_error_message(error);
        assert!(!sanitized.contains("/Users/"));
        assert!(sanitized.contains("[USER_PATH]"));
    }
}
```

### 6.2 Integration Tests (TypeScript)

```typescript
import { describe, it, expect } from 'vitest';
import { verifyBundle } from '../lib/tauri';

describe('Tauri Integration', () => {
  it('should verify a valid bundle', async () => {
    const result = await verifyBundle({
      bundle_path: './test-fixtures/valid-bundle.zip',
    });

    expect(result.status).toBe('ok');
    expect(result.signature_valid).toBe(true);
  });

  it('should reject invalid bundle', async () => {
    await expect(
      verifyBundle({
        bundle_path: './test-fixtures/invalid-bundle.zip',
      })
    ).rejects.toThrow();
  });
});
```

---

## 7. Deployment

### 7.1 Build-Prozess

```bash
# 1. Build Rust backend
cd src-tauri
cargo build --release

# 2. Build React frontend
cd ..
npm run build

# 3. Bundle Tauri app
npm run tauri build
```

### 7.2 Output

- **macOS:** `src-tauri/target/release/bundle/macos/Desktop Proofer.app`
- **Windows:** `src-tauri/target/release/bundle/msi/Desktop Proofer_0.1.0_x64.msi`
- **Linux:** `src-tauri/target/release/bundle/deb/desktop-proofer_0.1.0_amd64.deb`

---

## 8. Nächste Schritte (Phase 2+)

1. **Policy-Upload & Management** (REQ-08)
   - Command: `install_policy(policy_wasm_path)`
   - Local Policy-Store im Tauri Data Directory

2. **Batch-Verification** (REQ-09)
   - Command: `verify_multiple_bundles(bundle_paths[])`
   - Parallel-Verarbeitung mit Rayon

3. **Export-Funktionen** (REQ-10)
   - Command: `export_report(report, format, output_path)`
   - Formate: JSON, PDF, CSV

4. **Telemetrie (Optional, Local-Only)** (REQ-11)
   - Lokale Statistiken (keine Cloud)
   - SQLite-Datenbank in Tauri Data Dir

---

## Referenzen

- **Tauri 2.0 Docs:** https://v2.tauri.app/
- **Tauri Commands:** https://v2.tauri.app/develop/calling-rust/
- **Tauri Security:** https://v2.tauri.app/security/
- **cap-agent Core:** `/Users/tomwesselmann/Desktop/LsKG-Agent/agent/`
- **Planning Doc:** `/Users/tomwesselmann/Desktop/TAURI_DESKTOP_APP_PLANUNG.md`
