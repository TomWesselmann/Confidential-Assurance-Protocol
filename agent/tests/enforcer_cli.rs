//! # Enforcer CLI Tests (Week 6 - B1)
//!
//! Tests für CLI-Integration des Adaptive Enforcer.

use cap_agent::orchestrator::{EnforceOptions, Enforcer, OrchestratorContext};
use cap_agent::policy_v2::types::{IrExpression, IrRule, IrV1};
use std::collections::HashMap;

/// Helper: Erstellt Test-IR mit minimalen Regeln
fn create_test_ir() -> IrV1 {
    IrV1 {
        ir_version: "1.0".to_string(),
        policy_id: "test.v1".to_string(),
        policy_hash: "sha3-256:0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
            .to_string(),
        rules: vec![IrRule {
            id: "rule1".to_string(),
            op: "eq".to_string(),
            lhs: IrExpression::Var {
                var: "supplier_hash".to_string(),
            },
            rhs: IrExpression::Literal(serde_json::Value::String("0xabc".to_string())),
        }],
        adaptivity: None,
        ir_hash: "sha3-256:def".to_string(),
    }
}

/// Helper: Erstellt Test-Context
fn create_test_context() -> OrchestratorContext {
    OrchestratorContext {
        supplier_hashes: vec!["0xabc".to_string()],
        ubo_hashes: vec![],
        company_commitment_root: Some("0x123".to_string()),
        sanctions_root: None,
        jurisdiction_root: None,
        variables: HashMap::new(),
    }
}

#[test]
fn test_defaults_shadow_mode() {
    // Test: Default-Optionen entsprechen Shadow-Mode
    let opts = EnforceOptions::default();

    assert!(!opts.enforce, "Default enforce should be false");
    assert_eq!(opts.rollout_percent, 0, "Default rollout should be 0");
    assert_eq!(
        opts.drift_max_ratio, 0.005,
        "Default drift_max should be 0.005"
    );
}

#[test]
fn test_parse_enforce_rollout_drift() {
    // Test: Flags werden korrekt in EnforceOptions gemappt
    let ir = create_test_ir();
    let ctx = create_test_context();

    // Shadow-Only (enforce=false, rollout=0)
    let opts_shadow = EnforceOptions {
        enforce: false,
        rollout_percent: 0,
        drift_max_ratio: 0.005,
    };
    let enforcer_shadow = Enforcer::new(&ir, opts_shadow).unwrap();
    let result = enforcer_shadow.decide(&ctx, "test-request-123").unwrap();
    assert!(
        !result.enforced_applied,
        "Shadow mode should not apply enforcement"
    );

    // Canary (enforce=true, rollout=25)
    let opts_canary = EnforceOptions {
        enforce: true,
        rollout_percent: 25,
        drift_max_ratio: 0.005,
    };
    let _enforcer_canary = Enforcer::new(&ir, opts_canary).unwrap();
    // Note: Mit rollout=25 kann es sein, dass enforcement angewendet wird oder nicht
    // Das hängt vom Hash der request_id ab

    // Full (enforce=true, rollout=100)
    let opts_full = EnforceOptions {
        enforce: true,
        rollout_percent: 100,
        drift_max_ratio: 0.005,
    };
    let enforcer_full = Enforcer::new(&ir, opts_full).unwrap();
    let result_full = enforcer_full.decide(&ctx, "test-request-456").unwrap();
    assert!(
        result_full.enforced_applied,
        "Full rollout should always apply enforcement"
    );
}

#[test]
fn test_deterministic_sampling() {
    // Test: Für gleiche request_id wird immer die gleiche Enforcement-Entscheidung getroffen
    let ir = create_test_ir();
    let ctx = create_test_context();

    let opts = EnforceOptions {
        enforce: true,
        rollout_percent: 50, // 50% rollout
        drift_max_ratio: 0.005,
    };

    let enforcer = Enforcer::new(&ir, opts).unwrap();

    // Gleiche request_id sollte immer gleiche Entscheidung liefern
    let request_id = "deterministic-test-123";
    let result1 = enforcer.decide(&ctx, request_id).unwrap();
    let result2 = enforcer.decide(&ctx, request_id).unwrap();
    let result3 = enforcer.decide(&ctx, request_id).unwrap();

    assert_eq!(
        result1.enforced_applied, result2.enforced_applied,
        "Same request_id should produce same enforcement decision (result1 vs result2)"
    );
    assert_eq!(
        result2.enforced_applied, result3.enforced_applied,
        "Same request_id should produce same enforcement decision (result2 vs result3)"
    );

    // Verschiedene request_ids können unterschiedliche Entscheidungen haben
    let result_a = enforcer.decide(&ctx, "request-a").unwrap();
    let result_b = enforcer.decide(&ctx, "request-b").unwrap();
    // Wir testen nur Determinismus, nicht dass sie unterschiedlich sind
    // (könnte zufällig gleich sein bei 50% rollout)

    // Aber jede request_id sollte konsistent sein
    let result_a2 = enforcer.decide(&ctx, "request-a").unwrap();
    let result_b2 = enforcer.decide(&ctx, "request-b").unwrap();
    assert_eq!(result_a.enforced_applied, result_a2.enforced_applied);
    assert_eq!(result_b.enforced_applied, result_b2.enforced_applied);
}

#[test]
fn test_rollout_percentage_distribution() {
    // Test: Rollout-Prozentsatz führt zu ungefähr korrekter Verteilung
    let ir = create_test_ir();
    let ctx = create_test_context();

    let opts = EnforceOptions {
        enforce: true,
        rollout_percent: 25, // 25% rollout
        drift_max_ratio: 0.005,
    };

    let enforcer = Enforcer::new(&ir, opts).unwrap();

    // Teste mit 100 verschiedenen request_ids
    let mut enforced_count = 0;
    for i in 0..100 {
        let result = enforcer.decide(&ctx, &format!("request-{}", i)).unwrap();
        if result.enforced_applied {
            enforced_count += 1;
        }
    }

    // Bei 25% rollout erwarten wir ca. 25 von 100
    // Mit ±15% Toleranz (10-40) wegen Hash-basiertem Sampling
    assert!(
        (10..=40).contains(&enforced_count),
        "Expected ~25% rollout, got {}%",
        enforced_count
    );
}

#[test]
fn test_enforce_disabled_overrides_rollout() {
    // Test: enforce=false überschreibt rollout-Einstellung
    let ir = create_test_ir();
    let ctx = create_test_context();

    let opts = EnforceOptions {
        enforce: false,       // Enforcement disabled
        rollout_percent: 100, // Aber 100% rollout
        drift_max_ratio: 0.005,
    };

    let enforcer = Enforcer::new(&ir, opts).unwrap();
    let result = enforcer.decide(&ctx, "test-request").unwrap();

    assert!(
        !result.enforced_applied,
        "enforce=false should override rollout_percent"
    );
}

#[test]
fn test_drift_max_ratio_option() {
    // Test: drift_max_ratio Option wird korrekt gesetzt
    let ir = create_test_ir();

    let opts_default = EnforceOptions {
        enforce: true,
        rollout_percent: 100,
        drift_max_ratio: 0.005, // Default 0.5%
    };

    let opts_custom = EnforceOptions {
        enforce: true,
        rollout_percent: 100,
        drift_max_ratio: 0.01, // Custom 1%
    };

    let enforcer_default = Enforcer::new(&ir, opts_default.clone()).unwrap();
    let enforcer_custom = Enforcer::new(&ir, opts_custom.clone()).unwrap();

    assert_eq!(enforcer_default.options().drift_max_ratio, 0.005);
    assert_eq!(enforcer_custom.options().drift_max_ratio, 0.01);
}
