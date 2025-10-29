# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**LkSG Proof Agent** (Version 0.2.0) - A local CLI tool for generating cryptographically verifiable commitments (Merkle roots) for supply chain and sanctions compliance checks. Part of the Confidential Assurance Protocol (CAP).

**Core Purpose:** Process company data (CSV/JSON) completely offline and generate cryptographic fingerprints (Merkle roots) as foundation for Zero-Knowledge proofs, policy validation, manifests, and auditable compliance documentation.

## Version History

- **Tag 1 (v0.1.0)**: Core MVP - Commitments, Merkle Roots, Audit Log
- **Tag 2 (v0.2.0)**: Policy Layer - Policy validation, Manifest building, Mock proofs, Ed25519 signing

## Build & Test Commands

```bash
# Build the project
cd agent && cargo build

# Run all tests (20 unit tests)
cargo test

# Run linter (must pass with zero warnings)
cargo clippy -- -D warnings

# Run the tool
cargo run -- <command> [options]
```

## CLI Commands

### Tag 1 Commands (Commitments)

```bash
# Generate commitments from CSV files
cargo run -- prepare --suppliers ../examples/suppliers.csv --ubos ../examples/ubos.csv

# Inspect generated commitments
cargo run -- inspect build/commitments.json

# Show version
cargo run -- version
```

### Tag 2 Commands (Policy & Manifest)

```bash
# Validate a policy file
cargo run -- policy validate --file ../examples/policy.lksg.v1.yml

# Build manifest from commitments and policy
cargo run -- manifest build --policy ../examples/policy.lksg.v1.yml

# Generate mock proof
cargo run -- proof mock --policy ../examples/policy.lksg.v1.yml --manifest build/manifest.json

# Verify mock proof
cargo run -- proof verify --proof build/proof.mock.json

# Generate Ed25519 keypair
cargo run -- sign keygen

# Sign manifest
cargo run -- sign manifest --key keys/company.ed25519 --manifest-in build/manifest.json --out build/manifest.signed.json

# Verify signed manifest
cargo run -- sign verify-manifest --pub-key keys/company.pub --signed-in build/manifest.signed.json
```

## Architecture

### Module Structure (Tag 2)

- **main.rs**: CLI entry point with clap-based command parsing (prepare, inspect, policy, manifest, proof, sign, version)
- **io.rs**: CSV/JSON parsing for Supplier and UBO data structures
- **commitment.rs**: BLAKE3-based Merkle root computation engine
- **audit.rs**: SHA3-256 hash-chain audit logging system (JSONL format)
- **policy.rs**: Policy loader, validator, and hash computation (YAML/JSON support)
- **manifest.rs**: Manifest builder combining commitments, policy, and audit information
- **proof_mock.rs**: Mock proof generation and verification (placeholder for ZKP)
- **sign.rs**: Ed25519 key generation, signing, and verification

### Data Flow (Complete Pipeline)

```
1. CSV Input (suppliers + UBOs)
   ↓
2. io.rs → Parse to Supplier/UBO structs
   ↓
3. commitment.rs → BLAKE3 hashing → Merkle roots
   ↓
4. audit.rs → Log all operations with SHA3-256 chain
   ↓
5. policy.rs → Load & validate policy (YAML/JSON)
   ↓
6. manifest.rs → Build manifest (commitments + policy + audit)
   ↓
7. proof_mock.rs → Generate mock proof (policy checks)
   ↓
8. sign.rs → Ed25519 signature over manifest
   ↓
9. Output: commitments.json, manifest.json, proof.mock.json, manifest.signed.json
```

### Key Technical Constraints

- **Offline-only**: NO network access permitted
- **Deterministic**: Same input MUST produce identical roots (critical for reproducibility)
- **Hash-chain integrity**: Every audit log entry cryptographically links to previous entry
- **No warnings**: Code must pass `cargo clippy -- -D warnings`
- **Policy validation**: Schema + semantic checks for LkSG compliance rules

### Cryptographic Specifications

| Component | Algorithm | Format | Usage |
|-----------|-----------|--------|-------|
| **Commitment hashing** | BLAKE3 | `0x<64 chars>` | Merkle roots, Company root |
| **Audit log hashing** | SHA3-256 | `0x<64 chars>` | Hash-chain integrity |
| **Policy hashing** | SHA3-256 | `0x<64 chars>` | Policy fingerprint |
| **Signing** | Ed25519 | `0x<128 chars>` | Manifest signatures |
| **Timestamps** | RFC3339 (UTC) | ISO 8601 | All events |

## Output Files

### Tag 1 Outputs
- `build/commitments.json`: Contains supplier_root, ubo_root, company_commitment_root
- `build/agent.audit.jsonl`: JSONL file with hash-chained audit events

### Tag 2 Outputs
- `build/manifest.json`: Manifest with commitments, policy, audit info, proof status
- `build/proof.mock.json`: Mock proof with policy checks (type: "mock", status: "ok"/"failed")
- `build/manifest.signed.json`: Signed manifest with Ed25519 signature
- `keys/company.ed25519`: Private signing key (32 bytes)
- `keys/company.pub`: Public verification key (32 bytes)

## Policy Schema (LkSG v1)

```yaml
version: "lksg.v1"
name: "Policy Name"
created_at: "2025-10-25T09:00:00Z"
constraints:
  require_at_least_one_ubo: true
  supplier_count_max: 10
notes: "Optional description"
```

## Manifest Structure

```json
{
  "version": "manifest.v0",
  "created_at": "2025-10-25T10:00:00Z",
  "supplier_root": "0x...",
  "ubo_root": "0x...",
  "company_commitment_root": "0x...",
  "policy": {
    "name": "LkSG-Demo",
    "version": "lksg.v1",
    "hash": "0x..."
  },
  "audit": {
    "tail_digest": "0x...",
    "events_count": 9
  },
  "proof": {
    "type": "none",
    "status": "none"
  },
  "signatures": []
}
```

## Testing Philosophy

- **Unit tests (20 total)**: Each module verifies core functionality (hashing, determinism, chain integrity, policy validation, signing)
- **Determinism test**: Delete `build/` and re-run `prepare` → roots must match exactly
- **Audit log**: Must form valid hash chain (each entry's prev_digest matches previous entry's digest)
- **Policy validation**: Invalid policies must be rejected
- **Signature verification**: Wrong keys must fail verification

## Audit Log Events (Tag 2)

New events added in Tag 2:
- `policy_loaded`: Policy file loaded
- `policy_validated`: Policy passed validation
- `manifest_built`: Manifest created
- `mock_proof_generated`: Mock proof generated
- `mock_proof_verified`: Mock proof verified
- `manifest_signed`: Manifest signed with Ed25519

## Development Notes

- All code includes German comments explaining functionality (per project requirement)
- Merkle tree implementation is simplified (concatenated hashing) - production version would use proper tree structure
- Mock proof is NOT a real Zero-Knowledge proof - it's a structured placeholder for demonstration
- Ed25519 keys stored as raw bytes (32 bytes private, 32 bytes public)
- Policy hash computed over canonical JSON representation (deterministic)
- `current_seq()` in audit.rs and `update_proof()` in manifest.rs marked `#[allow(dead_code)]` as used only in tests

## Complete Workflow Example

```bash
# 1. Generate commitments
cargo run -- prepare --suppliers ../examples/suppliers.csv --ubos ../examples/ubos.csv

# 2. Validate policy
cargo run -- policy validate --file ../examples/policy.lksg.v1.yml

# 3. Build manifest
cargo run -- manifest build --policy ../examples/policy.lksg.v1.yml

# 4. Generate and verify proof
cargo run -- proof mock --policy ../examples/policy.lksg.v1.yml --manifest build/manifest.json
cargo run -- proof verify --proof build/proof.mock.json

# 5. Sign and verify manifest
cargo run -- sign keygen
cargo run -- sign manifest --key keys/company.ed25519 --manifest-in build/manifest.json --out build/manifest.signed.json
cargo run -- sign verify-manifest --pub-key keys/company.pub --signed-in build/manifest.signed.json
```

## Future Extensions (Tag 3+)

According to the roadmap:
- Real Zero-Knowledge proof integration (replacing mock)
- Advanced policy rules and validators
- Multi-signature support
- Blockchain anchoring (optional)
- REST API for external integration
