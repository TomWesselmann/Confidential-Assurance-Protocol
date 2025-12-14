/**
 * CAP VerificationResult Model Tests
 *
 * @description Unit tests for VerificationResult type guard
 */

import { describe, it, expect } from 'vitest';
import { isValidVerificationResult } from '../../core/models/VerificationResult';
import type { VerificationResult } from '../../core/models/VerificationResult';

describe('VerificationResult model', () => {
  describe('isValidVerificationResult', () => {
    const validResult: VerificationResult = {
      status: 'valid',
      proof_valid: true,
      policy_hash: '0x' + 'a'.repeat(64),
      manifest_hash: '0x' + 'b'.repeat(64),
      constraints_checked: 5,
      constraints_passed: 5,
      constraints_failed: 0,
      constraint_results: [],
      verified_at: '2025-12-14T10:30:00.000Z',
    };

    it('should accept valid result with status "valid"', () => {
      expect(isValidVerificationResult(validResult)).toBe(true);
    });

    it('should accept result with status "invalid"', () => {
      const invalidResult: VerificationResult = {
        ...validResult,
        status: 'invalid',
        proof_valid: false,
        constraints_passed: 3,
        constraints_failed: 2,
      };
      expect(isValidVerificationResult(invalidResult)).toBe(true);
    });

    it('should accept result with status "error"', () => {
      const errorResult: VerificationResult = {
        ...validResult,
        status: 'error',
        proof_valid: false,
        error: 'Verification failed due to invalid format',
      };
      expect(isValidVerificationResult(errorResult)).toBe(true);
    });

    it('should accept result with constraint_results', () => {
      const resultWithConstraints: VerificationResult = {
        ...validResult,
        constraint_results: [
          { constraint: 'supplier_count', status: 'pass' },
          { constraint: 'ubo_threshold', status: 'pass', message: 'All UBOs verified' },
          { constraint: 'high_risk_check', status: 'fail', message: 'Found high risk supplier' },
        ],
      };
      expect(isValidVerificationResult(resultWithConstraints)).toBe(true);
    });

    it('should reject null', () => {
      expect(isValidVerificationResult(null)).toBe(false);
    });

    it('should reject undefined', () => {
      expect(isValidVerificationResult(undefined)).toBe(false);
    });

    it('should reject non-object', () => {
      expect(isValidVerificationResult('string')).toBe(false);
      expect(isValidVerificationResult(123)).toBe(false);
      expect(isValidVerificationResult(true)).toBe(false);
      expect(isValidVerificationResult([])).toBe(false);
    });

    it('should reject invalid status', () => {
      const invalid = {
        ...validResult,
        status: 'unknown',
      };
      expect(isValidVerificationResult(invalid)).toBe(false);
    });

    it('should reject missing status', () => {
      const { status, ...rest } = validResult;
      expect(isValidVerificationResult(rest)).toBe(false);
    });

    it('should reject missing proof_valid', () => {
      const { proof_valid, ...rest } = validResult;
      expect(isValidVerificationResult(rest)).toBe(false);
    });

    it('should reject proof_valid as non-boolean', () => {
      const invalid = {
        ...validResult,
        proof_valid: 'true',
      };
      expect(isValidVerificationResult(invalid)).toBe(false);
    });

    it('should reject missing policy_hash', () => {
      const { policy_hash, ...rest } = validResult;
      expect(isValidVerificationResult(rest)).toBe(false);
    });

    it('should reject policy_hash as non-string', () => {
      const invalid = {
        ...validResult,
        policy_hash: 12345,
      };
      expect(isValidVerificationResult(invalid)).toBe(false);
    });

    it('should reject missing manifest_hash', () => {
      const { manifest_hash, ...rest } = validResult;
      expect(isValidVerificationResult(rest)).toBe(false);
    });

    it('should reject manifest_hash as non-string', () => {
      const invalid = {
        ...validResult,
        manifest_hash: null,
      };
      expect(isValidVerificationResult(invalid)).toBe(false);
    });

    it('should reject missing constraint_results', () => {
      const { constraint_results, ...rest } = validResult;
      expect(isValidVerificationResult(rest)).toBe(false);
    });

    it('should reject constraint_results as non-array', () => {
      const invalid = {
        ...validResult,
        constraint_results: 'not-an-array',
      };
      expect(isValidVerificationResult(invalid)).toBe(false);
    });

    it('should accept empty constraint_results array', () => {
      expect(isValidVerificationResult(validResult)).toBe(true);
      expect(validResult.constraint_results).toEqual([]);
    });

    it('should accept result without optional error field', () => {
      const { error, ...withoutError } = {
        ...validResult,
        error: 'some error',
      };
      expect(isValidVerificationResult(withoutError)).toBe(true);
    });

    it('should accept result with error field', () => {
      const withError: VerificationResult = {
        ...validResult,
        status: 'error',
        error: 'Something went wrong',
      };
      expect(isValidVerificationResult(withError)).toBe(true);
    });

    it('should handle all three status values', () => {
      const statuses: Array<'valid' | 'invalid' | 'error'> = ['valid', 'invalid', 'error'];

      statuses.forEach((status) => {
        const result: VerificationResult = {
          ...validResult,
          status,
        };
        expect(isValidVerificationResult(result)).toBe(true);
      });
    });
  });
});
