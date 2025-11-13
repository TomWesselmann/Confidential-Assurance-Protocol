/// WASM Executor - Orchestrates Bundle Verification
///
/// Loads bundle files and executes WASM verifier with fallback to native.
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::loader::{WasmLimits, WasmVerifier};
use crate::verifier::core::{self as verifier_core, VerifyOptions, VerifyReport};

/// Executor ABI Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorConfig {
    pub abi_version: String,
    pub entry_function: String,
    pub input_encoding: String,
    pub output_encoding: String,
    pub limits: WasmLimits,
    pub expectations: Expectations,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expectations {
    pub manifest_schema: String,
    pub proof_container_version: String,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            abi_version: "wasm-verify.v1".to_string(),
            entry_function: "verify".to_string(),
            input_encoding: "json".to_string(),
            output_encoding: "json".to_string(),
            limits: WasmLimits::default(),
            expectations: Expectations {
                manifest_schema: "manifest.v1.0".to_string(),
                proof_container_version: "capz.v2".to_string(),
            },
        }
    }
}

impl ExecutorConfig {
    /// Load from executor.json file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: ExecutorConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Save to executor.json file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// Bundle Verifier Executor
pub struct BundleExecutor {
    bundle_path: String,
    config: ExecutorConfig,
    use_wasm: bool,
}

impl BundleExecutor {
    /// Create new executor for bundle
    pub fn new(bundle_path: String) -> Result<Self> {
        // Try to load executor.json
        let executor_json_path = format!("{}/executor.json", bundle_path);
        let config = if Path::new(&executor_json_path).exists() {
            ExecutorConfig::from_file(&executor_json_path)?
        } else {
            ExecutorConfig::default()
        };

        // Check if verifier.wasm exists
        let wasm_path = format!("{}/verifier.wasm", bundle_path);
        let use_wasm = Path::new(&wasm_path).exists();

        Ok(Self {
            bundle_path,
            config,
            use_wasm,
        })
    }

    /// Execute verification (WASM or native fallback)
    pub fn verify(&self, options: &VerifyOptions) -> Result<VerifyReport> {
        if self.use_wasm {
            self.verify_wasm(options)
        } else {
            self.verify_native(options)
        }
    }

    /// Verify using WASM verifier
    fn verify_wasm(&self, options: &VerifyOptions) -> Result<VerifyReport> {
        // Load files
        let manifest_path = format!("{}/manifest.json", self.bundle_path);
        let proof_path = format!("{}/proof.capz", self.bundle_path);
        let wasm_path = format!("{}/verifier.wasm", self.bundle_path);

        let manifest_bytes = std::fs::read(&manifest_path)?;
        let proof_bytes = std::fs::read(&proof_path)?;
        let options_json = serde_json::to_vec(options)?;

        // Load WASM verifier
        let verifier = WasmVerifier::from_file(&wasm_path, self.config.limits.clone())?;

        // Execute verification
        let result_json = verifier.verify(&manifest_bytes, &proof_bytes, &options_json)?;

        // Parse result
        let report: VerifyReport = serde_json::from_str(&result_json)?;
        Ok(report)
    }

    /// Verify using native verifier (fallback)
    fn verify_native(&self, options: &VerifyOptions) -> Result<VerifyReport> {
        use crate::proof::CapzContainer;

        // Load files
        let manifest_path = format!("{}/manifest.json", self.bundle_path);
        let proof_path = format!("{}/proof.capz", self.bundle_path);

        let manifest_bytes = std::fs::read(&manifest_path)?;

        // Load and decode CAPZ container
        let proof_container = CapzContainer::read_from_file(&proof_path)?;
        let proof_bytes = &proof_container.payload;

        // Parse manifest as JSON
        let manifest_json: serde_json::Value = serde_json::from_slice(&manifest_bytes)?;

        // Extract statement
        let stmt = verifier_core::extract_statement_from_manifest(&manifest_json)?;

        // Call native verifier
        let report = verifier_core::verify(&manifest_json, proof_bytes, &stmt, options)?;

        Ok(report)
    }

    /// Check if WASM verifier is available
    pub fn has_wasm(&self) -> bool {
        self.use_wasm
    }

    /// Get executor config
    pub fn config(&self) -> &ExecutorConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_config_default() {
        let config = ExecutorConfig::default();
        assert_eq!(config.abi_version, "wasm-verify.v1");
        assert_eq!(config.entry_function, "verify");
        assert_eq!(config.limits.max_memory_mb, 128);
    }

    #[test]
    fn test_executor_config_roundtrip() {
        let config = ExecutorConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: ExecutorConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.abi_version, config.abi_version);
    }

    // Full executor tests in test_verify_bundle.rs with real bundles
}
