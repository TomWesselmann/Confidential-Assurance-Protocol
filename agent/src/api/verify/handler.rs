//! Verify Handler - Main request processing logic
//!
//! Handles POST /verify requests with Mode A (policy_id) and Mode B (embedded IR).

use crate::policy_v2::IrV1;
use crate::verifier::core::VerifyOptions;
use anyhow::{anyhow, Result};

use super::manifest::build_manifest_from_ir;
use super::proof::create_mock_proof;
use super::types::{VerifyRequest, VerifyResponse};

/// Resolved mode from request validation
struct ResolvedMode {
    ir: IrV1,
    mode_name: &'static str,
    policy_id: Option<String>,
}

/// Handles POST /verify request (Week 4: Supports Mode A & Mode B)
///
/// Mode A (Policy ID Reference):
///   - Uses policy_id to retrieve IR from cache/storage
///   - Builds manifest from context + policy
///
/// Mode B (Embedded IR):
///   - Uses provided IR directly
///   - Builds manifest from context + embedded IR
pub fn handle_verify(req: VerifyRequest) -> Result<VerifyResponse> {
    // 1. Validate request and resolve mode
    let mode = resolve_mode(&req)?;

    // 2. Build manifest from context + IR
    let manifest = build_manifest_from_ir(&req, &mode.ir)?;

    // 3. Extract statement from manifest
    let stmt = crate::verifier::core::extract_statement_from_manifest(&manifest)?;

    // 4. Create mock proof (Phase 2: Mock, Phase 3: ZK)
    let proof_bytes = create_mock_proof(&stmt)?;

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

    // 7. Build response
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
            "mode": mode.mode_name,
            "policy_id": mode.policy_id,
            "adaptive": req.options.adaptive,
        })),
        signature: None, // TODO: Phase 3 - Sign with Ed25519
        timestamp: None, // TODO: Phase 3 - Add RFC3161 timestamp
        report,
    })
}

/// Resolves the verification mode from request
///
/// Returns the IR, mode name, and optional policy_id for tracing
fn resolve_mode(req: &VerifyRequest) -> Result<ResolvedMode> {
    match (&req.policy_id, &req.ir) {
        (Some(policy_id), None) => {
            // Mode A: Policy ID Reference - retrieve IR from LRU cache
            let ir = retrieve_ir_from_cache(policy_id)?;
            Ok(ResolvedMode {
                ir,
                mode_name: "policy_id",
                policy_id: Some(policy_id.clone()),
            })
        }
        (None, Some(ir)) => {
            // Mode B: Embedded IR
            Ok(ResolvedMode {
                ir: ir.clone(),
                mode_name: "embedded_ir",
                policy_id: None,
            })
        }
        (Some(_), Some(_)) => {
            Err(anyhow!(
                "Cannot specify both policy_id and ir - use one or the other"
            ))
        }
        (None, None) => {
            Err(anyhow!("Must specify either policy_id or ir"))
        }
    }
}

/// Retrieves IR from policy cache by policy_id
fn retrieve_ir_from_cache(policy_id: &str) -> Result<IrV1> {
    use crate::api::policy_compiler::{get_cache, get_id_index};

    // Lookup policy_hash from ID index
    let id_index = get_id_index();
    let index = id_index
        .lock()
        .map_err(|e| anyhow!("Lock error accessing policy ID index: {}", e))?;

    let policy_hash = index
        .get(policy_id)
        .ok_or_else(|| {
            anyhow!(
                "Policy not found: {}. Did you compile and persist it with POST /policy/compile?",
                policy_id
            )
        })?
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

    Ok(entry.ir.clone())
}
