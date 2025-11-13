use super::types::PolicyV2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LintMode {
    Strict,
    Relaxed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintDiagnostic {
    pub code: String, // e.g., "E1001", "W1002"
    pub level: Level,
    pub message: String,
    pub rule_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Error,
    Warning,
}

/// Lint error codes (machine-readable)
#[derive(Debug, Clone, Copy)]
pub enum LintCode {
    // E1xxx - Policy structure errors
    E1001, // Unknown rule ID in activation
    E1002, // Missing legal_basis
    E1003, // Duplicate rule ID

    // E2xxx - Expression/operator errors
    E2001, // Invalid operator
    E2003, // Unknown input reference

    // E3xxx - Constraint errors
    E3002, // Invalid range_min expression

    // W1xxx - Warnings
    W1002, // Missing description
}

impl LintCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            LintCode::E1001 => "E1001",
            LintCode::E1002 => "E1002",
            LintCode::E1003 => "E1003",
            LintCode::E2001 => "E2001",
            LintCode::E2003 => "E2003",
            LintCode::E3002 => "E3002",
            LintCode::W1002 => "W1002",
        }
    }

    pub fn http_status(&self) -> u16 {
        match self {
            LintCode::E1001
            | LintCode::E1002
            | LintCode::E1003
            | LintCode::E2001
            | LintCode::E2003
            | LintCode::E3002 => 422,
            LintCode::W1002 => 200,
        }
    }

    pub fn level(&self) -> Level {
        match self {
            LintCode::E1001
            | LintCode::E1002
            | LintCode::E1003
            | LintCode::E2001
            | LintCode::E2003
            | LintCode::E3002 => Level::Error,
            LintCode::W1002 => Level::Warning,
        }
    }
}

/// Lint a policy and return diagnostics
pub fn lint(policy: &PolicyV2, mode: LintMode) -> Vec<LintDiagnostic> {
    let mut diagnostics = Vec::new();

    // Check legal_basis (E1002)
    if policy.legal_basis.is_empty() {
        let level = match mode {
            LintMode::Strict => Level::Error,
            LintMode::Relaxed => Level::Warning,
        };
        diagnostics.push(LintDiagnostic {
            code: LintCode::E1002.as_str().to_string(),
            level,
            message: "missing `legal_basis`".to_string(),
            rule_id: None,
        });
    }

    // Check description (W1002)
    if policy.description.is_empty() {
        diagnostics.push(LintDiagnostic {
            code: LintCode::W1002.as_str().to_string(),
            level: Level::Warning,
            message: "description missing".to_string(),
            rule_id: None,
        });
    }

    // Check rule IDs uniqueness (E1003)
    let mut seen_ids = std::collections::HashSet::new();
    for rule in &policy.rules {
        if !seen_ids.insert(&rule.id) {
            diagnostics.push(LintDiagnostic {
                code: LintCode::E1003.as_str().to_string(),
                level: Level::Error,
                message: format!("duplicate rule ID '{}'", rule.id),
                rule_id: Some(rule.id.clone()),
            });
        }
    }

    // Check valid operators (E2001)
    const VALID_OPS: &[&str] = &["non_membership", "eq", "range_min"];
    for rule in &policy.rules {
        if !VALID_OPS.contains(&rule.op.as_str()) {
            diagnostics.push(LintDiagnostic {
                code: LintCode::E2001.as_str().to_string(),
                level: Level::Error,
                message: format!(
                    "invalid op '{}' (allowed: non_membership, eq, range_min)",
                    rule.op
                ),
                rule_id: Some(rule.id.clone()),
            });
        }
    }

    // Check adaptivity if present
    if let Some(ref adaptivity) = policy.adaptivity {
        // Collect all rule IDs
        let rule_ids: std::collections::HashSet<_> = policy.rules.iter().map(|r| &r.id).collect();

        // Check activations reference valid rule IDs (E1001)
        for activation in &adaptivity.activations {
            for rule_id in &activation.rules {
                if !rule_ids.contains(rule_id) {
                    diagnostics.push(LintDiagnostic {
                        code: LintCode::E1001.as_str().to_string(),
                        level: Level::Error,
                        message: format!(
                            "unknown rule id '{}' in activation '{}'",
                            rule_id, activation.when
                        ),
                        rule_id: None,
                    });
                }
            }
        }
    }

    // TODO: E2003 - Check lhs/rhs references exist in inputs (Week 2)
    // TODO: E3002 - Check range_min expression validity (Week 2)

    diagnostics
}

/// Check if diagnostics contain errors
pub fn has_errors(diagnostics: &[LintDiagnostic]) -> bool {
    diagnostics.iter().any(|d| d.level == Level::Error)
}

/// Get HTTP status code from diagnostics (for API responses)
/// Returns 422 if errors present, 200 otherwise
pub fn http_status_from_diagnostics(diagnostics: &[LintDiagnostic]) -> u16 {
    if has_errors(diagnostics) {
        422 // Unprocessable Entity
    } else {
        200 // OK (warnings only)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy_v2::types::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_lint_missing_legal_basis_strict() {
        let policy = PolicyV2 {
            id: "test".to_string(),
            version: "1.0".to_string(),
            legal_basis: vec![],
            description: "Test policy".to_string(),
            inputs: BTreeMap::new(),
            rules: vec![],
            adaptivity: None,
        };

        let diagnostics = lint(&policy, LintMode::Strict);
        assert!(has_errors(&diagnostics));
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].level, Level::Error);
        assert_eq!(diagnostics[0].code, "E1002");
    }

    #[test]
    fn test_lint_missing_legal_basis_relaxed() {
        let policy = PolicyV2 {
            id: "test".to_string(),
            version: "1.0".to_string(),
            legal_basis: vec![],
            description: "Test policy".to_string(),
            inputs: BTreeMap::new(),
            rules: vec![],
            adaptivity: None,
        };

        let diagnostics = lint(&policy, LintMode::Relaxed);
        assert!(!has_errors(&diagnostics));
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].level, Level::Warning);
        assert_eq!(diagnostics[0].code, "E1002");
    }

    #[test]
    fn test_lint_invalid_operator() {
        let policy = PolicyV2 {
            id: "test".to_string(),
            version: "1.0".to_string(),
            legal_basis: vec![LegalBasisItem {
                directive: Some("LkSG".to_string()),
                article: None,
            }],
            description: "Test policy".to_string(),
            inputs: BTreeMap::new(),
            rules: vec![Rule {
                id: "r1".to_string(),
                op: "invalid_op".to_string(),
                lhs: serde_json::json!("var1"),
                rhs: serde_json::json!("var2"),
            }],
            adaptivity: None,
        };

        let diagnostics = lint(&policy, LintMode::Strict);
        assert!(has_errors(&diagnostics));
        assert!(diagnostics.iter().any(|d| d.code == "E2001"));
        assert!(diagnostics.iter().any(|d| d.message.contains("invalid op")));
    }

    #[test]
    fn test_lint_duplicate_rule_ids() {
        let policy = PolicyV2 {
            id: "test".to_string(),
            version: "1.0".to_string(),
            legal_basis: vec![LegalBasisItem {
                directive: Some("LkSG".to_string()),
                article: None,
            }],
            description: "Test policy".to_string(),
            inputs: BTreeMap::new(),
            rules: vec![
                Rule {
                    id: "duplicate".to_string(),
                    op: "eq".to_string(),
                    lhs: serde_json::json!("a"),
                    rhs: serde_json::json!("b"),
                },
                Rule {
                    id: "duplicate".to_string(),
                    op: "eq".to_string(),
                    lhs: serde_json::json!("c"),
                    rhs: serde_json::json!("d"),
                },
            ],
            adaptivity: None,
        };

        let diagnostics = lint(&policy, LintMode::Strict);
        assert!(has_errors(&diagnostics));
        assert!(diagnostics.iter().any(|d| d.code == "E1003"));
        assert!(diagnostics
            .iter()
            .any(|d| d.message.contains("duplicate rule ID")));
    }

    #[test]
    fn test_lint_valid_policy() {
        let policy = PolicyV2 {
            id: "test".to_string(),
            version: "1.0".to_string(),
            legal_basis: vec![LegalBasisItem {
                directive: Some("LkSG".to_string()),
                article: None,
            }],
            description: "Test policy".to_string(),
            inputs: BTreeMap::new(),
            rules: vec![Rule {
                id: "r1".to_string(),
                op: "eq".to_string(),
                lhs: serde_json::json!("a"),
                rhs: serde_json::json!("b"),
            }],
            adaptivity: None,
        };

        let diagnostics = lint(&policy, LintMode::Strict);
        assert!(!has_errors(&diagnostics));
        assert_eq!(diagnostics.len(), 0); // No errors or warnings
    }
}
