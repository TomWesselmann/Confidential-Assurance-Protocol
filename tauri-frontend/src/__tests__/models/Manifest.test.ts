/**
 * CAP Manifest Model Tests
 *
 * @description Unit tests for Manifest type guard
 */

import { describe, it, expect } from 'vitest';
import { isValidManifest } from '../../core/models/Manifest';
import type { Manifest } from '../../core/models/Manifest';

describe('Manifest model', () => {
  describe('isValidManifest', () => {
    const validManifest: Manifest = {
      version: 'manifest.v0',
      created_at: '2025-12-14T10:30:00.000Z',
      supplier_root: '0x' + 'a'.repeat(64),
      ubo_root: '0x' + 'b'.repeat(64),
      company_commitment_root: '0x' + 'c'.repeat(64),
      policy: {
        name: 'LkSG Policy',
        version: 'lksg.v1',
        hash: '0x' + 'd'.repeat(64),
      },
      audit: {
        tail_digest: '0x' + 'e'.repeat(64),
        events_count: 10,
      },
      proof: {
        type: 'mock',
        status: 'ok',
      },
      signatures: [],
    };

    it('should accept valid manifest', () => {
      expect(isValidManifest(validManifest)).toBe(true);
    });

    it('should accept manifest with signatures', () => {
      const manifestWithSig: Manifest = {
        ...validManifest,
        signatures: [
          {
            kid: '0123456789abcdef',
            algorithm: 'Ed25519',
            signature: 'base64signature',
            signed_at: '2025-12-14T10:35:00.000Z',
          },
        ],
      };
      expect(isValidManifest(manifestWithSig)).toBe(true);
    });

    it('should accept manifest with time_anchor', () => {
      const manifestWithAnchor: Manifest = {
        ...validManifest,
        time_anchor: {
          blockchain: 'ethereum',
          tx_hash: '0x' + 'f'.repeat(64),
          block_number: 12345678,
          timestamp: '2025-12-14T10:40:00.000Z',
        },
      };
      expect(isValidManifest(manifestWithAnchor)).toBe(true);
    });

    it('should accept manifest with extended audit info', () => {
      const manifestWithExtendedAudit: Manifest = {
        ...validManifest,
        audit: {
          tail_digest: '0x' + 'e'.repeat(64),
          events_count: 10,
          time_range: {
            start: '2025-12-01T00:00:00.000Z',
            end: '2025-12-14T10:30:00.000Z',
          },
          event_categories: {
            data_changes: 5,
            compliance: 3,
            system: 2,
          },
          last_event_type: 'manifest_built',
          hash_function: 'SHA3-256',
          chain_type: 'linear_hash_chain',
          integrity: 'verified',
          audit_chain_version: 1,
        },
      };
      expect(isValidManifest(manifestWithExtendedAudit)).toBe(true);
    });

    it('should reject null', () => {
      expect(isValidManifest(null)).toBe(false);
    });

    it('should reject undefined', () => {
      expect(isValidManifest(undefined)).toBe(false);
    });

    it('should reject non-object', () => {
      expect(isValidManifest('string')).toBe(false);
      expect(isValidManifest(123)).toBe(false);
      expect(isValidManifest(true)).toBe(false);
    });

    it('should reject missing version', () => {
      const { version, ...rest } = validManifest;
      expect(isValidManifest(rest)).toBe(false);
    });

    it('should reject missing created_at', () => {
      const { created_at, ...rest } = validManifest;
      expect(isValidManifest(rest)).toBe(false);
    });

    it('should reject missing supplier_root', () => {
      const { supplier_root, ...rest } = validManifest;
      expect(isValidManifest(rest)).toBe(false);
    });

    it('should reject missing ubo_root', () => {
      const { ubo_root, ...rest } = validManifest;
      expect(isValidManifest(rest)).toBe(false);
    });

    it('should reject missing company_commitment_root', () => {
      const { company_commitment_root, ...rest } = validManifest;
      expect(isValidManifest(rest)).toBe(false);
    });

    it('should reject missing policy', () => {
      const { policy, ...rest } = validManifest;
      expect(isValidManifest(rest)).toBe(false);
    });

    it('should reject missing audit', () => {
      const { audit, ...rest } = validManifest;
      expect(isValidManifest(rest)).toBe(false);
    });

    it('should reject missing proof', () => {
      const { proof, ...rest } = validManifest;
      expect(isValidManifest(rest)).toBe(false);
    });

    it('should reject missing signatures', () => {
      const { signatures, ...rest } = validManifest;
      expect(isValidManifest(rest)).toBe(false);
    });

    it('should reject signatures as non-array', () => {
      const invalid = {
        ...validManifest,
        signatures: 'not-an-array',
      };
      expect(isValidManifest(invalid)).toBe(false);
    });

    it('should reject policy as non-object', () => {
      const invalid = {
        ...validManifest,
        policy: 'not-an-object',
      };
      expect(isValidManifest(invalid)).toBe(false);
    });

    it('should reject audit as non-object', () => {
      const invalid = {
        ...validManifest,
        audit: 'not-an-object',
      };
      expect(isValidManifest(invalid)).toBe(false);
    });

    it('should reject proof as non-object', () => {
      const invalid = {
        ...validManifest,
        proof: 'not-an-object',
      };
      expect(isValidManifest(invalid)).toBe(false);
    });

    it('should reject version as non-string', () => {
      const invalid = {
        ...validManifest,
        version: 123,
      };
      expect(isValidManifest(invalid)).toBe(false);
    });

    it('should reject created_at as non-string', () => {
      const invalid = {
        ...validManifest,
        created_at: new Date(),
      };
      expect(isValidManifest(invalid)).toBe(false);
    });

    it('should accept empty array for signatures', () => {
      expect(isValidManifest(validManifest)).toBe(true);
      expect(validManifest.signatures).toEqual([]);
    });
  });
});
