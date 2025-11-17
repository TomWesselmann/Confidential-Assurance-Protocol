/// Mapper: SAP Business Partner → CAP Context
/// Deterministic transformation with BLAKE3 hashing
/// Follows CAP Engineering Guide: "Mapping-Funktion schreiben"

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use crate::sanitizer::SanitizedBusinessPartner;

/// CAP context supplier entry (hashed)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapSupplier {
    /// BLAKE3 hash of (BusinessPartner ID + Name)
    pub id_hash: String,

    /// ISO 3166-1 alpha-2 country code (clear text for policy evaluation)
    pub country: String,

    /// Supply chain tier: "1", "2", or "3" (clear text for policy evaluation)
    pub tier: String,

    /// Optional: Last audit date (ISO 8601 YYYY-MM-DD)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audit_date: Option<String>,

    /// Optional: Number of Ultimate Beneficial Owners
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ubo_count: Option<u32>,
}

/// CAP context structure for Verifier API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapContext {
    /// Schema version for backward compatibility
    pub schema_version: String,

    /// List of hashed suppliers
    pub suppliers: Vec<CapSupplier>,

    /// Total supplier count (for quick validation)
    pub total_count: usize,

    /// Metadata: Data source and extraction timestamp
    pub metadata: CapMetadata,
}

/// Metadata for CAP context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapMetadata {
    pub source: String,
    pub extracted_at: String,
    pub adapter_version: String,
}

/// Hash a business partner deterministically
///
/// # Arguments
/// * `id` - Business Partner ID (sanitized, uppercase)
/// * `name` - Business Partner Name (sanitized, trimmed)
///
/// # Returns
/// Hex-encoded BLAKE3 hash with "0x" prefix
///
/// # Determinism Guarantee
/// Same inputs always produce the same hash (critical for auditability)
fn hash_business_partner(id: &str, name: &str) -> String {
    let input = format!("{}:{}", id, name);
    let mut hasher = Hasher::new();
    hasher.update(input.as_bytes());
    format!("0x{}", hasher.finalize().to_hex())
}

/// Map a single sanitized business partner to CAP supplier
///
/// # Arguments
/// * `bp` - Sanitized business partner from SAP
///
/// # Returns
/// CAP supplier with hashed ID and clear-text metadata
pub fn map_to_cap_supplier(bp: &SanitizedBusinessPartner) -> CapSupplier {
    CapSupplier {
        id_hash: hash_business_partner(&bp.id, &bp.name),
        country: bp.country.clone(),
        tier: bp.tier.clone(),
        audit_date: bp.audit_date.clone(),
        ubo_count: Some(bp.ubo_count),
    }
}

/// Map a batch of sanitized business partners to CAP context
///
/// # Arguments
/// * `partners` - Slice of sanitized business partners (already sorted by ID)
/// * `source` - Data source identifier (e.g., "SAP S/4HANA Production")
/// * `extracted_at` - ISO 8601 timestamp (e.g., "2025-11-17T10:30:00Z")
///
/// # Returns
/// Complete CAP context ready for Verifier API
///
/// # Determinism Guarantee
/// - Input partners MUST be sorted by ID (enforced by sanitizer)
/// - Hash function is deterministic (BLAKE3)
/// - Result is always identical for same inputs
pub fn map_to_cap_context(
    partners: &[SanitizedBusinessPartner],
    source: &str,
    extracted_at: &str,
) -> CapContext {
    let suppliers: Vec<CapSupplier> = partners
        .iter()
        .map(map_to_cap_supplier)
        .collect();

    CapContext {
        schema_version: "v1".to_string(),
        suppliers,
        total_count: partners.len(),
        metadata: CapMetadata {
            source: source.to_string(),
            extracted_at: extracted_at.to_string(),
            adapter_version: env!("CARGO_PKG_VERSION").to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_sanitized_partner(id: &str, name: &str, country: &str, tier: &str) -> SanitizedBusinessPartner {
        SanitizedBusinessPartner {
            id: id.to_string(),
            name: name.to_string(),
            country: country.to_string(),
            city: None,
            street: None,
            audit_date: Some("2025-11-17".to_string()),
            tier: tier.to_string(),
            ubo_count: 2,
        }
    }

    #[test]
    fn test_hash_business_partner_deterministic() {
        let hash1 = hash_business_partner("100001", "Acme GmbH");
        let hash2 = hash_business_partner("100001", "Acme GmbH");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_business_partner_unique() {
        let hash1 = hash_business_partner("100001", "Acme GmbH");
        let hash2 = hash_business_partner("100002", "Beta AG");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_business_partner_format() {
        let hash = hash_business_partner("100001", "Acme GmbH");
        assert!(hash.starts_with("0x"));
        assert_eq!(hash.len(), 66); // "0x" + 64 hex chars (BLAKE3 256-bit)
    }

    #[test]
    fn test_hash_business_partner_known_value() {
        // Regression test: Ensure hash remains stable across code changes
        let hash = hash_business_partner("100001", "Acme Steel GmbH");
        assert_eq!(hash, "0xd847ab7566eab1c8c77417a157648d62483de89b010e955cd9cf298297ff803b");
    }

    #[test]
    fn test_map_to_cap_supplier() {
        let bp = mock_sanitized_partner("100001", "Acme GmbH", "DE", "1");
        let supplier = map_to_cap_supplier(&bp);

        assert!(supplier.id_hash.starts_with("0x"));
        assert_eq!(supplier.country, "DE");
        assert_eq!(supplier.tier, "1");
        assert_eq!(supplier.audit_date, Some("2025-11-17".to_string()));
        assert_eq!(supplier.ubo_count, Some(2));
    }

    #[test]
    fn test_map_to_cap_context() {
        let partners = vec![
            mock_sanitized_partner("100001", "Acme GmbH", "DE", "1"),
            mock_sanitized_partner("100002", "Beta AG", "FR", "2"),
        ];

        let context = map_to_cap_context(
            &partners,
            "SAP S/4HANA Test",
            "2025-11-17T10:30:00Z",
        );

        assert_eq!(context.schema_version, "v1");
        assert_eq!(context.total_count, 2);
        assert_eq!(context.suppliers.len(), 2);
        assert_eq!(context.metadata.source, "SAP S/4HANA Test");
        assert_eq!(context.metadata.extracted_at, "2025-11-17T10:30:00Z");
    }

    #[test]
    fn test_map_to_cap_context_deterministic_ordering() {
        let partners = vec![
            mock_sanitized_partner("100001", "A GmbH", "DE", "1"),
            mock_sanitized_partner("100002", "B AG", "FR", "2"),
            mock_sanitized_partner("100003", "C Ltd", "GB", "3"),
        ];

        let context1 = map_to_cap_context(&partners, "Test", "2025-11-17T10:00:00Z");
        let context2 = map_to_cap_context(&partners, "Test", "2025-11-17T10:00:00Z");

        // Hashes must be identical (determinism)
        assert_eq!(
            context1.suppliers[0].id_hash,
            context2.suppliers[0].id_hash
        );
        assert_eq!(
            context1.suppliers[1].id_hash,
            context2.suppliers[1].id_hash
        );
        assert_eq!(
            context1.suppliers[2].id_hash,
            context2.suppliers[2].id_hash
        );
    }

    #[test]
    fn test_cap_context_json_serialization() {
        let partners = vec![mock_sanitized_partner("100001", "Acme GmbH", "DE", "1")];
        let context = map_to_cap_context(&partners, "Test", "2025-11-17T10:00:00Z");

        let json = serde_json::to_string_pretty(&context).unwrap();

        assert!(json.contains("\"schema_version\""));
        assert!(json.contains("\"suppliers\""));
        assert!(json.contains("\"total_count\""));
        assert!(json.contains("\"metadata\""));
        assert!(json.contains("\"id_hash\""));
        assert!(json.contains("0x"));
    }

    #[test]
    fn test_cap_context_json_roundtrip() {
        let partners = vec![mock_sanitized_partner("100001", "Acme GmbH", "DE", "1")];
        let context = map_to_cap_context(&partners, "Test", "2025-11-17T10:00:00Z");

        let json = serde_json::to_string(&context).unwrap();
        let deserialized: CapContext = serde_json::from_str(&json).unwrap();

        assert_eq!(context.total_count, deserialized.total_count);
        assert_eq!(
            context.suppliers[0].id_hash,
            deserialized.suppliers[0].id_hash
        );
    }
}
