//! Package Verifier Module
//!
//! I/O-based proof package verification supporting both modern (cap-bundle.v1)
//! and legacy bundle formats.
//!
//! ## Module Structure (v0.11 Refactoring)
//!
//! - `types`: VerificationResult, BundleVerifyResult, BundleType
//! - `validation`: File hash validation with TOCTOU mitigation
//! - `verifier`: Main Verifier struct and implementation
//! - `summary`: Package summary display

pub mod summary;
pub mod types;
pub mod validation;
pub mod verifier;

// Public API re-exports for backward compatibility
pub use summary::show_package_summary;
#[allow(unused_imports)]
pub use types::{BundleType, BundleVerifyResult, VerificationResult};
#[allow(unused_imports)]
pub use validation::{load_and_validate_bundle, validate_file_hash, MAX_FILE_SIZE};
#[allow(unused_imports)]
pub use verifier::{detect_bundle_type, BundleVerifier, Verifier};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{AuditInfo, Manifest, ProofInfo};
    use crate::policy::PolicyInfo;
    use crate::proof_engine::{ConstraintCheck, Proof, ProofData};
    use std::fs;

    fn create_test_manifest() -> Manifest {
        Manifest {
            version: "manifest.v1.0".to_string(),
            created_at: "2025-10-25T10:00:00Z".to_string(),
            supplier_root: "0xabc".to_string(),
            ubo_root: "0xdef".to_string(),
            company_commitment_root: "0x123".to_string(),
            policy: PolicyInfo {
                name: "Test Policy".to_string(),
                version: "lksg.v1".to_string(),
                hash: "0xpolicy".to_string(),
            },
            audit: AuditInfo {
                tail_digest: "0xtail".to_string(),
                events_count: 10,
            },
            proof: ProofInfo {
                proof_type: "mock".to_string(),
                status: "ok".to_string(),
            },
            signatures: Vec::new(),
            time_anchor: None,
        }
    }

    #[test]
    fn test_verifier_check_integrity() {
        let test_dir = "/tmp/test_proof_package_mod";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

        fs::write(format!("{}/manifest.json", test_dir), "{}").unwrap();
        fs::write(format!("{}/proof.dat", test_dir), "dummy").unwrap();

        let verifier = Verifier::new(test_dir);
        let result = verifier.check_package_integrity().unwrap();
        assert!(result.contains("vollständig"));

        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_verifier_missing_files() {
        let test_dir = "/tmp/test_proof_package_empty_mod";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

        let verifier = Verifier::new(test_dir);
        let result = verifier.check_package_integrity();
        assert!(result.is_err());

        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_package_summary() {
        let test_dir = "/tmp/test_proof_summary_mod";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

        let manifest = create_test_manifest();
        let manifest_hash = Proof::compute_manifest_hash(&manifest).unwrap();

        let proof = Proof {
            version: "proof.v0".to_string(),
            proof_type: "mock".to_string(),
            statement: "policy:lksg.v1".to_string(),
            manifest_hash,
            policy_hash: "0xpolicy".to_string(),
            proof_data: ProofData {
                checked_constraints: vec![ConstraintCheck {
                    name: "test_check".to_string(),
                    ok: true,
                }],
            },
            status: "ok".to_string(),
        };

        manifest
            .save(format!("{}/manifest.json", test_dir))
            .unwrap();
        proof
            .save_as_dat(format!("{}/proof.dat", test_dir))
            .unwrap();

        let summary = show_package_summary(test_dir).unwrap();
        assert!(summary.contains("PROOF-PAKET"));
        assert!(summary.contains("Test Policy"));

        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_verify_success() {
        let test_dir = "/tmp/test_verify_success_mod";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

        let manifest = create_test_manifest();
        let manifest_hash = Proof::compute_manifest_hash(&manifest).unwrap();

        let proof = Proof {
            version: "proof.v0".to_string(),
            proof_type: "mock".to_string(),
            statement: "policy:lksg.v1".to_string(),
            manifest_hash: manifest_hash.clone(),
            policy_hash: "0xpolicy".to_string(),
            proof_data: ProofData {
                checked_constraints: vec![
                    ConstraintCheck {
                        name: "check1".to_string(),
                        ok: true,
                    },
                    ConstraintCheck {
                        name: "check2".to_string(),
                        ok: true,
                    },
                ],
            },
            status: "ok".to_string(),
        };

        manifest
            .save(format!("{}/manifest.json", test_dir))
            .unwrap();
        proof
            .save_as_dat(format!("{}/proof.dat", test_dir))
            .unwrap();

        let verifier = Verifier::new(test_dir);
        let result = verifier.verify().unwrap();

        assert!(result.success);
        assert_eq!(result.manifest_hash, manifest_hash);

        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_verify_manifest_not_found() {
        let test_dir = "/tmp/test_verify_no_manifest_mod";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

        let verifier = Verifier::new(test_dir);
        let result = verifier.verify();

        assert!(result.is_err());

        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_extract_manifest_success() {
        let test_dir = "/tmp/test_extract_manifest_mod";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

        let manifest = create_test_manifest();
        manifest
            .save(format!("{}/manifest.json", test_dir))
            .unwrap();

        let verifier = Verifier::new(test_dir);
        let extracted = verifier.extract_manifest().unwrap();

        assert_eq!(extracted.version, "manifest.v1.0");

        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_show_audit_trail_success() {
        let test_dir = "/tmp/test_audit_trail_mod";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

        let manifest = create_test_manifest();
        manifest
            .save(format!("{}/manifest.json", test_dir))
            .unwrap();

        let verifier = Verifier::new(test_dir);
        let (tail_digest, events_count) = verifier.show_audit_trail().unwrap();

        assert_eq!(tail_digest, "0xtail");
        assert_eq!(events_count, 10);

        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_integrity_with_signature() {
        let test_dir = "/tmp/test_integrity_with_sig_mod";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

        fs::write(format!("{}/manifest.json", test_dir), "{}").unwrap();
        fs::write(format!("{}/proof.dat", test_dir), "dummy").unwrap();
        fs::write(format!("{}/signature.json", test_dir), "{}").unwrap();

        let verifier = Verifier::new(test_dir);
        let result = verifier.check_package_integrity().unwrap();

        assert!(result.contains("ja"));

        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_detect_bundle_type_modern() {
        let test_dir = "/tmp/test_bundle_type_modern";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

        fs::write(format!("{}/_meta.json", test_dir), "{}").unwrap();

        assert_eq!(
            detect_bundle_type(std::path::Path::new(test_dir)),
            BundleType::Modern
        );

        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_detect_bundle_type_legacy() {
        let test_dir = "/tmp/test_bundle_type_legacy";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

        assert_eq!(
            detect_bundle_type(std::path::Path::new(test_dir)),
            BundleType::Legacy
        );

        fs::remove_dir_all(test_dir).ok();
    }
}
