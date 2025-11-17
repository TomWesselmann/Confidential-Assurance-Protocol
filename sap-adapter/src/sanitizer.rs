/// Sanitizer for SAP Business Partner data
/// Validates, cleanses, and normalizes input before processing
/// Follows CAP Engineering Guide: "Sanitizer definieren"

use anyhow::{Context, Result};
use crate::odata_client::SapBusinessPartner;

/// Validation errors for business partner data
#[derive(Debug, thiserror::Error)]
pub enum SanitizationError {
    #[error("Business partner ID is empty")]
    EmptyBusinessPartnerId,

    #[error("Business partner name is empty")]
    EmptyBusinessPartnerName,

    #[error("Country code is invalid: {0}")]
    InvalidCountryCode(String),

    #[error("Tier value is invalid: {0} (must be 1, 2, or 3)")]
    InvalidTier(String),

    #[error("Audit date format is invalid: {0} (expected ISO 8601 YYYY-MM-DD)")]
    InvalidAuditDate(String),

    #[error("UBO count is invalid: {0} (must be 0-999)")]
    InvalidUboCount(u32),
}

/// Sanitized and validated business partner
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SanitizedBusinessPartner {
    pub id: String,
    pub name: String,
    pub country: String,
    pub city: Option<String>,
    pub street: Option<String>,
    pub audit_date: Option<String>,
    pub tier: String,
    pub ubo_count: u32,
}

/// Sanitize a single business partner
///
/// # Validation Rules
/// - Business Partner ID: Non-empty, trimmed, uppercase
/// - Name: Non-empty, trimmed
/// - Country: Valid ISO 3166-1 alpha-2 code (2 chars, uppercase)
/// - Tier: Must be "1", "2", or "3" (defaults to "3" if missing)
/// - Audit Date: ISO 8601 format YYYY-MM-DD (optional)
/// - UBO Count: 0-999 (defaults to 0 if missing)
///
/// # Errors
/// Returns `SanitizationError` if validation fails
pub fn sanitize_business_partner(bp: &SapBusinessPartner) -> Result<SanitizedBusinessPartner> {
    // 1. Validate Business Partner ID
    let id = bp.business_partner.trim().to_uppercase();
    if id.is_empty() {
        return Err(SanitizationError::EmptyBusinessPartnerId.into());
    }

    // 2. Validate Name
    let name = bp.name.trim();
    if name.is_empty() {
        return Err(SanitizationError::EmptyBusinessPartnerName.into());
    }

    // 3. Validate Country Code (ISO 3166-1 alpha-2)
    let country = bp.country.trim().to_uppercase();
    if country.len() != 2 || !country.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(SanitizationError::InvalidCountryCode(bp.country.clone()).into());
    }

    // 4. Sanitize optional City
    let city = bp.city.as_ref().map(|s| s.trim().to_string()).filter(|s| !s.is_empty());

    // 5. Sanitize optional Street
    let street = bp.street.as_ref().map(|s| s.trim().to_string()).filter(|s| !s.is_empty());

    // 6. Validate Tier (1, 2, or 3)
    let tier = match bp.tier.as_deref() {
        Some("1") | Some("2") | Some("3") => bp.tier.clone().unwrap_or_else(|| "3".to_string()),
        Some(invalid) => return Err(SanitizationError::InvalidTier(invalid.to_string()).into()),
        None => "3".to_string(), // Default to tier 3 if missing
    };

    // 7. Validate Audit Date (ISO 8601: YYYY-MM-DD)
    let audit_date = match &bp.audit_date {
        Some(date) => {
            let trimmed = date.trim();
            if !trimmed.is_empty() {
                validate_iso_date(trimmed)
                    .with_context(|| SanitizationError::InvalidAuditDate(date.clone()))?;
                Some(trimmed.to_string())
            } else {
                None
            }
        }
        None => None,
    };

    // 8. Validate UBO Count (0-999)
    let ubo_count = bp.ubo_count.unwrap_or(0);
    if ubo_count > 999 {
        return Err(SanitizationError::InvalidUboCount(ubo_count).into());
    }

    Ok(SanitizedBusinessPartner {
        id,
        name: name.to_string(),
        country,
        city,
        street,
        audit_date,
        tier,
        ubo_count,
    })
}

/// Validate ISO 8601 date format (YYYY-MM-DD)
///
/// # Returns
/// Ok(()) if valid, Err otherwise
fn validate_iso_date(date: &str) -> Result<()> {
    // Simple validation: YYYY-MM-DD format
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        anyhow::bail!("Expected format YYYY-MM-DD, got: {}", date);
    }

    let year: u32 = parts[0].parse().context("Invalid year")?;
    let month: u32 = parts[1].parse().context("Invalid month")?;
    let day: u32 = parts[2].parse().context("Invalid day")?;

    if !(1900..=2100).contains(&year) {
        anyhow::bail!("Year out of range: {}", year);
    }

    if !(1..=12).contains(&month) {
        anyhow::bail!("Month out of range: {}", month);
    }

    if !(1..=31).contains(&day) {
        anyhow::bail!("Day out of range: {}", day);
    }

    Ok(())
}

/// Sanitize a batch of business partners
///
/// # Arguments
/// * `partners` - Slice of raw SAP business partners
///
/// # Returns
/// Tuple of (sanitized_partners, errors)
/// - Sanitized partners are sorted by ID (deterministic)
/// - Errors contain partner ID and error message for failed validations
pub fn sanitize_batch(
    partners: &[SapBusinessPartner],
) -> (Vec<SanitizedBusinessPartner>, Vec<(String, String)>) {
    let mut sanitized = Vec::with_capacity(partners.len());
    let mut errors = Vec::new();

    for bp in partners {
        match sanitize_business_partner(bp) {
            Ok(sanitized_bp) => sanitized.push(sanitized_bp),
            Err(e) => {
                errors.push((bp.business_partner.clone(), e.to_string()));
            }
        }
    }

    // Ensure deterministic ordering
    sanitized.sort_by(|a, b| a.id.cmp(&b.id));

    (sanitized, errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_business_partner(id: &str, name: &str, country: &str) -> SapBusinessPartner {
        SapBusinessPartner {
            business_partner: id.to_string(),
            name: name.to_string(),
            country: country.to_string(),
            city: None,
            street: None,
            audit_date: None,
            tier: Some("1".to_string()),
            ubo_count: Some(0),
        }
    }

    #[test]
    fn test_sanitize_valid_business_partner() {
        let bp = mock_business_partner("100001", "Acme GmbH", "DE");
        let result = sanitize_business_partner(&bp);
        assert!(result.is_ok());

        let sanitized = result.unwrap();
        assert_eq!(sanitized.id, "100001");
        assert_eq!(sanitized.name, "Acme GmbH");
        assert_eq!(sanitized.country, "DE");
        assert_eq!(sanitized.tier, "1");
    }

    #[test]
    fn test_sanitize_empty_id() {
        let bp = mock_business_partner("", "Acme GmbH", "DE");
        let result = sanitize_business_partner(&bp);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_sanitize_empty_name() {
        let bp = mock_business_partner("100001", "   ", "DE");
        let result = sanitize_business_partner(&bp);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_sanitize_invalid_country() {
        let bp = mock_business_partner("100001", "Acme GmbH", "DEU"); // Should be 2 chars
        let result = sanitize_business_partner(&bp);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Country code is invalid"));
    }

    #[test]
    fn test_sanitize_invalid_tier() {
        let mut bp = mock_business_partner("100001", "Acme GmbH", "DE");
        bp.tier = Some("5".to_string()); // Invalid tier
        let result = sanitize_business_partner(&bp);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Tier value is invalid"));
    }

    #[test]
    fn test_sanitize_missing_tier_defaults_to_3() {
        let mut bp = mock_business_partner("100001", "Acme GmbH", "DE");
        bp.tier = None;
        let result = sanitize_business_partner(&bp);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().tier, "3");
    }

    #[test]
    fn test_sanitize_valid_audit_date() {
        let mut bp = mock_business_partner("100001", "Acme GmbH", "DE");
        bp.audit_date = Some("2025-11-17".to_string());
        let result = sanitize_business_partner(&bp);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().audit_date, Some("2025-11-17".to_string()));
    }

    #[test]
    fn test_sanitize_invalid_audit_date_format() {
        let mut bp = mock_business_partner("100001", "Acme GmbH", "DE");
        bp.audit_date = Some("17.11.2025".to_string()); // Wrong format
        let result = sanitize_business_partner(&bp);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Audit date format is invalid"));
    }

    #[test]
    fn test_sanitize_batch_deterministic_ordering() {
        let partners = vec![
            mock_business_partner("100003", "C GmbH", "DE"),
            mock_business_partner("100001", "A GmbH", "DE"),
            mock_business_partner("100002", "B GmbH", "DE"),
        ];

        let (sanitized, errors) = sanitize_batch(&partners);

        assert_eq!(errors.len(), 0);
        assert_eq!(sanitized.len(), 3);
        assert_eq!(sanitized[0].id, "100001");
        assert_eq!(sanitized[1].id, "100002");
        assert_eq!(sanitized[2].id, "100003");
    }

    #[test]
    fn test_sanitize_batch_partial_failures() {
        let partners = vec![
            mock_business_partner("100001", "Valid GmbH", "DE"),
            mock_business_partner("", "Invalid GmbH", "DE"), // Empty ID
            mock_business_partner("100003", "Valid AG", "FR"),
        ];

        let (sanitized, errors) = sanitize_batch(&partners);

        assert_eq!(sanitized.len(), 2);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].1.contains("empty"));
    }

    #[test]
    fn test_validate_iso_date() {
        assert!(validate_iso_date("2025-11-17").is_ok());
        assert!(validate_iso_date("2025-01-01").is_ok());
        assert!(validate_iso_date("2025-12-31").is_ok());

        assert!(validate_iso_date("2025-13-01").is_err()); // Invalid month
        assert!(validate_iso_date("2025-11-32").is_err()); // Invalid day
        assert!(validate_iso_date("17.11.2025").is_err()); // Wrong format
        assert!(validate_iso_date("2025/11/17").is_err()); // Wrong separator
    }

    #[test]
    fn test_ubo_count_validation() {
        let mut bp = mock_business_partner("100001", "Acme GmbH", "DE");
        bp.ubo_count = Some(1000); // Too large
        let result = sanitize_business_partner(&bp);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("UBO count is invalid"));
    }
}
