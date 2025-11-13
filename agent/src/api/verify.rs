/// Verify API - Request/Response Types and Handler (Week 4: Embedded IR Support)
///
/// Provides REST API types that map to the core verification logic.
/// Supports two modes:
/// - Mode A: policy_id (reference to stored policy)
/// - Mode B: ir (embedded IR v1 object)

use serde::{Deserialize, Serialize};
use anyhow::{anyhow, Result};
use crate::verifier::core::{ProofStatement, VerifyOptions, VerifyReport};
use crate::policy_v2::IrV1;
use crate::crypto;

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
    let (ir, mode_name, policy_id_for_trace): (IrV1, &str, Option<String>) = match (&req.policy_id, &req.ir) {
        (Some(policy_id), None) => {
            // Mode A: Policy ID Reference - retrieve IR from LRU cache
            use crate::api::policy_compiler::{get_cache, get_id_index};

            // Lookup policy_hash from ID index
            let id_index = get_id_index();
            let index = id_index.lock()
                .map_err(|e| anyhow!("Lock error accessing policy ID index: {}", e))?;

            let policy_hash = index.get(policy_id)
                .ok_or_else(|| anyhow!("Policy not found: {}. Did you compile and persist it with POST /policy/compile?", policy_id))?
                .clone();
            drop(index); // Release lock early

            // Retrieve IR from LRU cache
            let cache = get_cache();
            let mut lru = cache.lock()
                .map_err(|e| anyhow!("Lock error accessing policy cache: {}", e))?;

            let entry = lru.get(&policy_hash)
                .ok_or_else(|| anyhow!("Policy not in cache: {}. Policy hash: {}", policy_id, policy_hash))?
                .clone();
            drop(lru); // Release lock early

            (entry.ir.clone(), "policy_id", Some(policy_id.clone()))
        }
        (None, Some(ir)) => {
            // Mode B: Embedded IR
            (ir.clone(), "embedded_ir", None)
        }
        (Some(_), Some(_)) => {
            return Err(anyhow!("Cannot specify both policy_id and ir - use one or the other"));
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

    // 6. Run core verification
    let report = crate::verifier::core::verify(&manifest, &proof_bytes, &stmt, &opts)?;

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
    // For mock: just hash the concatenated hashes
    let mut combined = String::new();
    for h in &ctx.supplier_hashes {
        combined.push_str(h);
    }
    for h in &ctx.ubo_hashes {
        combined.push_str(h);
    }

    if combined.is_empty() {
        return Err(anyhow!("No supplier or UBO hashes provided"));
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
