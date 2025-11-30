//! Verify API - Request/Response Types and Handler (Week 4: Embedded IR Support)
//!
//! Provides REST API types that map to the core verification logic.
//! Supports two modes:
//! - Mode A: policy_id (reference to stored policy)
//! - Mode B: ir (embedded IR v1 object)

use crate::crypto;
use crate::policy_v2::IrV1;
use crate::verifier::core::{ProofStatement, VerifyOptions, VerifyReport};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

// ============================================================================
// API Request/Response Types
// ============================================================================

/// POST /verify - Request Body (Week 4: Supports embedded IR)
///
/// Mode A (Policy ID Reference):
/// ```json
/// {
///   "policy_id": "lksg.v1",
///   "context": {...},
///   "backend": "mock"
/// }
/// ```
///
/// Mode B (Embedded IR):
/// ```json
/// {
///   "ir": { "ir_version": "1.0", ... },
///   "context": {...},
///   "backend": "mock"
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    /// Policy ID (Mode A - reference to stored policy)
    #[serde(default)]
    pub policy_id: Option<String>,

    /// Embedded IR v1 (Mode B - direct IR verification)
    #[serde(default)]
    pub ir: Option<IrV1>,

    /// Context containing proof data
    pub context: VerifyContext,

    /// Backend type (mock, zkvm, halo2)
    #[serde(default = "default_backend")]
    pub backend: String,

    /// Optional verification options
    #[serde(default)]
    pub options: VerifyRequestOptions,
}

fn default_backend() -> String {
    "mock".to_string()
}

/// Verification context (proof data)
#[derive(Debug, Deserialize)]
pub struct VerifyContext {
    /// Supplier hashes (BLAKE3, 0x-prefixed)
    #[serde(default)]
    pub supplier_hashes: Vec<String>,

    /// UBO hashes (BLAKE3, 0x-prefixed)
    #[serde(default)]
    pub ubo_hashes: Vec<String>,

    /// Company commitment root (BLAKE3, 0x-prefixed)
    pub company_commitment_root: Option<String>,

    /// Sanctions list root (BLAKE3, 0x-prefixed)
    pub sanctions_root: Option<String>,

    /// Jurisdiction list root (BLAKE3, 0x-prefixed)
    pub jurisdiction_root: Option<String>,
}

/// Request options
#[derive(Debug, Deserialize, Default)]
pub struct VerifyRequestOptions {
    /// Adaptive mode (activates rules based on context)
    #[serde(default)]
    pub adaptive: bool,

    /// Check timestamp validity
    #[serde(default = "default_true")]
    pub check_timestamp: bool,

    /// Check registry match
    #[serde(default = "default_true")]
    pub check_registry: bool,
}

fn default_true() -> bool {
    true
}

/// POST /verify - Response Body
#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    /// Result: "OK", "FAIL", or "WARN"
    pub result: String,

    /// Manifest hash (SHA3-256, 0x-prefixed)
    pub manifest_hash: String,

    /// Proof hash (SHA3-256, 0x-prefixed)
    pub proof_hash: String,

    /// Rule trace (which rules were checked)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<serde_json::Value>,

    /// Ed25519 signature (base64)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,

    /// RFC3161 timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,

    /// Detailed verification report
    pub report: VerifyReport,
}

// ============================================================================
// Handler Logic
// ============================================================================

/// Handles POST /verify request (Week 4: Supports Mode A & Mode B)
///
/// Mode A (Policy ID Reference):
///   - Uses policy_id to retrieve IR from cache/storage
///   - Builds manifest from context + policy
///
/// Mode B (Embedded IR):
///   - Uses provided IR directly
///   - Builds manifest from context + embedded IR
///
/// This function orchestrates the verification process:
/// 1. Determine mode (A or B) and extract/retrieve IR
/// 2. Build manifest from context + IR
/// 3. Extract statement from manifest
/// 4. Create mock proof (or call ZK backend)
/// 5. Run verifier::core::verify()
/// 6. Return structured response
pub fn handle_verify(req: VerifyRequest) -> Result<VerifyResponse> {
    // 1. Validate request (exactly one of policy_id or ir must be provided)
    let (ir, mode_name, policy_id_for_trace): (IrV1, &str, Option<String>) = match (
        &req.policy_id,
        &req.ir,
    ) {
        (Some(policy_id), None) => {
            // Mode A: Policy ID Reference - retrieve IR from LRU cache
            use crate::api::policy_compiler::{get_cache, get_id_index};

            // Lookup policy_hash from ID index
            let id_index = get_id_index();
            let index = id_index
                .lock()
                .map_err(|e| anyhow!("Lock error accessing policy ID index: {}", e))?;

            let policy_hash = index.get(policy_id)
                .ok_or_else(|| anyhow!("Policy not found: {}. Did you compile and persist it with POST /policy/compile?", policy_id))?
                .clone();
            drop(index); // Release lock early

            // Retrieve IR from LRU cache
            let cache = get_cache();
            let mut lru = cache
                .lock()
                .map_err(|e| anyhow!("Lock error accessing policy cache: {}", e))?;

            let entry = lru
                .get(&policy_hash)
                .ok_or_else(|| {
                    anyhow!(
                        "Policy not in cache: {}. Policy hash: {}",
                        policy_id,
                        policy_hash
                    )
                })?
                .clone();
            drop(lru); // Release lock early

            (entry.ir.clone(), "policy_id", Some(policy_id.clone()))
        }
        (None, Some(ir)) => {
            // Mode B: Embedded IR
            (ir.clone(), "embedded_ir", None)
        }
        (Some(_), Some(_)) => {
            return Err(anyhow!(
                "Cannot specify both policy_id and ir - use one or the other"
            ));
        }
        (None, None) => {
            return Err(anyhow!("Must specify either policy_id or ir"));
        }
    };

    // 2. Build manifest from context + IR
    let manifest = build_manifest_from_ir(&req, &ir)?;

    // 3. Extract statement from manifest
    let stmt = crate::verifier::core::extract_statement_from_manifest(&manifest)?;

    // 4. Create mock proof (Phase 2: Mock, Phase 3: ZK)
    let proof_bytes = create_mock_proof(&req, &stmt)?;

    // 5. Setup verify options
    let opts = VerifyOptions {
        check_timestamp: req.options.check_timestamp,
        check_registry: req.options.check_registry,
    };

    // 6. Run core verification (with metrics)
    let start = std::time::Instant::now();
    let report = crate::verifier::core::verify(&manifest, &proof_bytes, &stmt, &opts)?;
    let duration_secs = start.elapsed().as_secs_f64();

    // Record proof verification duration metric
    crate::metrics::get_metrics().record_proof_verification_duration(duration_secs);

    // 7. Build response (Week 4: trace shows mode and policy_id if available)
    let result = if report.status == "ok" {
        "OK".to_string()
    } else {
        "FAIL".to_string()
    };

    Ok(VerifyResponse {
        result,
        manifest_hash: report.manifest_hash.clone(),
        proof_hash: report.proof_hash.clone(),
        trace: Some(serde_json::json!({
            "backend": req.backend,
            "mode": mode_name,
            "policy_id": policy_id_for_trace,
            "adaptive": req.options.adaptive,
        })),
        signature: None, // TODO: Phase 3 - Sign with Ed25519
        timestamp: None, // TODO: Phase 3 - Add RFC3161 timestamp
        report,
    })
}

/// Builds a manifest JSON from embedded IR (Week 4: Mode B)
///
/// Uses the provided IR to construct the manifest instead of looking up policy_id
fn build_manifest_from_ir(req: &VerifyRequest, ir: &IrV1) -> Result<serde_json::Value> {
    // Compute company commitment root if not provided
    let company_root = if let Some(root) = &req.context.company_commitment_root {
        root.clone()
    } else {
        // Compute from supplier and UBO hashes
        compute_company_root(&req.context)?
    };

    // Use IR's policy_hash directly
    let policy_hash = ir.policy_hash.clone();

    // Build manifest structure (using IR metadata)
    let manifest = serde_json::json!({
        "version": "manifest.v1.0",
        "created_at": chrono::Utc::now().to_rfc3339(),
        "company_commitment_root": company_root,
        "policy": {
            "name": ir.policy_id.clone(),  // Use policy_id from IR
            "version": "lksg.v1",  // From IR version
            "hash": policy_hash,   // From IR policy_hash
        },
        "sanctions_root": req.context.sanctions_root,
        "jurisdiction_root": req.context.jurisdiction_root,
        "proof": {
            "proof_type": &req.backend,
            "status": "ok",
        },
    });

    Ok(manifest)
}

/// Builds a manifest JSON from the request context (Legacy - Mode A)
///
/// Note: This function is kept for backward compatibility but will be replaced
/// by build_manifest_from_ir once Mode A is fully integrated with IR cache
#[allow(dead_code)]
fn build_manifest_from_context(req: &VerifyRequest) -> Result<serde_json::Value> {
    // Compute company commitment root if not provided
    let company_root = if let Some(root) = &req.context.company_commitment_root {
        root.clone()
    } else {
        // Compute from supplier and UBO hashes
        compute_company_root(&req.context)?
    };

    // Compute policy hash (mock for now)
    let policy_id = req.policy_id.as_deref().unwrap_or("unknown");
    let policy_hash = compute_policy_hash(policy_id);

    // Build manifest structure
    let manifest = serde_json::json!({
        "version": "manifest.v1.0",
        "created_at": chrono::Utc::now().to_rfc3339(),
        "company_commitment_root": company_root,
        "policy": {
            "name": req.policy_id,
            "version": "lksg.v1",
            "hash": policy_hash,
        },
        "sanctions_root": req.context.sanctions_root,
        "jurisdiction_root": req.context.jurisdiction_root,
        "proof": {
            "proof_type": &req.backend,
            "status": "ok",
        },
    });

    Ok(manifest)
}

/// Computes company commitment root from supplier and UBO hashes
fn compute_company_root(ctx: &VerifyContext) -> Result<String> {
    // If company_commitment_root is already provided (from manifest), use it
    if let Some(root) = &ctx.company_commitment_root {
        return Ok(root.clone());
    }

    // Otherwise compute from individual hashes (mock implementation)
    let mut combined = String::new();
    for h in &ctx.supplier_hashes {
        combined.push_str(h);
    }
    for h in &ctx.ubo_hashes {
        combined.push_str(h);
    }

    if combined.is_empty() {
        return Err(anyhow!(
            "No supplier or UBO hashes provided and no company_commitment_root"
        ));
    }

    let hash = crypto::blake3_256(combined.as_bytes());
    Ok(crypto::hex_lower_prefixed32(hash))
}

/// Computes policy hash (mock implementation)
fn compute_policy_hash(policy_id: &str) -> String {
    let hash = crypto::sha3_256(policy_id.as_bytes());
    crypto::hex_lower_prefixed32(hash)
}

/// Creates a mock proof (Phase 2 implementation)
///
/// In Phase 3, this will call the ZK backend (zkvm, halo2, etc.)
fn create_mock_proof(_req: &VerifyRequest, stmt: &ProofStatement) -> Result<Vec<u8>> {
    // Mock proof: just serialize the statement with a marker
    let mock_proof = serde_json::json!({
        "version": "proof.mock.v0",
        "type": "mock",
        "statement": stmt,
        "proof_data": {
            "mock": true,
            "verified": true,
        },
    });

    Ok(serde_json::to_vec(&mock_proof)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy_v2::IrV1;

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

        // Should be a valid 0x-prefixed 64-char hex string
        assert!(result.starts_with("0x"));
        assert_eq!(result.len(), 66); // 0x + 64 hex chars

        // Should be deterministic
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

        // Should return the provided root
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
    fn test_compute_policy_hash_deterministic() {
        let hash1 = compute_policy_hash("test-policy");
        let hash2 = compute_policy_hash("test-policy");

        // Should be deterministic
        assert_eq!(hash1, hash2);

        // Should be 0x-prefixed 64-char hex
        assert!(hash1.starts_with("0x"));
        assert_eq!(hash1.len(), 66);

        // Different input should give different hash
        let hash3 = compute_policy_hash("other-policy");
        assert_ne!(hash1, hash3);
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

        // Check manifest structure
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

        let req = VerifyRequest {
            policy_id: None,
            ir: Some(create_test_ir()),
            context: create_test_context(),
            backend: "mock".to_string(),
            options: VerifyRequestOptions::default(),
        };

        let proof_bytes = create_mock_proof(&req, &stmt).unwrap();

        // Should be valid JSON
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
        assert_eq!(req.backend, "mock"); // default
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

        // When "options" field is missing entirely, Default::default() is used (all false)
        assert!(!req.options.adaptive);
        assert!(!req.options.check_timestamp);
        assert!(!req.options.check_registry);
    }

    #[test]
    fn test_verify_request_options_partial() {
        // When "options" object exists but fields are missing, field-level defaults apply
        let json = r#"{
            "policy_id": "lksg.v1",
            "context": {
                "supplier_hashes": [],
                "ubo_hashes": []
            },
            "options": {}
        }"#;

        let req: VerifyRequest = serde_json::from_str(json).unwrap();

        // Field-level defaults from #[serde(default = "default_true")]
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
    fn test_default_backend() {
        assert_eq!(default_backend(), "mock");
    }

    #[test]
    fn test_default_true() {
        assert!(default_true());
    }

    // ========================================================================
    // Handler Integration Tests
    // ========================================================================

    #[test]
    fn test_handle_verify_mode_b_success() {
        // Initialize metrics for tests that call handle_verify
        crate::metrics::init_metrics();

        // Mode B: Embedded IR verification
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

        // Verify handler produces valid output (result can be OK, WARN, or FAIL depending on verification)
        assert!(response.result == "OK" || response.result == "WARN" || response.result == "FAIL");
        assert!(response.manifest_hash.starts_with("0x"));
        assert!(response.proof_hash.starts_with("0x"));
        assert!(
            response.report.status == "ok"
                || response.report.status == "warn"
                || response.report.status == "fail"
        );

        // Verify trace contains Mode B metadata
        assert!(response.trace.is_some());
        let trace = response.trace.unwrap();
        assert_eq!(trace["mode"], "embedded_ir");
        assert_eq!(trace["backend"], "mock");
    }

    #[test]
    fn test_handle_verify_both_fields_error() {
        // Error case: Both policy_id and ir specified
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
        // Error case: Neither policy_id nor ir specified
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
        // Initialize metrics for tests that call handle_verify
        crate::metrics::init_metrics();

        // Mode B with custom options
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

        // Check that trace contains correct backend and options
        assert!(response.trace.is_some());
        let trace = response.trace.unwrap();
        assert_eq!(trace["backend"], "zkvm");
        assert_eq!(trace["mode"], "embedded_ir");
        assert_eq!(trace["adaptive"], true);
    }

    #[test]
    fn test_build_manifest_from_ir_complete() {
        // Test manifest building with all optional fields
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

        // Verify all fields
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
        // Test manifest building with computed company root
        let ir = create_test_ir();
        let mut ctx = create_test_context();
        ctx.company_commitment_root = None; // Force computation

        let req = VerifyRequest {
            policy_id: None,
            ir: Some(ir.clone()),
            context: ctx,
            backend: "mock".to_string(),
            options: VerifyRequestOptions::default(),
        };

        let manifest = build_manifest_from_ir(&req, &ir).unwrap();

        // Company root should be computed from supplier + UBO hashes
        let computed_root = manifest["company_commitment_root"].as_str().unwrap();
        assert!(computed_root.starts_with("0x"));
        assert_eq!(computed_root.len(), 66);
    }

    #[test]
    fn test_handle_verify_mode_a_with_cached_policy() {
        use crate::api::policy_compiler::{get_cache, get_id_index, PolicyEntry};
        use std::sync::Arc;

        // Initialize metrics
        crate::metrics::init_metrics();

        // Setup: Add policy to cache
        let ir = create_test_ir();
        let policy_id = "test-policy".to_string();
        let policy_hash = ir.policy_hash.clone();

        // Create a mock PolicyV2 for the entry
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

        // Add to cache and ID index
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

        // Test Mode A verification
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

        // Verify handler produces valid output
        assert!(response.result == "OK" || response.result == "WARN" || response.result == "FAIL");
        assert!(response.manifest_hash.starts_with("0x"));
        assert!(response.proof_hash.starts_with("0x"));

        // Verify trace contains Mode A metadata
        assert!(response.trace.is_some());
        let trace = response.trace.unwrap();
        assert_eq!(trace["mode"], "policy_id");
        assert_eq!(trace["backend"], "mock");
        assert_eq!(trace["policy_id"], "test-policy");
    }

    #[test]
    fn test_handle_verify_mode_a_policy_not_found() {
        // Initialize metrics
        crate::metrics::init_metrics();

        // Test Mode A with non-existent policy
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
