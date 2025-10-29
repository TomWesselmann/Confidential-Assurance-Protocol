# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**LkSG Proof Agent** - A local CLI tool for generating cryptographically verifiable commitments (Merkle roots) for supply chain and sanctions compliance checks. Part of the Confidential Assurance Protocol (CAP).

**Core Purpose:** Process company data (CSV/JSON) completely offline and generate cryptographic fingerprints (Merkle roots) as foundation for Zero-Knowledge proofs and auditable manifests.

## Build & Test Commands

```bash
# Build the project
cd agent && cargo build

# Run all tests
cargo test

# Run linter (must pass with zero warnings)
cargo clippy -- -D warnings

# Run the tool
cargo run -- <command> [options]
```

## CLI Commands

```bash
# Generate commitments from CSV files
cargo run -- prepare --suppliers ../examples/suppliers.csv --ubos ../examples/ubos.csv

# Inspect generated commitments
cargo run -- inspect build/commitments.json

# Show version
cargo run -- version
```

## Architecture

### Module Structure

- **main.rs**: CLI entry point with clap-based command parsing (prepare, inspect, version)
- **io.rs**: CSV/JSON parsing for Supplier and UBO data structures
- **commitment.rs**: BLAKE3-based Merkle root computation engine
- **audit.rs**: SHA3-256 hash-chain audit logging system (JSONL format)

### Data Flow

1. CSV input (suppliers + UBOs) → parsed by `io.rs`
2. Individual records hashed with BLAKE3 → `commitment.rs::hash_record()`
3. Hashes combined into Merkle roots → `commitment.rs::compute_merkle_root()`
4. Company root computed from supplier + UBO roots → `commitment.rs::compute_company_root()`
5. All operations logged to audit trail → `audit.rs::AuditLog`
6. Results saved to `build/commitments.json` + `build/agent.audit.jsonl`

### Key Technical Constraints

- **Offline-only**: NO network access permitted
- **Deterministic**: Same input MUST produce identical roots (critical for reproducibility)
- **Hash-chain integrity**: Every audit log entry cryptographically links to previous entry
- **No warnings**: Code must pass `cargo clippy -- -D warnings`

### Cryptographic Specifications

- **Commitment hashing**: BLAKE3
- **Audit log hashing**: SHA3-256
- **Output format**: Hex strings prefixed with `0x`
- **Timestamp format**: RFC3339 (UTC)

## Output Files

- `build/commitments.json`: Contains supplier_root, ubo_root, company_commitment_root
- `build/agent.audit.jsonl`: JSONL file with hash-chained audit events (seq, ts, event, details, prev_digest, digest)

## Testing Philosophy

- Unit tests in each module verify core functionality (hashing, determinism, chain integrity)
- Determinism test: Delete `build/` and re-run `prepare` → roots must match exactly
- Audit log must form valid hash chain: each entry's prev_digest must match previous entry's digest

## Development Notes

- All code includes German comments explaining functionality (per project requirement)
- Merkle tree implementation is simplified (concatenated hashing) - production version would use proper tree structure
- `current_seq()` method in `audit.rs` marked `#[allow(dead_code)]` as it's used only in tests
