//! Statement Extraction - Extract proof statements from manifests
//!
//! Provides functions to extract and validate cryptographic commitments
//! from manifest JSON structures.

use anyhow::{anyhow, Result};

use super::types::ProofStatement;

/// Extracts proof statement from manifest
///
/// Reads the manifest JSON and extracts the cryptographic commitments
/// and policy information into a structured ProofStatement.
///
/// # Arguments
/// * `manifest_json` - Parsed manifest JSON object
///
/// # Returns
/// ProofStatement with validated hex fields
///
/// # Errors
/// - Missing required fields (policy.hash, company_commitment_root)
/// - Invalid hex format (not 0x-prefixed or wrong length)
pub fn extract_statement_from_manifest(
    manifest_json: &serde_json::Value,
) -> Result<ProofStatement> {
    // Extract policy hash
    let policy_hash = manifest_json
        .get("policy")
        .and_then(|p| p.get("hash"))
        .and_then(|h| h.as_str())
        .ok_or_else(|| anyhow!("Missing policy.hash in manifest"))?
        .to_string();

    // Validate policy hash format
    validate_hex32(&policy_hash, "policy.hash")?;

    // Extract company commitment root
    let company_commitment_root = manifest_json
        .get("company_commitment_root")
        .and_then(|r| r.as_str())
        .ok_or_else(|| anyhow!("Missing company_commitment_root in manifest"))?
        .to_string();

    // Validate company commitment root format
    validate_hex32(&company_commitment_root, "company_commitment_root")?;

    // Extract optional roots
    let sanctions_root = manifest_json
        .get("sanctions_root")
        .and_then(|r| r.as_str())
        .map(|s| s.to_string());

    if let Some(ref root) = sanctions_root {
        validate_hex32(root, "sanctions_root")?;
    }

    let jurisdiction_root = manifest_json
        .get("jurisdiction_root")
        .and_then(|r| r.as_str())
        .map(|s| s.to_string());

    if let Some(ref root) = jurisdiction_root {
        validate_hex32(root, "jurisdiction_root")?;
    }

    // Extract optional extensions
    let extensions = manifest_json.get("extensions").cloned();

    Ok(ProofStatement {
        policy_hash,
        company_commitment_root,
        sanctions_root,
        jurisdiction_root,
        extensions,
    })
}

/// Validates that a string is a valid 32-byte hex hash with 0x or sha3-256: prefix
pub fn validate_hex32(hex_str: &str, field_name: &str) -> Result<()> {
    // Support both 0x and sha3-256: prefixes (for compatibility with PolicyV2)
    let hex_part = if let Some(stripped) = hex_str.strip_prefix("0x") {
        stripped
    } else if let Some(stripped) = hex_str.strip_prefix("sha3-256:") {
        stripped
    } else {
        return Err(anyhow!(
            "{}: must start with '0x' or 'sha3-256:'",
            field_name
        ));
    };

    if hex_part.len() != 64 {
        return Err(anyhow!(
            "{}: expected 64 hex characters (32 bytes), got {}",
            field_name,
            hex_part.len()
        ));
    }

    if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(anyhow!("{}: contains invalid hex characters", field_name));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn mock_manifest() -> serde_json::Value {
        json!({
            "version": "manifest.v1.0",
            "company_commitment_root": "0x0000000000000000000000000000000000000000000000000000000000000003",
            "policy": {
                "name": "Test Policy",
                "version": "lksg.v1",
                "hash": "0x0000000000000000000000000000000000000000000000000000000000000004"
            }
        })
    }

    #[test]
    fn test_extract_statement_ok() {
        let manifest = mock_manifest();
        let stmt = extract_statement_from_manifest(&manifest).unwrap();

        assert_eq!(
            stmt.policy_hash,
            "0x0000000000000000000000000000000000000000000000000000000000000004"
        );
        assert_eq!(
            stmt.company_commitment_root,
            "0x0000000000000000000000000000000000000000000000000000000000000003"
        );
    }

    #[test]
    fn test_extract_statement_missing_policy() {
        let mut manifest = mock_manifest();
        manifest.as_object_mut().unwrap().remove("policy");

        let result = extract_statement_from_manifest(&manifest);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing policy.hash"));
    }

    #[test]
    fn test_validate_hex32_with_sha3_prefix() {
        let valid = "sha3-256:0000000000000000000000000000000000000000000000000000000000000000";
        assert!(validate_hex32(valid, "test").is_ok());
    }

    #[test]
    fn test_validate_hex32_invalid_prefix() {
        let invalid = "invalid:0000000000000000000000000000000000000000000000000000000000000000";
        let result = validate_hex32(invalid, "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_hex32_wrong_length() {
        let short = "0x00001234";
        let result = validate_hex32(short, "test");
        assert!(result.is_err());
    }
}
