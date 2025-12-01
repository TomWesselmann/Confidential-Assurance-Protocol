//! Manifest Building - Constructs manifest from IR and context
//!
//! Provides helpers to build manifest JSON from verification context.

use crate::crypto;
use crate::policy_v2::IrV1;
use anyhow::{anyhow, Result};

use super::types::{VerifyContext, VerifyRequest};

/// Builds a manifest JSON from embedded IR (Week 4: Mode B)
///
/// Uses the provided IR to construct the manifest instead of looking up policy_id
pub fn build_manifest_from_ir(req: &VerifyRequest, ir: &IrV1) -> Result<serde_json::Value> {
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

/// Computes company commitment root from supplier and UBO hashes
pub fn compute_company_root(ctx: &VerifyContext) -> Result<String> {
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
