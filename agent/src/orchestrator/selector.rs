use crate::orchestrator::OrchestratorContext;
/// Rule Selector - IR-based predicate evaluation and rule activation
///
/// Evaluates predicates from IR adaptivity section and determines which rules
/// should be active based on runtime context.
use crate::policy_v2::types::{Activation, IrAdaptivity, IrPredicate, IrV1};
use anyhow::{anyhow, Result};
use std::collections::{HashMap, HashSet};

/// Predicate evaluator - evaluates IR expressions to bool
pub struct PredicateEvaluator;

impl PredicateEvaluator {
    /// Evaluates a predicate expression against context
    ///
    /// Currently supports simplified evaluation for Week 5 MVP:
    /// - Simple boolean literals
    /// - Basic comparisons (future: full IR expression evaluation)
    pub fn evaluate(expr: &serde_json::Value, context: &OrchestratorContext) -> Result<bool> {
        match expr {
            // Simple boolean literal
            serde_json::Value::Bool(b) => Ok(*b),

            // String (variable reference or literal "true"/"false")
            serde_json::Value::String(s) => {
                if s == "true" {
                    Ok(true)
                } else if s == "false" {
                    Ok(false)
                } else {
                    // Try to resolve as variable
                    context
                        .variables
                        .get(s)
                        .and_then(|v| v.as_bool())
                        .ok_or_else(|| anyhow!("Cannot evaluate variable: {}", s))
                }
            }

            // Object (function call or complex expression)
            serde_json::Value::Object(obj) => {
                // Check for function call { "func": "lt", "args": [...] }
                if let Some(func) = obj.get("func").and_then(|v| v.as_str()) {
                    Self::evaluate_function(func, obj.get("args"), context)
                } else {
                    Err(anyhow!("Unsupported expression object: {:?}", obj))
                }
            }

            _ => Err(anyhow!("Unsupported predicate expression type: {:?}", expr)),
        }
    }

    /// Evaluates a function call (e.g., lt, gt, eq)
    fn evaluate_function(
        func: &str,
        args: Option<&serde_json::Value>,
        context: &OrchestratorContext,
    ) -> Result<bool> {
        match func {
            "lt" => {
                // Less than: lt(a, b) → a < b
                let args = args.ok_or_else(|| anyhow!("Missing args for lt"))?;
                let args_arr = args
                    .as_array()
                    .ok_or_else(|| anyhow!("lt args must be array"))?;
                if args_arr.len() != 2 {
                    return Err(anyhow!("lt requires exactly 2 arguments"));
                }

                // For MVP: Support simple numeric comparisons
                let a = Self::extract_number(&args_arr[0], context)?;
                let b = Self::extract_number(&args_arr[1], context)?;
                Ok(a < b)
            }

            "gt" => {
                // Greater than: gt(a, b) → a > b
                let args = args.ok_or_else(|| anyhow!("Missing args for gt"))?;
                let args_arr = args
                    .as_array()
                    .ok_or_else(|| anyhow!("gt args must be array"))?;
                if args_arr.len() != 2 {
                    return Err(anyhow!("gt requires exactly 2 arguments"));
                }

                let a = Self::extract_number(&args_arr[0], context)?;
                let b = Self::extract_number(&args_arr[1], context)?;
                Ok(a > b)
            }

            "eq" => {
                // Equality: eq(a, b) → a == b
                let args = args.ok_or_else(|| anyhow!("Missing args for eq"))?;
                let args_arr = args
                    .as_array()
                    .ok_or_else(|| anyhow!("eq args must be array"))?;
                if args_arr.len() != 2 {
                    return Err(anyhow!("eq requires exactly 2 arguments"));
                }

                // For MVP: Support simple value comparisons
                Ok(args_arr[0] == args_arr[1])
            }

            _ => Err(anyhow!("Unsupported function: {}", func)),
        }
    }

    /// Extracts a number from an expression (literal or variable)
    fn extract_number(expr: &serde_json::Value, context: &OrchestratorContext) -> Result<f64> {
        match expr {
            serde_json::Value::Number(n) => n
                .as_f64()
                .ok_or_else(|| anyhow!("Cannot convert number to f64")),
            serde_json::Value::String(var) => context
                .variables
                .get(var)
                .and_then(|v| v.as_f64())
                .ok_or_else(|| anyhow!("Variable {} not found or not a number", var)),
            _ => Err(anyhow!("Expected number or variable, got {:?}", expr)),
        }
    }
}

/// Rule selector - determines active rules based on predicates
pub struct RuleSelector {
    predicates: HashMap<String, IrPredicate>,
    activations: Vec<Activation>,
}

impl RuleSelector {
    /// Creates a new rule selector from IR adaptivity
    pub fn new(adaptivity: Option<&IrAdaptivity>) -> Self {
        let (predicates, activations) = if let Some(adapt) = adaptivity {
            let pred_map: HashMap<String, IrPredicate> = adapt
                .predicates
                .iter()
                .map(|p| (p.id.clone(), p.clone()))
                .collect();
            (pred_map, adapt.activations.clone())
        } else {
            (HashMap::new(), vec![])
        };

        Self {
            predicates,
            activations,
        }
    }

    /// Selects active rules based on context
    ///
    /// Returns a set of rule IDs that should be active
    pub fn select(&self, context: &OrchestratorContext) -> Result<HashSet<String>> {
        let mut active_rules = HashSet::new();

        // Evaluate each activation
        for activation in &self.activations {
            // Find the predicate
            let predicate = self
                .predicates
                .get(&activation.when)
                .ok_or_else(|| anyhow!("Predicate not found: {}", activation.when))?;

            // Evaluate predicate
            let is_true = PredicateEvaluator::evaluate(&predicate.expr, context)?;

            // If true, activate all associated rules
            if is_true {
                for rule_id in &activation.rules {
                    active_rules.insert(rule_id.clone());
                }
            }
        }

        Ok(active_rules)
    }
}

/// Main selector - coordinates predicate evaluation and rule selection
pub struct Selector {
    /// All rules from IR (baseline)
    all_rules: Vec<String>,

    /// Rule selector (handles adaptivity)
    rule_selector: RuleSelector,

    /// Whether adaptivity is present
    has_adaptivity: bool,
}

impl Selector {
    /// Creates a new selector from IR
    pub fn new(ir: &IrV1) -> Result<Self> {
        let all_rules: Vec<String> = ir.rules.iter().map(|r| r.id.clone()).collect();

        let has_adaptivity = ir.adaptivity.is_some();
        let rule_selector = RuleSelector::new(ir.adaptivity.as_ref());

        Ok(Self {
            all_rules,
            rule_selector,
            has_adaptivity,
        })
    }

    /// Selects active rules based on context
    ///
    /// If no adaptivity is present, returns all rules.
    /// If adaptivity is present, returns baseline + conditionally activated rules.
    pub fn select_active_rules(&self, context: &OrchestratorContext) -> Result<Vec<String>> {
        if !self.has_adaptivity {
            // No adaptivity → all rules are always active
            return Ok(self.all_rules.clone());
        }

        // With adaptivity: start with empty set, add activated rules
        let activated = self.rule_selector.select(context)?;

        // Return activated rules in deterministic order (sorted)
        let mut active: Vec<String> = activated.into_iter().collect();
        active.sort();

        Ok(active)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_predicate_evaluator_bool_literal() {
        let ctx = OrchestratorContext {
            supplier_hashes: vec![],
            ubo_hashes: vec![],
            company_commitment_root: None,
            sanctions_root: None,
            jurisdiction_root: None,
            variables: HashMap::new(),
        };

        assert!(PredicateEvaluator::evaluate(&json!(true), &ctx).unwrap());
        assert!(!PredicateEvaluator::evaluate(&json!(false), &ctx).unwrap());
    }

    #[test]
    fn test_predicate_evaluator_string_literal() {
        let ctx = OrchestratorContext {
            supplier_hashes: vec![],
            ubo_hashes: vec![],
            company_commitment_root: None,
            sanctions_root: None,
            jurisdiction_root: None,
            variables: HashMap::new(),
        };

        assert!(PredicateEvaluator::evaluate(&json!("true"), &ctx).unwrap());
        assert!(!PredicateEvaluator::evaluate(&json!("false"), &ctx).unwrap());
    }

    #[test]
    fn test_predicate_evaluator_variable() {
        let mut vars = HashMap::new();
        vars.insert("is_recent".to_string(), json!(true));

        let ctx = OrchestratorContext {
            supplier_hashes: vec![],
            ubo_hashes: vec![],
            company_commitment_root: None,
            sanctions_root: None,
            jurisdiction_root: None,
            variables: vars,
        };

        assert!(PredicateEvaluator::evaluate(&json!("is_recent"), &ctx).unwrap());
    }

    #[test]
    fn test_predicate_evaluator_lt_function() {
        let mut vars = HashMap::new();
        vars.insert("age".to_string(), json!(17));

        let ctx = OrchestratorContext {
            supplier_hashes: vec![],
            ubo_hashes: vec![],
            company_commitment_root: None,
            sanctions_root: None,
            jurisdiction_root: None,
            variables: vars,
        };

        // age < 18 → true
        let expr = json!({
            "func": "lt",
            "args": ["age", 18]
        });
        assert!(PredicateEvaluator::evaluate(&expr, &ctx).unwrap());

        // age < 10 → false
        let expr2 = json!({
            "func": "lt",
            "args": ["age", 10]
        });
        assert!(!PredicateEvaluator::evaluate(&expr2, &ctx).unwrap());
    }

    #[test]
    fn test_rule_selector_no_adaptivity() {
        let selector = RuleSelector::new(None);

        let ctx = OrchestratorContext {
            supplier_hashes: vec![],
            ubo_hashes: vec![],
            company_commitment_root: None,
            sanctions_root: None,
            jurisdiction_root: None,
            variables: HashMap::new(),
        };

        let active = selector.select(&ctx).unwrap();
        assert!(active.is_empty());
    }

    #[test]
    fn test_rule_selector_with_activation() {
        use crate::policy_v2::types::{Activation, IrAdaptivity, IrPredicate};

        let adaptivity = IrAdaptivity {
            predicates: vec![IrPredicate {
                id: "is_high_risk".to_string(),
                expr: json!(true),
            }],
            activations: vec![Activation {
                when: "is_high_risk".to_string(),
                rules: vec!["check_sanctions".to_string(), "verify_country".to_string()],
            }],
        };

        let selector = RuleSelector::new(Some(&adaptivity));

        let ctx = OrchestratorContext {
            supplier_hashes: vec![],
            ubo_hashes: vec![],
            company_commitment_root: None,
            sanctions_root: None,
            jurisdiction_root: None,
            variables: HashMap::new(),
        };

        let active = selector.select(&ctx).unwrap();
        assert_eq!(active.len(), 2);
        assert!(active.contains("check_sanctions"));
        assert!(active.contains("verify_country"));
    }

    #[test]
    fn test_selector_all_rules_no_adaptivity() {
        use crate::policy_v2::types::{IrExpression, IrRule, IrV1};

        let ir = IrV1 {
            ir_version: "1.0".to_string(),
            policy_id: "test.v1".to_string(),
            policy_hash: "sha3-256:abc".to_string(),
            rules: vec![
                IrRule {
                    id: "rule1".to_string(),
                    op: "eq".to_string(),
                    lhs: IrExpression::Var {
                        var: "x".to_string(),
                    },
                    rhs: IrExpression::Literal(json!(1)),
                },
                IrRule {
                    id: "rule2".to_string(),
                    op: "eq".to_string(),
                    lhs: IrExpression::Var {
                        var: "y".to_string(),
                    },
                    rhs: IrExpression::Literal(json!(2)),
                },
            ],
            adaptivity: None,
            ir_hash: "sha3-256:def".to_string(),
        };

        let selector = Selector::new(&ir).unwrap();

        let ctx = OrchestratorContext {
            supplier_hashes: vec![],
            ubo_hashes: vec![],
            company_commitment_root: None,
            sanctions_root: None,
            jurisdiction_root: None,
            variables: HashMap::new(),
        };

        let active = selector.select_active_rules(&ctx).unwrap();
        assert_eq!(active.len(), 2);
        assert_eq!(active[0], "rule1");
        assert_eq!(active[1], "rule2");
    }
}
