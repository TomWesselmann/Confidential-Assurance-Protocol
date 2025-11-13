/// Integration tests for Adaptive Proof Orchestrator (Week 5 Track B1)
///
/// Tests the end-to-end orchestration flow:
/// - Predicate evaluation
/// - Rule selection
/// - Deterministic planning
/// - Cost-based ordering
use cap_agent::orchestrator::{Orchestrator, OrchestratorContext};
use cap_agent::policy_v2::types::{
    Activation, IrAdaptivity, IrExpression, IrPredicate, IrRule, IrV1,
};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_orchestrator_no_adaptivity_all_rules_active() {
    // IR without adaptivity → all rules should be active
    let ir = IrV1 {
        ir_version: "1.0".to_string(),
        policy_id: "test.v1".to_string(),
        policy_hash: "sha3-256:abc123".to_string(),
        rules: vec![
            IrRule {
                id: "check_sanctions".to_string(),
                op: "non_membership".to_string(),
                lhs: IrExpression::Var {
                    var: "supplier_hash".to_string(),
                },
                rhs: IrExpression::Var {
                    var: "sanctions_root".to_string(),
                },
            },
            IrRule {
                id: "check_age".to_string(),
                op: "range_min".to_string(),
                lhs: IrExpression::Var {
                    var: "age".to_string(),
                },
                rhs: IrExpression::Literal(json!(18)),
            },
            IrRule {
                id: "check_status".to_string(),
                op: "eq".to_string(),
                lhs: IrExpression::Var {
                    var: "status".to_string(),
                },
                rhs: IrExpression::Literal(json!("active")),
            },
        ],
        adaptivity: None,
        ir_hash: "sha3-256:def456".to_string(),
    };

    let orchestrator = Orchestrator::new(&ir).unwrap();

    let context = OrchestratorContext {
        supplier_hashes: vec![],
        ubo_hashes: vec![],
        company_commitment_root: None,
        sanctions_root: None,
        jurisdiction_root: None,
        variables: HashMap::new(),
    };

    let plan = orchestrator.orchestrate(&context).unwrap();

    // All 3 rules should be active
    assert_eq!(plan.steps.len(), 3);
    assert_eq!(plan.metadata.active_rules, 3);

    // Verify cost-based ordering:
    // 1. check_status (eq, cost=1)
    // 2. check_age (range_min, cost=2)
    // 3. check_sanctions (non_membership, cost=10)
    assert_eq!(plan.steps[0].rule_id, "check_status");
    assert_eq!(plan.steps[0].cost, 1);
    assert_eq!(plan.steps[0].step_index, 0);

    assert_eq!(plan.steps[1].rule_id, "check_age");
    assert_eq!(plan.steps[1].cost, 2);
    assert_eq!(plan.steps[1].step_index, 1);

    assert_eq!(plan.steps[2].rule_id, "check_sanctions");
    assert_eq!(plan.steps[2].cost, 10);
    assert_eq!(plan.steps[2].step_index, 2);

    assert_eq!(plan.total_cost, 13); // 1 + 2 + 10
}

#[test]
fn test_orchestrator_with_adaptivity_predicate_true() {
    // IR with adaptivity: when "is_high_risk" is true, activate extra checks
    let ir = IrV1 {
        ir_version: "1.0".to_string(),
        policy_id: "adaptive_policy.v1".to_string(),
        policy_hash: "sha3-256:adaptive_hash".to_string(),
        rules: vec![
            IrRule {
                id: "baseline_check".to_string(),
                op: "eq".to_string(),
                lhs: IrExpression::Var {
                    var: "status".to_string(),
                },
                rhs: IrExpression::Literal(json!("active")),
            },
            IrRule {
                id: "sanctions_check".to_string(),
                op: "non_membership".to_string(),
                lhs: IrExpression::Var {
                    var: "hash".to_string(),
                },
                rhs: IrExpression::Var {
                    var: "sanctions_root".to_string(),
                },
            },
            IrRule {
                id: "jurisdiction_check".to_string(),
                op: "non_membership".to_string(),
                lhs: IrExpression::Var {
                    var: "hash".to_string(),
                },
                rhs: IrExpression::Var {
                    var: "jurisdiction_root".to_string(),
                },
            },
        ],
        adaptivity: Some(IrAdaptivity {
            predicates: vec![IrPredicate {
                id: "is_high_risk".to_string(),
                expr: json!(true), // Always true for this test
            }],
            activations: vec![Activation {
                when: "is_high_risk".to_string(),
                rules: vec![
                    "sanctions_check".to_string(),
                    "jurisdiction_check".to_string(),
                ],
            }],
        }),
        ir_hash: "sha3-256:ir_hash".to_string(),
    };

    let orchestrator = Orchestrator::new(&ir).unwrap();

    let context = OrchestratorContext {
        supplier_hashes: vec![],
        ubo_hashes: vec![],
        company_commitment_root: None,
        sanctions_root: Some("0xsanctions".to_string()),
        jurisdiction_root: Some("0xjurisdictions".to_string()),
        variables: HashMap::new(),
    };

    let plan = orchestrator.orchestrate(&context).unwrap();

    // Predicate is true → should activate 2 rules
    assert_eq!(plan.steps.len(), 2);
    assert_eq!(plan.metadata.active_rules, 2);

    // Both rules have same cost (non_membership=10) → sorted by rule_id
    assert_eq!(plan.steps[0].rule_id, "jurisdiction_check");
    assert_eq!(plan.steps[0].cost, 10);

    assert_eq!(plan.steps[1].rule_id, "sanctions_check");
    assert_eq!(plan.steps[1].cost, 10);

    assert_eq!(plan.total_cost, 20); // 10 + 10
}

#[test]
fn test_orchestrator_with_adaptivity_predicate_false() {
    // IR with adaptivity: when "is_high_risk" is false, no rules activated
    let ir = IrV1 {
        ir_version: "1.0".to_string(),
        policy_id: "adaptive_policy.v1".to_string(),
        policy_hash: "sha3-256:adaptive_hash".to_string(),
        rules: vec![IrRule {
            id: "sanctions_check".to_string(),
            op: "non_membership".to_string(),
            lhs: IrExpression::Var {
                var: "hash".to_string(),
            },
            rhs: IrExpression::Var {
                var: "sanctions_root".to_string(),
            },
        }],
        adaptivity: Some(IrAdaptivity {
            predicates: vec![IrPredicate {
                id: "is_high_risk".to_string(),
                expr: json!(false), // False → no activation
            }],
            activations: vec![Activation {
                when: "is_high_risk".to_string(),
                rules: vec!["sanctions_check".to_string()],
            }],
        }),
        ir_hash: "sha3-256:ir_hash".to_string(),
    };

    let orchestrator = Orchestrator::new(&ir).unwrap();

    let context = OrchestratorContext {
        supplier_hashes: vec![],
        ubo_hashes: vec![],
        company_commitment_root: None,
        sanctions_root: Some("0xsanctions".to_string()),
        jurisdiction_root: None,
        variables: HashMap::new(),
    };

    let plan = orchestrator.orchestrate(&context).unwrap();

    // Predicate is false → no rules activated
    assert_eq!(plan.steps.len(), 0);
    assert_eq!(plan.metadata.active_rules, 0);
    assert_eq!(plan.total_cost, 0);
}

#[test]
fn test_orchestrator_with_variable_predicate() {
    // Predicate evaluation based on context variables
    let ir = IrV1 {
        ir_version: "1.0".to_string(),
        policy_id: "variable_adaptive.v1".to_string(),
        policy_hash: "sha3-256:var_hash".to_string(),
        rules: vec![
            IrRule {
                id: "age_check".to_string(),
                op: "range_min".to_string(),
                lhs: IrExpression::Var {
                    var: "age".to_string(),
                },
                rhs: IrExpression::Literal(json!(18)),
            },
            IrRule {
                id: "enhanced_check".to_string(),
                op: "eq".to_string(),
                lhs: IrExpression::Var {
                    var: "verified".to_string(),
                },
                rhs: IrExpression::Literal(json!(true)),
            },
        ],
        adaptivity: Some(IrAdaptivity {
            predicates: vec![IrPredicate {
                id: "is_young".to_string(),
                expr: json!({
                    "func": "lt",
                    "args": ["age", 25]
                }),
            }],
            activations: vec![Activation {
                when: "is_young".to_string(),
                rules: vec!["enhanced_check".to_string()],
            }],
        }),
        ir_hash: "sha3-256:ir_hash".to_string(),
    };

    let orchestrator = Orchestrator::new(&ir).unwrap();

    // Case 1: age < 25 → predicate true → enhanced check activated
    let mut variables1 = HashMap::new();
    variables1.insert("age".to_string(), json!(20));

    let context1 = OrchestratorContext {
        supplier_hashes: vec![],
        ubo_hashes: vec![],
        company_commitment_root: None,
        sanctions_root: None,
        jurisdiction_root: None,
        variables: variables1,
    };

    let plan1 = orchestrator.orchestrate(&context1).unwrap();
    assert_eq!(plan1.steps.len(), 1);
    assert_eq!(plan1.steps[0].rule_id, "enhanced_check");

    // Case 2: age >= 25 → predicate false → no rules activated
    let mut variables2 = HashMap::new();
    variables2.insert("age".to_string(), json!(30));

    let context2 = OrchestratorContext {
        supplier_hashes: vec![],
        ubo_hashes: vec![],
        company_commitment_root: None,
        sanctions_root: None,
        jurisdiction_root: None,
        variables: variables2,
    };

    let plan2 = orchestrator.orchestrate(&context2).unwrap();
    assert_eq!(plan2.steps.len(), 0);
}

#[test]
fn test_orchestrator_deterministic_ordering() {
    // Test that rules with same cost are ordered deterministically by rule_id
    let ir = IrV1 {
        ir_version: "1.0".to_string(),
        policy_id: "deterministic.v1".to_string(),
        policy_hash: "sha3-256:det_hash".to_string(),
        rules: vec![
            IrRule {
                id: "z_rule".to_string(),
                op: "eq".to_string(),
                lhs: IrExpression::Var {
                    var: "x".to_string(),
                },
                rhs: IrExpression::Literal(json!(1)),
            },
            IrRule {
                id: "a_rule".to_string(),
                op: "eq".to_string(),
                lhs: IrExpression::Var {
                    var: "y".to_string(),
                },
                rhs: IrExpression::Literal(json!(2)),
            },
            IrRule {
                id: "m_rule".to_string(),
                op: "eq".to_string(),
                lhs: IrExpression::Var {
                    var: "z".to_string(),
                },
                rhs: IrExpression::Literal(json!(3)),
            },
        ],
        adaptivity: None,
        ir_hash: "sha3-256:ir_hash".to_string(),
    };

    let orchestrator = Orchestrator::new(&ir).unwrap();

    let context = OrchestratorContext {
        supplier_hashes: vec![],
        ubo_hashes: vec![],
        company_commitment_root: None,
        sanctions_root: None,
        jurisdiction_root: None,
        variables: HashMap::new(),
    };

    let plan = orchestrator.orchestrate(&context).unwrap();

    // All rules have same cost (eq=1) → should be sorted lexicographically
    assert_eq!(plan.steps.len(), 3);
    assert_eq!(plan.steps[0].rule_id, "a_rule");
    assert_eq!(plan.steps[1].rule_id, "m_rule");
    assert_eq!(plan.steps[2].rule_id, "z_rule");
}

#[test]
fn test_orchestrator_mixed_costs() {
    // Test complex scenario with mixed operation costs
    let ir = IrV1 {
        ir_version: "1.0".to_string(),
        policy_id: "mixed_costs.v1".to_string(),
        policy_hash: "sha3-256:mixed_hash".to_string(),
        rules: vec![
            IrRule {
                id: "expensive_check".to_string(),
                op: "non_membership".to_string(), // cost=10
                lhs: IrExpression::Var {
                    var: "hash".to_string(),
                },
                rhs: IrExpression::Var {
                    var: "root".to_string(),
                },
            },
            IrRule {
                id: "cheap_check_1".to_string(),
                op: "eq".to_string(), // cost=1
                lhs: IrExpression::Var {
                    var: "a".to_string(),
                },
                rhs: IrExpression::Literal(json!(1)),
            },
            IrRule {
                id: "medium_check".to_string(),
                op: "range_min".to_string(), // cost=2
                lhs: IrExpression::Var {
                    var: "age".to_string(),
                },
                rhs: IrExpression::Literal(json!(18)),
            },
            IrRule {
                id: "cheap_check_2".to_string(),
                op: "eq".to_string(), // cost=1
                lhs: IrExpression::Var {
                    var: "b".to_string(),
                },
                rhs: IrExpression::Literal(json!(2)),
            },
            IrRule {
                id: "very_expensive".to_string(),
                op: "threshold".to_string(), // cost=20
                lhs: IrExpression::Var {
                    var: "count".to_string(),
                },
                rhs: IrExpression::Literal(json!(0.8)),
            },
        ],
        adaptivity: None,
        ir_hash: "sha3-256:ir_hash".to_string(),
    };

    let orchestrator = Orchestrator::new(&ir).unwrap();

    let context = OrchestratorContext {
        supplier_hashes: vec![],
        ubo_hashes: vec![],
        company_commitment_root: None,
        sanctions_root: None,
        jurisdiction_root: None,
        variables: HashMap::new(),
    };

    let plan = orchestrator.orchestrate(&context).unwrap();

    assert_eq!(plan.steps.len(), 5);

    // Expected ordering:
    // 1. cheap_check_1 (eq, cost=1, id=cheap_check_1)
    // 2. cheap_check_2 (eq, cost=1, id=cheap_check_2)
    // 3. medium_check (range_min, cost=2)
    // 4. expensive_check (non_membership, cost=10)
    // 5. very_expensive (threshold, cost=20)
    assert_eq!(plan.steps[0].rule_id, "cheap_check_1");
    assert_eq!(plan.steps[0].cost, 1);

    assert_eq!(plan.steps[1].rule_id, "cheap_check_2");
    assert_eq!(plan.steps[1].cost, 1);

    assert_eq!(plan.steps[2].rule_id, "medium_check");
    assert_eq!(plan.steps[2].cost, 2);

    assert_eq!(plan.steps[3].rule_id, "expensive_check");
    assert_eq!(plan.steps[3].cost, 10);

    assert_eq!(plan.steps[4].rule_id, "very_expensive");
    assert_eq!(plan.steps[4].cost, 20);

    assert_eq!(plan.total_cost, 34); // 1 + 1 + 2 + 10 + 20
}
