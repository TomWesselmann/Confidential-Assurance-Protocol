//! Mock Proof Creation - Phase 2 implementation
//!
//! In Phase 3, this will call the ZK backend (zkvm, halo2, etc.)

use crate::verifier::core::ProofStatement;
use anyhow::Result;

/// Creates a mock proof (Phase 2 implementation)
///
/// In Phase 3, this will call the ZK backend (zkvm, halo2, etc.)
pub fn create_mock_proof(stmt: &ProofStatement) -> Result<Vec<u8>> {
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
