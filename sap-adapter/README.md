# 🧩 SAP Adapter - CAP Verifier Integration

**Version:** 0.1.0 (Week 1 Skeleton)
**Status:** Functional Prototype
**Purpose:** Pull supplier data from SAP (mock), hash sensitive fields, send to CAP Verifier

---

## Features

✅ Mock SAP S/4 Supplier Data (OData/CDS style)
✅ BLAKE3 Hashing (no raw PII transmitted)
✅ DSGVO-compliant data handling
✅ Configurable output (context.json)
✅ Ready for HTTPS integration with Verifier API

---

## Quick Start

### 1. Build
```bash
cargo build --release
```

### 2. Run (Dry-Run)
```bash
cargo run -- --dry-run --output context.json
```

**Output:**
```
🧩 SAP Adapter v0.1.0 (Week 1 Skeleton)
📂 Reading: examples/suppliers.json
✅ Loaded 10 suppliers
🔐 Hashed 10 suppliers with BLAKE3
💾 Saved to: context.json
```

### 3. Inspect context.json
```bash
cat context.json | jq .
```

**Example:**
```json
{
  "suppliers": [
    {
      "id_hash": "0xd847ab7566eab1c8c77417a157648d62483de89b010e955cd9cf298297ff803b",
      "country": "DE",
      "tier": "1"
    },
    ...
  ],
  "total_count": 10
}
```

---

## CLI Options

```bash
USAGE:
    sap-adapter [OPTIONS]

OPTIONS:
    -s, --suppliers <FILE>    SAP mock data file [default: examples/suppliers.json]
    -o, --output <FILE>       Output context.json to file
        --dry-run             Don't send to verifier (just show context)
    -h, --help                Print help
```

---

## Mock SAP Data Structure

**Input:** `examples/suppliers.json`

```json
{
  "suppliers": [
    {
      "LIFNR": "100001",
      "NAME1": "Acme Steel GmbH",
      "LAND1": "DE",
      "ORT01": "Duisburg",
      "STRAS": "Werksstraße 123",
      "AUDIT_DATE": "2025-10-15",
      "TIER": "1",
      "UBO_COUNT": 2
    },
    ...
  ]
}
```

**Sensitive Fields (Hashed):**
- `LIFNR` (Supplier ID)
- `NAME1` (Supplier Name)

**Clear-Text Fields (Metadata only):**
- `LAND1` (Country)
- `TIER` (Supply Chain Tier)

---

## Data Flow

```
┌─────────────────────────────────────────────────────────┐
│ 1. SAP S/4 (Mock OData/CDS)                            │
│    examples/suppliers.json                              │
└─────────────┬───────────────────────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────────────────────┐
│ 2. SAP Adapter                                          │
│    - Load JSON                                          │
│    - BLAKE3 Hash: id_hash = BLAKE3(LIFNR + NAME1)      │
│    - Build context.json                                 │
└─────────────┬───────────────────────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────────────────────┐
│ 3. context.json                                         │
│    {                                                    │
│      "suppliers": [                                     │
│        {                                                │
│          "id_hash": "0x...",  ← BLAKE3 (no raw data)  │
│          "country": "DE",                               │
│          "tier": "1"                                    │
│        }                                                │
│      ]                                                  │
│    }                                                    │
└─────────────┬───────────────────────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────────────────────┐
│ 4. POST to Verifier API (Week 2)                       │
│    https://localhost:8443/verify                        │
│    Authorization: Bearer <token>                        │
└─────────────────────────────────────────────────────────┘
```

---

## Security & Privacy

### DSGVO Compliance
- ✅ **No Raw PII Transmitted:** Names, addresses hashed before transmission
- ✅ **BLAKE3 Cryptographic Hashing:** Collision-resistant, fast
- ✅ **Minimal Metadata:** Only country + tier in clear text (not PII)

### Hashing Strategy
```rust
fn hash_field(input: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(input.as_bytes());
    format!("0x{}", hasher.finalize().to_hex())
}

// Example:
// Input:  "100001:Acme Steel GmbH"
// Output: "0xd847ab7566eab1c8c77417a157648d62483de89b010e955cd9cf298297ff803b"
```

---

## Integration with Verifier API (Week 2)

### Planned Features
- [ ] HTTPS POST to `/verify` endpoint
- [ ] OAuth2 Bearer token authentication
- [ ] Self-signed TLS acceptance (`--accept-invalid-certs`)
- [ ] mTLS optional (`--require-mtls`)
- [ ] Response parsing (manifest_hash, proof_hash, result)

### Example (Week 2)
```bash
export VERIFIER_TOKEN="eyJ0eXAiOiJKV1QiLCJhbGc..."

cargo run -- \
  --suppliers examples/suppliers.json \
  --output context.json \
  --verifier-url https://localhost:8443 \
  --accept-invalid-certs
```

---

## Development

### Dependencies
- `blake3` - Cryptographic hashing
- `clap` - CLI argument parsing
- `serde` + `serde_json` - Serialization
- `anyhow` - Error handling

### Build
```bash
cargo build
```

### Test
```bash
cargo test
cargo run -- --dry-run
```

### Lint
```bash
cargo clippy -- -D warnings
```

---

## Week 1 Deliverables

✅ **Mock SAP Data Source** (`examples/suppliers.json`)
✅ **BLAKE3 Hashing** (no raw PII)
✅ **Context Builder** (SAP → context.json mapping)
✅ **CLI Tool** (working dry-run mode)
✅ **README** (this document)

---

## Next Steps (Week 2)

1. **HTTPS Integration**
   - Add `reqwest` client
   - POST to Verifier `/verify` endpoint
   - Parse VerifyResponse

2. **TLS Configuration**
   - Generate self-signed certs (`openssl req -new -x509`)
   - Accept invalid certs flag
   - Optional mTLS support

3. **End-to-End Testing**
   - Start Verifier API
   - Run Adapter with real HTTP call
   - Validate response

4. **CI/CD Pipeline**
   - GitHub Actions workflow
   - Trivy/Grype security scans
   - SBOM generation

---

## License

**Proprietary** - BASF/EuroDat Integration Project

---

**Contact:** CAP Team
**Last Updated:** 2025-11-07
