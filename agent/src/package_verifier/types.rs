//! Package Verifier Types
//!
//! Core data structures for package verification.

use crate::verifier::VerifyStatus;

/// Bundle Type Detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BundleType {
    /// Modern bundle with _meta.json (cap-bundle.v1)
    Modern,
    /// Legacy bundle without _meta.json
    Legacy,
}

/// Verifier-Ergebnis
#[derive(Debug)]
pub struct VerificationResult {
    pub success: bool,
    pub manifest_hash: String,
    pub policy_hash: String,
    pub proof_status: String,
    pub checks_passed: usize,
    pub checks_total: usize,
}

/// Bundle-Verifikations-Ergebnis
#[derive(Debug)]
pub struct BundleVerifyResult {
    pub bundle_id: String,
    pub schema: String,
    #[allow(dead_code)] // Reserved for future reporting/serialization
    pub created_at: String,
    pub status: VerifyStatus,
    pub unit_results: Vec<(String, crate::verifier::core::VerifyReport)>,
}

/// Aggregiert den Gesamtstatus aus allen Unit-Ergebnissen
pub fn aggregate_status(
    unit_results: &[(String, crate::verifier::core::VerifyReport)],
) -> VerifyStatus {
    unit_results
        .iter()
        .fold(VerifyStatus::Ok, |acc, (_id, report)| {
            let unit_status = match report.status.as_str() {
                "ok" => VerifyStatus::Ok,
                "fail" => VerifyStatus::Fail,
                _ => VerifyStatus::Warn,
            };
            match (acc, unit_status) {
                (VerifyStatus::Error, _) | (_, VerifyStatus::Error) => VerifyStatus::Error,
                (VerifyStatus::Fail, _) | (_, VerifyStatus::Fail) => VerifyStatus::Fail,
                (VerifyStatus::Warn, _) | (_, VerifyStatus::Warn) => VerifyStatus::Warn,
                _ => VerifyStatus::Ok,
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bundle_type_equality() {
        assert_eq!(BundleType::Modern, BundleType::Modern);
        assert_ne!(BundleType::Modern, BundleType::Legacy);
    }
}
