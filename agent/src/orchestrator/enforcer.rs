//! # Adaptive Enforcer (Week 6 - v0.2)
//!
//! Implements progressive enforcement mode with shadow/enforced verdict pairs.
//! Supports gradual rollout (0% → 25% → 100%) with drift monitoring.

use crate::orchestrator::{Orchestrator, OrchestratorContext};
use crate::policy_v2::types::IrV1;
use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Enforcement options for adaptive orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforceOptions {
    /// Enable enforcement mode (if false, shadow-only)
    pub enforce: bool,

    /// Rollout percentage (0-100)
    /// - 0%: Shadow only, no enforcement
    /// - 25%: 25% of requests use enforced path
    /// - 100%: All requests use enforced path
    pub rollout_percent: u8,

    /// Maximum allowed drift ratio (e.g., 0.005 = 0.5%)
    /// Drift = percentage of requests where shadow != enforced verdict
    pub drift_max_ratio: f64,
}

impl Default for EnforceOptions {
    fn default() -> Self {
        Self {
            enforce: false,
            rollout_percent: 0,
            drift_max_ratio: 0.005, // 0.5% max drift
        }
    }
}

/// Verdict result (simplified for enforcer)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict {
    /// Verification passed
    Ok,
    /// Verification passed with warnings
    Warn,
    /// Verification failed
    Fail,
}

impl Verdict {
    /// Convert string result to Verdict
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s {
            "ok" => Verdict::Ok,
            "warn" => Verdict::Warn,
            "fail" => Verdict::Fail,
            _ => Verdict::Fail,
        }
    }

    /// Check if verdict is successful (Ok or Warn)
    pub fn is_success(&self) -> bool {
        matches!(self, Verdict::Ok | Verdict::Warn)
    }
}

/// Verdict pair (shadow + enforced)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerdictPair {
    /// Shadow verdict (always computed)
    pub shadow: Verdict,

    /// Enforced verdict (computed only if rollout applies)
    pub enforced: Verdict,

    /// Whether enforcement was applied for this request
    pub enforced_applied: bool,
}

impl VerdictPair {
    /// Check if drift occurred (shadow != enforced)
    pub fn has_drift(&self) -> bool {
        self.enforced_applied && self.shadow != self.enforced
    }
}

/// Enforcement decision engine
pub struct Enforcer {
    /// Orchestrator for plan execution
    orchestrator: Orchestrator,

    /// Enforcement options
    options: EnforceOptions,
}

impl Enforcer {
    /// Create new enforcer with IR and options
    pub fn new(ir: &IrV1, options: EnforceOptions) -> Result<Self> {
        let orchestrator = Orchestrator::new(ir)?;

        Ok(Self {
            orchestrator,
            options,
        })
    }

    /// Decide enforcement for a request
    ///
    /// # Logic
    /// 1. Shadow: Always compute plan + result (no enforcement)
    /// 2. Enforce: If rollout_percent applies, compute enforced plan + result
    /// 3. Return VerdictPair with drift flag
    ///
    /// # Rollout
    /// - 0%: enforced_applied = false (shadow only)
    /// - 25%: enforced_applied = true for 25% of requests (random sampling)
    /// - 100%: enforced_applied = true for all requests
    pub fn decide(&self, ctx: &OrchestratorContext, request_id: &str) -> Result<VerdictPair> {
        // Step 1: Shadow execution (always)
        let shadow_verdict = self.compute_shadow_verdict(ctx)?;

        // Step 2: Determine if enforcement should be applied
        let should_enforce = self.should_enforce(request_id);

        // Step 3: Enforced execution (conditional)
        let (enforced_verdict, enforced_applied) = if should_enforce {
            let verdict = self.compute_enforced_verdict(ctx)?;
            (verdict, true)
        } else {
            // No enforcement, return shadow as enforced
            (shadow_verdict.clone(), false)
        };

        Ok(VerdictPair {
            shadow: shadow_verdict,
            enforced: enforced_verdict,
            enforced_applied,
        })
    }

    /// Compute shadow verdict (dry-run mode)
    fn compute_shadow_verdict(&self, ctx: &OrchestratorContext) -> Result<Verdict> {
        // Use orchestrator to compute plan
        let plan = self
            .orchestrator
            .orchestrate(ctx)
            .context("Failed to compute shadow plan")?;

        // Simulate verification (in real implementation, this would execute ZK proofs)
        // For now, return Ok if plan has steps
        if !plan.steps.is_empty() {
            Ok(Verdict::Ok)
        } else {
            Ok(Verdict::Warn)
        }
    }

    /// Compute enforced verdict (active enforcement)
    fn compute_enforced_verdict(&self, ctx: &OrchestratorContext) -> Result<Verdict> {
        // Use orchestrator to compute plan (same as shadow for now)
        let plan = self
            .orchestrator
            .orchestrate(ctx)
            .context("Failed to compute enforced plan")?;

        // In production, this would execute the plan actively
        // For now, return Ok if plan has steps
        if !plan.steps.is_empty() {
            Ok(Verdict::Ok)
        } else {
            Ok(Verdict::Fail)
        }
    }

    /// Determine if enforcement should be applied for this request
    ///
    /// # Sampling Strategy
    /// - Use deterministic hash-based sampling (consistent per request_id)
    /// - Hash(request_id) % 100 < rollout_percent → enforce
    fn should_enforce(&self, request_id: &str) -> bool {
        // If enforce is disabled, never enforce
        if !self.options.enforce {
            return false;
        }

        // If rollout is 0%, never enforce
        if self.options.rollout_percent == 0 {
            return false;
        }

        // If rollout is 100%, always enforce
        if self.options.rollout_percent >= 100 {
            return true;
        }

        // Hash-based sampling (deterministic)
        let hash = Self::hash_request_id(request_id);
        let sample = hash % 100;

        sample < self.options.rollout_percent as u64
    }

    /// Hash request ID for sampling (deterministic, uniform distribution)
    fn hash_request_id(request_id: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        request_id.hash(&mut hasher);
        hasher.finish()
    }

    /// Get enforcement options
    pub fn options(&self) -> &EnforceOptions {
        &self.options
    }
}

/// Drift tracker for monitoring shadow/enforced divergence
#[derive(Debug, Clone)]
pub struct DriftTracker {
    /// Total requests processed
    total_requests: usize,

    /// Total drift events (shadow != enforced)
    drift_events: usize,

    /// Drift events by policy_id
    drift_by_policy: HashMap<String, usize>,
}

impl DriftTracker {
    /// Create new drift tracker
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            drift_events: 0,
            drift_by_policy: HashMap::new(),
        }
    }

    /// Record a verdict pair
    pub fn record(&mut self, policy_id: &str, verdict_pair: &VerdictPair) {
        self.total_requests += 1;

        if verdict_pair.has_drift() {
            self.drift_events += 1;
            *self
                .drift_by_policy
                .entry(policy_id.to_string())
                .or_insert(0) += 1;
        }
    }

    /// Calculate current drift ratio
    pub fn drift_ratio(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }

        self.drift_events as f64 / self.total_requests as f64
    }

    /// Get drift events for specific policy
    pub fn drift_for_policy(&self, policy_id: &str) -> usize {
        self.drift_by_policy.get(policy_id).copied().unwrap_or(0)
    }

    /// Check if drift exceeds threshold
    pub fn exceeds_threshold(&self, threshold: f64) -> bool {
        self.drift_ratio() > threshold
    }

    /// Get statistics
    pub fn stats(&self) -> DriftStats {
        DriftStats {
            total_requests: self.total_requests,
            drift_events: self.drift_events,
            drift_ratio: self.drift_ratio(),
        }
    }

    /// Reset tracker
    pub fn reset(&mut self) {
        self.total_requests = 0;
        self.drift_events = 0;
        self.drift_by_policy.clear();
    }
}

impl Default for DriftTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Drift statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftStats {
    /// Total requests processed
    pub total_requests: usize,

    /// Total drift events
    pub drift_events: usize,

    /// Current drift ratio (0.0 - 1.0)
    pub drift_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy_v2::types::{IrExpression, IrRule};

    fn create_test_ir() -> IrV1 {
        IrV1 {
            ir_version: "1.0".to_string(),
            policy_id: "test.v1".to_string(),
            policy_hash: "sha3-256:0x1234567890abcdef".to_string(),
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

    fn create_test_context() -> OrchestratorContext {
        OrchestratorContext {
            supplier_hashes: vec!["0xabc".to_string()],
            ubo_hashes: vec![],
            company_commitment_root: Some("0x123".to_string()),
            sanctions_root: None,
            jurisdiction_root: None,
            variables: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_enforce_options_default() {
        let opts = EnforceOptions::default();
        assert!(!opts.enforce);
        assert_eq!(opts.rollout_percent, 0);
        assert_eq!(opts.drift_max_ratio, 0.005);
    }

    #[test]
    fn test_verdict_from_str() {
        assert_eq!(Verdict::from_str("ok"), Verdict::Ok);
        assert_eq!(Verdict::from_str("warn"), Verdict::Warn);
        assert_eq!(Verdict::from_str("fail"), Verdict::Fail);
        assert_eq!(Verdict::from_str("unknown"), Verdict::Fail);
    }

    #[test]
    fn test_verdict_is_success() {
        assert!(Verdict::Ok.is_success());
        assert!(Verdict::Warn.is_success());
        assert!(!Verdict::Fail.is_success());
    }

    #[test]
    fn test_verdict_pair_has_drift() {
        // No drift: shadow == enforced
        let pair1 = VerdictPair {
            shadow: Verdict::Ok,
            enforced: Verdict::Ok,
            enforced_applied: true,
        };
        assert!(!pair1.has_drift());

        // Drift: shadow != enforced
        let pair2 = VerdictPair {
            shadow: Verdict::Ok,
            enforced: Verdict::Fail,
            enforced_applied: true,
        };
        assert!(pair2.has_drift());

        // No drift: enforcement not applied
        let pair3 = VerdictPair {
            shadow: Verdict::Ok,
            enforced: Verdict::Ok,
            enforced_applied: false,
        };
        assert!(!pair3.has_drift());
    }

    #[test]
    fn test_enforcer_shadow_only() {
        let ir = create_test_ir();
        let ctx = create_test_context();

        let opts = EnforceOptions {
            enforce: false,
            rollout_percent: 0,
            drift_max_ratio: 0.005,
        };

        let enforcer = Enforcer::new(&ir, opts).unwrap();
        let result = enforcer.decide(&ctx, "request-123").unwrap();

        // Should have shadow verdict
        assert!(result.shadow.is_success());

        // Enforcement should not be applied
        assert!(!result.enforced_applied);

        // No drift (enforcement not applied)
        assert!(!result.has_drift());
    }

    #[test]
    fn test_enforcer_full_rollout() {
        let ir = create_test_ir();
        let ctx = create_test_context();

        let opts = EnforceOptions {
            enforce: true,
            rollout_percent: 100,
            drift_max_ratio: 0.005,
        };

        let enforcer = Enforcer::new(&ir, opts).unwrap();
        let result = enforcer.decide(&ctx, "request-123").unwrap();

        // Should have shadow verdict
        assert!(result.shadow.is_success());

        // Enforcement should be applied
        assert!(result.enforced_applied);

        // Should have enforced verdict
        assert!(result.enforced.is_success());
    }

    #[test]
    fn test_enforcer_partial_rollout_deterministic() {
        let ir = create_test_ir();
        let ctx = create_test_context();

        let opts = EnforceOptions {
            enforce: true,
            rollout_percent: 50, // 50% rollout
            drift_max_ratio: 0.005,
        };

        let enforcer = Enforcer::new(&ir, opts).unwrap();

        // Same request_id should produce same enforcement decision
        let result1 = enforcer.decide(&ctx, "request-123").unwrap();
        let result2 = enforcer.decide(&ctx, "request-123").unwrap();

        assert_eq!(result1.enforced_applied, result2.enforced_applied);

        // Different request_ids should have different (but deterministic) outcomes
        let _result3 = enforcer.decide(&ctx, "request-456").unwrap();
        // Cannot assert equality here, but it should be deterministic

        // Test multiple requests to verify ~50% rollout
        let mut enforced_count = 0;
        for i in 0..100 {
            let result = enforcer.decide(&ctx, &format!("request-{}", i)).unwrap();
            if result.enforced_applied {
                enforced_count += 1;
            }
        }

        // Should be roughly 50% (allow ±20% variance due to hash distribution)
        assert!(
            (30..=70).contains(&enforced_count),
            "Expected ~50% rollout, got {}%",
            enforced_count
        );
    }

    #[test]
    fn test_drift_tracker() {
        let mut tracker = DriftTracker::new();

        // Record non-drift event
        let pair1 = VerdictPair {
            shadow: Verdict::Ok,
            enforced: Verdict::Ok,
            enforced_applied: true,
        };
        tracker.record("policy1", &pair1);

        assert_eq!(tracker.total_requests, 1);
        assert_eq!(tracker.drift_events, 0);
        assert_eq!(tracker.drift_ratio(), 0.0);

        // Record drift event
        let pair2 = VerdictPair {
            shadow: Verdict::Ok,
            enforced: Verdict::Fail,
            enforced_applied: true,
        };
        tracker.record("policy1", &pair2);

        assert_eq!(tracker.total_requests, 2);
        assert_eq!(tracker.drift_events, 1);
        assert_eq!(tracker.drift_ratio(), 0.5);

        // Check threshold
        assert!(tracker.exceeds_threshold(0.1));
        assert!(!tracker.exceeds_threshold(0.9));

        // Check policy-specific drift
        assert_eq!(tracker.drift_for_policy("policy1"), 1);
        assert_eq!(tracker.drift_for_policy("policy2"), 0);

        // Reset
        tracker.reset();
        assert_eq!(tracker.total_requests, 0);
        assert_eq!(tracker.drift_events, 0);
    }

    #[test]
    fn test_drift_stats() {
        let mut tracker = DriftTracker::new();

        // Add some events
        for i in 0..10 {
            let pair = VerdictPair {
                shadow: Verdict::Ok,
                enforced: if i % 3 == 0 {
                    Verdict::Fail
                } else {
                    Verdict::Ok
                },
                enforced_applied: true,
            };
            tracker.record("policy1", &pair);
        }

        let stats = tracker.stats();
        assert_eq!(stats.total_requests, 10);
        assert_eq!(stats.drift_events, 4); // 0, 3, 6, 9
        assert!((stats.drift_ratio - 0.4).abs() < 0.001);
    }
}
