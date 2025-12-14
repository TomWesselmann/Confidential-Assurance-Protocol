/**
 * CAP Data Formatters
 *
 * @description Pure functions for data formatting and display
 * @pure All functions are deterministic with no side effects
 */

/**
 * Formats hash for display (truncated)
 * @pure No side effects
 * @param hash - Full hash string
 * @param prefixLength - Number of chars to show at start
 * @param suffixLength - Number of chars to show at end
 * @returns Truncated hash with ellipsis
 */
export function formatHashShort(
  hash: string,
  prefixLength: number = 10,
  suffixLength: number = 8
): string {
  if (hash.length <= prefixLength + suffixLength) {
    return hash;
  }

  const prefix = hash.slice(0, prefixLength);
  const suffix = hash.slice(-suffixLength);

  return `${prefix}...${suffix}`;
}

/**
 * Formats timestamp to human-readable string
 * @pure No side effects (given same locale)
 * @param timestamp - ISO 8601 timestamp
 * @returns Formatted date/time string
 */
export function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp);

  return new Intl.DateTimeFormat('de-DE', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    timeZoneName: 'short',
  }).format(date);
}

/**
 * Formats file size in bytes to human-readable string
 * @pure No side effects
 * @param bytes - File size in bytes
 * @returns Formatted file size (e.g., "1.23 MB")
 */
export function formatFileSize(bytes: number): string {
  const units = ['B', 'KB', 'MB', 'GB'];
  let size = bytes;
  let unitIndex = 0;

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }

  return `${size.toFixed(2)} ${units[unitIndex]}`;
}

/**
 * Formats verification status to display string
 * @pure No side effects
 * @param status - Verification status
 * @returns User-friendly status text
 */
export function formatVerificationStatus(status: 'valid' | 'invalid' | 'error'): string {
  const statusMap: Record<typeof status, string> = {
    valid: 'Gültig',
    invalid: 'Ungültig',
    error: 'Fehler',
  };

  return statusMap[status];
}

/**
 * Formats proof type to display string
 * @pure No side effects
 * @param proofType - Proof type
 * @returns User-friendly proof type text
 */
export function formatProofType(proofType: string): string {
  const typeMap: Record<string, string> = {
    none: 'None',
    mock: 'Mock (Development)',
    zk: 'Zero-Knowledge Proof',
    halo2: 'Halo2 Zero-Knowledge Proof',
    spartan: 'Spartan Zero-Knowledge Proof',
    risc0: 'RISC Zero zkVM',
  };

  return typeMap[proofType] || proofType;
}
