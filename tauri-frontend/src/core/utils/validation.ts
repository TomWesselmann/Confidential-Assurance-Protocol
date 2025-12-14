/**
 * CAP Validation Utilities
 *
 * @description Pure functions for data validation
 * @pure All functions are deterministic with no side effects
 */

/**
 * Validates BLAKE3 or SHA3-256 hash format
 * @pure No side effects
 * @param hash - Hash string to validate
 * @returns true if valid hex hash with 0x prefix
 */
export function isValidHash(hash: string): boolean {
  if (!hash.startsWith('0x')) return false;

  const hexPart = hash.slice(2);

  // BLAKE3 = 64 hex chars (32 bytes)
  // SHA3-256 = 64 hex chars (32 bytes)
  if (hexPart.length !== 64) return false;

  return /^[0-9a-f]{64}$/i.test(hexPart);
}

/**
 * Validates Ed25519 signature format
 * @pure No side effects
 * @param signature - Base64-encoded signature
 * @returns true if valid 64-byte signature
 */
export function isValidSignature(signature: string): boolean {
  try {
    const decoded = atob(signature);
    return decoded.length === 64;
  } catch {
    return false;
  }
}

/**
 * Validates KID (Key Identifier) format
 * @pure No side effects
 * @param kid - Key identifier (16-char hex from BLAKE3 hash)
 * @returns true if valid KID format
 */
export function isValidKID(kid: string): boolean {
  return /^[0-9a-f]{16}$/i.test(kid);
}

/**
 * Validates ISO 8601 timestamp
 * @pure No side effects
 * @param timestamp - ISO timestamp string
 * @returns true if valid ISO 8601 format
 */
export function isValidTimestamp(timestamp: string): boolean {
  const date = new Date(timestamp);
  return !isNaN(date.getTime()) && date.toISOString() === timestamp;
}

/**
 * Validates proof type
 * @pure No side effects
 * @param proofType - Proof type string
 * @returns true if recognized proof type
 */
export function isValidProofType(proofType: string): boolean {
  const validTypes = ['none', 'mock', 'zk', 'halo2', 'spartan', 'risc0'];
  return validTypes.includes(proofType);
}

/**
 * Validates manifest version string
 * @pure No side effects
 * @param version - Version string
 * @returns true if valid semantic version or manifest version
 */
export function isValidVersion(version: string): boolean {
  // Semantic version: X.Y.Z
  const semverPattern = /^\d+\.\d+\.\d+$/;

  // Manifest version: manifest.vX
  const manifestPattern = /^manifest\.v\d+$/;

  // Policy version: lksg.vX
  const policyPattern = /^lksg\.v\d+$/;

  return semverPattern.test(version) ||
         manifestPattern.test(version) ||
         policyPattern.test(version);
}
