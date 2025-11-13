/// Policy Compiler Determinism Tests (Week 3)
///
/// Validates that:
/// 1. Policy hash is deterministic across 100 compilations
/// 2. IR hash is deterministic across 100 compilations
/// 3. Canonical JSON ordering is stable
/// 4. Rule sorting is consistent

use cap_agent::policy_v2::{
    parse_yaml, lint, LintMode, generate_ir, canonicalize, sha3_256_hex,
};
use std::collections::HashSet;

/// Test policy hash determinism (100 runs)
#[test]
fn test_policy_hash_determinism_100_runs() {
    let policy = parse_yaml("examples/lksg_v1.policy.yml")
        .expect("Failed to parse policy");

    let mut hashes = HashSet::new();

    for _ in 0..100 {
        let policy_json = serde_json::to_string(&policy)
            .expect("Failed to serialize policy");
        let policy_hash = sha3_256_hex(&policy_json);
        hashes.insert(policy_hash);
    }

    // All 100 hashes must be identical
    assert_eq!(hashes.len(), 1, "Policy hash is non-deterministic! Found {} unique hashes in 100 runs", hashes.len());

    println!("✅ Policy hash deterministic across 100 runs: {}", hashes.iter().next().unwrap());
}

/// Test IR hash determinism (100 runs)
#[test]
fn test_ir_hash_determinism_100_runs() {
    let policy = parse_yaml("examples/lksg_v1.policy.yml")
        .expect("Failed to parse policy");

    let diagnostics = lint(&policy, LintMode::Strict);
    assert!(diagnostics.iter().all(|d| !matches!(d.level, cap_agent::policy_v2::Level::Error)),
            "Policy has lint errors");

    let policy_json = serde_json::to_string(&policy).expect("Failed to serialize policy");
    let policy_hash = sha3_256_hex(&policy_json);

    let mut ir_hashes = HashSet::new();

    for _ in 0..100 {
        let ir = generate_ir(&policy, policy_hash.clone())
            .expect("Failed to generate IR");

        let ir_canonical = canonicalize(&ir)
            .expect("Failed to canonicalize IR");

        let ir_hash = sha3_256_hex(&ir_canonical);
        ir_hashes.insert(ir_hash);
    }

    // All 100 IR hashes must be identical
    assert_eq!(ir_hashes.len(), 1, "IR hash is non-deterministic! Found {} unique hashes in 100 runs", ir_hashes.len());

    println!("✅ IR hash deterministic across 100 runs: {}", ir_hashes.iter().next().unwrap());
}

/// Test full compilation determinism (policy + IR)
#[test]
fn test_full_compilation_determinism_100_runs() {
    let policy = parse_yaml("examples/lksg_v1.policy.yml")
        .expect("Failed to parse policy");

    let mut results = HashSet::new();

    for _ in 0..100 {
        // Policy hash
        let policy_json = serde_json::to_string(&policy).expect("Serialization failed");
        let policy_hash = sha3_256_hex(&policy_json);

        // IR hash
        let ir = generate_ir(&policy, policy_hash.clone()).expect("IR generation failed");
        let ir_canonical = canonicalize(&ir).expect("Canonicalization failed");
        let ir_hash = sha3_256_hex(&ir_canonical);

        results.insert((policy_hash, ir_hash));
    }

    // All 100 runs must produce identical (policy_hash, ir_hash) pairs
    assert_eq!(results.len(), 1, "Compilation is non-deterministic! Found {} unique (policy_hash, ir_hash) pairs in 100 runs", results.len());

    let (policy_hash, ir_hash) = results.iter().next().unwrap();
    println!("✅ Full compilation deterministic across 100 runs:");
    println!("   Policy Hash: {}", policy_hash);
    println!("   IR Hash: {}", ir_hash);
}

/// Test canonical JSON ordering is stable
#[test]
fn test_canonical_json_ordering() {
    let policy = parse_yaml("examples/lksg_v1.policy.yml")
        .expect("Failed to parse policy");

    let policy_json = serde_json::to_string(&policy).expect("Serialization failed");
    let policy_hash = sha3_256_hex(&policy_json);

    let ir = generate_ir(&policy, policy_hash).expect("IR generation failed");

    let mut canonical_jsons = HashSet::new();

    for _ in 0..100 {
        let canonical = canonicalize(&ir).expect("Canonicalization failed");
        canonical_jsons.insert(canonical);
    }

    // All canonical JSONs must be byte-identical
    assert_eq!(canonical_jsons.len(), 1, "Canonical JSON ordering is non-deterministic!");

    let canonical_json = canonical_jsons.iter().next().unwrap();
    println!("✅ Canonical JSON ordering stable across 100 runs ({} bytes)", canonical_json.len());
}

/// Test rule sorting is consistent
#[test]
fn test_rule_sorting_consistency() {
    let policy = parse_yaml("examples/lksg_v1.policy.yml")
        .expect("Failed to parse policy");

    let policy_json = serde_json::to_string(&policy).expect("Serialization failed");
    let policy_hash = sha3_256_hex(&policy_json);

    let mut rule_orderings = HashSet::new();

    for _ in 0..100 {
        let ir = generate_ir(&policy, policy_hash.clone()).expect("IR generation failed");

        // Extract rule IDs in order
        let rule_ids: Vec<String> = ir.rules.iter().map(|r| r.id.clone()).collect();
        rule_orderings.insert(rule_ids);
    }

    // All rule orderings must be identical
    assert_eq!(rule_orderings.len(), 1, "Rule sorting is non-deterministic!");

    let rule_order = rule_orderings.iter().next().unwrap();
    println!("✅ Rule sorting consistent across 100 runs: {:?}", rule_order);
}

/// Benchmark compilation performance (informational)
#[test]
#[ignore] // Run with: cargo test --test test_policy_determinism -- --ignored
fn bench_compilation_performance() {
    use std::time::Instant;

    let policy = parse_yaml("examples/lksg_v1.policy.yml")
        .expect("Failed to parse policy");

    let mut durations = Vec::new();

    for _ in 0..100 {
        let start = Instant::now();

        let policy_json = serde_json::to_string(&policy).expect("Serialization failed");
        let policy_hash = sha3_256_hex(&policy_json);
        let ir = generate_ir(&policy, policy_hash).expect("IR generation failed");
        let _ir_canonical = canonicalize(&ir).expect("Canonicalization failed");

        durations.push(start.elapsed());
    }

    let total: std::time::Duration = durations.iter().sum();
    let avg = total / 100;
    let max = durations.iter().max().unwrap();
    let min = durations.iter().min().unwrap();

    println!("📊 Compilation Performance (100 runs):");
    println!("   Average: {:?}", avg);
    println!("   Min: {:?}", min);
    println!("   Max: {:?}", max);
    println!("   Total: {:?}", total);

    // Week 3 spec: p95 ≤ 50 ms (warm)
    durations.sort();
    let p95_idx = (durations.len() as f64 * 0.95) as usize;
    let p95 = durations[p95_idx];

    println!("   P95: {:?}", p95);

    // This is informational, not a hard assert
    if p95 > std::time::Duration::from_millis(50) {
        println!("⚠️  P95 exceeds 50ms target (found {:?})", p95);
    } else {
        println!("✅ P95 within 50ms target");
    }
}
