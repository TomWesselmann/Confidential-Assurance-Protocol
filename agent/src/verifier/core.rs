/// Verifier Core – Pure Verification Logic
///
/// This module provides portable, I/O-free verification logic that can be used
/// in CLI, tests, WASM, zkVM, and registry sandboxes.
///
/// Key invariants:
/// - No file system access (std::fs)
/// - No console output (println!/eprintln!)
/// - No external dependencies beyond crypto primitives
/// - All inputs are in-memory data structures
/// - Deterministic verification results
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use crate::crypto;

// ============================================================================
// Core Types
// ============================================================================

/// Proof statement extracted from manifest
///
/// Represents the cryptographic commitments and policy requirements
/// that the proof must satisfy.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProofStatement {
    /// Policy hash (SHA3-256, 0x-prefixed)
    pub policy_hash: String,

    /// Company commitment root (BLAKE3 Merkle root, 0x-prefixed)
    pub company_commitment_root: String,

    /// Optional sanctions list root (BLAKE3, 0x-prefixed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sanctions_root: Option<String>,

    /// Optional jurisdiction list root (BLAKE3, 0x-prefixed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jurisdiction_root: Option<String>,

    /// Optional extensions (future use)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Value>,
}

/// Verification options
///
/// Controls which verification checks should be performed.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerifyOptions {
    /// Check timestamp validity (requires timestamp data in manifest)
    pub check_timestamp: bool,

    /// Check registry match (requires registry entry data)
    pub check_registry: bool,
}

impl Default for VerifyOptions {
    fn default() -> Self {
        Self {
            check_timestamp: true,
            check_registry: true,
        }
    }
}

/// Verification report
///
/// Contains structured results of verification checks, including
/// detailed error information for failed checks.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerifyReport {
    /// Overall status: "ok" or "fail"
    pub status: String,

    /// Manifest hash (SHA3-256, 0x-prefixed)
    pub manifest_hash: String,

    /// Proof hash (SHA3-256, 0x-prefixed)
    pub proof_hash: String,

    /// Signature validation result
    pub signature_valid: bool,

    /// Timestamp validation result (None if check disabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp_valid: Option<bool>,

    /// Registry match result (None if check disabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry_match: Option<bool>,

    /// Structured details about verification findings
    pub details: serde_json::Value,
}

// ============================================================================
// Statement Extraction
// ============================================================================

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
pub fn extract_statement_from_manifest(manifest_json: &serde_json::Value) -> Result<ProofStatement> {
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

/// Validates that a string is a valid 32-byte hex hash with 0x prefix
fn validate_hex32(hex_str: &str, field_name: &str) -> Result<()> {
    if !hex_str.starts_with("0x") {
        return Err(anyhow!("{}: must start with '0x'", field_name));
    }

    let hex_part = &hex_str[2..];
    if hex_part.len() != 64 {
        return Err(anyhow!("{}: expected 64 hex characters (32 bytes), got {}", field_name, hex_part.len()));
    }

    if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(anyhow!("{}: contains invalid hex characters", field_name));
    }

    Ok(())
}

// ============================================================================
// Core Verification
// ============================================================================

/// Pure verification function (I/O-free)
///
/// Verifies a proof package against a manifest using provided data.
/// All inputs are in-memory, no file system access is performed.
///
/// # Arguments
/// * `manifest` - Parsed manifest JSON object
/// * `proof_bytes` - Raw proof bytes
/// * `stmt` - Proof statement (use extract_statement_from_manifest)
/// * `opts` - Verification options
///
/// # Returns
/// VerifyReport with detailed results
///
/// # Verification Steps
/// 1. Hash computation (manifest & proof)
/// 2. Statement validation (manifest matches statement)
/// 3. Signature check (if present in manifest)
/// 4. Timestamp validation (optional, if enabled)
/// 5. Registry match (optional, if enabled)
///
/// # Example
/// ```
/// use cap_agent::verifier::core::*;
/// use serde_json::json;
///
/// let manifest = json!({
///     "policy": {"hash": "0x1234..."},
///     "company_commitment_root": "0xabcd...",
///     "signatures": []
/// });
/// let proof_bytes = b"proof data";
/// let stmt = extract_statement_from_manifest(&manifest).unwrap();
/// let opts = VerifyOptions::default();
///
/// let report = verify(&manifest, proof_bytes, &stmt, &opts).unwrap();
/// ```
pub fn verify(
    manifest: &serde_json::Value,
    proof_bytes: &[u8],
    stmt: &ProofStatement,
    opts: &VerifyOptions,
) -> Result<VerifyReport> {
    let mut details = serde_json::Map::new();
    let mut checks_passed = 0;
    let mut checks_total = 0;

    // 1. Compute hashes
    let manifest_bytes = serde_json::to_vec(manifest)?;
    let manifest_hash_bytes = crypto::sha3_256(&manifest_bytes);
    let manifest_hash = crypto::hex_lower_prefixed32(manifest_hash_bytes);

    let proof_hash_bytes = crypto::sha3_256(proof_bytes);
    let proof_hash = crypto::hex_lower_prefixed32(proof_hash_bytes);

    details.insert("manifest_hash".to_string(), serde_json::json!(manifest_hash));
    details.insert("proof_hash".to_string(), serde_json::json!(proof_hash));

    // 2. Validate statement matches manifest
    checks_total += 1;
    let statement_valid = validate_statement_matches_manifest(manifest, stmt, &mut details)?;
    if statement_valid {
        checks_passed += 1;
    }

    // 3. Check signature presence
    checks_total += 1;
    let signature_valid = check_signature_presence(manifest, &mut details);
    if signature_valid {
        checks_passed += 1;
    }

    // 4. Optional timestamp check
    let timestamp_valid = if opts.check_timestamp {
        checks_total += 1;
        let valid = check_timestamp_in_manifest(manifest, &mut details);
        if valid {
            checks_passed += 1;
        }
        Some(valid)
    } else {
        details.insert("timestamp_check".to_string(), serde_json::json!("disabled"));
        None
    };

    // 5. Optional registry check
    let registry_match = if opts.check_registry {
        checks_total += 1;
        // Note: Registry check requires external data (registry entries)
        // This is a placeholder - actual check needs registry data passed in
        details.insert("registry_check".to_string(), serde_json::json!("not_implemented"));
        Some(false)
    } else {
        details.insert("registry_check".to_string(), serde_json::json!("disabled"));
        None
    };

    // 6. Determine overall status
    let all_required_passed = statement_valid && signature_valid;
    let status = if all_required_passed {
        "ok"
    } else {
        "fail"
    }.to_string();

    details.insert("checks_passed".to_string(), serde_json::json!(checks_passed));
    details.insert("checks_total".to_string(), serde_json::json!(checks_total));

    Ok(VerifyReport {
        status,
        manifest_hash,
        proof_hash,
        signature_valid,
        timestamp_valid,
        registry_match,
        details: serde_json::Value::Object(details),
    })
}

/// Validates that statement matches manifest content
fn validate_statement_matches_manifest(
    manifest: &serde_json::Value,
    stmt: &ProofStatement,
    details: &mut serde_json::Map<String, serde_json::Value>,
) -> Result<bool> {
    let mut checks = Vec::new();
    let mut all_ok = true;

    // Check policy hash
    if let Some(policy) = manifest.get("policy") {
        if let Some(hash) = policy.get("hash").and_then(|h| h.as_str()) {
            if hash == stmt.policy_hash {
                checks.push(serde_json::json!({"field": "policy.hash", "status": "ok"}));
            } else {
                checks.push(serde_json::json!({
                    "field": "policy.hash",
                    "status": "mismatch",
                    "expected": stmt.policy_hash,
                    "found": hash
                }));
                all_ok = false;
            }
        }
    }

    // Check company commitment root
    if let Some(root) = manifest.get("company_commitment_root").and_then(|r| r.as_str()) {
        if root == stmt.company_commitment_root {
            checks.push(serde_json::json!({"field": "company_commitment_root", "status": "ok"}));
        } else {
            checks.push(serde_json::json!({
                "field": "company_commitment_root",
                "status": "mismatch",
                "expected": stmt.company_commitment_root,
                "found": root
            }));
            all_ok = false;
        }
    }

    details.insert("statement_validation".to_string(), serde_json::json!(checks));
    Ok(all_ok)
}

/// Checks if signatures are present in manifest
fn check_signature_presence(
    manifest: &serde_json::Value,
    details: &mut serde_json::Map<String, serde_json::Value>,
) -> bool {
    let has_signatures = manifest
        .get("signatures")
        .and_then(|s| s.as_array())
        .map(|arr| !arr.is_empty())
        .unwrap_or(false);

    details.insert("signature_present".to_string(), serde_json::json!(has_signatures));

    if has_signatures {
        // Count signatures
        let count = manifest
            .get("signatures")
            .and_then(|s| s.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);
        details.insert("signature_count".to_string(), serde_json::json!(count));
    }

    has_signatures
}

/// Checks timestamp in manifest (mock/placeholder)
fn check_timestamp_in_manifest(
    manifest: &serde_json::Value,
    details: &mut serde_json::Map<String, serde_json::Value>,
) -> bool {
    // Check if time_anchor field exists
    let has_time_anchor = manifest.get("time_anchor").is_some();

    details.insert("timestamp_present".to_string(), serde_json::json!(has_time_anchor));

    if !has_time_anchor {
        // No timestamp anchor present - this is acceptable
        return true;
    }

    // Extract time_anchor
    let anchor = match manifest.get("time_anchor") {
        Some(a) => a,
        None => return true, // Should not happen, but safe fallback
    };

    details.insert("timestamp_info".to_string(), anchor.clone());

    // Check dual-anchor structure (v0.9.0)
    let has_private = anchor.get("private").is_some();
    let has_public = anchor.get("public").is_some();

    details.insert("dual_anchor_private".to_string(), serde_json::json!(has_private));
    details.insert("dual_anchor_public".to_string(), serde_json::json!(has_public));

    // Validate private anchor consistency if present
    if has_private {
        if let Some(private) = anchor.get("private") {
            let private_audit_tip = private.get("audit_tip_hex").and_then(|v| v.as_str());
            let anchor_audit_tip = anchor.get("audit_tip_hex").and_then(|v| v.as_str());

            if let (Some(priv_tip), Some(anc_tip)) = (private_audit_tip, anchor_audit_tip) {
                if priv_tip != anc_tip {
                    details.insert(
                        "dual_anchor_error".to_string(),
                        serde_json::json!("Private anchor audit_tip_hex does not match time_anchor.audit_tip_hex"),
                    );
                    return false;
                }
            }
        }
    }

    // Validate public anchor format if present
    if has_public {
        if let Some(public) = anchor.get("public") {
            let digest = public.get("digest").and_then(|v| v.as_str());
            let txid = public.get("txid").and_then(|v| v.as_str());

            if let Some(digest_str) = digest {
                if !digest_str.starts_with("0x") || digest_str.len() != 66 {
                    details.insert(
                        "dual_anchor_error".to_string(),
                        serde_json::json!("Public anchor digest has invalid format"),
                    );
                    return false;
                }
            }

            if let Some(txid_str) = txid {
                if txid_str.is_empty() {
                    details.insert(
                        "dual_anchor_error".to_string(),
                        serde_json::json!("Public anchor txid cannot be empty"),
                    );
                    return false;
                }
            }
        }
    }

    // Dual-anchor validation passed or not present (both are ok)
    true
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn mock_manifest() -> serde_json::Value {
        json!({
            "version": "manifest.v1.0",
            "created_at": "2025-10-30T12:00:00Z",
            "supplier_root": "0x0000000000000000000000000000000000000000000000000000000000000001",
            "ubo_root": "0x0000000000000000000000000000000000000000000000000000000000000002",
            "company_commitment_root": "0x0000000000000000000000000000000000000000000000000000000000000003",
            "policy": {
                "name": "Test Policy",
                "version": "lksg.v1",
                "hash": "0x0000000000000000000000000000000000000000000000000000000000000004"
            },
            "audit": {
                "tail_digest": "0x0000000000000000000000000000000000000000000000000000000000000005",
                "events_count": 10
            },
            "proof": {
                "proof_type": "mock",
                "status": "ok"
            },
            "signatures": []
        })
    }

    #[test]
    fn test_extract_statement_roundtrip_ok() {
        let manifest = mock_manifest();
        let stmt = extract_statement_from_manifest(&manifest).unwrap();

        assert_eq!(stmt.policy_hash, "0x0000000000000000000000000000000000000000000000000000000000000004");
        assert_eq!(stmt.company_commitment_root, "0x0000000000000000000000000000000000000000000000000000000000000003");
        assert!(stmt.sanctions_root.is_none());
        assert!(stmt.jurisdiction_root.is_none());
    }

    #[test]
    fn test_extract_statement_missing_policy_hash() {
        let mut manifest = mock_manifest();
        manifest.as_object_mut().unwrap().remove("policy");

        let result = extract_statement_from_manifest(&manifest);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing policy.hash"));
    }

    #[test]
    fn test_extract_statement_invalid_hex_format() {
        let mut manifest = mock_manifest();
        manifest["policy"]["hash"] = json!("not_a_hex_hash");

        let result = extract_statement_from_manifest(&manifest);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_ok_minimal() {
        let manifest = mock_manifest();
        let proof_bytes = b"mock proof data";
        let stmt = extract_statement_from_manifest(&manifest).unwrap();
        let opts = VerifyOptions {
            check_timestamp: false,
            check_registry: false,
        };

        let report = verify(&manifest, proof_bytes, &stmt, &opts).unwrap();

        // With no signatures, status will be "fail"
        assert_eq!(report.status, "fail");
        assert!(!report.signature_valid);
        assert!(report.manifest_hash.starts_with("0x"));
        assert!(report.proof_hash.starts_with("0x"));
    }

    #[test]
    fn test_verify_ok_with_signature() {
        let mut manifest = mock_manifest();
        manifest["signatures"] = json!([
            {
                "alg": "Ed25519",
                "signer": "TestCompany",
                "pubkey_hex": "0x0000000000000000000000000000000000000000000000000000000000000006",
                "sig_hex": "0x0000000000000000000000000000000000000000000000000000000000000007"
            }
        ]);

        let proof_bytes = b"mock proof data";
        let stmt = extract_statement_from_manifest(&manifest).unwrap();
        let opts = VerifyOptions {
            check_timestamp: false,
            check_registry: false,
        };

        let report = verify(&manifest, proof_bytes, &stmt, &opts).unwrap();

        assert_eq!(report.status, "ok");
        assert!(report.signature_valid);
    }

    #[test]
    fn test_verify_fail_tampered_policy_hash() {
        let manifest = mock_manifest();
        let proof_bytes = b"mock proof data";

        // Create statement with different policy hash
        let mut stmt = extract_statement_from_manifest(&manifest).unwrap();
        stmt.policy_hash = "0x9999999999999999999999999999999999999999999999999999999999999999".to_string();

        let opts = VerifyOptions {
            check_timestamp: false,
            check_registry: false,
        };

        let report = verify(&manifest, proof_bytes, &stmt, &opts).unwrap();

        assert_eq!(report.status, "fail");

        // Check details for mismatch
        let details = report.details.as_object().unwrap();
        let validation = details.get("statement_validation").unwrap().as_array().unwrap();
        let policy_check = &validation[0];
        assert_eq!(policy_check["status"], "mismatch");
    }

    #[test]
    fn test_verify_options_disable_checks() {
        let manifest = mock_manifest();
        let proof_bytes = b"mock proof data";
        let stmt = extract_statement_from_manifest(&manifest).unwrap();

        let opts = VerifyOptions {
            check_timestamp: false,
            check_registry: false,
        };

        let report = verify(&manifest, proof_bytes, &stmt, &opts).unwrap();

        assert!(report.timestamp_valid.is_none());
        assert!(report.registry_match.is_none());

        let details = report.details.as_object().unwrap();
        assert_eq!(details.get("timestamp_check").unwrap(), "disabled");
        assert_eq!(details.get("registry_check").unwrap(), "disabled");
    }
}
