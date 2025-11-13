# Claude Task: Abstract Timestamp Interface (v1.0) — Provider Pattern

## 🎯 Goal
Introduce a pluggable **TimestampProvider** so that the current mock RFC3161 implementation keeps working,
and a future *real* RFC3161 TSA can be added **without touching the rest of the app**.

No networking required now — only interface + wiring + CLI flags.

---

## 🧱 Scope Overview

### What changes
- New trait `TimestampProvider` with two variants:
  - `mock_rfc3161` (existing behavior)
  - `real_rfc3161` (stub; not implemented yet)
- CLI flag to choose provider:
  ```bash
  cap audit timestamp --provider mock|rfc3161 --tsa-url <url>
  ```
  > For `mock`, `--tsa-url` is ignored.
- Internal refactor: existing timestamp creation/verification calls route through the provider.

---

## 🔧 Implementation

### 1) Trait & Provider Enum
**File:** `agent/src/registry.rs` (or new `agent/src/timestamp.rs` and re-export from `registry.rs`)

```rust
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Timestamp {
    pub version: String,       // "tsr.v1"
    pub audit_tip_hex: String, // 0x.. sha3 tip
    pub created_at: String,    // RFC3339
    pub tsa: String,           // "local-mock" | URL/string
    pub signature: String,     // hex(SHA3(tip + created_at)) -- for mock
    pub status: String,        // "ok" | "fail"
}

pub trait TimestampProvider {
    fn create(&self, audit_tip_hex: &str) -> Result<Timestamp>;
    fn verify(&self, audit_tip_hex: &str, ts: &Timestamp) -> Result<bool>;
    fn name(&self) -> &'static str;
}

pub enum ProviderKind {
    MockRfc3161,
    RealRfc3161 { tsa_url: String },
}
```

### 2) Provider Selector
**File:** `agent/src/registry.rs` (or `timestamp.rs`)
```rust
pub fn provider_from_cli(kind: &str, tsa_url: Option<String>) -> ProviderKind {
    match kind {
        "rfc3161" => ProviderKind::RealRfc3161 { tsa_url: tsa_url.unwrap_or_default() },
        _ => ProviderKind::MockRfc3161,
    }
}
```

### 3) Mock Implementation (current behavior extracted)
```rust
use chrono::Utc;
use sha3::{Digest, Sha3_256};

pub struct MockRfc3161;

impl TimestampProvider for MockRfc3161 {
    fn create(&self, audit_tip_hex: &str) -> Result<Timestamp> {
        let now = Utc::now().to_rfc3339();
        let mut hasher = Sha3_256::new();
        hasher.update(format!("{audit_tip_hex}{now}"));
        let sig = format!("0x{:x}", hasher.finalize());

        Ok(Timestamp {
            version: "tsr.v1".into(),
            audit_tip_hex: audit_tip_hex.into(),
            created_at: now,
            tsa: "local-mock".into(),
            signature: sig,
            status: "ok".into(),
        })
    }

    fn verify(&self, audit_tip_hex: &str, ts: &Timestamp) -> Result<bool> {
        let mut hasher = Sha3_256::new();
        hasher.update(format!("{}{}", audit_tip_hex, ts.created_at));
        let expected = format!("0x{:x}", hasher.finalize());
        Ok(ts.signature == expected && ts.status == "ok")
    }

    fn name(&self) -> &'static str { "mock_rfc3161" }
}
```

### 4) Real RFC3161 Stub (no network yet)
```rust
pub struct RealRfc3161 { pub tsa_url: String }

impl TimestampProvider for RealRfc3161 {
    fn create(&self, audit_tip_hex: &str) -> Result<Timestamp> {
        // Stub: return a placeholder error for now, or mimic mock until real networking lands.
        anyhow::bail!("real RFC3161 provider not implemented yet (tsa_url={})", self.tsa_url);
    }
    fn verify(&self, _audit_tip_hex: &str, _ts: &Timestamp) -> Result<bool> {
        anyhow::bail!("real RFC3161 provider not implemented yet");
    }
    fn name(&self) -> &'static str { "real_rfc3161" }
}
```

### 5) Factory & Routing
```rust
pub fn make_provider(kind: ProviderKind) -> Box<dyn TimestampProvider> {
    match kind {
        ProviderKind::MockRfc3161 => Box::new(MockRfc3161),
        ProviderKind::RealRfc3161 { tsa_url } => Box::new(RealRfc3161 { tsa_url }),
    }
}
```

### 6) Wire into existing CLI — `audit timestamp`
**File:** `agent/src/main.rs`

```rust
.subcommand(
  Command::new("audit")
    .about("Audit & timestamp ops")
    .subcommand(
        Command::new("timestamp")
          .about("Create timestamp from audit tip")
          .arg(arg!(--head <FILE> "Path to audit.head"))
          .arg(arg!(--provider [PROVIDER] "mock|rfc3161").required(false))
          .arg(arg!(--"tsa-url" [URL] "TSA endpoint (ignored for mock)").required(false))
          .arg(arg!(--out [FILE] "Output TSR path").required(false))
    )
)
```

**Command handler (sketch):**
```rust
pub fn cmd_audit_timestamp(head: &str, provider: &str, tsa_url: Option<String>, out: Option<&str>) -> anyhow::Result<()> {
    let audit_tip_hex = std::fs::read_to_string(head)?.trim().to_string();
    let kind = provider_from_cli(provider, tsa_url);
    let prov = make_provider(kind);
    let ts = prov.create(&audit_tip_hex)?;
    let out_path = out.unwrap_or("build/timestamp.tsr");
    std::fs::write(out_path, serde_json::to_string_pretty(&ts)?)?;
    println!("✅ timestamp created via provider: {}", prov.name());
    Ok(())
}
```

### 7) Wire into verification — `audit verify-timestamp`
Reuse chosen provider to verify:
```rust
pub fn cmd_audit_verify_timestamp(head: &str, ts_path: &str, provider: &str, tsa_url: Option<String>) -> anyhow::Result<()> {
    let audit_tip_hex = std::fs::read_to_string(head)?.trim().to_string();
    let ts: Timestamp = serde_json::from_str(&std::fs::read_to_string(ts_path)?)?;
    let prov = make_provider(provider_from_cli(provider, tsa_url));
    let ok = prov.verify(&audit_tip_hex, &ts)?;
    if ok { println!("✅ timestamp valid ({})", prov.name()); } else { println!("❌ timestamp invalid"); }
    Ok(())
}
```

---

## 🧪 Tests

**File:** `agent/tests/test_timestamp_provider.rs`
```rust
#[test]
fn mock_create_and_verify_ok() {
    let audit_tip_hex = "0x83a8779dc1f6a3b0..."; // fixture
    let prov = MockRfc3161;
    let ts = prov.create(audit_tip_hex).unwrap();
    assert_eq!(ts.status, "ok");
    assert!(prov.verify(audit_tip_hex, &ts).unwrap());
}

#[test]
fn mock_verify_fail_on_tamper() {
    let audit_tip_hex = "0x83a8779dc1f6a3b0...";
    let prov = MockRfc3161;
    let mut ts = prov.create(audit_tip_hex).unwrap();
    ts.signature = "0xdeadbeef".into();
    assert!(!prov.verify(audit_tip_hex, &ts).unwrap());
}
```

---

## 📘 README Update

```markdown
### ⏱️ Timestamp Providers

Choose a timestamp provider (default: mock):

```bash
# Create timestamp (mock)
cap audit timestamp --head build/audit.head --provider mock --out build/timestamp.tsr

# Verify timestamp (mock)
cap audit verify-timestamp --head build/audit.head --timestamp build/timestamp.tsr --provider mock

# Future (real RFC3161 provider)
cap audit timestamp --head build/audit.head --provider rfc3161 --tsa-url https://tsa.example.org/rfc3161
```
```

---

## 📦 Cargo.toml

```toml
chrono = "0.4"
sha3 = "0.10"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
# Future (real provider): reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
```

---

## ✅ Acceptance Criteria

| Criterion | Description |
|-----------|-------------|
| ✔ Trait | `TimestampProvider` defined with `create` and `verify` |
| ✔ Mock | Current behavior moved to `MockRfc3161` provider |
| ✔ CLI | `--provider mock|rfc3161` (mock works; rfc3161 returns clear TODO error) |
| ✔ Verify | `audit verify-timestamp` uses provider as well |
| ✔ Backwards | Default provider is mock; no behavior change without flags |
| ✔ Tests | Unit tests for mock provider pass |

---

## 🔭 Future
- Implement real RFC3161: DER request/response build, nonce, TSA signing cert chain verification, and signature check against `audit_tip_hex`.\n