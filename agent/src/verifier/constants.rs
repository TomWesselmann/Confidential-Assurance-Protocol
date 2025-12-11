//! Cryptographic constants for verification
//!
//! Central definitions of signature and hash lengths to avoid magic numbers.

/// Ed25519 signature length in bytes
pub const ED25519_SIGNATURE_LEN: usize = 64;

/// Ed25519 public key length in bytes
pub const ED25519_PUBKEY_LEN: usize = 32;

/// SHA3-256 hash length in bytes
pub const SHA3_256_HASH_LEN: usize = 32;

/// BLAKE3-256 hash length in bytes
pub const BLAKE3_256_HASH_LEN: usize = 32;

/// Hex-encoded SHA3-256 hash length (64 characters)
pub const SHA3_256_HEX_LEN: usize = 64;

/// Hex-encoded SHA3-256 hash length with "0x" prefix (66 characters)
pub const SHA3_256_HEX_PREFIXED_LEN: usize = 66;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ed25519_constants() {
        // Verify constants match expected values
        assert_eq!(ED25519_SIGNATURE_LEN, 64);
        assert_eq!(ED25519_PUBKEY_LEN, 32);
    }

    #[test]
    fn test_hash_constants() {
        assert_eq!(SHA3_256_HASH_LEN, 32);
        assert_eq!(BLAKE3_256_HASH_LEN, 32);
        assert_eq!(SHA3_256_HEX_LEN, SHA3_256_HASH_LEN * 2);
        assert_eq!(SHA3_256_HEX_PREFIXED_LEN, SHA3_256_HEX_LEN + 2);
    }
}
