//! # Orchestrator Enforce Integration Tests (Week 6 - B2)
//!
//! End-to-End Tests für Orchestrator + Enforcer + Drift Analysis

use cap_agent::orchestrator::{
    DriftAnalyzer, EnforceOptions, Enforcer, OrchestratorContext, Verdict, VerdictPair,
};
use cap_agent::policy_v2::types::{IrExpression, IrRule, IrV1};
use std::collections::HashMap;
use std::time::Duration;

/// Helper: Erstellt Test-IR mit mehreren Regeln
fn create_test_ir_multi_rules() -> IrV1 {
    IrV1 {
        ir_version: "1.0".to_string(),
        policy_id: "test_enforce.v1".to_string(),
        policy_hash: "sha3-256:0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
            .to_string(),
        rules: vec![
            IrRule {
                id: "supplier_check".to_string(),
                op: "eq".to_string(),
                lhs: IrExpression::Var {
                    var: "supplier_hash".to_string(),
                },
                rhs: IrExpression::Literal(serde_json::Value::String("0xabc".to_string())),
            },
            IrRule {
                id: "ubo_check".to_string(),
                op: "ne".to_string(),
                lhs: IrExpression::Var {
                    var: "ubo_hash".to_string(),
                },
                rhs: IrExpression::Literal(serde_json::Value::String("0xsanctioned".to_string())),
            },
        ],
        adaptivity: None,
        ir_hash: "sha3-256:def".to_string(),
    }
}

/// Helper: Erstellt Test-Context
fn create_test_context() -> OrchestratorContext {
    OrchestratorContext {
        supplier_hashes: vec!["0xabc".to_string()],
        ubo_hashes: vec!["0xgood".to_string()],
        company_commitment_root: Some("0x123".to_string()),
        sanctions_root: None,
        jurisdiction_root: None,
        variables: HashMap::new(),
    }
}

#[test]
fn test_orchestrator_enforce_end_to_end_shadow() {
    // Test: Vollständiger Shadow-Mode-Durchlauf
    let ir = create_test_ir_multi_rules();
    let ctx = create_test_context();

    let opts = EnforceOptions {
        enforce: false,
        rollout_percent: 0,
        drift_max_ratio: 0.005,
    };

    let enforcer = Enforcer::new(&ir, opts).unwrap();
    let result = enforcer.decide(&ctx, "test-request-shadow").unwrap();

    // Shadow-Modus: Enforcement sollte nicht angewendet werden
    assert!(!result.enforced_applied);
    assert_eq!(result.shadow, result.enforced); // Beide sollten gleich sein
    assert!(!result.has_drift()); // Kein Drift in Shadow-Modus
}

#[test]
fn test_orchestrator_enforce_end_to_end_full_rollout() {
    // Test: Vollständiger Enforce-Modus (100% Rollout)
    let ir = create_test_ir_multi_rules();
    let ctx = create_test_context();

    let opts = EnforceOptions {
        enforce: true,
        rollout_percent: 100,
        drift_max_ratio: 0.005,
    };

    let enforcer = Enforcer::new(&ir, opts).unwrap();
    let result = enforcer.decide(&ctx, "test-request-enforced").unwrap();

    // Enforce-Modus: Enforcement sollte angewendet werden
    assert!(result.enforced_applied);
    assert!(result.shadow.is_success());
    assert!(result.enforced.is_success());
}

#[test]
fn test_orchestrator_enforce_with_drift_tracking() {
    // Test: Enforcement mit Drift-Tracking über DriftAnalyzer
    let ir = create_test_ir_multi_rules();
    let ctx = create_test_context();

    let opts = EnforceOptions {
        enforce: true,
        rollout_percent: 100,
        drift_max_ratio: 0.005,
    };

    let enforcer = Enforcer::new(&ir, opts).unwrap();
    let mut drift_analyzer = DriftAnalyzer::default();

    // Simuliere mehrere Requests
    for i in 0..10 {
        let result = enforcer.decide(&ctx, &format!("request-{}", i)).unwrap();
        drift_analyzer.record_verdict_pair(&result, ir.policy_id.clone(), format!("request-{}", i));
    }

    // Prüfe Drift-Statistiken
    let stats = drift_analyzer.stats_5m();
    assert_eq!(stats.total_events, 10);

    // In diesem Test sollte kein Drift auftreten (alles Ok)
    assert_eq!(stats.drift_events, 0);
    assert_eq!(stats.drift_ratio, 0.0);
}

#[test]
fn test_orchestrator_enforce_canary_rollout() {
    // Test: Canary Rollout (25%)
    let ir = create_test_ir_multi_rules();
    let ctx = create_test_context();

    let opts = EnforceOptions {
        enforce: true,
        rollout_percent: 25,
        drift_max_ratio: 0.005,
    };

    let enforcer = Enforcer::new(&ir, opts).unwrap();
    let mut drift_analyzer = DriftAnalyzer::default();

    // Simuliere 100 Requests
    let mut enforced_count = 0;
    for i in 0..100 {
        let result = enforcer.decide(&ctx, &format!("canary-{}", i)).unwrap();
        drift_analyzer.record_verdict_pair(&result, ir.policy_id.clone(), format!("canary-{}", i));

        if result.enforced_applied {
            enforced_count += 1;
        }
    }

    // Bei 25% Rollout erwarten wir ca. 25 von 100 (mit Toleranz)
    assert!(
        (10..=40).contains(&enforced_count),
        "Expected ~25% rollout, got {}%",
        enforced_count
    );

    // Alle Requests sollten im Drift-Analyzer sein
    let stats = drift_analyzer.stats_5m();
    assert_eq!(stats.total_events, 100);
}

#[test]
fn test_orchestrator_enforce_drift_detection() {
    // Test: Drift-Detection mit simuliertem Unterschied Shadow/Enforced
    let mut drift_analyzer = DriftAnalyzer::default();

    // Simuliere Drift-Szenarien
    for i in 0..100 {
        let pair = if i < 5 {
            // 5% Drift: Shadow Ok, Enforced Fail
            VerdictPair {
                shadow: Verdict::Ok,
                enforced: Verdict::Fail,
                enforced_applied: true,
            }
        } else {
            // Kein Drift
            VerdictPair {
                shadow: Verdict::Ok,
                enforced: Verdict::Ok,
                enforced_applied: true,
            }
        };

        drift_analyzer.record_verdict_pair(&pair, "test.v1".to_string(), format!("req-{}", i));
    }

    let stats = drift_analyzer.stats_5m();
    assert_eq!(stats.total_events, 100);
    assert_eq!(stats.drift_events, 5);
    assert_eq!(stats.drift_ratio, 0.05); // 5%

    // Drift überschreitet 1% Threshold
    assert!(drift_analyzer.exceeds_threshold(0.01));
    // Drift überschreitet nicht 10% Threshold
    assert!(!drift_analyzer.exceeds_threshold(0.10));
}

#[test]
fn test_orchestrator_enforce_rolling_window() {
    // Test: Rolling Window Drift-Tracking
    let mut drift_analyzer = DriftAnalyzer::new(
        1000,
        Duration::from_secs(60), // 1 minute max age
        Duration::from_secs(30), // 30 second window
    );

    // Add events
    for i in 0..50 {
        let pair = VerdictPair {
            shadow: Verdict::Ok,
            enforced: if i % 10 == 0 {
                Verdict::Fail
            } else {
                Verdict::Ok
            },
            enforced_applied: true,
        };
        drift_analyzer.record_verdict_pair(&pair, "test.v1".to_string(), format!("req-{}", i));
    }

    // Custom window query
    let drift_ratio_30s = drift_analyzer.drift_ratio_custom(Duration::from_secs(30));
    assert!(drift_ratio_30s > 0.0); // Should have some drift

    let drift_events = drift_analyzer.drift_events_5m();
    assert_eq!(drift_events.len(), 5); // 5 drift events (i % 10 == 0)
}

#[test]
fn test_orchestrator_enforce_empty_buffer() {
    // Test: Verhalten bei leerem Drift-Analyzer
    let drift_analyzer = DriftAnalyzer::default();

    assert_eq!(drift_analyzer.drift_ratio_5m(), 0.0);
    assert!(!drift_analyzer.exceeds_threshold(0.005));
    assert_eq!(drift_analyzer.buffer_size(), 0);

    let stats = drift_analyzer.stats_5m();
    assert_eq!(stats.total_events, 0);
    assert_eq!(stats.drift_events, 0);
    assert_eq!(stats.drift_ratio, 0.0);
}

#[test]
fn test_orchestrator_enforce_request_rate() {
    // Test: Request-Rate-Berechnung im Drift-Analyzer
    let mut drift_analyzer = DriftAnalyzer::default();

    // Add multiple events quickly
    for i in 0..100 {
        let pair = VerdictPair {
            shadow: Verdict::Ok,
            enforced: Verdict::Ok,
            enforced_applied: true,
        };
        drift_analyzer.record_verdict_pair(&pair, "test.v1".to_string(), format!("req-{}", i));
    }

    let request_rate = drift_analyzer.request_rate_5m();
    // Request rate sollte > 0 sein (alle Events im Fenster)
    assert!(request_rate > 0.0, "Request rate should be positive");
}

#[test]
fn test_orchestrator_enforce_determinism() {
    // Test: Deterministisches Verhalten bei gleichen Inputs
    let ir = create_test_ir_multi_rules();
    let ctx = create_test_context();

    let opts = EnforceOptions {
        enforce: true,
        rollout_percent: 50,
        drift_max_ratio: 0.005,
    };

    let enforcer = Enforcer::new(&ir, opts).unwrap();

    // Gleiche request_id sollte immer gleiches Ergebnis liefern
    let request_id = "deterministic-test-456";
    let result1 = enforcer.decide(&ctx, request_id).unwrap();
    let result2 = enforcer.decide(&ctx, request_id).unwrap();
    let result3 = enforcer.decide(&ctx, request_id).unwrap();

    assert_eq!(result1.enforced_applied, result2.enforced_applied);
    assert_eq!(result2.enforced_applied, result3.enforced_applied);
    assert_eq!(result1.shadow, result2.shadow);
    assert_eq!(result1.enforced, result2.enforced);
}
