//! Verify API - Request/Response Types and Handler (Week 4: Embedded IR Support)
//!
//! Provides REST API types that map to the core verification logic.
//! Supports two modes:
//! - Mode A: policy_id (reference to stored policy)
//! - Mode B: ir (embedded IR v1 object)
//!
//! ## Module Structure (v0.11 Refactoring)
//!
//! - `types`: VerifyRequest, VerifyContext, VerifyRequestOptions, VerifyResponse
//! - `handler`: handle_verify, mode resolution
//! - `manifest`: build_manifest_from_ir, compute_company_root
//! - `proof`: create_mock_proof

pub mod handler;
pub mod manifest;
pub mod proof;
pub mod types;

// Re-exports for backward compatibility
pub use handler::handle_verify;
pub use manifest::{build_manifest_from_ir, compute_company_root};
pub use proof::create_mock_proof;
pub use types::{VerifyContext, VerifyRequest, VerifyRequestOptions, VerifyResponse};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy_v2::IrV1;
    use crate::verifier::core::{ProofStatement, VerifyReport};

    fn create_test_ir() -> IrV1 {
        IrV1 {
            ir_version: "1.0".to_string(),
            policy_id: "test-policy".to_string(),
            policy_hash: "0x1234567890123456789012345678901234567890123456789012345678901234"
                .to_string(),
            rules: vec![],
            adaptivity: None,
            ir_hash: "0x5555555555555555555555555555555555555555555555555555555555555555"
                .to_string(),
        }
    }

    fn create_test_context() -> VerifyContext {
        VerifyContext {
            supplier_hashes: vec![
                "0x1111111111111111111111111111111111111111111111111111111111111111".to_string(),
                "0x2222222222222222222222222222222222222222222222222222222222222222".to_string(),
            ],
            ubo_hashes: vec![
                "0x3333333333333333333333333333333333333333333333333333333333333333".to_string(),
            ],
            company_commitment_root: Some(
                "0x4444444444444444444444444444444444444444444444444444444444444444".to_string(),
            ),
            sanctions_root: None,
            jurisdiction_root: None,
        }
    }

    #[test]
    fn test_compute_company_root_from_hashes() {
        let ctx = VerifyContext {
            supplier_hashes: vec![
                "0x1111111111111111111111111111111111111111111111111111111111111111".to_string(),
                "0x2222222222222222222222222222222222222222222222222222222222222222".to_string(),
            ],
            ubo_hashes: vec![
                "0x3333333333333333333333333333333333333333333333333333333333333333".to_string(),
            ],
            company_commitment_root: None,
            sanctions_root: None,
            jurisdiction_root: None,
        };

        let result = compute_company_root(&ctx).unwrap();

        assert!(result.starts_with("0x"));
        assert_eq!(result.len(), 66);

        let result2 = compute_company_root(&ctx).unwrap();
        assert_eq!(result, result2);
    }

    #[test]
    fn test_compute_company_root_from_provided() {
        let ctx = VerifyContext {
            supplier_hashes: vec![],
            ubo_hashes: vec![],
            company_commitment_root: Some(
                "0x4444444444444444444444444444444444444444444444444444444444444444".to_string(),
            ),
            sanctions_root: None,
            jurisdiction_root: None,
        };

        let result = compute_company_root(&ctx).unwrap();

        assert_eq!(
            result,
            "0x4444444444444444444444444444444444444444444444444444444444444444"
        );
    }

    #[test]
    fn test_compute_company_root_no_data_error() {
        let ctx = VerifyContext {
            supplier_hashes: vec![],
            ubo_hashes: vec![],
            company_commitment_root: None,
            sanctions_root: None,
            jurisdiction_root: None,
        };

        let result = compute_company_root(&ctx);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No supplier or UBO hashes"));
    }

    #[test]
    fn test_build_manifest_from_ir() {
        let ir = create_test_ir();
        let ctx = create_test_context();

        let req = VerifyRequest {
            policy_id: None,
            ir: Some(ir.clone()),
            context: ctx,
            backend: "mock".to_string(),
            options: VerifyRequestOptions::default(),
        };

        let manifest = build_manifest_from_ir(&req, &ir).unwrap();

        assert_eq!(manifest["version"], "manifest.v1.0");
        assert_eq!(manifest["policy"]["name"], "test-policy");
        assert_eq!(manifest["policy"]["version"], "lksg.v1");
        assert_eq!(
            manifest["policy"]["hash"],
            "0x1234567890123456789012345678901234567890123456789012345678901234"
        );
        assert_eq!(manifest["proof"]["proof_type"], "mock");
    }

    #[test]
    fn test_create_mock_proof() {
        let stmt = ProofStatement {
            policy_hash: "0x1234567890123456789012345678901234567890123456789012345678901234"
                .to_string(),
            company_commitment_root:
                "0x4444444444444444444444444444444444444444444444444444444444444444".to_string(),
            sanctions_root: None,
            jurisdiction_root: None,
            extensions: None,
        };

        let proof_bytes = create_mock_proof(&stmt).unwrap();

        let proof_json: serde_json::Value = serde_json::from_slice(&proof_bytes).unwrap();

        assert_eq!(proof_json["version"], "proof.mock.v0");
        assert_eq!(proof_json["type"], "mock");
        assert_eq!(proof_json["proof_data"]["mock"], true);
        assert_eq!(proof_json["proof_data"]["verified"], true);
    }

    #[test]
    fn test_verify_request_deserialization_mode_a() {
        let json = r#"{
            "policy_id": "lksg.v1",
            "context": {
                "supplier_hashes": ["0x1111111111111111111111111111111111111111111111111111111111111111"],
                "ubo_hashes": ["0x2222222222222222222222222222222222222222222222222222222222222222"],
                "company_commitment_root": "0x3333333333333333333333333333333333333333333333333333333333333333"
            },
            "backend": "mock"
        }"#;

        let req: VerifyRequest = serde_json::from_str(json).unwrap();

        assert_eq!(req.policy_id, Some("lksg.v1".to_string()));
        assert!(req.ir.is_none());
        assert_eq!(req.backend, "mock");
        assert_eq!(req.context.supplier_hashes.len(), 1);
        assert_eq!(req.context.ubo_hashes.len(), 1);
    }

    #[test]
    fn test_verify_request_deserialization_mode_b() {
        let json = r#"{
            "ir": {
                "ir_version": "1.0",
                "policy_id": "test-policy",
                "policy_hash": "0x1234567890123456789012345678901234567890123456789012345678901234",
                "rules": [],
                "ir_hash": "0x5555555555555555555555555555555555555555555555555555555555555555"
            },
            "context": {
                "supplier_hashes": [],
                "ubo_hashes": []
            }
        }"#;

        let req: VerifyRequest = serde_json::from_str(json).unwrap();

        assert!(req.policy_id.is_none());
        assert!(req.ir.is_some());
        assert_eq!(req.backend, "mock");
    }

    #[test]
    fn test_verify_request_options_defaults() {
        let json = r#"{
            "policy_id": "lksg.v1",
            "context": {
                "supplier_hashes": [],
                "ubo_hashes": []
            }
        }"#;

        let req: VerifyRequest = serde_json::from_str(json).unwrap();

        assert!(!req.options.adaptive);
        assert!(!req.options.check_timestamp);
        assert!(!req.options.check_registry);
    }

    #[test]
    fn test_verify_request_options_partial() {
        let json = r#"{
            "policy_id": "lksg.v1",
            "context": {
                "supplier_hashes": [],
                "ubo_hashes": []
            },
            "options": {}
        }"#;

        let req: VerifyRequest = serde_json::from_str(json).unwrap();

        assert!(!req.options.adaptive);
        assert!(req.options.check_timestamp);
        assert!(req.options.check_registry);
    }

    #[test]
    fn test_verify_response_serialization() {
        let report = VerifyReport {
            status: "ok".to_string(),
            manifest_hash: "0x1111111111111111111111111111111111111111111111111111111111111111"
                .to_string(),
            proof_hash: "0x2222222222222222222222222222222222222222222222222222222222222222"
                .to_string(),
            signature_valid: true,
            timestamp_valid: Some(true),
            registry_match: Some(true),
            details: serde_json::json!([]),
        };

        let response = VerifyResponse {
            result: "OK".to_string(),
            manifest_hash: "0x1111111111111111111111111111111111111111111111111111111111111111"
                .to_string(),
            proof_hash: "0x2222222222222222222222222222222222222222222222222222222222222222"
                .to_string(),
            trace: Some(serde_json::json!({"test": "value"})),
            signature: Some("signature_base64".to_string()),
            timestamp: None,
            report,
        };

        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("OK"));
        assert!(json.contains("0x1111111111111111111111111111111111111111111111111111111111111111"));
        assert!(json.contains("signature_base64"));
    }

    #[test]
    fn test_handle_verify_mode_b_success() {
        crate::metrics::init_metrics();

        let ir = create_test_ir();
        let ctx = create_test_context();

        let req = VerifyRequest {
            policy_id: None,
            ir: Some(ir),
            context: ctx,
            backend: "mock".to_string(),
            options: VerifyRequestOptions::default(),
        };

        let result = handle_verify(req);

        assert!(
            result.is_ok(),
            "Mode B verification handler should complete successfully"
        );
        let response = result.unwrap();

        assert!(response.result == "OK" || response.result == "WARN" || response.result == "FAIL");
        assert!(response.manifest_hash.starts_with("0x"));
        assert!(response.proof_hash.starts_with("0x"));

        assert!(response.trace.is_some());
        let trace = response.trace.unwrap();
        assert_eq!(trace["mode"], "embedded_ir");
        assert_eq!(trace["backend"], "mock");
    }

    #[test]
    fn test_handle_verify_both_fields_error() {
        let ir = create_test_ir();
        let ctx = create_test_context();

        let req = VerifyRequest {
            policy_id: Some("lksg.v1".to_string()),
            ir: Some(ir),
            context: ctx,
            backend: "mock".to_string(),
            options: VerifyRequestOptions::default(),
        };

        let result = handle_verify(req);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cannot specify both policy_id and ir"));
    }

    #[test]
    fn test_handle_verify_neither_field_error() {
        let ctx = create_test_context();

        let req = VerifyRequest {
            policy_id: None,
            ir: None,
            context: ctx,
            backend: "mock".to_string(),
            options: VerifyRequestOptions::default(),
        };

        let result = handle_verify(req);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Must specify either policy_id or ir"));
    }

    #[test]
    fn test_handle_verify_mode_b_with_options() {
        crate::metrics::init_metrics();

        let ir = create_test_ir();
        let ctx = create_test_context();

        let req = VerifyRequest {
            policy_id: None,
            ir: Some(ir),
            context: ctx,
            backend: "zkvm".to_string(),
            options: VerifyRequestOptions {
                adaptive: true,
                check_timestamp: true,
                check_registry: true,
            },
        };

        let result = handle_verify(req);

        assert!(result.is_ok());
        let response = result.unwrap();

        assert!(response.trace.is_some());
        let trace = response.trace.unwrap();
        assert_eq!(trace["backend"], "zkvm");
        assert_eq!(trace["mode"], "embedded_ir");
        assert_eq!(trace["adaptive"], true);
    }

    #[test]
    fn test_build_manifest_from_ir_complete() {
        let ir = create_test_ir();
        let mut ctx = create_test_context();
        ctx.sanctions_root =
            Some("0x5555555555555555555555555555555555555555555555555555555555555555".to_string());
        ctx.jurisdiction_root =
            Some("0x6666666666666666666666666666666666666666666666666666666666666666".to_string());

        let req = VerifyRequest {
            policy_id: None,
            ir: Some(ir.clone()),
            context: ctx,
            backend: "mock".to_string(),
            options: VerifyRequestOptions::default(),
        };

        let manifest = build_manifest_from_ir(&req, &ir).unwrap();

        assert_eq!(manifest["version"], "manifest.v1.0");
        assert_eq!(manifest["policy"]["name"], "test-policy");
        assert_eq!(
            manifest["policy"]["hash"],
            "0x1234567890123456789012345678901234567890123456789012345678901234"
        );
        assert_eq!(
            manifest["company_commitment_root"],
            "0x4444444444444444444444444444444444444444444444444444444444444444"
        );
        assert_eq!(
            manifest["sanctions_root"],
            "0x5555555555555555555555555555555555555555555555555555555555555555"
        );
        assert_eq!(
            manifest["jurisdiction_root"],
            "0x6666666666666666666666666666666666666666666666666666666666666666"
        );
    }

    #[test]
    fn test_build_manifest_from_ir_computed_root() {
        let ir = create_test_ir();
        let mut ctx = create_test_context();
        ctx.company_commitment_root = None;

        let req = VerifyRequest {
            policy_id: None,
            ir: Some(ir.clone()),
            context: ctx,
            backend: "mock".to_string(),
            options: VerifyRequestOptions::default(),
        };

        let manifest = build_manifest_from_ir(&req, &ir).unwrap();

        let computed_root = manifest["company_commitment_root"].as_str().unwrap();
        assert!(computed_root.starts_with("0x"));
        assert_eq!(computed_root.len(), 66);
    }

    #[test]
    fn test_handle_verify_mode_a_with_cached_policy() {
        use crate::api::policy_compiler::{get_cache, get_id_index, PolicyEntry};
        use std::sync::Arc;

        crate::metrics::init_metrics();

        let ir = create_test_ir();
        let policy_id = "test-policy".to_string();
        let policy_hash = ir.policy_hash.clone();

        let policy = crate::policy_v2::PolicyV2 {
            id: policy_id.clone(),
            version: "1.0.0".to_string(),
            legal_basis: vec![],
            description: "Test policy".to_string(),
            inputs: std::collections::BTreeMap::new(),
            rules: vec![],
            adaptivity: None,
        };

        let entry = PolicyEntry {
            policy,
            policy_hash: policy_hash.clone(),
            ir: ir.clone(),
            ir_hash: ir.ir_hash.clone(),
        };

        {
            let cache = get_cache();
            let mut lru = cache.lock().unwrap();
            lru.put(policy_hash.clone(), Arc::new(entry));
        }
        {
            let id_index = get_id_index();
            let mut index = id_index.lock().unwrap();
            index.insert(policy_id.clone(), policy_hash.clone());
        }

        let ctx = create_test_context();
        let req = VerifyRequest {
            policy_id: Some(policy_id),
            ir: None,
            context: ctx,
            backend: "mock".to_string(),
            options: VerifyRequestOptions::default(),
        };

        let result = handle_verify(req);

        assert!(result.is_ok(), "Mode A verification should succeed");
        let response = result.unwrap();

        assert!(response.result == "OK" || response.result == "WARN" || response.result == "FAIL");
        assert!(response.manifest_hash.starts_with("0x"));
        assert!(response.proof_hash.starts_with("0x"));

        assert!(response.trace.is_some());
        let trace = response.trace.unwrap();
        assert_eq!(trace["mode"], "policy_id");
        assert_eq!(trace["backend"], "mock");
        assert_eq!(trace["policy_id"], "test-policy");
    }

    #[test]
    fn test_handle_verify_mode_a_policy_not_found() {
        crate::metrics::init_metrics();

        let ctx = create_test_context();
        let req = VerifyRequest {
            policy_id: Some("non-existent-policy".to_string()),
            ir: None,
            context: ctx,
            backend: "mock".to_string(),
            options: VerifyRequestOptions::default(),
        };

        let result = handle_verify(req);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Policy not found"));
    }
}
