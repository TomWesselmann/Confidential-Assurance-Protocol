# CAP Proof Bundle v2 Specification

## Version: cap-proof.v2.0

**Status:** Implemented (v0.9.0)
**Date:** 2025-10-30

---

## Overview

Bundle v2 ist ein self-contained Proof-Package-Format, das neben Manifest und Proof auch einen WASM-Verifier und Executor-Konfiguration enthält. Dies ermöglicht offline-verif

ication ohne externe Software-Abhängigkeiten.

## Bundle Structure

```
cap-proof-v2/
├─ manifest.json          # Compliance manifest (manifest.v1.0)
├─ proof.capz             # Proof container (CAPZ v2 format)
├─ verifier.wasm          # WASM verifier (optional)
├─ executor.json          # Executor ABI config (optional)
├─ timestamp.tsr          # Timestamp (optional)
├─ registry.json          # Registry (optional)
├─ _meta.json             # Bundle metadata + hashes
└─ README.txt             # Human-readable instructions (optional)
```

---

## File Specifications

### `_meta.json` (Required)

Bundle metadata with SHA3-256 hashes for integrity verification.

```json
{
  "bundle_version": "cap-proof.v2.0",
  "created_at": "2025-10-30T12:00:00Z",
  "hashes": {
    "manifest_sha3": "0x...",
    "proof_sha3": "0x...",
    "verifier_wasm_sha3": "0x...",
    "executor_json_sha3": "0x..."
  },
  "backend": "mock|halo2|zkvm",
  "vk_hash": "0x...",
  "params_hash": "0x..."
}
```

### `proof.capz` (Required)

Binary proof container with versioned header.

**Header Format (78 bytes, little endian):**
```
magic[4]      = b"CAPZ"
version[2]    = 0x0002 (u16 LE)
backend[1]    = 0=mock, 1=zkvm, 2=halo2
reserved[1]   = 0x00
vk_hash[32]   = verification key hash (zeros if N/A)
params_hash[32] = params hash (zeros if N/A)
payload_len[4]  = u32 LE
payload[payload_len] = proof data (JSON or binary)
```

**Constraints:**
- Max payload size: 100 MB
- Backend values: 0 (Mock), 1 (ZkVm), 2 (Halo2)

### `executor.json` (Optional)

WASM executor configuration.

```json
{
  "abi_version": "wasm-verify.v1",
  "entry_function": "verify",
  "input_encoding": "json",
  "output_encoding": "json",
  "limits": {
    "max_memory_mb": 128,
    "max_fuel": 5000000,
    "timeout_ms": 3000
  },
  "expectations": {
    "manifest_schema": "manifest.v1.0",
    "proof_container_version": "capz.v2"
  }
}
```

### `verifier.wasm` (Optional)

WASM module with verification logic.

**Required exports:**
- `memory`: Linear memory for data exchange
- `alloc(size: i32) -> ptr: i32`: Allocate memory
- `verify(manifest_ptr: i32, manifest_len: i32,
         proof_ptr: i32, proof_len: i32,
         options_ptr: i32, options_len: i32) -> result_ptr: i32`

**Input/Output:**
- Inputs: JSON-encoded bytes
- Output: `[length: i32][data: bytes]` where data is JSON VerifyReport

**Sandbox Constraints:**
- No file system access
- No network access
- Memory limit: 128 MB (default)
- Execution timeout: 3 seconds (default)

---

## CLI Commands

### Create Bundle v2

```bash
cap-agent bundle-v2 \
  --manifest build/manifest.json \
  --proof build/proof.capz \
  [--verifier-wasm build/verifier.wasm] \
  --out build/cap-proof-v2 \
  [--zip] \
  [--force]
```

**Behavior:**
- Copies manifest and proof to output directory
- Copies WASM verifier if provided
- Creates executor.json with default config
- Generates _meta.json with SHA3-256 hashes
- Optional: Creates ZIP archive (not yet implemented)

### Verify Bundle

```bash
cap-agent verify-bundle \
  --bundle build/cap-proof-v2 \
  [--out build/verification.report.json]
```

**Behavior:**
1. Loads bundle metadata
2. Checks for verifier.wasm (uses native fallback if missing)
3. Executes verification (WASM or native)
4. Generates VerifyReport
5. Saves report if --out specified

**Fallback:**
- If no verifier.wasm: uses native `verifier::core::verify()`
- No breaking changes for v1 bundles

---

## Verification Report

```json
{
  "status": "ok|fail",
  "manifest_hash": "0x...",
  "proof_hash": "0x...",
  "signature_valid": true,
  "timestamp_valid": null,
  "registry_match": null,
  "details": {
    "checks_passed": 2,
    "checks_total": 2
  }
}
```

---

## Security Considerations

### WASM Sandbox

- **No I/O:** WASM modules cannot access file system or network
- **Memory Limits:** Configurable max memory (default 128 MB)
- **Execution Limits:** Timeout (default 3s) and fuel metering
- **Pure Functions:** All inputs via memory, no side effects

### Integrity

- **Hash Verification:** All files hashed with SHA3-256 in _meta.json
- **Tamper Detection:** Mismatched hashes → verification failure
- **Version Checks:** CAPZ header validates version compatibility

---

## Backward Compatibility

- **v1 Bundles:** Can be verified with native fallback
- **Optional WASM:** Bundles without verifier.wasm use native verifier
- **Additive Schema:** New fields don't break old parsers

---

## Implementation Status

- ✅ CAPZ container format (proof/capz.rs)
- ✅ WASM loader with sandbox (wasm/loader.rs)
- ✅ Bundle executor (wasm/executor.rs)
- ✅ CLI commands (bundle-v2, verify-bundle)
- ⏳ ZIP archive creation (planned)
- ⏳ Full WASM verifier implementation (planned)

---

## Future Extensions

1. **Multi-Verifier Support:** Multiple WASM verifiers in one bundle
2. **Proof Composition:** Nested proofs with sub-verifiers
3. **Remote Registry:** HTTPS registry endpoint support
4. **Streaming Verification:** Large proof handling
5. **Signature Chains:** Multi-party verification workflows

---

## References

- Manifest Schema: docs/manifest.schema.json
- CAPZ Format: src/proof/capz.rs
- WASM ABI: src/wasm/executor.rs
- Verifier Core: src/verifier/core.rs
