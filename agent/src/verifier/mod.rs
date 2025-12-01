//! Verifier Module – Portable Verification Core
//!
//! This module provides a pure, portable verification core that can be used
//! across different environments (CLI, tests, WASM, zkVM, registry sandboxes).
//!
//! ## Module Structure (v0.11 Refactoring)
//!
//! - `types`: Core data structures (ProofStatement, VerifyOptions, VerifyReport)
//! - `statement`: Statement extraction from manifests
//! - `verify`: Pure verification logic
//! - `core`: Re-export layer for backward compatibility
//! - `core_verify`: Alternative verification interface

// Core modules (v0.11 split)
pub mod statement;
pub mod types;
pub mod verify;

// Re-export layer
pub mod core;

// Alternative verification interface
pub mod core_verify;

// Re-export main types for convenience (via core for backward compatibility)
pub use core::{
    extract_statement_from_manifest, verify, verify_from_source, ProofStatement, VerifyOptions,
    VerifyReport,
};
pub use core_verify::{
    verify_core, CheckResult, CoreVerifyInput, CoreVerifyOptions, CoreVerifyResult, VerifyStatus,
};
