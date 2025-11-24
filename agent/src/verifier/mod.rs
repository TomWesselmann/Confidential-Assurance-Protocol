/// Verifier Module – Portable Verification Core
///
/// This module provides a pure, portable verification core that can be used
/// across different environments (CLI, tests, WASM, zkVM, registry sandboxes).
///
/// The verification logic is decoupled from I/O operations, making it:
/// - Testable: Easy to unit test with mock data
/// - Portable: Can run in constrained environments (no std::fs, no println!)
/// - Deterministic: Same inputs always produce same outputs
/// - Composable: Can be integrated into larger systems
///
/// For I/O-based package verification, see the `verifier` module in main.rs.
pub mod core;
pub mod core_verify;

// Re-export main types for convenience
pub use core::{
    extract_statement_from_manifest, verify, ProofStatement, VerifyOptions, VerifyReport,
};
pub use core_verify::{
    verify_core, CheckResult, CoreVerifyInput, CoreVerifyOptions, CoreVerifyResult, VerifyStatus,
};
