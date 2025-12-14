/**
 * CAP Verification Store Tests
 *
 * @description Unit tests for Zustand verification state management
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { useVerificationStore } from '../../store/verificationStore';
import type { VerifyBundleResponse } from '../../lib/tauri';

describe('verificationStore', () => {
  beforeEach(() => {
    // Reset store before each test
    useVerificationStore.getState().reset();
  });

  describe('initial state', () => {
    it('should have null verification result initially', () => {
      const state = useVerificationStore.getState();
      expect(state.verificationResult).toBeNull();
    });

    it('should have null verification error initially', () => {
      const state = useVerificationStore.getState();
      expect(state.verificationError).toBeNull();
    });

    it('should not be verifying initially', () => {
      const state = useVerificationStore.getState();
      expect(state.isVerifying).toBe(false);
    });
  });

  describe('setVerificationResult', () => {
    it('should set verification result', () => {
      const mockResult: VerifyBundleResponse = {
        status: 'ok',
        bundleId: 'test-bundle-id',
        manifestHash: '0x' + 'a'.repeat(64),
        proofHash: '0x' + 'b'.repeat(64),
        signatureValid: true,
        timestampValid: true,
        registryMatch: false,
        details: {
          manifest_hash: '0x' + 'a'.repeat(64),
          proof_hash: '0x' + 'b'.repeat(64),
          checks_passed: 5,
          checks_total: 5,
          statement_validation: [],
          signature_present: true,
          signature_count: 1,
        },
      };

      useVerificationStore.getState().setVerificationResult(mockResult);

      const state = useVerificationStore.getState();
      expect(state.verificationResult).toEqual(mockResult);
      expect(state.verificationError).toBeNull();
      expect(state.isVerifying).toBe(false);
    });

    it('should clear error when setting result', () => {
      // First set an error
      useVerificationStore.getState().setVerificationError('Some error');

      // Then set result
      const mockResult: VerifyBundleResponse = {
        status: 'ok',
        bundleId: 'test',
        manifestHash: '0x' + 'a'.repeat(64),
        proofHash: '0x' + 'b'.repeat(64),
        signatureValid: true,
        details: {
          manifest_hash: '0x' + 'a'.repeat(64),
          proof_hash: '0x' + 'b'.repeat(64),
          checks_passed: 5,
          checks_total: 5,
          statement_validation: [],
          signature_present: true,
        },
      };

      useVerificationStore.getState().setVerificationResult(mockResult);

      expect(useVerificationStore.getState().verificationError).toBeNull();
    });

    it('should stop verifying when setting result', () => {
      // First start verifying
      useVerificationStore.getState().setIsVerifying(true);

      // Then set result
      const mockResult: VerifyBundleResponse = {
        status: 'fail',
        bundleId: 'test',
        manifestHash: '0x' + 'a'.repeat(64),
        proofHash: '0x' + 'b'.repeat(64),
        signatureValid: false,
        details: {
          manifest_hash: '0x' + 'a'.repeat(64),
          proof_hash: '0x' + 'b'.repeat(64),
          checks_passed: 3,
          checks_total: 5,
          statement_validation: [
            { field: 'test', status: 'mismatch', expected: '1', found: '2' },
          ],
          signature_present: false,
        },
      };

      useVerificationStore.getState().setVerificationResult(mockResult);

      expect(useVerificationStore.getState().isVerifying).toBe(false);
    });

    it('should handle null result', () => {
      // First set a result
      const mockResult: VerifyBundleResponse = {
        status: 'ok',
        bundleId: 'test',
        manifestHash: '0x' + 'a'.repeat(64),
        proofHash: '0x' + 'b'.repeat(64),
        signatureValid: true,
        details: {
          manifest_hash: '0x' + 'a'.repeat(64),
          proof_hash: '0x' + 'b'.repeat(64),
          checks_passed: 5,
          checks_total: 5,
          statement_validation: [],
          signature_present: true,
        },
      };
      useVerificationStore.getState().setVerificationResult(mockResult);

      // Then clear it
      useVerificationStore.getState().setVerificationResult(null);

      expect(useVerificationStore.getState().verificationResult).toBeNull();
    });
  });

  describe('setVerificationError', () => {
    it('should set verification error', () => {
      useVerificationStore.getState().setVerificationError('Test error message');

      const state = useVerificationStore.getState();
      expect(state.verificationError).toBe('Test error message');
      expect(state.verificationResult).toBeNull();
      expect(state.isVerifying).toBe(false);
    });

    it('should clear result when setting error', () => {
      // First set a result
      const mockResult: VerifyBundleResponse = {
        status: 'ok',
        bundleId: 'test',
        manifestHash: '0x' + 'a'.repeat(64),
        proofHash: '0x' + 'b'.repeat(64),
        signatureValid: true,
        details: {
          manifest_hash: '0x' + 'a'.repeat(64),
          proof_hash: '0x' + 'b'.repeat(64),
          checks_passed: 5,
          checks_total: 5,
          statement_validation: [],
          signature_present: true,
        },
      };
      useVerificationStore.getState().setVerificationResult(mockResult);

      // Then set error
      useVerificationStore.getState().setVerificationError('Error occurred');

      expect(useVerificationStore.getState().verificationResult).toBeNull();
    });

    it('should stop verifying when setting error', () => {
      useVerificationStore.getState().setIsVerifying(true);
      useVerificationStore.getState().setVerificationError('Error');

      expect(useVerificationStore.getState().isVerifying).toBe(false);
    });

    it('should handle null error', () => {
      useVerificationStore.getState().setVerificationError('Some error');
      useVerificationStore.getState().setVerificationError(null);

      expect(useVerificationStore.getState().verificationError).toBeNull();
    });
  });

  describe('setIsVerifying', () => {
    it('should set isVerifying to true', () => {
      useVerificationStore.getState().setIsVerifying(true);

      expect(useVerificationStore.getState().isVerifying).toBe(true);
    });

    it('should set isVerifying to false', () => {
      useVerificationStore.getState().setIsVerifying(true);
      useVerificationStore.getState().setIsVerifying(false);

      expect(useVerificationStore.getState().isVerifying).toBe(false);
    });
  });

  describe('reset', () => {
    it('should reset all state to initial values', () => {
      // Set various state
      const mockResult: VerifyBundleResponse = {
        status: 'ok',
        bundleId: 'test',
        manifestHash: '0x' + 'a'.repeat(64),
        proofHash: '0x' + 'b'.repeat(64),
        signatureValid: true,
        details: {
          manifest_hash: '0x' + 'a'.repeat(64),
          proof_hash: '0x' + 'b'.repeat(64),
          checks_passed: 5,
          checks_total: 5,
          statement_validation: [],
          signature_present: true,
        },
      };
      useVerificationStore.getState().setVerificationResult(mockResult);
      useVerificationStore.getState().setIsVerifying(true);

      // Reset
      useVerificationStore.getState().reset();

      const state = useVerificationStore.getState();
      expect(state.verificationResult).toBeNull();
      expect(state.verificationError).toBeNull();
      expect(state.isVerifying).toBe(false);
    });
  });

  describe('verification flow simulation', () => {
    it('should handle successful verification flow', () => {
      // Start verification
      useVerificationStore.getState().setIsVerifying(true);
      expect(useVerificationStore.getState().isVerifying).toBe(true);

      // Complete with success
      const mockResult: VerifyBundleResponse = {
        status: 'ok',
        bundleId: 'bundle-123',
        manifestHash: '0x' + 'a'.repeat(64),
        proofHash: '0x' + 'b'.repeat(64),
        signatureValid: true,
        timestampValid: true,
        registryMatch: true,
        details: {
          manifest_hash: '0x' + 'a'.repeat(64),
          proof_hash: '0x' + 'b'.repeat(64),
          checks_passed: 5,
          checks_total: 5,
          statement_validation: [],
          signature_present: true,
          signature_count: 1,
        },
      };
      useVerificationStore.getState().setVerificationResult(mockResult);

      const state = useVerificationStore.getState();
      expect(state.isVerifying).toBe(false);
      expect(state.verificationResult?.status).toBe('ok');
      expect(state.verificationError).toBeNull();
    });

    it('should handle failed verification flow', () => {
      // Start verification
      useVerificationStore.getState().setIsVerifying(true);

      // Complete with failure
      const mockResult: VerifyBundleResponse = {
        status: 'fail',
        bundleId: 'bundle-123',
        manifestHash: '0x' + 'a'.repeat(64),
        proofHash: '0x' + 'b'.repeat(64),
        signatureValid: false,
        details: {
          manifest_hash: '0x' + 'a'.repeat(64),
          proof_hash: '0x' + 'b'.repeat(64),
          checks_passed: 2,
          checks_total: 5,
          statement_validation: [
            { field: 'supplier_root', status: 'mismatch', expected: '0x1', found: '0x2' },
          ],
          signature_present: false,
        },
      };
      useVerificationStore.getState().setVerificationResult(mockResult);

      const state = useVerificationStore.getState();
      expect(state.verificationResult?.status).toBe('fail');
      expect(state.verificationResult?.signatureValid).toBe(false);
    });

    it('should handle error during verification', () => {
      // Start verification
      useVerificationStore.getState().setIsVerifying(true);

      // Error occurs
      useVerificationStore.getState().setVerificationError('Bundle not found');

      const state = useVerificationStore.getState();
      expect(state.isVerifying).toBe(false);
      expect(state.verificationResult).toBeNull();
      expect(state.verificationError).toBe('Bundle not found');
    });

    it('should handle new verification after error', () => {
      // First verification fails with error
      useVerificationStore.getState().setIsVerifying(true);
      useVerificationStore.getState().setVerificationError('First error');

      // Start new verification
      useVerificationStore.getState().reset();
      useVerificationStore.getState().setIsVerifying(true);

      // Complete successfully
      const mockResult: VerifyBundleResponse = {
        status: 'ok',
        bundleId: 'bundle-456',
        manifestHash: '0x' + 'c'.repeat(64),
        proofHash: '0x' + 'd'.repeat(64),
        signatureValid: true,
        details: {
          manifest_hash: '0x' + 'c'.repeat(64),
          proof_hash: '0x' + 'd'.repeat(64),
          checks_passed: 5,
          checks_total: 5,
          statement_validation: [],
          signature_present: true,
        },
      };
      useVerificationStore.getState().setVerificationResult(mockResult);

      const state = useVerificationStore.getState();
      expect(state.verificationError).toBeNull();
      expect(state.verificationResult?.bundleId).toBe('bundle-456');
    });
  });
});
