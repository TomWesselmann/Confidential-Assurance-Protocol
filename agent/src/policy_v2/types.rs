use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// PolicyV2 YAML structure
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PolicyV2 {
    pub id: String,
    pub version: String,
    pub legal_basis: Vec<LegalBasisItem>,
    #[serde(default)]
    pub description: String,
    pub inputs: BTreeMap<String, InputDef>,
    pub rules: Vec<Rule>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adaptivity: Option<Adaptivity>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LegalBasisItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directive: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub article: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InputDef {
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Rule {
    pub id: String,
    pub op: String,
    pub lhs: serde_json::Value,  // Can be string or object
    pub rhs: serde_json::Value,  // Can be string or object
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Adaptivity {
    pub predicates: Vec<Predicate>,
    pub activations: Vec<Activation>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Predicate {
    pub id: String,
    pub expr: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Activation {
    pub when: String,
    pub rules: Vec<String>,
}

/// IR v1 Structure (output)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IrV1 {
    pub ir_version: String,
    pub policy_id: String,
    pub policy_hash: String,
    pub rules: Vec<IrRule>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adaptivity: Option<IrAdaptivity>,
    pub ir_hash: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IrRule {
    pub id: String,
    pub op: String,
    pub lhs: IrExpression,
    pub rhs: IrExpression,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum IrExpression {
    Var { var: String },
    Literal(serde_json::Value),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IrAdaptivity {
    pub predicates: Vec<IrPredicate>,
    pub activations: Vec<Activation>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IrPredicate {
    pub id: String,
    pub expr: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_v2_roundtrip() {
        let policy = PolicyV2 {
            id: "test.v1".to_string(),
            version: "1.0".to_string(),
            legal_basis: vec![LegalBasisItem {
                directive: Some("LkSG".to_string()),
                article: None,
            }],
            description: "Test policy".to_string(),
            inputs: BTreeMap::new(),
            rules: vec![],
            adaptivity: None,
        };

        let json = serde_json::to_string(&policy).unwrap();
        let parsed: PolicyV2 = serde_json::from_str(&json).unwrap();

        assert_eq!(policy.id, parsed.id);
        assert_eq!(policy.version, parsed.version);
    }

    #[test]
    fn test_ir_v1_roundtrip() {
        let ir = IrV1 {
            ir_version: "1.0".to_string(),
            policy_id: "test.v1".to_string(),
            policy_hash: "sha3-256:abc123".to_string(),
            rules: vec![],
            adaptivity: None,
            ir_hash: "sha3-256:def456".to_string(),
        };

        let json = serde_json::to_string(&ir).unwrap();
        let parsed: IrV1 = serde_json::from_str(&json).unwrap();

        assert_eq!(ir.policy_id, parsed.policy_id);
        assert_eq!(ir.ir_version, parsed.ir_version);
    }

    #[test]
    fn test_ir_expression_var() {
        let expr = IrExpression::Var {
            var: "supplier_hashes".to_string(),
        };

        let json = serde_json::to_string(&expr).unwrap();
        let parsed: IrExpression = serde_json::from_str(&json).unwrap();

        match parsed {
            IrExpression::Var { var } => assert_eq!(var, "supplier_hashes"),
            _ => panic!("Expected Var variant"),
        }
    }

    #[test]
    fn test_ir_expression_literal() {
        let expr = IrExpression::Literal(serde_json::json!(42));

        let json = serde_json::to_string(&expr).unwrap();
        let parsed: IrExpression = serde_json::from_str(&json).unwrap();

        match parsed {
            IrExpression::Literal(val) => assert_eq!(val, 42),
            _ => panic!("Expected Literal variant"),
        }
    }
}
