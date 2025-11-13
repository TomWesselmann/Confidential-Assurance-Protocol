use super::types::{PolicyV2, IrV1, IrRule, IrExpression, IrAdaptivity, IrPredicate};
use anyhow::Result;

/// Generate IR v1 from PolicyV2
pub fn generate_ir(policy: &PolicyV2, policy_hash: String) -> Result<IrV1> {
    // Convert rules
    let mut ir_rules: Vec<IrRule> = policy.rules.iter().map(|r| {
        IrRule {
            id: r.id.clone(),
            op: r.op.clone(),
            lhs: convert_expression(&r.lhs),
            rhs: convert_expression(&r.rhs),
        }
    }).collect();

    // IMPORTANT: Canonical ordering - sort by rule ID
    ir_rules.sort_by(|a, b| a.id.cmp(&b.id));

    // Convert adaptivity (if present)
    let ir_adaptivity = policy.adaptivity.as_ref().map(|adapt| {
        IrAdaptivity {
            predicates: adapt.predicates.iter().map(|p| {
                IrPredicate {
                    id: p.id.clone(),
                    expr: p.expr.clone(),
                }
            }).collect(),
            activations: adapt.activations.clone(),
        }
    });

    let ir = IrV1 {
        ir_version: "1.0".to_string(),
        policy_id: policy.id.clone(),
        policy_hash,
        rules: ir_rules,
        adaptivity: ir_adaptivity,
        ir_hash: String::new(), // Will be filled by hasher
    };

    Ok(ir)
}

fn convert_expression(expr: &serde_json::Value) -> IrExpression {
    match expr {
        serde_json::Value::String(s) => {
            // Simple variable reference
            IrExpression::Var { var: s.clone() }
        }
        other => {
            // Complex expression or literal
            IrExpression::Literal(other.clone())
        }
    }
}

/// Canonicalize IR for hashing
/// Uses serde_json with BTreeMap to ensure stable ordering of keys
pub fn canonicalize(ir: &IrV1) -> Result<String> {
    // Use serde_json with BTreeMap for stable ordering
    let json = serde_json::to_string(ir)?;
    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy_v2::types::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_rules_sorted_by_id() {
        let policy = PolicyV2 {
            id: "test".to_string(),
            version: "1.0".to_string(),
            legal_basis: vec![],
            description: "".to_string(),
            inputs: BTreeMap::new(),
            rules: vec![
                Rule {
                    id: "z_rule".to_string(),
                    op: "eq".to_string(),
                    lhs: serde_json::json!("a"),
                    rhs: serde_json::json!("b"),
                },
                Rule {
                    id: "a_rule".to_string(),
                    op: "eq".to_string(),
                    lhs: serde_json::json!("c"),
                    rhs: serde_json::json!("d"),
                },
            ],
            adaptivity: None,
        };

        let ir = generate_ir(&policy, "hash123".to_string()).unwrap();
        assert_eq!(ir.rules[0].id, "a_rule");
        assert_eq!(ir.rules[1].id, "z_rule");
    }

    #[test]
    fn test_convert_expression_var() {
        let expr = serde_json::json!("supplier_hashes");
        let ir_expr = convert_expression(&expr);

        match ir_expr {
            IrExpression::Var { var } => assert_eq!(var, "supplier_hashes"),
            _ => panic!("Expected Var variant"),
        }
    }

    #[test]
    fn test_convert_expression_literal() {
        let expr = serde_json::json!(42);
        let ir_expr = convert_expression(&expr);

        match ir_expr {
            IrExpression::Literal(val) => assert_eq!(val, 42),
            _ => panic!("Expected Literal variant"),
        }
    }

    #[test]
    fn test_canonicalize() {
        let ir = IrV1 {
            ir_version: "1.0".to_string(),
            policy_id: "test.v1".to_string(),
            policy_hash: "sha3-256:abc123".to_string(),
            rules: vec![],
            adaptivity: None,
            ir_hash: "sha3-256:def456".to_string(),
        };

        let canonical = canonicalize(&ir).unwrap();
        assert!(canonical.contains("\"ir_version\":\"1.0\""));
        assert!(canonical.contains("\"policy_id\":\"test.v1\""));
    }

    #[test]
    fn test_generate_ir_with_adaptivity() {
        let policy = PolicyV2 {
            id: "test".to_string(),
            version: "1.0".to_string(),
            legal_basis: vec![],
            description: "".to_string(),
            inputs: BTreeMap::new(),
            rules: vec![],
            adaptivity: Some(Adaptivity {
                predicates: vec![Predicate {
                    id: "pred1".to_string(),
                    expr: serde_json::json!("now() > 2025-01-01"),
                }],
                activations: vec![Activation {
                    when: "pred1".to_string(),
                    rules: vec!["rule1".to_string()],
                }],
            }),
        };

        let ir = generate_ir(&policy, "hash123".to_string()).unwrap();
        assert!(ir.adaptivity.is_some());
        let adaptivity = ir.adaptivity.unwrap();
        assert_eq!(adaptivity.predicates.len(), 1);
        assert_eq!(adaptivity.activations.len(), 1);
    }

    #[test]
    fn test_canonicalize_determinism() {
        let ir = IrV1 {
            ir_version: "1.0".to_string(),
            policy_id: "test.v1".to_string(),
            policy_hash: "sha3-256:abc123".to_string(),
            rules: vec![],
            adaptivity: None,
            ir_hash: "sha3-256:def456".to_string(),
        };

        let canonical1 = canonicalize(&ir).unwrap();
        let canonical2 = canonicalize(&ir).unwrap();
        assert_eq!(canonical1, canonical2);
    }
}
