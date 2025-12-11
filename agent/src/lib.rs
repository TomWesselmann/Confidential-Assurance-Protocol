/// CAP Agent - Minimal Local Agent Library
///
/// Cryptographic Audit Proofs for Supply Chain Compliance
/// This library provides core modules for bundle creation, verification, and signing.

// Core cryptographic modules
pub mod commitment;
pub mod crypto;
pub mod sign;

// Audit and registry
pub mod audit;
pub mod registry;

// Bundle handling
pub mod bundle;
pub mod manifest;
pub mod io;
pub mod blob_store;

// Policy engine
pub mod policy;
pub mod policy_v2;

// Verification
pub mod verifier;
pub mod proof;
pub mod proof_engine;
pub mod proof_mock;
pub mod package_verifier;

// Key management
pub mod keys;
pub mod providers;

// CLI (for integration testing)
pub mod cli;
