/// API Module - REST API Types and Handlers
pub mod auth;
pub mod metrics_middleware;
pub mod policy;
pub mod policy_compiler; // Week 3: PolicyV2 compiler with IR generation
pub mod rate_limit; // Production: Rate limiting & request throttling
pub mod tls; // Phase 1: TLS/mTLS configuration
pub mod upload; // Proof package upload endpoint
pub mod verify;

// Re-export important types
pub use policy::PolicyState;
