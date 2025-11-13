/// LkSG Proof Agent - Library Interface
///
/// This library exposes core modules for benchmarking, testing, and external use.
/// The main application logic remains in the binary (main.rs).
pub mod api;
pub mod audit;
pub mod auth;
pub mod blob_store;
pub mod crypto;
pub mod http;
pub mod keys;
pub mod metrics;
pub mod orchestrator;
pub mod policy;
pub mod policy_v2;
pub mod proof;
pub mod providers;
pub mod registry;
pub mod tls;
pub mod verifier;
pub mod wasm;
