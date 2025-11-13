use sha3::{Digest, Sha3_256};

/// Compute SHA3-256 hash of input data
pub fn sha3_256_hex(data: &str) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(data.as_bytes());
    let result = hasher.finalize();
    format!("sha3-256:{}", hex::encode(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_hash() {
        let data = r#"{"id":"test","version":"1.0"}"#;
        let hash1 = sha3_256_hex(data);
        let hash2 = sha3_256_hex(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_format() {
        let data = "test data";
        let hash = sha3_256_hex(data);
        assert!(hash.starts_with("sha3-256:"));
        assert_eq!(hash.len(), 9 + 64); // "sha3-256:" + 64 hex chars
    }

    #[test]
    fn test_different_data_different_hash() {
        let hash1 = sha3_256_hex("data1");
        let hash2 = sha3_256_hex("data2");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_empty_string() {
        let hash = sha3_256_hex("");
        assert!(hash.starts_with("sha3-256:"));
    }
}
