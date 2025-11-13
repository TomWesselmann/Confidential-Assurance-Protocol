//! Audit Module (Track A: Structured Audit-Log / Hash-Chain)
//!
//! Modular audit structure with hash-chain verification and export.

pub mod v1_0;
pub mod hash_chain;

// Re-export v1.0 types for backwards compatibility
pub use v1_0::{AuditEntry, AuditLog};

// Re-export v2 types (Track A)
pub use hash_chain::{
    AuditEvent, AuditEventResult, AuditChain, VerifyReport, verify_chain, export_events
};
