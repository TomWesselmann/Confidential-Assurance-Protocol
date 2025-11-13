/// Execution Planner - Cost-based deterministic rule ordering
///
/// Creates an execution plan for active rules with:
/// - Deterministic ordering (sorted by cost, then by rule ID)
/// - Cost estimation based on operation type
/// - Dependency resolution (future: DAG-based scheduling)

use crate::policy_v2::types::{IrV1, IrRule};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Execution step in the plan
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlanStep {
    /// Rule ID
    pub rule_id: String,

    /// Operator type (non_membership, eq, range_min, etc.)
    pub op: String,

    /// Estimated cost (lower = cheaper)
    pub cost: u32,

    /// Step index in execution order (0-indexed)
    pub step_index: usize,
}

/// Execution plan - ordered sequence of rule executions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    /// Ordered steps
    pub steps: Vec<PlanStep>,

    /// Total estimated cost
    pub total_cost: u32,

    /// Metadata
    pub metadata: PlanMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanMetadata {
    /// Policy ID
    pub policy_id: String,

    /// Number of active rules
    pub active_rules: usize,

    /// Planning strategy ("cost_based_v1")
    pub strategy: String,
}

/// Cost estimator - assigns costs to operations
pub struct CostEstimator;

impl CostEstimator {
    /// Estimates cost for a rule based on its operator
    ///
    /// Cost model (Week 5 MVP):
    /// - eq: 1 (cheapest - simple equality)
    /// - range_min: 2 (range check)
    /// - range_max: 2 (range check)
    /// - non_membership: 10 (Merkle proof verification)
    /// - non_intersection: 15 (set operation)
    /// - threshold: 20 (percentage calculation)
    /// - custom: 100 (unknown operation)
    pub fn estimate_cost(op: &str) -> u32 {
        match op {
            "eq" => 1,
            "range_min" => 2,
            "range_max" => 2,
            "gt" => 2,
            "lt" => 2,
            "gte" => 2,
            "lte" => 2,
            "non_membership" => 10,
            "membership" => 10,
            "non_intersection" => 15,
            "intersection" => 15,
            "threshold" => 20,
            _ => 100, // Unknown operations have high cost
        }
    }
}

/// Execution planner
pub struct Planner {
    /// All rules from IR (indexed by ID)
    rules: HashMap<String, IrRule>,

    /// Policy ID for metadata
    policy_id: String,
}

impl Planner {
    /// Creates a new planner from IR
    pub fn new(ir: &IrV1) -> Result<Self> {
        let rules: HashMap<String, IrRule> = ir.rules.iter()
            .map(|r| (r.id.clone(), r.clone()))
            .collect();

        Ok(Self {
            rules,
            policy_id: ir.policy_id.clone(),
        })
    }

    /// Creates an execution plan for active rules
    ///
    /// Planning strategy:
    /// 1. Filter to active rules only
    /// 2. Estimate cost for each rule
    /// 3. Sort by cost (ascending), then by rule ID (lexicographic)
    /// 4. Assign step indices
    pub fn plan(&self, active_rule_ids: &[String]) -> Result<ExecutionPlan> {
        // Step 1: Collect active rules with costs
        let mut steps: Vec<PlanStep> = Vec::new();

        for rule_id in active_rule_ids {
            let rule = self.rules.get(rule_id)
                .ok_or_else(|| anyhow!("Rule not found: {}", rule_id))?;

            let cost = CostEstimator::estimate_cost(&rule.op);

            steps.push(PlanStep {
                rule_id: rule_id.clone(),
                op: rule.op.clone(),
                cost,
                step_index: 0, // Will be assigned after sorting
            });
        }

        // Step 2: Sort by cost (ascending), then by rule_id (lexicographic)
        steps.sort_by(|a, b| {
            a.cost.cmp(&b.cost)
                .then_with(|| a.rule_id.cmp(&b.rule_id))
        });

        // Step 3: Assign step indices
        for (idx, step) in steps.iter_mut().enumerate() {
            step.step_index = idx;
        }

        // Step 4: Compute total cost
        let total_cost: u32 = steps.iter().map(|s| s.cost).sum();

        Ok(ExecutionPlan {
            steps,
            total_cost,
            metadata: PlanMetadata {
                policy_id: self.policy_id.clone(),
                active_rules: active_rule_ids.len(),
                strategy: "cost_based_v1".to_string(),
            },
        })
    }

    /// Creates an empty plan (no active rules)
    pub fn empty_plan(&self) -> ExecutionPlan {
        ExecutionPlan {
            steps: vec![],
            total_cost: 0,
            metadata: PlanMetadata {
                policy_id: self.policy_id.clone(),
                active_rules: 0,
                strategy: "cost_based_v1".to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy_v2::types::{IrV1, IrRule, IrExpression};
    use serde_json::json;

    fn make_test_ir() -> IrV1 {
        IrV1 {
            ir_version: "1.0".to_string(),
            policy_id: "test.v1".to_string(),
            policy_hash: "sha3-256:abc".to_string(),
            rules: vec![
                IrRule {
                    id: "rule_eq".to_string(),
                    op: "eq".to_string(),
                    lhs: IrExpression::Var { var: "x".to_string() },
                    rhs: IrExpression::Literal(json!(1)),
                },
                IrRule {
                    id: "rule_membership".to_string(),
                    op: "non_membership".to_string(),
                    lhs: IrExpression::Var { var: "hash".to_string() },
                    rhs: IrExpression::Var { var: "root".to_string() },
                },
                IrRule {
                    id: "rule_range".to_string(),
                    op: "range_min".to_string(),
                    lhs: IrExpression::Var { var: "age".to_string() },
                    rhs: IrExpression::Literal(json!(18)),
                },
            ],
            adaptivity: None,
            ir_hash: "sha3-256:def".to_string(),
        }
    }

    #[test]
    fn test_cost_estimator() {
        assert_eq!(CostEstimator::estimate_cost("eq"), 1);
        assert_eq!(CostEstimator::estimate_cost("range_min"), 2);
        assert_eq!(CostEstimator::estimate_cost("non_membership"), 10);
        assert_eq!(CostEstimator::estimate_cost("threshold"), 20);
        assert_eq!(CostEstimator::estimate_cost("unknown_op"), 100);
    }

    #[test]
    fn test_planner_empty_plan() {
        let ir = make_test_ir();
        let planner = Planner::new(&ir).unwrap();

        let plan = planner.empty_plan();
        assert_eq!(plan.steps.len(), 0);
        assert_eq!(plan.total_cost, 0);
        assert_eq!(plan.metadata.active_rules, 0);
    }

    #[test]
    fn test_planner_all_rules() {
        let ir = make_test_ir();
        let planner = Planner::new(&ir).unwrap();

        let active_rules = vec![
            "rule_eq".to_string(),
            "rule_membership".to_string(),
            "rule_range".to_string(),
        ];

        let plan = planner.plan(&active_rules).unwrap();

        // Should have 3 steps
        assert_eq!(plan.steps.len(), 3);

        // Verify cost-based ordering:
        // 1. rule_eq (cost=1)
        // 2. rule_range (cost=2)
        // 3. rule_membership (cost=10)
        assert_eq!(plan.steps[0].rule_id, "rule_eq");
        assert_eq!(plan.steps[0].cost, 1);
        assert_eq!(plan.steps[0].step_index, 0);

        assert_eq!(plan.steps[1].rule_id, "rule_range");
        assert_eq!(plan.steps[1].cost, 2);
        assert_eq!(plan.steps[1].step_index, 1);

        assert_eq!(plan.steps[2].rule_id, "rule_membership");
        assert_eq!(plan.steps[2].cost, 10);
        assert_eq!(plan.steps[2].step_index, 2);

        // Total cost
        assert_eq!(plan.total_cost, 13); // 1 + 2 + 10
    }

    #[test]
    fn test_planner_deterministic_ordering() {
        // Create IR with same-cost rules (should sort by rule_id)
        let ir = IrV1 {
            ir_version: "1.0".to_string(),
            policy_id: "test.v1".to_string(),
            policy_hash: "sha3-256:abc".to_string(),
            rules: vec![
                IrRule {
                    id: "z_rule".to_string(),
                    op: "eq".to_string(),
                    lhs: IrExpression::Var { var: "x".to_string() },
                    rhs: IrExpression::Literal(json!(1)),
                },
                IrRule {
                    id: "a_rule".to_string(),
                    op: "eq".to_string(),
                    lhs: IrExpression::Var { var: "y".to_string() },
                    rhs: IrExpression::Literal(json!(2)),
                },
                IrRule {
                    id: "m_rule".to_string(),
                    op: "eq".to_string(),
                    lhs: IrExpression::Var { var: "z".to_string() },
                    rhs: IrExpression::Literal(json!(3)),
                },
            ],
            adaptivity: None,
            ir_hash: "sha3-256:def".to_string(),
        };

        let planner = Planner::new(&ir).unwrap();

        let active_rules = vec![
            "z_rule".to_string(),
            "m_rule".to_string(),
            "a_rule".to_string(),
        ];

        let plan = planner.plan(&active_rules).unwrap();

        // Same cost (eq=1) → should sort by rule_id lexicographically
        assert_eq!(plan.steps[0].rule_id, "a_rule");
        assert_eq!(plan.steps[1].rule_id, "m_rule");
        assert_eq!(plan.steps[2].rule_id, "z_rule");
    }

    #[test]
    fn test_planner_partial_rules() {
        let ir = make_test_ir();
        let planner = Planner::new(&ir).unwrap();

        // Only activate 2 out of 3 rules
        let active_rules = vec![
            "rule_membership".to_string(),
            "rule_eq".to_string(),
        ];

        let plan = planner.plan(&active_rules).unwrap();

        assert_eq!(plan.steps.len(), 2);
        assert_eq!(plan.steps[0].rule_id, "rule_eq"); // cost=1
        assert_eq!(plan.steps[1].rule_id, "rule_membership"); // cost=10
        assert_eq!(plan.total_cost, 11);
    }
}
