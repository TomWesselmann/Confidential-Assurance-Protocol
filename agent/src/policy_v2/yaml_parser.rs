use super::types::PolicyV2;
use anyhow::{Context, Result};
use std::path::Path;

/// Parse PolicyV2 from YAML file
pub fn parse_yaml<P: AsRef<Path>>(path: P) -> Result<PolicyV2> {
    let contents = std::fs::read_to_string(path.as_ref()).context("Failed to read policy file")?;

    let policy: PolicyV2 = serde_yaml::from_str(&contents).context("Failed to parse YAML")?;

    Ok(policy)
}

/// Parse PolicyV2 from YAML string
pub fn parse_yaml_str(yaml: &str) -> Result<PolicyV2> {
    let policy: PolicyV2 = serde_yaml::from_str(yaml).context("Failed to parse YAML string")?;

    Ok(policy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_policy() {
        let yaml = r#"
id: lksg.v1
version: "1.0"
legal_basis:
  - directive: "LkSG"
inputs:
  supplier_hashes:
    type: array
    items: hex
  sanctions_root:
    type: hex
rules:
  - id: no_sanctions
    op: non_membership
    lhs: supplier_hashes
    rhs: sanctions_root
"#;
        let policy = parse_yaml_str(yaml).unwrap();
        assert_eq!(policy.id, "lksg.v1");
        assert_eq!(policy.version, "1.0");
        assert_eq!(policy.rules.len(), 1);
        assert_eq!(policy.rules[0].id, "no_sanctions");
        assert_eq!(policy.rules[0].op, "non_membership");
    }

    #[test]
    fn test_parse_policy_with_description() {
        let yaml = r#"
id: test.v1
version: "1.0"
description: "Test policy with description"
legal_basis:
  - directive: "LkSG"
    article: "Art. 3"
inputs: {}
rules: []
"#;
        let policy = parse_yaml_str(yaml).unwrap();
        assert_eq!(policy.description, "Test policy with description");
        assert_eq!(policy.legal_basis[0].article, Some("Art. 3".to_string()));
    }

    #[test]
    fn test_parse_policy_with_adaptivity() {
        let yaml = r#"
id: test.v1
version: "1.0"
legal_basis:
  - directive: "LkSG"
inputs: {}
rules:
  - id: rule1
    op: eq
    lhs: a
    rhs: b
adaptivity:
  predicates:
    - id: pred1
      expr: "now() > 2025-01-01"
  activations:
    - when: pred1
      rules: [rule1]
"#;
        let policy = parse_yaml_str(yaml).unwrap();
        assert!(policy.adaptivity.is_some());
        let adaptivity = policy.adaptivity.unwrap();
        assert_eq!(adaptivity.predicates.len(), 1);
        assert_eq!(adaptivity.activations.len(), 1);
        assert_eq!(adaptivity.predicates[0].id, "pred1");
    }

    #[test]
    fn test_parse_invalid_yaml() {
        let yaml = "invalid: yaml: structure: [";
        let result = parse_yaml_str(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_required_fields() {
        let yaml = r#"
id: test.v1
# Missing version, legal_basis, inputs, rules
"#;
        let result = parse_yaml_str(yaml);
        assert!(result.is_err());
    }
}
