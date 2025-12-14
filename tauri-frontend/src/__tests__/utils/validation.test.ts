/**
 * CAP Validation Utils Tests
 *
 * @description Unit tests for pure validation functions
 * @pure Tests deterministic, side-effect-free functions
 */

import { describe, it, expect } from 'vitest';
import {
  isValidHash,
  isValidKID,
  isValidTimestamp,
  isValidProofType,
  isValidVersion,
  isValidSignature,
} from '../../core/utils/validation';

describe('validation utils', () => {
  describe('isValidHash', () => {
    it('should accept valid BLAKE3/SHA3-256 hash', () => {
      const validHash = '0x' + 'a'.repeat(64);
      expect(isValidHash(validHash)).toBe(true);
    });

    it('should reject hash without 0x prefix', () => {
      const invalidHash = 'a'.repeat(64);
      expect(isValidHash(invalidHash)).toBe(false);
    });

    it('should reject hash with wrong length', () => {
      const invalidHash = '0x' + 'a'.repeat(32);
      expect(isValidHash(invalidHash)).toBe(false);
    });

    it('should reject non-hex characters', () => {
      const invalidHash = '0x' + 'g'.repeat(64);
      expect(isValidHash(invalidHash)).toBe(false);
    });
  });

  describe('isValidKID', () => {
    it('should accept valid 16-char hex KID', () => {
      expect(isValidKID('0123456789abcdef')).toBe(true);
    });

    it('should reject KID with wrong length', () => {
      expect(isValidKID('0123456789abcd')).toBe(false);
    });

    it('should reject non-hex characters', () => {
      expect(isValidKID('0123456789abcdeg')).toBe(false);
    });
  });

  describe('isValidProofType', () => {
    it('should accept "none" proof type', () => {
      expect(isValidProofType('none')).toBe(true);
    });

    it('should accept "mock" proof type', () => {
      expect(isValidProofType('mock')).toBe(true);
    });

    it('should accept "halo2" proof type', () => {
      expect(isValidProofType('halo2')).toBe(true);
    });

    it('should accept "zk" proof type', () => {
      expect(isValidProofType('zk')).toBe(true);
    });

    it('should reject unknown proof types', () => {
      expect(isValidProofType('unknown')).toBe(false);
    });
  });

  describe('isValidVersion', () => {
    it('should accept semantic version', () => {
      expect(isValidVersion('1.2.3')).toBe(true);
    });

    it('should accept manifest version', () => {
      expect(isValidVersion('manifest.v0')).toBe(true);
    });

    it('should accept policy version', () => {
      expect(isValidVersion('lksg.v1')).toBe(true);
    });

    it('should reject invalid version', () => {
      expect(isValidVersion('invalid')).toBe(false);
    });
  });

  describe('isValidTimestamp', () => {
    it('should accept valid ISO 8601 timestamp', () => {
      const now = new Date().toISOString();
      expect(isValidTimestamp(now)).toBe(true);
    });

    it('should reject invalid timestamp', () => {
      expect(isValidTimestamp('not-a-timestamp')).toBe(false);
    });

    it('should reject partial ISO timestamp', () => {
      expect(isValidTimestamp('2025-12-14')).toBe(false);
    });

    it('should reject timestamp without Z suffix', () => {
      expect(isValidTimestamp('2025-12-14T10:30:00')).toBe(false);
    });
  });

  describe('isValidSignature', () => {
    it('should accept valid 64-byte base64 signature', () => {
      // Create a valid 64-byte signature (512 bits = 64 bytes)
      const validSig = btoa(String.fromCharCode(...new Array(64).fill(65)));
      expect(isValidSignature(validSig)).toBe(true);
    });

    it('should reject signature with wrong length', () => {
      // 32-byte signature (too short)
      const shortSig = btoa(String.fromCharCode(...new Array(32).fill(65)));
      expect(isValidSignature(shortSig)).toBe(false);
    });

    it('should reject signature that is too long', () => {
      // 128-byte signature (too long)
      const longSig = btoa(String.fromCharCode(...new Array(128).fill(65)));
      expect(isValidSignature(longSig)).toBe(false);
    });

    it('should reject invalid base64', () => {
      expect(isValidSignature('not-valid-base64!!!')).toBe(false);
    });

    it('should reject empty string', () => {
      expect(isValidSignature('')).toBe(false);
    });
  });
});
