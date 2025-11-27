/**
 * CAP REST API Types
 * Source: WEBUI_BACKEND_STATUS.md Section 1
 *
 * @description Type-safe API contracts
 */

import type { Manifest } from '../models/Manifest';

/**
 * Health Check Response
 * GET /healthz
 */
export interface HealthResponse {
  status: 'OK' | 'ERROR';
  version: string;
  build_hash: string | null;
}

/**
 * Readiness Check Response
 * GET /readyz
 */
export interface ReadinessResponse {
  status: 'READY' | 'NOT_READY';
  checks: {
    registry: boolean;
    blob_store: boolean;
    key_store: boolean;
  };
}

/**
 * Verify Context (Backend API v0.11.0)
 * Contains proof data for verification
 */
export interface VerifyContext {
  supplier_hashes?: string[];
  ubo_hashes?: string[];
  company_commitment_root?: string;
  sanctions_root?: string;
  jurisdiction_root?: string;
}

/**
 * Verify Request Options
 */
export interface VerifyRequestOptions {
  adaptive?: boolean;
  check_timestamp?: boolean;
  check_registry?: boolean;
}

/**
 * Verify Request Payload (Backend API v0.11.0)
 * POST /verify
 *
 * Supports two modes:
 * - Mode A: policy_id (reference to stored policy)
 * - Mode B: embedded IR
 */
export interface VerifyRequest {
  policy_id?: string;
  context: VerifyContext;
  backend?: string; // "mock" | "zkvm" | "halo2"
  options?: VerifyRequestOptions;
}

/**
 * Verify Response (Backend API v0.11.0)
 * POST /verify
 */
export interface VerifyResponse {
  result: string; // "OK" | "FAIL" | "WARN"
  manifest_hash: string;
  proof_hash: string;
  trace?: unknown;
  signature?: string;
  timestamp?: string;
  report: {
    status: string;
    manifest_hash: string;
    proof_hash: string;
    signature_valid: boolean;
    details: unknown[];
  };
}

/**
 * Upload Response (POST /proof/upload)
 */
export interface UploadResponse {
  manifest: Manifest;
  proof_base64: string;
  company_commitment_root: string;
  package_info: {
    size_bytes: number;
    file_count: number;
    files: string[];
  };
}

/**
 * API Error Response
 */
export interface ApiError {
  error: string;
  message: string;
  status_code: number;
  timestamp: string;
}

/**
 * Type guard for API Error
 * @pure No side effects
 */
export function isApiError(data: unknown): data is ApiError {
  if (typeof data !== 'object' || data === null) return false;

  const e = data as Partial<ApiError>;

  return (
    typeof e.error === 'string' &&
    typeof e.message === 'string' &&
    typeof e.status_code === 'number'
  );
}
