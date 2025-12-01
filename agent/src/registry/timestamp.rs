//! Timestamp Module - RFC3161 Timestamp support
//!
//! Provides timestamp creation and verification with pluggable providers:
//! - MockRfc3161Provider (local mock for testing)
//! - RealRfc3161Provider (stub for future implementation)

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::error::Error;
use std::fs;
use std::path::Path;

/// Timestamp-Struktur (RFC3161-Mock)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timestamp {
    pub version: String,
    pub audit_tip_hex: String,
    pub created_at: String, // RFC3339
    pub tsa: String,
    pub signature: String, // Base64-encoded mock signature
    pub status: String,
}

impl Timestamp {
    /// Erstellt einen neuen Mock-Timestamp für einen Audit-Tip
    ///
    /// Delegiert an MockRfc3161Provider für konsistente Implementierung.
    pub fn create_mock(audit_tip_hex: String) -> Self {
        MockRfc3161Provider
            .create(&audit_tip_hex)
            .expect("Mock provider should never fail")
    }

    /// Lädt einen Timestamp aus einer JSON-Datei
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let content = fs::read_to_string(path)?;
        let timestamp: Timestamp = serde_json::from_str(&content)?;
        Ok(timestamp)
    }

    /// Speichert den Timestamp in eine JSON-Datei
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Verifiziert einen Timestamp gegen einen Audit-Tip
    ///
    /// Delegiert an MockRfc3161Provider für konsistente Implementierung.
    pub fn verify(&self, audit_tip_hex: &str) -> bool {
        MockRfc3161Provider
            .verify(audit_tip_hex, self)
            .unwrap_or(false)
    }
}

// ============================================================================
// Timestamp Provider Abstraction (Pluggable Interface)
// ============================================================================

/// Timestamp Provider Trait - allows pluggable timestamp sources
#[allow(dead_code)]
pub trait TimestampProvider {
    /// Creates a timestamp for the given audit tip
    fn create(&self, audit_tip_hex: &str) -> Result<Timestamp, Box<dyn Error>>;

    /// Verifies a timestamp against an audit tip
    fn verify(&self, audit_tip_hex: &str, ts: &Timestamp) -> Result<bool, Box<dyn Error>>;

    /// Returns the provider name
    fn name(&self) -> &'static str;
}

/// Provider kind selector
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ProviderKind {
    MockRfc3161,
    RealRfc3161 { tsa_url: String },
}

/// Factory function to create a timestamp provider
#[allow(dead_code)]
pub fn make_provider(kind: ProviderKind) -> Box<dyn TimestampProvider> {
    match kind {
        ProviderKind::MockRfc3161 => Box::new(MockRfc3161Provider),
        ProviderKind::RealRfc3161 { tsa_url } => Box::new(RealRfc3161Provider { tsa_url }),
    }
}

/// Helper to parse provider from CLI string
#[allow(dead_code)]
pub fn provider_from_cli(kind: &str, tsa_url: Option<String>) -> ProviderKind {
    match kind {
        "rfc3161" => ProviderKind::RealRfc3161 {
            tsa_url: tsa_url.unwrap_or_else(|| "https://freetsa.org/tsr".to_string()),
        },
        _ => ProviderKind::MockRfc3161,
    }
}

// ============================================================================
// Mock RFC3161 Provider (current behavior)
// ============================================================================

/// Mock RFC3161 Timestamp Provider
#[allow(dead_code)]
pub struct MockRfc3161Provider;

impl TimestampProvider for MockRfc3161Provider {
    fn create(&self, audit_tip_hex: &str) -> Result<Timestamp, Box<dyn Error>> {
        let now = Utc::now().to_rfc3339();
        let mut hasher = Sha3_256::new();
        hasher.update(audit_tip_hex.as_bytes());
        hasher.update(now.as_bytes());
        let sig = hasher.finalize();
        let signature = hex::encode(&sig[..]);

        Ok(Timestamp {
            version: "tsr.v1".to_string(),
            audit_tip_hex: audit_tip_hex.to_string(),
            created_at: now,
            tsa: "local-mock".to_string(),
            signature,
            status: "ok".to_string(),
        })
    }

    fn verify(&self, audit_tip_hex: &str, ts: &Timestamp) -> Result<bool, Box<dyn Error>> {
        if ts.audit_tip_hex != audit_tip_hex {
            return Ok(false);
        }

        // Verify mock signature
        let mut hasher = Sha3_256::new();
        hasher.update(ts.audit_tip_hex.as_bytes());
        hasher.update(ts.created_at.as_bytes());
        let expected_sig = hasher.finalize();
        let expected_sig_hex = hex::encode(&expected_sig[..]);

        Ok(ts.signature == expected_sig_hex && ts.status == "ok")
    }

    fn name(&self) -> &'static str {
        "mock_rfc3161"
    }
}

// ============================================================================
// Real RFC3161 Provider (stub for future implementation)
// ============================================================================

/// Real RFC3161 Timestamp Provider (not yet implemented)
#[allow(dead_code)]
pub struct RealRfc3161Provider {
    pub tsa_url: String,
}

impl TimestampProvider for RealRfc3161Provider {
    fn create(&self, _audit_tip_hex: &str) -> Result<Timestamp, Box<dyn Error>> {
        Err(format!(
            "Real RFC3161 provider not yet implemented (tsa_url={}). Use --provider mock for now.",
            self.tsa_url
        )
        .into())
    }

    fn verify(&self, _audit_tip_hex: &str, _ts: &Timestamp) -> Result<bool, Box<dyn Error>> {
        Err("Real RFC3161 provider not yet implemented. Use --provider mock for now.".into())
    }

    fn name(&self) -> &'static str {
        "real_rfc3161"
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Verifiziert einen Mock-Timestamp aus Datei
///
/// # Argumente
/// * `ts_path` - Pfad zur Timestamp-Datei
///
/// # Rückgabe
/// true wenn Timestamp-Status "ok" ist, false sonst
pub fn verify_timestamp_from_file(ts_path: &str) -> bool {
    match Timestamp::load(ts_path) {
        Ok(ts) => ts.status == "ok",
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_create_mock() {
        let tip = "0x1234567890abcdef".to_string();
        let ts = Timestamp::create_mock(tip.clone());

        assert_eq!(ts.version, "tsr.v1");
        assert_eq!(ts.audit_tip_hex, tip);
        assert_eq!(ts.tsa, "local-mock");
        assert_eq!(ts.status, "ok");
        assert!(!ts.signature.is_empty());
    }

    #[test]
    fn test_timestamp_verify_ok() {
        let tip = "0x1234567890abcdef".to_string();
        let ts = Timestamp::create_mock(tip.clone());

        assert!(ts.verify(&tip));
    }

    #[test]
    fn test_timestamp_verify_fail() {
        let tip = "0x1234567890abcdef".to_string();
        let ts = Timestamp::create_mock(tip);

        assert!(!ts.verify("0xwronghash"));
    }

    #[test]
    fn test_timestamp_save_load() {
        let tip = "0x1234567890abcdef".to_string();
        let ts = Timestamp::create_mock(tip.clone());

        let temp_path = std::env::temp_dir().join("test_timestamp_module.tsr");
        ts.save(&temp_path).unwrap();

        let loaded = Timestamp::load(&temp_path).unwrap();
        assert_eq!(loaded.audit_tip_hex, tip);
        assert!(loaded.verify(&tip));

        std::fs::remove_file(&temp_path).ok();
    }

    #[test]
    fn test_provider_from_cli() {
        let mock = provider_from_cli("mock", None);
        assert!(matches!(mock, ProviderKind::MockRfc3161));

        let real = provider_from_cli("rfc3161", Some("https://example.com".to_string()));
        assert!(matches!(real, ProviderKind::RealRfc3161 { .. }));
    }
}
