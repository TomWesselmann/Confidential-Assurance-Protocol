/**
 * Tauri Backend API Client
 *
 * @description Provides type-safe access to Tauri commands for offline proof verification
 * @architecture Imperative Shell - calls into Tauri (Rust) backend
 * @version 2.0 - Extended for Proofer Workflow
 */

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

// ============================================================================
// Request/Response Types (matches Rust backend)
// ============================================================================

export interface VerifyBundleRequest {
  /** Absolute path to bundle (ZIP file or directory) */
  bundlePath: string;

  /** Verification options */
  options?: {
    /** Check timestamp validity (default: false for offline) */
    checkTimestamp?: boolean;

    /** Check registry match (default: false for offline) */
    checkRegistry?: boolean;
  };
}

export interface VerifyBundleResponse {
  /** Verification status: "ok" or "fail" */
  status: 'ok' | 'fail';

  /** Bundle ID (UUID v4) */
  bundleId: string;

  /** Manifest hash (SHA3-256, 0x-prefixed) */
  manifestHash: string;

  /** Proof hash (SHA3-256, 0x-prefixed) */
  proofHash: string;

  /** Signature validation result */
  signatureValid: boolean;

  /** Timestamp validation result (optional) */
  timestampValid?: boolean;

  /** Registry match result (optional) */
  registryMatch?: boolean;

  /** Detailed verification results */
  details: {
    manifest_hash: string;
    proof_hash: string;
    checks_passed: number;
    checks_total: number;
    statement_validation: Array<{
      field: string;
      status: 'ok' | 'mismatch';
      expected?: string;
      found?: string;
    }>;
    signature_present: boolean;
    signature_count?: number;
  };
}

export interface BundleInfo {
  /** Bundle ID (UUID) */
  bundleId: string;

  /** Schema version (e.g., "cap-bundle.v1") */
  schema: string;

  /** Bundle creation timestamp (RFC3339) */
  createdAt: string;

  /** Proof units in bundle */
  proofUnits: Array<{
    id: string;
    policyId: string;
    backend: string;
  }>;

  /** Total number of files in bundle */
  fileCount: number;
}

// ============================================================================
// Tauri Command Wrappers
// ============================================================================

/**
 * Verifies a proof bundle using Tauri backend
 *
 * This function is completely offline - no network requests are made.
 * All verification happens locally using the cap-agent core library.
 *
 * @param request - Bundle path and verification options
 * @returns Verification report with detailed results
 * @throws Error if bundle not found, invalid, or verification fails
 *
 * @example
 * ```typescript
 * const result = await verifyBundle({
 *   bundlePath: '/path/to/bundle.zip',
 *   options: {
 *     checkTimestamp: false,
 *     checkRegistry: false
 *   }
 * });
 *
 * if (result.status === 'ok') {
 *   console.log('Verification successful!');
 * }
 * ```
 */
export async function verifyBundle(
  request: VerifyBundleRequest
): Promise<VerifyBundleResponse> {
  try {
    return await invoke<VerifyBundleResponse>('verify_bundle', { request });
  } catch (error) {
    // Re-throw with more context
    const errorMessage = error instanceof Error ? error.message : String(error);
    throw new Error(`Bundle verification failed: ${errorMessage}`);
  }
}

/**
 * Gets bundle metadata without verification (for preview)
 *
 * Useful for displaying bundle info before running full verification.
 *
 * @param bundlePath - Absolute path to bundle (ZIP or directory)
 * @returns Bundle metadata
 * @throws Error if bundle not found or invalid
 *
 * @example
 * ```typescript
 * const info = await getBundleInfo('/path/to/bundle.zip');
 * console.log(`Bundle contains ${info.fileCount} files`);
 * ```
 */
export async function getBundleInfo(bundlePath: string): Promise<BundleInfo> {
  try {
    return await invoke<BundleInfo>('get_bundle_info', { bundlePath });
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    throw new Error(`Failed to load bundle info: ${errorMessage}`);
  }
}

// ============================================================================
// File Dialog Helpers (using Tauri Dialog Plugin)
// ============================================================================

/**
 * Opens a file dialog to select a bundle (ZIP file)
 *
 * @returns Selected file path or null if cancelled
 */
export async function selectBundleFile(): Promise<string | null> {
  const { open } = await import('@tauri-apps/plugin-dialog');

  const selected = await open({
    multiple: false,
    directory: false,
    filters: [
      {
        name: 'Bundle',
        extensions: ['zip'],
      },
    ],
    title: 'Select Bundle ZIP File',
  });

  return selected as string | null;
}

/**
 * Opens a directory dialog to select a bundle directory
 *
 * @returns Selected directory path or null if cancelled
 */
export async function selectBundleDirectory(): Promise<string | null> {
  const { open } = await import('@tauri-apps/plugin-dialog');

  const selected = await open({
    multiple: false,
    directory: true,
    title: 'Select Bundle Directory',
  });

  return selected as string | null;
}

// ============================================================================
// PROOFER WORKFLOW TYPES (matches Rust backend types.rs)
// ============================================================================

/** CSV type for import (lowercase to match Rust serde) */
export type CsvType = 'suppliers' | 'ubos' | 'sanctions' | 'jurisdictions';

/** Project information */
export interface ProjectInfo {
  path: string;
  name: string;
  createdAt: string;
}

/** Project status with workflow progress */
export interface ProjectStatus {
  info: ProjectInfo;
  hasSuppliersCSv: boolean;
  hasUbosCsv: boolean;
  hasPolicy: boolean;
  hasCommitments: boolean;
  hasManifest: boolean;
  hasProof: boolean;
  currentStep: string;
}

/** CSV import result */
export interface ImportResult {
  csv_type: string;
  record_count: number;
  hash: string;
  destination: string;
}

/** Commitments (Merkle roots) result */
export interface CommitmentsResult {
  supplier_root: string;
  ubo_root: string;
  company_root: string;
  path: string;
}

/** Policy information */
export interface PolicyInfo {
  name: string;
  version: string;
  hash: string;
  rules_count: number;
  path: string;
}

/** Manifest build result */
export interface ManifestResult {
  manifest_hash: string;
  path: string;
  supplier_root: string;
  ubo_root: string;
  policy_hash: string;
}

/** Proof build result */
export interface ProofResult {
  proof_hash: string;
  path: string;
  backend: string;
}

/** Proof progress event payload */
export interface ProofProgress {
  percent: number;
  message: string;
}

/** Bundle export result */
export interface ExportResult {
  bundle_path: string;
  size_bytes: number;
  hash: string;
  files: string[];
}

// ============================================================================
// PROOFER WORKFLOW COMMANDS
// ============================================================================

/**
 * Creates a new project in the workspace
 */
export async function createProject(
  workspace: string,
  name: string
): Promise<ProjectInfo> {
  try {
    return await invoke<ProjectInfo>('create_project', { workspace, name });
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    throw new Error(`Failed to create project: ${errorMessage}`);
  }
}

/**
 * Lists all projects in a workspace
 */
export async function listProjects(workspace: string): Promise<ProjectInfo[]> {
  try {
    return await invoke<ProjectInfo[]>('list_projects', { workspace });
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    throw new Error(`Failed to list projects: ${errorMessage}`);
  }
}

// ============================================================================
// SIMPLIFIED PROJECT API (Uses configured workspace)
// ============================================================================

/**
 * Lists all projects in the configured workspace
 */
export async function listAllProjects(): Promise<ProjectInfo[]> {
  try {
    return await invoke<ProjectInfo[]>('list_all_projects');
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    throw new Error(`Failed to list projects: ${errorMessage}`);
  }
}

/**
 * Creates a new project with auto-generated name
 */
export async function createNewProject(): Promise<ProjectInfo> {
  try {
    return await invoke<ProjectInfo>('create_new_project');
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    throw new Error(`Failed to create project: ${errorMessage}`);
  }
}

/**
 * Creates a new project in a temporary directory
 * Best for workflows where user saves bundle at the end
 */
export async function createTempProject(): Promise<ProjectInfo> {
  try {
    return await invoke<ProjectInfo>('create_temp_project');
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    throw new Error(`Failed to create temp project: ${errorMessage}`);
  }
}

/**
 * Creates a new project in a specific folder chosen by user
 */
export async function createProjectInFolder(folder: string): Promise<ProjectInfo> {
  try {
    return await invoke<ProjectInfo>('create_project_in_folder', { folder });
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    throw new Error(`Failed to create project: ${errorMessage}`);
  }
}

/**
 * Opens a directory dialog to select a working folder
 */
export async function selectWorkingFolder(): Promise<string | null> {
  const { open } = await import('@tauri-apps/plugin-dialog');

  const selected = await open({
    multiple: false,
    directory: true,
    title: 'Arbeitsordner für Proof auswählen',
  });

  return selected as string | null;
}

// ============================================================================
// APP SETTINGS API
// ============================================================================

/** App info response */
export interface AppInfo {
  /** Current workspace path */
  workspacePath: string;
  /** Whether this is the first run */
  isFirstRun: boolean;
  /** Whether a custom path is set */
  hasCustomPath: boolean;
}

/**
 * Gets app info including workspace path and first-run status
 */
export async function getAppInfo(): Promise<AppInfo> {
  try {
    const result = await invoke<{
      workspace_path: string;
      is_first_run: boolean;
      has_custom_path: boolean;
    }>('get_app_info');
    return {
      workspacePath: result.workspace_path,
      isFirstRun: result.is_first_run,
      hasCustomPath: result.has_custom_path,
    };
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    throw new Error(`Failed to get app info: ${errorMessage}`);
  }
}

/**
 * Sets the workspace path
 */
export async function setWorkspacePath(path: string): Promise<string> {
  try {
    return await invoke<string>('set_workspace_path', { path });
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    throw new Error(`Failed to set workspace path: ${errorMessage}`);
  }
}

/**
 * Resets workspace to default (~/Documents/CAP-Proofs)
 */
export async function resetWorkspacePath(): Promise<string> {
  try {
    return await invoke<string>('reset_workspace_path');
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    throw new Error(`Failed to reset workspace path: ${errorMessage}`);
  }
}

/**
 * Opens a directory dialog to select a new workspace
 */
export async function selectWorkspaceFolder(): Promise<string | null> {
  const { open } = await import('@tauri-apps/plugin-dialog');

  const selected = await open({
    multiple: false,
    directory: true,
    title: 'Speicherort für Proofs auswählen',
  });

  return selected as string | null;
}

/**
 * Gets the status of a project
 */
export async function getProjectStatus(project: string): Promise<ProjectStatus> {
  try {
    return await invoke<ProjectStatus>('get_project_status', { project });
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    throw new Error(`Failed to get project status: ${errorMessage}`);
  }
}

/**
 * Imports a CSV file into the project
 */
export async function importCsv(
  project: string,
  csvType: CsvType,
  filePath: string
): Promise<ImportResult> {
  try {
    return await invoke<ImportResult>('import_csv', {
      project,
      csvType,
      filePath,
    });
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    throw new Error(`Failed to import CSV: ${errorMessage}`);
  }
}

/**
 * Creates commitments (Merkle roots) from imported CSVs
 */
export async function createCommitments(
  project: string
): Promise<CommitmentsResult> {
  try {
    return await invoke<CommitmentsResult>('create_commitments', { project });
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    throw new Error(`Failed to create commitments: ${errorMessage}`);
  }
}

/**
 * Loads a policy file into the project
 */
export async function loadPolicy(
  project: string,
  policyPath: string
): Promise<PolicyInfo> {
  try {
    return await invoke<PolicyInfo>('load_policy', { project, policyPath });
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    throw new Error(`Failed to load policy: ${errorMessage}`);
  }
}

/**
 * Builds the manifest from commitments and policy
 */
export async function buildManifest(project: string): Promise<ManifestResult> {
  try {
    return await invoke<ManifestResult>('build_manifest', { project });
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    throw new Error(`Failed to build manifest: ${errorMessage}`);
  }
}

/**
 * Builds the proof (long-running operation with progress events)
 *
 * @param project - Project path
 * @param onProgress - Callback for progress updates
 * @returns Proof result and unlisten function
 */
export async function buildProof(
  project: string,
  onProgress?: (progress: ProofProgress) => void
): Promise<ProofResult> {
  let unlisten: UnlistenFn | undefined;

  try {
    // Set up progress listener if callback provided
    if (onProgress) {
      unlisten = await listen<ProofProgress>('proof:progress', (event) => {
        onProgress(event.payload);
      });
    }

    // Call the command
    const result = await invoke<ProofResult>('build_proof', { project });

    return result;
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    throw new Error(`Failed to build proof: ${errorMessage}`);
  } finally {
    // Clean up listener
    if (unlisten) {
      unlisten();
    }
  }
}

/**
 * Exports the bundle as a ZIP file
 */
export async function exportBundle(
  project: string,
  output: string
): Promise<ExportResult> {
  try {
    return await invoke<ExportResult>('export_bundle', { project, output });
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    throw new Error(`Failed to export bundle: ${errorMessage}`);
  }
}

// ============================================================================
// Additional File Dialog Helpers
// ============================================================================

/**
 * Opens a file dialog to select a CSV file
 */
export async function selectCsvFile(): Promise<string | null> {
  const { open } = await import('@tauri-apps/plugin-dialog');

  const selected = await open({
    multiple: false,
    directory: false,
    filters: [
      {
        name: 'CSV',
        extensions: ['csv'],
      },
    ],
    title: 'CSV-Datei auswählen',
  });

  return selected as string | null;
}

/**
 * Opens a file dialog to select a policy file (YAML)
 */
export async function selectPolicyFile(): Promise<string | null> {
  const { open } = await import('@tauri-apps/plugin-dialog');

  const selected = await open({
    multiple: false,
    directory: false,
    filters: [
      {
        name: 'Policy',
        extensions: ['yml', 'yaml'],
      },
    ],
    title: 'Policy-Datei auswählen',
  });

  return selected as string | null;
}

/**
 * Opens a save dialog for bundle export
 */
export async function selectExportPath(): Promise<string | null> {
  const { save } = await import('@tauri-apps/plugin-dialog');

  const selected = await save({
    filters: [
      {
        name: 'Bundle',
        extensions: ['zip'],
      },
    ],
    title: 'Bundle speichern unter',
    defaultPath: `cap-bundle-${new Date().toISOString().split('T')[0]}.zip`,
  });

  return selected;
}

/**
 * Opens a directory dialog to select workspace
 */
export async function selectWorkspace(): Promise<string | null> {
  const { open } = await import('@tauri-apps/plugin-dialog');

  const selected = await open({
    multiple: false,
    directory: true,
    title: 'Workspace-Ordner auswählen',
  });

  return selected as string | null;
}

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Truncates a hash for display (0x1234...abcd)
 */
export function truncateHash(hash: string, chars: number = 8): string {
  if (!hash || hash.length <= chars * 2 + 3) return hash;
  return `${hash.slice(0, chars + 2)}...${hash.slice(-chars)}`;
}

/**
 * Formats file size in human readable format
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

// ============================================================================
// File Content Reading (for Detail Views)
// ============================================================================

/**
 * Reads file content from within a project directory
 * Uses the secure read_file_content Tauri command instead of the FS plugin
 *
 * @param projectPath - Path to the project directory
 * @param relativePath - Relative path within the project (e.g., "input/policy.yml")
 */
export async function readFileContent(projectPath: string, relativePath: string): Promise<string> {
  return await invoke<string>('read_file_content', {
    project: projectPath,
    relativePath: relativePath,
  });
}

/**
 * Reads policy file content (YAML)
 */
export async function readPolicyContent(projectPath: string): Promise<string> {
  return await readFileContent(projectPath, 'input/policy.yml');
}

/**
 * Reads manifest file content (JSON)
 */
export async function readManifestContent(projectPath: string): Promise<string> {
  return await readFileContent(projectPath, 'build/manifest.json');
}

// ============================================================================
// AUDIT TYPES AND COMMANDS
// ============================================================================

/** Audit event result status */
export type AuditEventResult = 'OK' | 'WARN' | 'FAIL';

/** Unified audit event (supports both V1.0 and V2.0 formats) */
export interface AuditEvent {
  /** Sequence number (V1.0) */
  seq?: number;

  /** Timestamp (ISO 8601) */
  ts: string;

  /** Event type */
  event: string;

  /** Event details (V1.0 format) */
  details?: Record<string, unknown>;

  /** Policy ID (V2.0) */
  policyId?: string;

  /** IR hash (V2.0) */
  irHash?: string;

  /** Manifest hash (V2.0) */
  manifestHash?: string;

  /** Result status (V2.0) */
  result?: AuditEventResult;

  /** Run ID (V2.0) */
  runId?: string;

  /** Previous hash */
  prevHash: string;

  /** Self hash */
  selfHash: string;
}

/** Audit log with events and chain status */
export interface AuditLog {
  /** List of audit events */
  events: AuditEvent[];

  /** Total event count (for pagination) */
  totalCount: number;

  /** Whether the hash chain is valid */
  chainValid: boolean;

  /** Offset used for pagination */
  offset: number;

  /** Limit used for pagination */
  limit: number;
}

/** Error in hash chain */
export interface ChainError {
  /** Index of the tampered event */
  index: number;

  /** Event timestamp */
  timestamp: string;

  /** Error type */
  errorType: string;

  /** Expected hash */
  expected: string;

  /** Found hash */
  found: string;
}

/** Result of hash chain verification */
export interface ChainVerifyResult {
  /** Whether the chain is valid */
  valid: boolean;

  /** Number of events verified */
  verifiedCount: number;

  /** List of errors found */
  errors: ChainError[];

  /** Tail hash (last event hash) */
  tailHash?: string;
}

/**
 * Gets audit log for a project
 *
 * @param project - Path to the project directory
 * @param limit - Maximum number of events to return (default: 100)
 * @param offset - Number of events to skip (default: 0)
 * @returns Audit log with events and chain status
 */
export async function getAuditLog(
  project: string,
  limit?: number,
  offset?: number
): Promise<AuditLog> {
  try {
    return await invoke<AuditLog>('get_audit_log', {
      project,
      limit,
      offset,
    });
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    throw new Error(`Failed to get audit log: ${errorMessage}`);
  }
}

/**
 * Verifies the audit chain integrity
 *
 * @param project - Path to the project directory
 * @returns Chain verification result
 */
export async function verifyAuditChain(project: string): Promise<ChainVerifyResult> {
  try {
    return await invoke<ChainVerifyResult>('verify_audit_chain', { project });
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    throw new Error(`Failed to verify audit chain: ${errorMessage}`);
  }
}

/**
 * Formats an audit event timestamp for display
 */
export function formatAuditTimestamp(ts: string): string {
  try {
    const date = new Date(ts);
    return date.toLocaleString('de-DE', {
      day: '2-digit',
      month: '2-digit',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });
  } catch {
    return ts;
  }
}

/**
 * Gets a human-readable event type name
 */
export function getEventTypeName(eventType: string): string {
  const names: Record<string, string> = {
    project_created: 'Projekt erstellt',
    csv_imported: 'CSV importiert',
    commitments_created: 'Commitments erstellt',
    policy_loaded: 'Policy geladen',
    manifest_built: 'Manifest erstellt',
    proof_built: 'Proof erstellt',
    bundle_exported: 'Bundle exportiert',
    bundle_verifier_run: 'Bundle verifiziert',
    registry_entry_added: 'Registry-Eintrag hinzugefügt',
    verify_response: 'Verifikationsantwort',
    policy_compile: 'Policy kompiliert',
    keys_generated: 'Schlüssel generiert',
    manifest_signed: 'Manifest signiert',
    signature_verified: 'Signatur verifiziert',
  };
  return names[eventType] || eventType;
}

// ============================================================================
// SIGNING TYPES (matches Rust backend types.rs)
// ============================================================================

/** Key information returned from generate_keys and list_keys */
export interface KeyInfo {
  /** Key ID (first 16 bytes of BLAKE3 hash, hex encoded) */
  kid: string;

  /** Human-readable signer name */
  signerName: string;

  /** Path to public key file */
  publicKeyPath: string;

  /** SHA-256 fingerprint (first 8 bytes, hex encoded) */
  fingerprint: string;

  /** Creation timestamp (RFC3339) */
  createdAt: string;
}

/** Result of signing a manifest */
export interface SignResult {
  /** Whether signing was successful */
  success: boolean;

  /** Signer name */
  signer: string;

  /** Truncated signature hash for display */
  signatureHash: string;

  /** Full signature hex (0x-prefixed) */
  signatureHex: string;

  /** Path to the signed manifest */
  manifestPath: string;
}

/** Result of verifying a signature */
export interface SignatureVerifyResult {
  /** Whether the signature is valid */
  valid: boolean;

  /** Signer name from signature */
  signer: string;

  /** Algorithm used (e.g., "Ed25519") */
  algorithm: string;

  /** Error message if verification failed */
  error?: string;
}

// ============================================================================
// SIGNING COMMANDS
// ============================================================================

/**
 * Generates a new Ed25519 key pair for signing
 *
 * @param project - Path to the project directory
 * @param signerName - Human-readable name for the signer (e.g., "Company Name")
 * @returns KeyInfo with key details
 * @throws Error if key generation fails or signer name is invalid
 *
 * @example
 * ```typescript
 * const key = await generateKeys('/path/to/project', 'My Company');
 * console.log(`Key generated: ${key.kid}`);
 * ```
 */
export async function generateKeys(
  project: string,
  signerName: string
): Promise<KeyInfo> {
  try {
    return await invoke<KeyInfo>('generate_keys', { project, signerName });
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    throw new Error(`Failed to generate keys: ${errorMessage}`);
  }
}

/**
 * Lists all available signing keys in a project
 *
 * @param project - Path to the project directory
 * @returns Array of KeyInfo for each key found (sorted by creation date, newest first)
 *
 * @example
 * ```typescript
 * const keys = await listKeys('/path/to/project');
 * if (keys.length === 0) {
 *   console.log('No keys found');
 * }
 * ```
 */
export async function listKeys(project: string): Promise<KeyInfo[]> {
  try {
    return await invoke<KeyInfo[]>('list_keys', { project });
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    throw new Error(`Failed to list keys: ${errorMessage}`);
  }
}

/**
 * Signs the manifest with the specified key
 *
 * @param project - Path to the project directory
 * @param signerName - Name of the signer (must match existing key)
 * @returns SignResult with signature details
 * @throws Error if manifest not found, key not found, or signing fails
 *
 * @example
 * ```typescript
 * const result = await signProjectManifest('/path/to/project', 'My Company');
 * if (result.success) {
 *   console.log(`Signed with hash: ${result.signatureHash}`);
 * }
 * ```
 */
export async function signProjectManifest(
  project: string,
  signerName: string
): Promise<SignResult> {
  try {
    return await invoke<SignResult>('sign_project_manifest', { project, signerName });
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    throw new Error(`Failed to sign manifest: ${errorMessage}`);
  }
}

/**
 * Verifies the signature on the manifest
 *
 * @param project - Path to the project directory
 * @returns SignatureVerifyResult with verification status
 * @throws Error if manifest not found
 *
 * @example
 * ```typescript
 * const result = await verifyManifestSignature('/path/to/project');
 * if (result.valid) {
 *   console.log(`Valid signature from: ${result.signer}`);
 * } else {
 *   console.log(`Invalid: ${result.error}`);
 * }
 * ```
 */
export async function verifyManifestSignature(
  project: string
): Promise<SignatureVerifyResult> {
  try {
    return await invoke<SignatureVerifyResult>('verify_manifest_signature', { project });
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    throw new Error(`Failed to verify signature: ${errorMessage}`);
  }
}

/**
 * Validates a signer name (client-side pre-validation)
 *
 * @param name - Signer name to validate
 * @returns true if valid, error message if invalid
 */
export function validateSignerName(name: string): true | string {
  if (!name || name.trim().length === 0) {
    return 'Signer-Name ist erforderlich';
  }
  if (name.length > 64) {
    return 'Signer-Name zu lang (max. 64 Zeichen)';
  }
  if (name.includes('..') || name.includes('/') || name.includes('\\')) {
    return 'Ungültiger Signer-Name: Pfad-Zeichen nicht erlaubt';
  }
  if (!/^[a-zA-Z0-9_\- ]+$/.test(name)) {
    return 'Ungültiger Signer-Name: Nur Buchstaben, Zahlen, Unterstrich, Bindestrich und Leerzeichen erlaubt';
  }
  return true;
}
