/**
 * CAP Manifest Data Model
 * Source: WEBUI_BACKEND_STATUS.md Section 2.1
 *
 * @description Immutable data structure representing a CAP compliance manifest
 * @deterministic Hash calculation is reproducible
 */

export interface TimeAnchor {
  blockchain: string;
  tx_hash: string;
  block_number: number;
  timestamp: string;
}

export interface PolicyInfo {
  name: string;
  version: string;
  hash: string;
  description?: string;
}

export interface AuditEventCategories {
  data_changes?: number;
  compliance?: number;
  system?: number;
}

export interface AuditTimeRange {
  start: string; // RFC3339 timestamp
  end: string;   // RFC3339 timestamp
}

export interface AuditInfo {
  // Core fields (always present)
  tail_digest: string;
  events_count: number;

  // Extended fields (optional, CAP Manifest v0.1+)
  time_range?: AuditTimeRange;
  event_categories?: AuditEventCategories;
  last_event_type?: string;
  hash_function?: string; // e.g., "SHA3-256"
  chain_type?: string; // e.g., "linear_hash_chain"
  integrity?: 'verified' | 'unverified' | 'failed';
  audit_chain_version?: number;

  // Legacy fields (backward compatibility)
  first_event_timestamp?: string;
  last_event_timestamp?: string;
}

export interface ProofInfo {
  type: string; // "none" | "mock" | "halo2" | "zk" | "spartan" | "risc0"
  status: string; // "none" | "ok" | "failed"
}

export interface SignatureInfo {
  kid: string;
  algorithm: string;
  signature: string;
  signed_at: string;
}

export interface Manifest {
  version: string;
  created_at: string;
  supplier_root: string;
  ubo_root: string;
  company_commitment_root: string;
  policy: PolicyInfo;
  audit: AuditInfo;
  proof: ProofInfo;
  signatures: SignatureInfo[];
  time_anchor?: TimeAnchor;
}

/**
 * Type guard to validate Manifest structure
 * @pure No side effects
 */
export function isValidManifest(data: unknown): data is Manifest {
  if (typeof data !== 'object' || data === null) return false;

  const m = data as Partial<Manifest>;

  return (
    typeof m.version === 'string' &&
    typeof m.created_at === 'string' &&
    typeof m.supplier_root === 'string' &&
    typeof m.ubo_root === 'string' &&
    typeof m.company_commitment_root === 'string' &&
    typeof m.policy === 'object' &&
    typeof m.audit === 'object' &&
    typeof m.proof === 'object' &&
    Array.isArray(m.signatures)
  );
}
