/**
 * CAP Verification Result Data Model
 * Source: WEBUI_BACKEND_STATUS.md Section 2.3
 *
 * @description Response from POST /verify endpoint
 * @deterministic Same inputs → same verification result
 */

export interface ConstraintResult {
  constraint: string;
  status: 'pass' | 'fail';
  message?: string;
}

export interface VerificationResult {
  status: 'valid' | 'invalid' | 'error';
  proof_valid: boolean;
  policy_hash: string;
  manifest_hash: string;
  constraints_checked: number;
  constraints_passed: number;
  constraints_failed: number;
  constraint_results: ConstraintResult[];
  error?: string;
  verified_at: string;
}

/**
 * Type guard for VerificationResult
 * @pure No side effects
 */
export function isValidVerificationResult(data: unknown): data is VerificationResult {
  if (typeof data !== 'object' || data === null) return false;

  const v = data as Partial<VerificationResult>;

  return (
    (v.status === 'valid' || v.status === 'invalid' || v.status === 'error') &&
    typeof v.proof_valid === 'boolean' &&
    typeof v.policy_hash === 'string' &&
    typeof v.manifest_hash === 'string' &&
    Array.isArray(v.constraint_results)
  );
}
