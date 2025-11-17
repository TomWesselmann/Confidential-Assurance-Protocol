/// Crypto Module – Centralized Cryptographic Operations
///
/// This module provides a unified API for all cryptographic operations:
/// - Hash functions (SHA3-256, BLAKE3)
/// - Digital signatures (Ed25519)
/// - Hex encoding/decoding
///
/// All crypto operations in the codebase should use this module to ensure
/// consistency and maintainability.
use anyhow::{anyhow, Result};
use blake3::Hasher as Blake3Hasher;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use sha3::{Digest, Sha3_256};

// ============================================================================
// Hash Functions
// ============================================================================

/// Computes SHA3-256 hash of input data
///
/// # Arguments
/// * `input` - Byte slice to hash
///
/// # Returns
/// 32-byte SHA3-256 hash
///
/// # Example
/// ```
/// use cap_agent::crypto::sha3_256;
/// let hash = sha3_256(b"hello world");
/// assert_eq!(hash.len(), 32);
/// ```
pub fn sha3_256(input: &[u8]) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    hasher.update(input);
    let result = hasher.finalize();
    result.into()
}

/// Computes BLAKE3 hash of input data
///
/// # Arguments
/// * `input` - Byte slice to hash
///
/// # Returns
/// 32-byte BLAKE3 hash
///
/// # Example
/// ```
/// use cap_agent::crypto::blake3_256;
/// let hash = blake3_256(b"hello world");
/// assert_eq!(hash.len(), 32);
/// ```
pub fn blake3_256(input: &[u8]) -> [u8; 32] {
    let mut hasher = Blake3Hasher::new();
    hasher.update(input);
    let result = hasher.finalize();
    *result.as_bytes()
}

// ============================================================================
// Ed25519 Digital Signatures
// ============================================================================

/// Ed25519 public key wrapper
#[derive(Clone, Debug)]
pub struct Ed25519PublicKey(VerifyingKey);

impl Ed25519PublicKey {
    /// Creates a public key from raw 32-byte array
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self> {
        let key = VerifyingKey::from_bytes(bytes)
            .map_err(|e| anyhow!("Invalid Ed25519 public key: {}", e))?;
        Ok(Self(key))
    }

    /// Exports public key as 32-byte array
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    /// Returns reference to inner VerifyingKey (for compatibility)
    pub fn inner(&self) -> &VerifyingKey {
        &self.0
    }
}

/// Ed25519 secret key wrapper
#[derive(Clone)]
pub struct Ed25519SecretKey(SigningKey);

impl Ed25519SecretKey {
    /// Creates a secret key from raw 32-byte array
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        let key = SigningKey::from_bytes(bytes);
        Self(key)
    }

    /// Exports secret key as 32-byte array
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    /// Returns the public key corresponding to this secret key
    pub fn verifying_key(&self) -> Ed25519PublicKey {
        Ed25519PublicKey(self.0.verifying_key())
    }

    /// Returns reference to inner SigningKey (for compatibility)
    pub fn inner(&self) -> &SigningKey {
        &self.0
    }
}

/// Ed25519 signature wrapper
#[derive(Clone, Debug)]
pub struct Ed25519Signature(Signature);

impl Ed25519Signature {
    /// Creates a signature from raw 64-byte array
    pub fn from_bytes(bytes: &[u8; 64]) -> Self {
        let sig = Signature::from_bytes(bytes);
        Self(sig)
    }

    /// Exports signature as 64-byte array
    pub fn to_bytes(&self) -> [u8; 64] {
        self.0.to_bytes()
    }

    /// Returns reference to inner Signature (for compatibility)
    pub fn inner(&self) -> &Signature {
        &self.0
    }
}

/// Signs a message with Ed25519 secret key
///
/// # Arguments
/// * `sk` - Ed25519 secret key
/// * `msg` - Message bytes to sign
///
/// # Returns
/// Ed25519 signature
///
/// # Example
/// ```
/// use cap_agent::crypto::{Ed25519SecretKey, ed25519_sign};
/// let sk = Ed25519SecretKey::from_bytes(&[0u8; 32]);
/// let sig = ed25519_sign(&sk, b"hello").unwrap();
/// ```
pub fn ed25519_sign(sk: &Ed25519SecretKey, msg: &[u8]) -> Result<Ed25519Signature> {
    let signature = sk.0.sign(msg);
    Ok(Ed25519Signature(signature))
}

/// Verifies an Ed25519 signature
///
/// # Arguments
/// * `pk` - Ed25519 public key
/// * `msg` - Message bytes that were signed
/// * `sig` - Signature to verify
///
/// # Returns
/// `Ok(true)` if signature is valid, `Ok(false)` if invalid
///
/// # Example
/// ```
/// use cap_agent::crypto::{Ed25519PublicKey, Ed25519Signature, ed25519_verify};
/// let pk = Ed25519PublicKey::from_bytes(&[0u8; 32]).unwrap();
/// let sig = Ed25519Signature::from_bytes(&[0u8; 64]);
/// let valid = ed25519_verify(&pk, b"hello", &sig);
/// ```
pub fn ed25519_verify(pk: &Ed25519PublicKey, msg: &[u8], sig: &Ed25519Signature) -> bool {
    pk.0.verify(msg, &sig.0).is_ok()
}

// ============================================================================
// Hex Encoding/Decoding
// ============================================================================

/// Encodes 32 bytes as lowercase hex string with "0x" prefix
///
/// # Arguments
/// * `bytes32` - 32-byte array to encode
///
/// # Returns
/// Hex string in format "0x..." (lowercase, 66 characters total)
///
/// # Example
/// ```
/// use cap_agent::crypto::hex_lower_prefixed32;
/// let hash = [0u8; 32];
/// let hex = hex_lower_prefixed32(hash);
/// assert_eq!(hex.len(), 66);
/// assert!(hex.starts_with("0x"));
/// ```
pub fn hex_lower_prefixed32(bytes32: [u8; 32]) -> String {
    format!("0x{}", hex::encode(bytes32))
}

/// Parses a hex string into 32 bytes (strict mode)
///
/// # Arguments
/// * `s` - Hex string with or without "0x" prefix
///
/// # Returns
/// 32-byte array or error if invalid format
///
/// # Errors
/// - Invalid hex characters
/// - Wrong length (must decode to exactly 32 bytes)
///
/// # Example
/// ```
/// use cap_agent::crypto::hex_to_32b;
/// let bytes = hex_to_32b("0x0000000000000000000000000000000000000000000000000000000000000000").unwrap();
/// assert_eq!(bytes.len(), 32);
/// ```
pub fn hex_to_32b(s: &str) -> Result<[u8; 32]> {
    // Strip "0x" prefix if present
    let hex_str = s.strip_prefix("0x").unwrap_or(s);

    // Decode hex
    let bytes = hex::decode(hex_str).map_err(|e| anyhow!("Invalid hex string: {}", e))?;

    // Check length
    if bytes.len() != 32 {
        return Err(anyhow!("Expected 32 bytes, got {}", bytes.len()));
    }

    // Convert to array
    let mut array = [0u8; 32];
    array.copy_from_slice(&bytes);
    Ok(array)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha3_256_deterministic() {
        let input = b"hello world";
        let hash1 = sha3_256(input);
        let hash2 = sha3_256(input);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 32);
    }

    #[test]
    fn test_blake3_256_deterministic() {
        let input = b"hello world";
        let hash1 = blake3_256(input);
        let hash2 = blake3_256(input);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 32);
    }

    #[test]
    fn test_ed25519_sign_verify_roundtrip() {
        let sk = Ed25519SecretKey::from_bytes(&[42u8; 32]);
        let pk = sk.verifying_key();
        let msg = b"test message";

        let sig = ed25519_sign(&sk, msg).unwrap();
        assert!(ed25519_verify(&pk, msg, &sig));
    }

    #[test]
    fn test_ed25519_verify_fails_wrong_message() {
        let sk = Ed25519SecretKey::from_bytes(&[42u8; 32]);
        let pk = sk.verifying_key();
        let msg1 = b"test message";
        let msg2 = b"different message";

        let sig = ed25519_sign(&sk, msg1).unwrap();
        assert!(!ed25519_verify(&pk, msg2, &sig));
    }

    #[test]
    fn test_ed25519_verify_fails_wrong_key() {
        let sk1 = Ed25519SecretKey::from_bytes(&[42u8; 32]);
        let sk2 = Ed25519SecretKey::from_bytes(&[99u8; 32]);
        let pk2 = sk2.verifying_key();
        let msg = b"test message";

        let sig = ed25519_sign(&sk1, msg).unwrap();
        assert!(!ed25519_verify(&pk2, msg, &sig));
    }

    #[test]
    fn test_hex_lower_prefixed32() {
        let bytes = [0xAB; 32];
        let hex = hex_lower_prefixed32(bytes);
        assert_eq!(hex.len(), 66); // "0x" + 64 hex chars
        assert!(hex.starts_with("0x"));
        assert!(hex
            .chars()
            .skip(2)
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    #[test]
    fn test_hex_to_32b_with_prefix() {
        let hex = "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let bytes = hex_to_32b(hex).unwrap();
        assert_eq!(bytes.len(), 32);
        assert_eq!(bytes[0], 0x01);
        assert_eq!(bytes[31], 0xef);
    }

    #[test]
    fn test_hex_to_32b_without_prefix() {
        let hex = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let bytes = hex_to_32b(hex).unwrap();
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn test_hex_to_32b_fails_wrong_length() {
        let hex = "0x0123"; // Too short
        assert!(hex_to_32b(hex).is_err());
    }

    #[test]
    fn test_hex_to_32b_fails_invalid_hex() {
        let hex = "0xGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG";
        assert!(hex_to_32b(hex).is_err());
    }

    #[test]
    fn test_hex_roundtrip() {
        let original = [
            0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45,
            0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01,
            0x23, 0x45, 0x67, 0x89,
        ];
        let hex = hex_lower_prefixed32(original);
        let decoded = hex_to_32b(&hex).unwrap();
        assert_eq!(original, decoded);
    }
}
