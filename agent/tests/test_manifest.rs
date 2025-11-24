/// Integration Tests für manifest.rs
///
/// Diese Tests wurden aus inline test modules extrahiert um Tarpaulin Coverage-Tracking zu ermöglichen.
/// Tarpaulin hat eine bekannte Limitation mit #[cfg(test)] inline modules.

use cap_agent::commitment::Commitments;
use cap_agent::manifest::{
    AuditInfo, Manifest, ProofInfo, PublicChain, SignatureInfo, SignedManifest,
    TimeAnchorPrivate, TimeAnchorPublic, MANIFEST_SCHEMA_VERSION,
};
use cap_agent::policy::PolicyInfo;
use chrono::Utc;
use std::fs;

// Helper function to create test manifest
fn create_test_manifest() -> Manifest {
    Manifest {
        version: MANIFEST_SCHEMA_VERSION.to_string(),
        created_at: Utc::now().to_rfc3339(),
        supplier_root: "0xabc".to_string(),
        ubo_root: "0xdef".to_string(),
        company_commitment_root: "0x123".to_string(),
        policy: PolicyInfo {
            name: "Test Policy".to_string(),
            version: "lksg.v1".to_string(),
            hash: "0xpolicyhash".to_string(),
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
fn test_manifest_struct_creation() {
    let manifest = create_test_manifest();

    assert_eq!(manifest.version, "manifest.v1.0");
    assert_eq!(manifest.supplier_root, "0xabc");
    assert_eq!(manifest.ubo_root, "0xdef");
    assert_eq!(manifest.company_commitment_root, "0x123");
    assert_eq!(manifest.policy.name, "Test Policy");
    assert_eq!(manifest.policy.version, "lksg.v1");
    assert_eq!(manifest.audit.tail_digest, "0xtail");
    assert_eq!(manifest.audit.events_count, 5);
    assert_eq!(manifest.proof.proof_type, "none");
    assert_eq!(manifest.proof.status, "none");
}

#[test]
fn test_manifest_build() {
    let commitments = Commitments {
        supplier_root: "0xabc123".to_string(),
        ubo_root: "0xdef456".to_string(),
        company_commitment_root: "0x789ghi".to_string(),
        supplier_count: Some(5),
        ubo_count: Some(2),
    };

    let policy_info = PolicyInfo {
        name: "Test Policy".to_string(),
        version: "lksg.v1".to_string(),
        hash: "0xpolicy123".to_string(),
    };

    // Create temporary audit log
    let temp_audit = "/tmp/test_manifest_build_audit.jsonl";
    fs::write(
        temp_audit,
        r#"{"digest":"0xtest123","seq":1}
{"digest":"0xtest456","seq":2}"#,
    )
    .unwrap();

    let manifest = Manifest::build(&commitments, policy_info, temp_audit).unwrap();

    assert_eq!(manifest.version, "manifest.v1.0");
    assert_eq!(manifest.supplier_root, "0xabc123");
    assert_eq!(manifest.ubo_root, "0xdef456");
    assert_eq!(manifest.company_commitment_root, "0x789ghi");
    assert_eq!(manifest.policy.name, "Test Policy");
    assert_eq!(manifest.audit.events_count, 2);
    assert_eq!(manifest.audit.tail_digest, "0xtest456");
    assert_eq!(manifest.proof.proof_type, "none");

    // Cleanup
    fs::remove_file(temp_audit).ok();
}

#[test]
fn test_manifest_save_and_load() {
    let manifest = create_test_manifest();
    let temp_path = "/tmp/test_manifest_save_load.json";

    // Save manifest
    manifest.save(temp_path).unwrap();

    // Load manifest
    let loaded = Manifest::load(temp_path).unwrap();

    assert_eq!(loaded.version, manifest.version);
    assert_eq!(loaded.supplier_root, manifest.supplier_root);
    assert_eq!(loaded.ubo_root, manifest.ubo_root);
    assert_eq!(
        loaded.company_commitment_root,
        manifest.company_commitment_root
    );
    assert_eq!(loaded.policy.name, manifest.policy.name);
    assert_eq!(loaded.audit.tail_digest, manifest.audit.tail_digest);

    // Cleanup
    fs::remove_file(temp_path).ok();
}

#[test]
fn test_manifest_update_proof() {
    let mut manifest = create_test_manifest();

    manifest.update_proof("mock".to_string(), "ok".to_string());

    assert_eq!(manifest.proof.proof_type, "mock");
    assert_eq!(manifest.proof.status, "ok");
}

#[test]
fn test_manifest_set_time_anchor() {
    let mut manifest = create_test_manifest();

    manifest.set_time_anchor(
        "tsa".to_string(),
        "./tsa/test.tsr".to_string(),
        "0x1234567890123456789012345678901234567890123456789012345678901234".to_string(),
    );

    assert!(manifest.time_anchor.is_some());
    let anchor = manifest.time_anchor.unwrap();
    assert_eq!(anchor.kind, "tsa");
    assert_eq!(anchor.reference, "./tsa/test.tsr");
    assert_eq!(
        anchor.audit_tip_hex,
        "0x1234567890123456789012345678901234567890123456789012345678901234"
    );
    assert!(!anchor.created_at.is_empty());
}

#[test]
fn test_manifest_set_private_anchor_success() {
    let mut manifest = create_test_manifest();
    let audit_tip =
        "0x1234567890123456789012345678901234567890123456789012345678901234".to_string();

    // First set time anchor
    manifest.set_time_anchor("tsa".to_string(), "./tsa/test.tsr".to_string(), audit_tip.clone());

    // Then set private anchor
    let result = manifest.set_private_anchor(audit_tip.clone(), None);
    assert!(result.is_ok());

    let anchor = manifest.time_anchor.unwrap();
    assert!(anchor.private.is_some());
    let private = anchor.private.unwrap();
    assert_eq!(private.audit_tip_hex, audit_tip);
    assert!(!private.created_at.is_empty());
}

#[test]
fn test_manifest_set_private_anchor_not_initialized() {
    let mut manifest = create_test_manifest();
    let audit_tip =
        "0x1234567890123456789012345678901234567890123456789012345678901234".to_string();

    // time_anchor is None
    let result = manifest.set_private_anchor(audit_tip, None);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("time_anchor must be initialized"));
}

#[test]
fn test_manifest_set_private_anchor_mismatch() {
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
fn test_manifest_set_public_anchor_ethereum() {
    let mut manifest = create_test_manifest();
    let audit_tip =
        "0x1234567890123456789012345678901234567890123456789012345678901234".to_string();

    manifest.set_time_anchor(
        "blockchain".to_string(),
        "ethereum".to_string(),
        audit_tip.clone(),
    );

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
fn test_manifest_set_public_anchor_hedera() {
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
fn test_manifest_set_public_anchor_btc() {
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
fn test_manifest_set_public_anchor_not_initialized() {
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
fn test_manifest_validate_dual_anchor_no_anchor() {
    let manifest = create_test_manifest();
    let result = manifest.validate_dual_anchor();
    assert!(result.is_ok()); // No anchor = OK
}

#[test]
fn test_manifest_validate_dual_anchor_private_mismatch() {
    let mut manifest = create_test_manifest();
    manifest.set_time_anchor(
        "tsa".to_string(),
        "./tsa/test.tsr".to_string(),
        "0x1111111111111111111111111111111111111111111111111111111111111111".to_string(),
    );

    // Manually set inconsistent private anchor
    if let Some(anchor) = &mut manifest.time_anchor {
        anchor.private = Some(TimeAnchorPrivate {
            audit_tip_hex:
                "0x2222222222222222222222222222222222222222222222222222222222222222".to_string(),
            created_at: Utc::now().to_rfc3339(),
        });
    }

    let result = manifest.validate_dual_anchor();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("mismatch"));
}

#[test]
fn test_manifest_validate_dual_anchor_invalid_public_digest() {
    let mut manifest = create_test_manifest();
    manifest.set_time_anchor(
        "blockchain".to_string(),
        "eth".to_string(),
        "0x1234567890123456789012345678901234567890123456789012345678901234".to_string(),
    );

    // Set public anchor with invalid digest (wrong format)
    if let Some(anchor) = &mut manifest.time_anchor {
        anchor.public = Some(TimeAnchorPublic {
            chain: PublicChain::Ethereum,
            txid: "0xabc".to_string(),
            digest: "invalid".to_string(), // Invalid format
            created_at: Utc::now().to_rfc3339(),
        });
    }

    let result = manifest.validate_dual_anchor();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("invalid format"));
}

#[test]
fn test_manifest_validate_dual_anchor_empty_txid() {
    let mut manifest = create_test_manifest();
    manifest.set_time_anchor(
        "blockchain".to_string(),
        "eth".to_string(),
        "0x1234567890123456789012345678901234567890123456789012345678901234".to_string(),
    );

    // Set public anchor with empty txid
    if let Some(anchor) = &mut manifest.time_anchor {
        anchor.public = Some(TimeAnchorPublic {
            chain: PublicChain::Ethereum,
            txid: "".to_string(), // Empty
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
fn test_manifest_validate_dual_anchor_success() {
    let mut manifest = create_test_manifest();
    let audit_tip =
        "0x1234567890123456789012345678901234567890123456789012345678901234".to_string();

    manifest.set_time_anchor(
        "blockchain".to_string(),
        "eth".to_string(),
        audit_tip.clone(),
    );
    manifest.set_private_anchor(audit_tip.clone(), None).unwrap();
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
fn test_manifest_to_canonical_json() {
    let manifest = create_test_manifest();
    let result = manifest.to_canonical_json();
    assert!(result.is_ok());

    let json_str = result.unwrap();
    assert!(json_str.contains("manifest.v1.0"));
    assert!(json_str.contains("0xabc"));
    assert!(json_str.contains("Test Policy"));
}

#[test]
fn test_signed_manifest_save_and_load() {
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

    let temp_path = "/tmp/test_signed_manifest.json";

    // Save
    signed.save(temp_path).unwrap();

    // Load
    let loaded = SignedManifest::load(temp_path).unwrap();

    assert_eq!(loaded.manifest.version, "manifest.v1.0");
    assert_eq!(loaded.signature.alg, "Ed25519");
    assert_eq!(loaded.signature.sig_hex, "0xdeadbeef");

    // Cleanup
    fs::remove_file(temp_path).ok();
}

#[test]
fn test_public_chain_serde() {
    // Test all PublicChain variants serialize/deserialize correctly
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

#[test]
fn test_read_audit_tail_empty_file() {
    let temp_audit = "/tmp/test_empty_audit.jsonl";
    fs::write(temp_audit, "").unwrap();

    // Access via helper since read_audit_tail is private - we test via build()
    let commitments = Commitments {
        supplier_root: "0xabc".to_string(),
        ubo_root: "0xdef".to_string(),
        company_commitment_root: "0x123".to_string(),
        supplier_count: Some(0),
        ubo_count: Some(0),
    };

    let policy_info = PolicyInfo {
        name: "Test".to_string(),
        version: "lksg.v1".to_string(),
        hash: "0xhash".to_string(),
    };

    let result = Manifest::build(&commitments, policy_info, temp_audit);
    assert!(result.is_ok());
    let manifest = result.unwrap();
    assert_eq!(
        manifest.audit.tail_digest,
        "0x0000000000000000000000000000000000000000000000000000000000000000"
    );
    assert_eq!(manifest.audit.events_count, 0);

    fs::remove_file(temp_audit).ok();
}

#[test]
fn test_read_audit_tail_multiple_entries() {
    let temp_audit = "/tmp/test_multiple_audit.jsonl";
    fs::write(
        temp_audit,
        r#"{"digest":"0x1111111111111111111111111111111111111111111111111111111111111111","seq":1}
{"digest":"0x2222222222222222222222222222222222222222222222222222222222222222","seq":2}
{"digest":"0x3333333333333333333333333333333333333333333333333333333333333333","seq":3}"#,
    )
    .unwrap();

    let commitments = Commitments {
        supplier_root: "0xabc".to_string(),
        ubo_root: "0xdef".to_string(),
        company_commitment_root: "0x123".to_string(),
        supplier_count: Some(0),
        ubo_count: Some(0),
    };

    let policy_info = PolicyInfo {
        name: "Test".to_string(),
        version: "lksg.v1".to_string(),
        hash: "0xhash".to_string(),
    };

    let result = Manifest::build(&commitments, policy_info, temp_audit);
    assert!(result.is_ok());
    let manifest = result.unwrap();
    assert_eq!(
        manifest.audit.tail_digest,
        "0x3333333333333333333333333333333333333333333333333333333333333333"
    ); // Last digest
    assert_eq!(manifest.audit.events_count, 3);

    fs::remove_file(temp_audit).ok();
}

#[test]
fn test_read_audit_tail_file_not_found() {
    let commitments = Commitments {
        supplier_root: "0xabc".to_string(),
        ubo_root: "0xdef".to_string(),
        company_commitment_root: "0x123".to_string(),
        supplier_count: Some(0),
        ubo_count: Some(0),
    };

    let policy_info = PolicyInfo {
        name: "Test".to_string(),
        version: "lksg.v1".to_string(),
        hash: "0xhash".to_string(),
    };

    let result = Manifest::build(&commitments, policy_info, "/nonexistent/path/audit.jsonl");
    assert!(result.is_err());
}

#[test]
fn test_audit_info_clone() {
    let audit = AuditInfo {
        tail_digest: "0xtest".to_string(),
        events_count: 10,
    };

    let cloned = audit.clone();

    assert_eq!(cloned.tail_digest, "0xtest");
    assert_eq!(cloned.events_count, 10);
}

#[test]
fn test_proof_info_clone() {
    let proof = ProofInfo {
        proof_type: "mock".to_string(),
        status: "ok".to_string(),
    };

    let cloned = proof.clone();

    assert_eq!(cloned.proof_type, "mock");
    assert_eq!(cloned.status, "ok");
}

#[test]
fn test_signature_info_clone() {
    let sig = SignatureInfo {
        alg: "Ed25519".to_string(),
        signer: "company".to_string(),
        pubkey_hex: "0xabc".to_string(),
        sig_hex: "0xdef".to_string(),
    };

    let cloned = sig.clone();

    assert_eq!(cloned.alg, "Ed25519");
    assert_eq!(cloned.signer, "company");
    assert_eq!(cloned.pubkey_hex, "0xabc");
    assert_eq!(cloned.sig_hex, "0xdef");
}
