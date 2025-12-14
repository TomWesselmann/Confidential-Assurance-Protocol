//! Manifest Module - Proof package metadata
//!
//! Provides the Manifest data structure for cryptographic proof packages,
//! including time anchoring and signature support.
//!
//! ## Module Structure (v0.11 Refactoring)
//!
//! - `types`: Manifest, AuditInfo, ProofInfo, SignatureInfo
//! - `anchor`: TimeAnchor, TimeAnchorPrivate, TimeAnchorPublic, PublicChain
//! - `signed`: SignedManifest
//! - `io`: build, save, load, anchor methods

pub mod anchor;
pub mod io;
pub mod signed;
pub mod types;

// Public API re-exports for backward compatibility
#[allow(unused_imports)]
pub use anchor::{PublicChain, TimeAnchor, TimeAnchorPrivate, TimeAnchorPublic};
#[allow(unused_imports)]
pub use io::read_audit_tail;
pub use signed::SignedManifest;
#[allow(unused_imports)]
pub use types::{AuditInfo, Manifest, ProofInfo, SignatureInfo, MANIFEST_SCHEMA_VERSION};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commitment::Commitments;
    use crate::policy::PolicyInfo;
    use chrono::Utc;

    fn create_test_manifest() -> Manifest {
        Manifest {
            version: "manifest.v1.0".to_string(),
            created_at: "2025-10-29T10:00:00Z".to_string(),
            supplier_root: "0xabc".to_string(),
            ubo_root: "0xdef".to_string(),
            company_commitment_root: "0x123".to_string(),
            policy: PolicyInfo {
                name: "Test".to_string(),
                version: "lksg.v1".to_string(),
                hash: "0xhash".to_string(),
            },
            audit: AuditInfo {
                tail_digest: "0xtail".to_string(),
                events_count: 5,
            },
            proof: ProofInfo {
                proof_type: "none".to_string(),
                status: "none".to_string(),
            },
            signatures: Vec::new(),
            time_anchor: None,
        }
    }

    #[test]
    fn test_manifest_creation() {
        let commitments = Commitments {
            supplier_root: "0xabc".to_string(),
            ubo_root: "0xdef".to_string(),
            company_commitment_root: "0x123".to_string(),
            supplier_count: Some(5),
            ubo_count: Some(2),
        };

        let policy_info = PolicyInfo {
            name: "Test".to_string(),
            version: "lksg.v1".to_string(),
            hash: "0xpolicy".to_string(),
        };

        let temp_audit = "/tmp/test_manifest_audit.jsonl";
        std::fs::write(temp_audit, r#"{"digest":"0xtest","seq":1}"#).unwrap();

        let manifest = Manifest::build(&commitments, policy_info, temp_audit).unwrap();

        assert_eq!(manifest.version, "manifest.v1.0");
        assert_eq!(manifest.supplier_root, "0xabc");
        assert_eq!(manifest.proof.proof_type, "none");

        std::fs::remove_file(temp_audit).ok();
    }

    #[test]
    fn test_manifest_update_proof() {
        let mut manifest = create_test_manifest();

        manifest.update_proof("mock".to_string(), "ok".to_string());
        assert_eq!(manifest.proof.proof_type, "mock");
        assert_eq!(manifest.proof.status, "ok");
    }

    #[test]
    fn time_anchor_roundtrip_ok() {
        use std::fs;

        let temp_path = "/tmp/test_manifest_anchor.json";

        let mut manifest = create_test_manifest();

        manifest.set_time_anchor(
            "tsa".to_string(),
            "./tsa/test.tsr".to_string(),
            "0x83a8779d".to_string(),
        );

        manifest.save(temp_path).unwrap();
        let loaded = Manifest::load(temp_path).unwrap();

        assert!(loaded.time_anchor.is_some());
        let anchor = loaded.time_anchor.unwrap();
        assert_eq!(anchor.kind, "tsa");
        assert_eq!(anchor.reference, "./tsa/test.tsr");
        assert_eq!(anchor.audit_tip_hex, "0x83a8779d");
        assert!(!anchor.created_at.is_empty());

        fs::remove_file(temp_path).ok();
    }

    #[test]
    fn test_set_private_anchor_success() {
        let mut manifest = create_test_manifest();
        let audit_tip =
            "0x1234567890123456789012345678901234567890123456789012345678901234".to_string();

        manifest.set_time_anchor(
            "tsa".to_string(),
            "./tsa/test.tsr".to_string(),
            audit_tip.clone(),
        );

        let result = manifest.set_private_anchor(audit_tip.clone(), None);
        assert!(result.is_ok());

        let anchor = manifest.time_anchor.unwrap();
        assert!(anchor.private.is_some());
        let private = anchor.private.unwrap();
        assert_eq!(private.audit_tip_hex, audit_tip);
        assert!(!private.created_at.is_empty());
    }

    #[test]
    fn test_set_private_anchor_not_initialized() {
        let mut manifest = create_test_manifest();
        let audit_tip =
            "0x1234567890123456789012345678901234567890123456789012345678901234".to_string();

        let result = manifest.set_private_anchor(audit_tip, None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("time_anchor must be initialized"));
    }

    #[test]
    fn test_set_private_anchor_mismatch() {
        let mut manifest = create_test_manifest();
        let anchor_tip =
            "0x1111111111111111111111111111111111111111111111111111111111111111".to_string();
        let private_tip =
            "0x2222222222222222222222222222222222222222222222222222222222222222".to_string();

        manifest.set_time_anchor("tsa".to_string(), "./tsa/test.tsr".to_string(), anchor_tip);

        let result = manifest.set_private_anchor(private_tip, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not match"));
    }

    #[test]
    fn test_set_public_anchor_ethereum_success() {
        let mut manifest = create_test_manifest();
        let audit_tip =
            "0x1234567890123456789012345678901234567890123456789012345678901234".to_string();

        manifest.set_time_anchor("blockchain".to_string(), "ethereum".to_string(), audit_tip);

        let result = manifest.set_public_anchor(
            PublicChain::Ethereum,
            "0xabc123def456".to_string(),
            "0x9999999999999999999999999999999999999999999999999999999999999999".to_string(),
            None,
        );

        assert!(result.is_ok());
        let anchor = manifest.time_anchor.unwrap();
        assert!(anchor.public.is_some());
        let public = anchor.public.unwrap();
        assert_eq!(public.chain, PublicChain::Ethereum);
        assert_eq!(public.txid, "0xabc123def456");
        assert!(!public.created_at.is_empty());
    }

    #[test]
    fn test_set_public_anchor_hedera() {
        let mut manifest = create_test_manifest();
        manifest.set_time_anchor(
            "blockchain".to_string(),
            "hedera".to_string(),
            "0x1234567890123456789012345678901234567890123456789012345678901234".to_string(),
        );

        let result = manifest.set_public_anchor(
            PublicChain::Hedera,
            "0.0.12345@1234567890.123456789".to_string(),
            "0x8888888888888888888888888888888888888888888888888888888888888888".to_string(),
            None,
        );

        assert!(result.is_ok());
        let public = manifest.time_anchor.unwrap().public.unwrap();
        assert_eq!(public.chain, PublicChain::Hedera);
    }

    #[test]
    fn test_set_public_anchor_btc() {
        let mut manifest = create_test_manifest();
        manifest.set_time_anchor(
            "blockchain".to_string(),
            "btc".to_string(),
            "0x1234567890123456789012345678901234567890123456789012345678901234".to_string(),
        );

        let result = manifest.set_public_anchor(
            PublicChain::Btc,
            "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_string(),
            "0x7777777777777777777777777777777777777777777777777777777777777777".to_string(),
            None,
        );

        assert!(result.is_ok());
        let public = manifest.time_anchor.unwrap().public.unwrap();
        assert_eq!(public.chain, PublicChain::Btc);
    }

    #[test]
    fn test_set_public_anchor_not_initialized() {
        let mut manifest = create_test_manifest();

        let result = manifest.set_public_anchor(
            PublicChain::Ethereum,
            "0xabc".to_string(),
            "0x1234567890123456789012345678901234567890123456789012345678901234".to_string(),
            None,
        );

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("time_anchor must be initialized"));
    }

    #[test]
    fn test_validate_dual_anchor_no_anchor() {
        let manifest = create_test_manifest();
        let result = manifest.validate_dual_anchor();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_dual_anchor_private_mismatch() {
        let mut manifest = create_test_manifest();
        manifest.set_time_anchor(
            "tsa".to_string(),
            "./tsa/test.tsr".to_string(),
            "0x1111111111111111111111111111111111111111111111111111111111111111".to_string(),
        );

        if let Some(anchor) = &mut manifest.time_anchor {
            anchor.private = Some(TimeAnchorPrivate {
                audit_tip_hex: "0x2222222222222222222222222222222222222222222222222222222222222222"
                    .to_string(),
                created_at: Utc::now().to_rfc3339(),
            });
        }

        let result = manifest.validate_dual_anchor();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("mismatch"));
    }

    #[test]
    fn test_validate_dual_anchor_invalid_public_digest() {
        let mut manifest = create_test_manifest();
        manifest.set_time_anchor(
            "blockchain".to_string(),
            "eth".to_string(),
            "0x1234567890123456789012345678901234567890123456789012345678901234".to_string(),
        );

        if let Some(anchor) = &mut manifest.time_anchor {
            anchor.public = Some(TimeAnchorPublic {
                chain: PublicChain::Ethereum,
                txid: "0xabc".to_string(),
                digest: "invalid".to_string(),
                created_at: Utc::now().to_rfc3339(),
            });
        }

        let result = manifest.validate_dual_anchor();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid format"));
    }

    #[test]
    fn test_validate_dual_anchor_empty_txid() {
        let mut manifest = create_test_manifest();
        manifest.set_time_anchor(
            "blockchain".to_string(),
            "eth".to_string(),
            "0x1234567890123456789012345678901234567890123456789012345678901234".to_string(),
        );

        if let Some(anchor) = &mut manifest.time_anchor {
            anchor.public = Some(TimeAnchorPublic {
                chain: PublicChain::Ethereum,
                txid: "".to_string(),
                digest: "0x1234567890123456789012345678901234567890123456789012345678901234"
                    .to_string(),
                created_at: Utc::now().to_rfc3339(),
            });
        }

        let result = manifest.validate_dual_anchor();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("txid cannot be empty"));
    }

    #[test]
    fn test_validate_dual_anchor_success() {
        let mut manifest = create_test_manifest();
        let audit_tip =
            "0x1234567890123456789012345678901234567890123456789012345678901234".to_string();

        manifest.set_time_anchor(
            "blockchain".to_string(),
            "eth".to_string(),
            audit_tip.clone(),
        );
        manifest
            .set_private_anchor(audit_tip.clone(), None)
            .unwrap();
        manifest
            .set_public_anchor(
                PublicChain::Ethereum,
                "0xabc123".to_string(),
                "0x5555555555555555555555555555555555555555555555555555555555555555".to_string(),
                None,
            )
            .unwrap();

        let result = manifest.validate_dual_anchor();
        assert!(result.is_ok());
    }

    #[test]
    fn test_read_audit_tail_empty_file() {
        use std::fs;
        let temp_audit = "/tmp/test_empty_audit.jsonl";
        fs::write(temp_audit, "").unwrap();

        let result = read_audit_tail(temp_audit);
        assert!(result.is_ok());
        let (digest, count) = result.unwrap();
        assert_eq!(
            digest,
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        );
        assert_eq!(count, 0);

        fs::remove_file(temp_audit).ok();
    }

    #[test]
    fn test_read_audit_tail_multiple_entries() {
        use std::fs;
        let temp_audit = "/tmp/test_multiple_audit.jsonl";
        fs::write(temp_audit,
            r#"{"digest":"0x1111111111111111111111111111111111111111111111111111111111111111","seq":1}
{"digest":"0x2222222222222222222222222222222222222222222222222222222222222222","seq":2}
{"digest":"0x3333333333333333333333333333333333333333333333333333333333333333","seq":3}"#
        ).unwrap();

        let result = read_audit_tail(temp_audit);
        assert!(result.is_ok());
        let (digest, count) = result.unwrap();
        assert_eq!(
            digest,
            "0x3333333333333333333333333333333333333333333333333333333333333333"
        );
        assert_eq!(count, 3);

        fs::remove_file(temp_audit).ok();
    }

    #[test]
    fn test_read_audit_tail_file_not_found() {
        let result = read_audit_tail("/nonexistent/path/audit.jsonl");
        assert!(result.is_err());
    }

    #[test]
    fn test_to_canonical_json() {
        let manifest = create_test_manifest();
        let result = manifest.to_canonical_json();
        assert!(result.is_ok());

        let json_str = result.unwrap();
        assert!(json_str.contains("manifest.v1.0"));
        assert!(json_str.contains("0xabc"));
        assert!(json_str.contains("Test"));
    }

    #[test]
    fn test_signed_manifest_roundtrip() {
        use std::fs;
        let temp_path = "/tmp/test_signed_manifest.json";

        let manifest = create_test_manifest();
        let signature = SignatureInfo {
            alg: "Ed25519".to_string(),
            signer: "company".to_string(),
            pubkey_hex: "0x123abc".to_string(),
            sig_hex: "0xdeadbeef".to_string(),
        };

        let signed = SignedManifest {
            manifest,
            signature,
        };
        signed.save(temp_path).unwrap();

        let loaded = SignedManifest::load(temp_path).unwrap();
        assert_eq!(loaded.manifest.version, "manifest.v1.0");
        assert_eq!(loaded.signature.alg, "Ed25519");
        assert_eq!(loaded.signature.sig_hex, "0xdeadbeef");

        fs::remove_file(temp_path).ok();
    }

    #[test]
    fn test_public_chain_serde() {
        let chains = vec![
            (PublicChain::Ethereum, "\"ethereum\""),
            (PublicChain::Hedera, "\"hedera\""),
            (PublicChain::Btc, "\"btc\""),
        ];

        for (chain, expected_json) in chains {
            let json = serde_json::to_string(&chain).unwrap();
            assert_eq!(json, expected_json);

            let deserialized: PublicChain = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, chain);
        }
    }
}
