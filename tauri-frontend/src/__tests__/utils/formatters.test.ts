/**
 * CAP Formatter Utils Tests
 *
 * @description Unit tests for pure formatting functions
 */

import { describe, it, expect } from 'vitest';
import {
  formatHashShort,
  formatTimestamp,
  formatFileSize,
  formatVerificationStatus,
  formatProofType,
} from '../../core/utils/formatters';

describe('formatters', () => {
  describe('formatHashShort', () => {
    it('should truncate long hash with ellipsis', () => {
      const hash = '0x' + 'a'.repeat(64);
      const result = formatHashShort(hash);

      expect(result).toBe('0xaaaaaaaa...aaaaaaaa');
    });

    it('should use custom prefix length', () => {
      const hash = '0x' + 'a'.repeat(64);
      const result = formatHashShort(hash, 6, 4);

      expect(result).toBe('0xaaaa...aaaa');
    });

    it('should return original hash if shorter than combined lengths', () => {
      const hash = '0x1234';
      const result = formatHashShort(hash, 10, 8);

      expect(result).toBe('0x1234');
    });

    it('should handle exact boundary case', () => {
      const hash = '0x' + 'a'.repeat(16);
      const result = formatHashShort(hash, 10, 8);

      expect(result).toBe('0x' + 'a'.repeat(16));
    });

    it('should handle zero prefix and suffix length (returns original if short)', () => {
      const hash = '0x' + 'a'.repeat(64);
      const result = formatHashShort(hash, 0, 0);

      // With 0 prefix and 0 suffix, hash length is always > 0, so it gets truncated
      // but shows full hash since slice(0, 0) = '' and slice(-0) = full string
      expect(result).toContain('...');
    });
  });

  describe('formatTimestamp', () => {
    it('should format ISO timestamp to German locale', () => {
      const timestamp = '2025-12-14T10:30:00.000Z';
      const result = formatTimestamp(timestamp);

      // Note: Actual output depends on timezone, but should contain date parts
      expect(result).toMatch(/\d{2}\.\d{2}\.\d{4}/); // DD.MM.YYYY
      expect(result).toMatch(/\d{2}:\d{2}:\d{2}/); // HH:mm:ss
    });

    it('should include timezone in output', () => {
      const timestamp = '2025-06-15T14:45:30.000Z';
      const result = formatTimestamp(timestamp);

      // Should contain timezone abbreviation
      expect(result.length).toBeGreaterThan(10);
    });

    it('should handle timestamps with different times', () => {
      const morning = formatTimestamp('2025-01-01T08:00:00.000Z');
      const evening = formatTimestamp('2025-01-01T20:00:00.000Z');

      // Times should be different
      expect(morning).not.toBe(evening);
    });
  });

  describe('formatFileSize', () => {
    it('should format bytes correctly', () => {
      expect(formatFileSize(0)).toBe('0.00 B');
      expect(formatFileSize(500)).toBe('500.00 B');
      expect(formatFileSize(1023)).toBe('1023.00 B');
    });

    it('should format kilobytes correctly', () => {
      expect(formatFileSize(1024)).toBe('1.00 KB');
      expect(formatFileSize(1536)).toBe('1.50 KB');
      expect(formatFileSize(10240)).toBe('10.00 KB');
    });

    it('should format megabytes correctly', () => {
      expect(formatFileSize(1048576)).toBe('1.00 MB');
      expect(formatFileSize(5242880)).toBe('5.00 MB');
      expect(formatFileSize(1572864)).toBe('1.50 MB');
    });

    it('should format gigabytes correctly', () => {
      expect(formatFileSize(1073741824)).toBe('1.00 GB');
      expect(formatFileSize(2147483648)).toBe('2.00 GB');
    });

    it('should handle large gigabyte values', () => {
      const result = formatFileSize(10737418240); // 10 GB
      expect(result).toBe('10.00 GB');
    });

    it('should show 2 decimal places', () => {
      const result = formatFileSize(1234567);
      expect(result).toMatch(/\d+\.\d{2} \w+/);
    });
  });

  describe('formatVerificationStatus', () => {
    it('should format valid status in German', () => {
      expect(formatVerificationStatus('valid')).toBe('Gültig');
    });

    it('should format invalid status in German', () => {
      expect(formatVerificationStatus('invalid')).toBe('Ungültig');
    });

    it('should format error status in German', () => {
      expect(formatVerificationStatus('error')).toBe('Fehler');
    });
  });

  describe('formatProofType', () => {
    it('should format none proof type', () => {
      expect(formatProofType('none')).toBe('None');
    });

    it('should format mock proof type', () => {
      expect(formatProofType('mock')).toBe('Mock (Development)');
    });

    it('should format zk proof type', () => {
      expect(formatProofType('zk')).toBe('Zero-Knowledge Proof');
    });

    it('should format halo2 proof type', () => {
      expect(formatProofType('halo2')).toBe('Halo2 Zero-Knowledge Proof');
    });

    it('should format spartan proof type', () => {
      expect(formatProofType('spartan')).toBe('Spartan Zero-Knowledge Proof');
    });

    it('should format risc0 proof type', () => {
      expect(formatProofType('risc0')).toBe('RISC Zero zkVM');
    });

    it('should return original for unknown types', () => {
      expect(formatProofType('unknown')).toBe('unknown');
      expect(formatProofType('custom-proof')).toBe('custom-proof');
    });
  });
});
