//! Verifier Core – Re-export layer for backward compatibility
//!
//! This module re-exports types and functions from the new modular structure:
//! - types.rs: ProofStatement, VerifyOptions, VerifyReport
//! - statement.rs: extract_statement_from_manifest, validate_hex32
//! - verify.rs: verify, verify_from_source

// Re-export from types module
pub use super::types::{ProofStatement, VerifyOptions, VerifyReport};

// Re-export from statement module
pub use super::statement::{extract_statement_from_manifest, validate_hex32};

// Re-export from verify module
pub use super::verify::{verify, verify_from_source};

// ============================================================================
// Tests (kept here for integration testing of re-exports)
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

        assert_eq!(
            stmt.policy_hash,
            "0x0000000000000000000000000000000000000000000000000000000000000004"
        );
        assert_eq!(
            stmt.company_commitment_root,
            "0x0000000000000000000000000000000000000000000000000000000000000003"
        );
        assert!(stmt.sanctions_root.is_none());
        assert!(stmt.jurisdiction_root.is_none());
    }

    #[test]
    fn test_extract_statement_missing_policy_hash() {
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

        let mut stmt = extract_statement_from_manifest(&manifest).unwrap();
        stmt.policy_hash =
            "0x9999999999999999999999999999999999999999999999999999999999999999".to_string();

        let opts = VerifyOptions {
            check_timestamp: false,
            check_registry: false,
        };

        let report = verify(&manifest, proof_bytes, &stmt, &opts).unwrap();

        assert_eq!(report.status, "fail");

        let details = report.details.as_object().unwrap();
        let validation = details
            .get("statement_validation")
            .unwrap()
            .as_array()
            .unwrap();
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

    #[test]
    fn test_validate_hex32_with_sha3_prefix() {
        let valid_hash =
            "sha3-256:0000000000000000000000000000000000000000000000000000000000000000";
        let result = validate_hex32(valid_hash, "test_field");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_hex32_invalid_prefix() {
        let invalid_hash =
            "invalid:0000000000000000000000000000000000000000000000000000000000000000";
        let result = validate_hex32(invalid_hash, "test_field");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must start with"));
    }

    #[test]
    fn test_validate_hex32_wrong_length() {
        let short_hash = "0x00001234";
        let result = validate_hex32(short_hash, "test_field");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("expected 64 hex characters"));
    }

    #[test]
    fn test_validate_hex32_non_hex_characters() {
        let invalid_chars = "0x000000000000000000000000000000000000000000000000000000000000gggg";
        let result = validate_hex32(invalid_chars, "test_field");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("invalid hex characters"));
    }

    #[test]
    fn test_extract_statement_with_sanctions_root() {
        let mut manifest = mock_manifest();
        manifest["sanctions_root"] =
            json!("0x1111111111111111111111111111111111111111111111111111111111111111");

        let stmt = extract_statement_from_manifest(&manifest).unwrap();
        assert!(stmt.sanctions_root.is_some());
        assert_eq!(
            stmt.sanctions_root.unwrap(),
            "0x1111111111111111111111111111111111111111111111111111111111111111"
        );
    }

    #[test]
    fn test_extract_statement_with_jurisdiction_root() {
        let mut manifest = mock_manifest();
        manifest["jurisdiction_root"] =
            json!("0x2222222222222222222222222222222222222222222222222222222222222222");

        let stmt = extract_statement_from_manifest(&manifest).unwrap();
        assert!(stmt.jurisdiction_root.is_some());
        assert_eq!(
            stmt.jurisdiction_root.unwrap(),
            "0x2222222222222222222222222222222222222222222222222222222222222222"
        );
    }

    #[test]
    fn test_extract_statement_with_extensions() {
        let mut manifest = mock_manifest();
        manifest["extensions"] = json!({"custom_field": "value"});

        let stmt = extract_statement_from_manifest(&manifest).unwrap();
        assert!(stmt.extensions.is_some());
        let ext = stmt.extensions.unwrap();
        assert_eq!(ext.get("custom_field").unwrap(), "value");
    }

    #[test]
    fn test_extract_statement_invalid_sanctions_root() {
        let mut manifest = mock_manifest();
        manifest["sanctions_root"] = json!("invalid_hex");

        let result = extract_statement_from_manifest(&manifest);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_statement_invalid_jurisdiction_root() {
        let mut manifest = mock_manifest();
        manifest["jurisdiction_root"] = json!("0x123");

        let result = extract_statement_from_manifest(&manifest);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_with_time_anchor() {
        let mut manifest = mock_manifest();
        manifest["signatures"] = json!([{"alg": "Ed25519"}]);
        manifest["time_anchor"] = json!({
            "kind": "tsa",
            "reference": "./test.tsr",
            "audit_tip_hex": "0x3333333333333333333333333333333333333333333333333333333333333333",
            "created_at": "2025-11-01T12:00:00Z"
        });

        let proof_bytes = b"proof";
        let stmt = extract_statement_from_manifest(&manifest).unwrap();
        let opts = VerifyOptions {
            check_timestamp: true,
            check_registry: false,
        };

        let report = verify(&manifest, proof_bytes, &stmt, &opts).unwrap();
        assert!(report.timestamp_valid.is_some());
        assert!(report.timestamp_valid.unwrap());
    }

    #[test]
    fn test_verify_with_dual_anchor_private() {
        let mut manifest = mock_manifest();
        manifest["signatures"] = json!([{"alg": "Ed25519"}]);
        manifest["time_anchor"] = json!({
            "kind": "tsa",
            "audit_tip_hex": "0x4444444444444444444444444444444444444444444444444444444444444444",
            "private": {
                "audit_tip_hex": "0x4444444444444444444444444444444444444444444444444444444444444444",
                "created_at": "2025-11-01T12:00:00Z"
            }
        });

        let proof_bytes = b"proof";
        let stmt = extract_statement_from_manifest(&manifest).unwrap();
        let opts = VerifyOptions {
            check_timestamp: true,
            check_registry: false,
        };

        let report = verify(&manifest, proof_bytes, &stmt, &opts).unwrap();
        assert!(report.timestamp_valid.unwrap());

        let details = report.details.as_object().unwrap();
        assert_eq!(details.get("dual_anchor_private").unwrap(), true);
    }

    #[test]
    fn test_verify_dual_anchor_mismatch() {
        let mut manifest = mock_manifest();
        manifest["signatures"] = json!([{"alg": "Ed25519"}]);
        manifest["time_anchor"] = json!({
            "audit_tip_hex": "0x5555555555555555555555555555555555555555555555555555555555555555",
            "private": {
                "audit_tip_hex": "0x6666666666666666666666666666666666666666666666666666666666666666",
                "created_at": "2025-11-01T12:00:00Z"
            }
        });

        let proof_bytes = b"proof";
        let stmt = extract_statement_from_manifest(&manifest).unwrap();
        let opts = VerifyOptions {
            check_timestamp: true,
            check_registry: false,
        };

        let report = verify(&manifest, proof_bytes, &stmt, &opts).unwrap();
        assert!(!report.timestamp_valid.unwrap());

        let details = report.details.as_object().unwrap();
        assert!(details.get("dual_anchor_error").is_some());
    }

    #[test]
    fn test_verify_public_anchor_invalid_digest() {
        let mut manifest = mock_manifest();
        manifest["signatures"] = json!([{"alg": "Ed25519"}]);
        manifest["time_anchor"] = json!({
            "audit_tip_hex": "0x7777777777777777777777777777777777777777777777777777777777777777",
            "public": {
                "chain": "ethereum",
                "txid": "0xabc123",
                "digest": "invalid_digest",
                "created_at": "2025-11-01T12:00:00Z"
            }
        });

        let proof_bytes = b"proof";
        let stmt = extract_statement_from_manifest(&manifest).unwrap();
        let opts = VerifyOptions {
            check_timestamp: true,
            check_registry: false,
        };

        let report = verify(&manifest, proof_bytes, &stmt, &opts).unwrap();
        assert!(!report.timestamp_valid.unwrap());

        let details = report.details.as_object().unwrap();
        assert!(details.get("dual_anchor_error").is_some());
    }

    #[test]
    fn test_verify_public_anchor_empty_txid() {
        let mut manifest = mock_manifest();
        manifest["signatures"] = json!([{"alg": "Ed25519"}]);
        manifest["time_anchor"] = json!({
            "audit_tip_hex": "0x8888888888888888888888888888888888888888888888888888888888888888",
            "public": {
                "chain": "ethereum",
                "txid": "",
                "digest": "0x9999999999999999999999999999999999999999999999999999999999999999",
                "created_at": "2025-11-01T12:00:00Z"
            }
        });

        let proof_bytes = b"proof";
        let stmt = extract_statement_from_manifest(&manifest).unwrap();
        let opts = VerifyOptions {
            check_timestamp: true,
            check_registry: false,
        };

        let report = verify(&manifest, proof_bytes, &stmt, &opts).unwrap();
        assert!(!report.timestamp_valid.unwrap());

        let details = report.details.as_object().unwrap();
        assert!(details.get("dual_anchor_error").is_some());
    }

    #[test]
    fn test_verify_fail_company_commitment_mismatch() {
        let manifest = mock_manifest();
        let proof_bytes = b"proof";

        let mut stmt = extract_statement_from_manifest(&manifest).unwrap();
        stmt.company_commitment_root =
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string();

        let opts = VerifyOptions {
            check_timestamp: false,
            check_registry: false,
        };

        let report = verify(&manifest, proof_bytes, &stmt, &opts).unwrap();
        assert_eq!(report.status, "fail");

        let details = report.details.as_object().unwrap();
        let validation = details
            .get("statement_validation")
            .unwrap()
            .as_array()
            .unwrap();
        let company_check = &validation[1];
        assert_eq!(company_check["status"], "mismatch");
    }

    #[test]
    fn test_verify_options_enable_timestamp_check() {
        let mut manifest = mock_manifest();
        manifest["signatures"] = json!([{"alg": "Ed25519"}]);

        let proof_bytes = b"proof";
        let stmt = extract_statement_from_manifest(&manifest).unwrap();
        let opts = VerifyOptions {
            check_timestamp: true,
            check_registry: false,
        };

        let report = verify(&manifest, proof_bytes, &stmt, &opts).unwrap();
        assert!(report.timestamp_valid.is_some());
    }

    #[test]
    fn test_verify_missing_company_commitment_root() {
        let mut manifest = mock_manifest();
        manifest
            .as_object_mut()
            .unwrap()
            .remove("company_commitment_root");

        let result = extract_statement_from_manifest(&manifest);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing company_commitment_root"));
    }
}
