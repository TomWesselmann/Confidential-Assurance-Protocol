/// Adaptive Proof Orchestrator - Week 5 Track B1
///
/// Provides IR-controlled activation of rules with deterministic ordering.
/// Consists of two main components:
/// - Selector: Evaluates predicates and determines active rules
/// - Planner: Creates cost-based deterministic execution plan

pub mod selector;
pub mod planner;
pub mod enforcer;  // Week 6: Adaptive Enforcement
pub mod metrics;   // Week 6: Prometheus Metrics
pub mod drift_analysis;  // Week 6: Advanced Drift Analysis

pub use selector::{Selector, PredicateEvaluator, RuleSelector};
pub use planner::{Planner, ExecutionPlan, PlanStep};
pub use enforcer::{Enforcer, EnforceOptions, Verdict, VerdictPair, DriftTracker, DriftStats};
pub use drift_analysis::{DriftAnalyzer, DriftEvent, DriftRingBuffer, DriftStats as DriftAnalysisStats};

use crate::policy_v2::types::IrV1;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Orchestrator context - runtime data for predicate evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorContext {
    /// Supplier hashes
    pub supplier_hashes: Vec<String>,

    /// UBO hashes
    pub ubo_hashes: Vec<String>,

    /// Company commitment root
    pub company_commitment_root: Option<String>,

    /// Sanctions list root
    pub sanctions_root: Option<String>,

    /// Jurisdiction list root
    pub jurisdiction_root: Option<String>,

    /// Additional runtime variables (e.g., "now", "audit_dates")
    pub variables: std::collections::HashMap<String, serde_json::Value>,
}

/// Main orchestrator - coordinates selector and planner
pub struct Orchestrator {
    selector: Selector,
    planner: Planner,
}

impl Orchestrator {
    /// Creates a new orchestrator from IR
    pub fn new(ir: &IrV1) -> Result<Self> {
        Ok(Self {
            selector: Selector::new(ir)?,
            planner: Planner::new(ir)?,
        })
    }

    /// Orchestrates proof generation: select rules → plan execution
    pub fn orchestrate(&self, context: &OrchestratorContext) -> Result<ExecutionPlan> {
        // Phase 1: Selector evaluates predicates and determines active rules
        let active_rules = self.selector.select_active_rules(context)?;

        // Phase 2: Planner creates deterministic execution plan
        let plan = self.planner.plan(&active_rules)?;

        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy_v2::types::{IrV1, IrRule, IrExpression};

    #[test]
    fn test_orchestrator_no_adaptivity() {
        let ir = IrV1 {
            ir_version: "1.0".to_string(),
            policy_id: "test.v1".to_string(),
            policy_hash: "sha3-256:abc".to_string(),
            rules: vec![
                IrRule {
                    id: "rule1".to_string(),
                    op: "eq".to_string(),
                    lhs: IrExpression::Var { var: "x".to_string() },
                    rhs: IrExpression::Literal(serde_json::json!(1)),
                },
            ],
            adaptivity: None,
            ir_hash: "sha3-256:def".to_string(),
        };

        let orch = Orchestrator::new(&ir).unwrap();
        let ctx = OrchestratorContext {
            supplier_hashes: vec![],
            ubo_hashes: vec![],
            company_commitment_root: None,
            sanctions_root: None,
            jurisdiction_root: None,
            variables: std::collections::HashMap::new(),
        };

        let plan = orch.orchestrate(&ctx).unwrap();
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].rule_id, "rule1");
    }
}
