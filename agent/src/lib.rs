/// LkSG Proof Agent - Library Interface
///
/// This library exposes core modules for benchmarking, testing, and external use.
/// The main application logic remains in the binary (main.rs).
#[cfg(feature = "api-server")]
pub mod api;
pub mod audit;
#[cfg(feature = "api-server")]
pub mod auth;
pub mod blob_store;
pub mod bundle;
pub mod crypto;
#[cfg(feature = "api-server")]
pub mod http;
pub mod keys;
#[cfg(feature = "api-server")]
pub mod metrics;
pub mod orchestrator;
pub mod policy;
pub mod policy_v2;
pub mod proof;
pub mod providers;
pub mod registry;
#[cfg(feature = "api-server")]
pub mod tls;
pub mod verifier;
pub mod wasm;

// Binary-only modules exported for integration testing
// These modules are primarily used by the CLI binary (main.rs)
// but are exported here to enable integration tests and Tarpaulin coverage
pub mod commitment;
pub mod io;
pub mod manifest;
pub mod sign;
