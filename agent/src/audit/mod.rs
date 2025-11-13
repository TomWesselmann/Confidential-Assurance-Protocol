//! Audit Module (Track A: Structured Audit-Log / Hash-Chain)
//!
//! Modular audit structure with hash-chain verification and export.

pub mod hash_chain;
pub mod v1_0;

// Re-export v1.0 types for backwards compatibility (used in tests)
#[allow(unused_imports)]
pub use v1_0::{AuditEntry, AuditLog};

// Re-export v2 types (Track A, used in tests)
#[allow(unused_imports)]
pub use hash_chain::{
    export_events, verify_chain, AuditChain, AuditEvent, AuditEventResult, VerifyReport,
};
