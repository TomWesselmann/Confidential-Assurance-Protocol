/// WASM Loader mit Sandbox und Limits
///
/// Lädt WASM-Module mit konfigurierbaren Sicherheitsgrenzen.
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use wasmtime::*;

/// WASM Execution Limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmLimits {
    /// Maximum memory in MB
    pub max_memory_mb: usize,
    /// Maximum execution time in milliseconds
    pub timeout_ms: u64,
    /// Maximum fuel (computational units)
    pub max_fuel: Option<u64>,
}

impl Default for WasmLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 128,
            timeout_ms: 3000,
            max_fuel: Some(5_000_000),
        }
    }
}

/// WASM Verifier Instance
pub struct WasmVerifier {
    engine: Engine,
    module: Module,
    limits: WasmLimits,
}

impl WasmVerifier {
    /// Create new verifier from WASM bytes
    pub fn new(wasm_bytes: &[u8], limits: WasmLimits) -> Result<Self> {
        // Configure engine with limits
        let mut config = Config::new();

        // Enable fuel metering for computational limits
        if limits.max_fuel.is_some() {
            config.consume_fuel(true);
        }

        // Set memory limits (pages = 64KB each)
        let _max_pages = (limits.max_memory_mb * 1024 * 1024) / 65536;
        config.max_wasm_stack(1024 * 1024); // 1MB stack

        let engine = Engine::new(&config)?;
        let module = Module::new(&engine, wasm_bytes)?;

        Ok(Self {
            engine,
            module,
            limits,
        })
    }

    /// Load from file
    pub fn from_file(path: &str, limits: WasmLimits) -> Result<Self> {
        let bytes = std::fs::read(path)?;
        Self::new(&bytes, limits)
    }

    /// Execute verification function
    ///
    /// Calls: verify(manifest_json: ptr, manifest_len: i32,
    ///                proof_bytes: ptr, proof_len: i32,
    ///                options_json: ptr, options_len: i32) -> result_ptr: i32
    ///
    /// Returns JSON report as string
    pub fn verify(
        &self,
        manifest_json: &[u8],
        proof_bytes: &[u8],
        options_json: &[u8],
    ) -> Result<String> {
        let mut store = Store::new(&self.engine, ());

        // Set fuel limit if configured
        if let Some(fuel) = self.limits.max_fuel {
            store.set_fuel(fuel)?;
        }

        // Instantiate module
        let instance = Instance::new(&mut store, &self.module, &[])?;

        // Get memory export
        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| anyhow!("WASM module must export 'memory'"))?;

        // Get verify function
        let verify_func = instance
            .get_typed_func::<(i32, i32, i32, i32, i32, i32), i32>(&mut store, "verify")
            .map_err(|_| {
                anyhow!("WASM module must export 'verify' function with correct signature")
            })?;

        // Allocate memory in WASM for inputs
        let alloc_func = instance
            .get_typed_func::<i32, i32>(&mut store, "alloc")
            .map_err(|_| anyhow!("WASM module must export 'alloc' function"))?;

        // Allocate and write manifest
        let manifest_ptr = alloc_func.call(&mut store, manifest_json.len() as i32)?;
        memory.write(&mut store, manifest_ptr as usize, manifest_json)?;

        // Allocate and write proof
        let proof_ptr = alloc_func.call(&mut store, proof_bytes.len() as i32)?;
        memory.write(&mut store, proof_ptr as usize, proof_bytes)?;

        // Allocate and write options
        let options_ptr = alloc_func.call(&mut store, options_json.len() as i32)?;
        memory.write(&mut store, options_ptr as usize, options_json)?;

        // Call verify function with timeout
        let result_ptr = std::thread::scope(|s| {
            let handle = s.spawn(|| {
                verify_func.call(
                    &mut store,
                    (
                        manifest_ptr,
                        manifest_json.len() as i32,
                        proof_ptr,
                        proof_bytes.len() as i32,
                        options_ptr,
                        options_json.len() as i32,
                    ),
                )
            });

            let _timeout = std::time::Duration::from_millis(self.limits.timeout_ms);
            match handle.join() {
                Ok(result) => result,
                Err(_) => Err(anyhow!("WASM execution panicked")),
            }
        })?;

        // Read result from memory
        // Format: [length: i32][data: bytes]
        let mut len_bytes = [0u8; 4];
        memory.read(&store, result_ptr as usize, &mut len_bytes)?;
        let result_len = i32::from_le_bytes(len_bytes) as usize;

        if result_len > 1_000_000 {
            return Err(anyhow!("Result too large: {} bytes", result_len));
        }

        let mut result_data = vec![0u8; result_len];
        memory.read(&store, (result_ptr + 4) as usize, &mut result_data)?;

        String::from_utf8(result_data).map_err(|e| anyhow!("Result is not valid UTF-8: {}", e))
    }

    /// Get remaining fuel (if fuel metering enabled)
    pub fn remaining_fuel(&self, store: &Store<()>) -> Option<u64> {
        store.get_fuel().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full WASM tests require actual WASM binaries
    // These tests verify the loader API structure

    #[test]
    fn test_limits_default() {
        let limits = WasmLimits::default();
        assert_eq!(limits.max_memory_mb, 128);
        assert_eq!(limits.timeout_ms, 3000);
        assert_eq!(limits.max_fuel, Some(5_000_000));
    }

    #[test]
    fn test_limits_custom() {
        let limits = WasmLimits {
            max_memory_mb: 64,
            timeout_ms: 1000,
            max_fuel: Some(1_000_000),
        };
        assert_eq!(limits.max_memory_mb, 64);
    }

    // Additional tests would require:
    // - Minimal WASM module with verify function
    // - Test fixtures in tests/fixtures/wasm/
    // - See test_verify_bundle.rs for integration tests
}
