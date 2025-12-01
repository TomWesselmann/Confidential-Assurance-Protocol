//! Core Verification - Pure verification logic
//!
//! Provides I/O-free verification functions that can be used in
//! CLI, tests, WASM, zkVM, and registry sandboxes.

use anyhow::{anyhow, Result};

use crate::bundle::{load_bundle_atomic, BundleSource};
use crate::crypto;

use super::statement::extract_statement_from_manifest;
use super::types::{ProofStatement, VerifyOptions, VerifyReport};

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

    details.insert(
        "manifest_hash".to_string(),
        serde_json::json!(manifest_hash),
    );
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
        details.insert(
            "registry_check".to_string(),
            serde_json::json!("not_implemented"),
        );
        Some(false)
    } else {
        details.insert("registry_check".to_string(), serde_json::json!("disabled"));
        None
    };

    // 6. Determine overall status
    let all_required_passed = statement_valid && signature_valid;
    let status = if all_required_passed { "ok" } else { "fail" }.to_string();

    details.insert(
        "checks_passed".to_string(),
        serde_json::json!(checks_passed),
    );
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

/// Verifies a proof package from a BundleSource (REQ-03, REQ-07)
///
/// High-level verification function that loads a bundle atomically from
/// a source (Directory or ZipFile) and verifies it with default offline options.
pub fn verify_from_source(
    source: &BundleSource,
    opts: Option<&VerifyOptions>,
) -> Result<VerifyReport> {
    // Load bundle atomically (REQ-04: TOCTOU prevention)
    let bundle_data = load_bundle_atomic(source)?;

    // Find the first proof unit (MVP: single proof unit only)
    let proof_unit = bundle_data
        .meta
        .proof_units
        .first()
        .ok_or_else(|| anyhow!("No proof units found in bundle"))?;

    // Extract manifest file
    let manifest_bytes = bundle_data
        .files
        .get(&proof_unit.manifest_file)
        .ok_or_else(|| anyhow!("Manifest file not found: {}", proof_unit.manifest_file))?;

    let manifest: serde_json::Value = serde_json::from_slice(manifest_bytes)?;

    // Extract proof file
    let proof_bytes = bundle_data
        .files
        .get(&proof_unit.proof_file)
        .ok_or_else(|| anyhow!("Proof file not found: {}", proof_unit.proof_file))?;

    // Extract statement from manifest
    let stmt = extract_statement_from_manifest(&manifest)?;

    // Use provided options or default (offline)
    let default_opts = VerifyOptions::default();
    let verify_opts = opts.unwrap_or(&default_opts);

    // Verify with existing pure function
    verify(&manifest, proof_bytes, &stmt, verify_opts)
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
    if let Some(root) = manifest
        .get("company_commitment_root")
        .and_then(|r| r.as_str())
    {
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

    details.insert(
        "statement_validation".to_string(),
        serde_json::json!(checks),
    );
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

    details.insert(
        "signature_present".to_string(),
        serde_json::json!(has_signatures),
    );

    if has_signatures {
        let count = manifest
            .get("signatures")
            .and_then(|s| s.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);
        details.insert("signature_count".to_string(), serde_json::json!(count));
    }

    has_signatures
}

/// Checks timestamp in manifest (validates dual-anchor structure)
fn check_timestamp_in_manifest(
    manifest: &serde_json::Value,
    details: &mut serde_json::Map<String, serde_json::Value>,
) -> bool {
    let has_time_anchor = manifest.get("time_anchor").is_some();

    details.insert(
        "timestamp_present".to_string(),
        serde_json::json!(has_time_anchor),
    );

    if !has_time_anchor {
        return true;
    }

    let anchor = match manifest.get("time_anchor") {
        Some(a) => a,
        None => return true,
    };

    details.insert("timestamp_info".to_string(), anchor.clone());

    // Check dual-anchor structure (v0.9.0)
    let has_private = anchor.get("private").is_some();
    let has_public = anchor.get("public").is_some();

    details.insert(
        "dual_anchor_private".to_string(),
        serde_json::json!(has_private),
    );
    details.insert(
        "dual_anchor_public".to_string(),
        serde_json::json!(has_public),
    );

    // Validate private anchor consistency
    if has_private {
        if let Some(private) = anchor.get("private") {
            let private_audit_tip = private.get("audit_tip_hex").and_then(|v| v.as_str());
            let anchor_audit_tip = anchor.get("audit_tip_hex").and_then(|v| v.as_str());

            if let (Some(priv_tip), Some(anc_tip)) = (private_audit_tip, anchor_audit_tip) {
                if priv_tip != anc_tip {
                    details.insert(
                        "dual_anchor_error".to_string(),
                        serde_json::json!(
                            "Private anchor audit_tip_hex does not match time_anchor.audit_tip_hex"
                        ),
                    );
                    return false;
                }
            }
        }
    }

    // Validate public anchor format
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

    true
}
