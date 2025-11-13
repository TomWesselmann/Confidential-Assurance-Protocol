/// Compiler Benchmarks - Week 4 Performance Testing
///
/// Targets:
/// - p95 ≤ 50 ms (warm cache)
/// - p95 ≤ 200 ms (cold cache)
/// - Memory < 64 MiB

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use cap_agent::policy_v2::{
    PolicyV2, parse_yaml_str, lint, LintMode, generate_ir,
    canonicalize, sha3_256_hex,
};
use std::collections::HashMap;

/// Example PolicyV2 YAML for benchmarking (from examples/lksg_v1.policy.yml)
const EXAMPLE_POLICY_YAML: &str = include_str!("../examples/lksg_v1.policy.yml");

/// Cold compilation: Parse + Lint + IR generation + Hash (no cache)
fn compile_cold(yaml: &str) -> Result<(String, String), String> {
    // 1. Parse YAML
    let policy = parse_yaml_str(yaml)
        .map_err(|e| format!("Parse error: {}", e))?;

    // 2. Lint (strict mode)
    let diagnostics = lint(&policy, LintMode::Strict);
    if !diagnostics.is_empty() {
        return Err(format!("Lint errors: {:?}", diagnostics));
    }

    // 3. Compute policy hash
    let policy_json = serde_json::to_string(&policy)
        .map_err(|e| format!("Serialization error: {}", e))?;
    let policy_hash = sha3_256_hex(&policy_json);

    // 4. Generate IR
    let mut ir = generate_ir(&policy, policy_hash.clone())
        .map_err(|e| format!("IR generation error: {}", e))?;

    // 5. Canonicalize and hash IR
    let ir_canonical = canonicalize(&ir)
        .map_err(|e| format!("Canonicalization error: {}", e))?;
    let ir_hash = sha3_256_hex(&ir_canonical);
    ir.ir_hash = ir_hash.clone();

    Ok((policy_hash, ir_hash))
}

/// Warm cache context for simulating cached compilation
struct WarmCache {
    /// Cache: policy_hash → ir_hash
    cache: HashMap<String, String>,
}

impl WarmCache {
    fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Warm compilation: Check cache, fall back to cold if miss
    fn compile_warm(&mut self, yaml: &str) -> Result<(String, String), String> {
        // Compute policy hash (quick)
        let policy = parse_yaml_str(yaml)
            .map_err(|e| format!("Parse error: {}", e))?;

        let policy_json = serde_json::to_string(&policy)
            .map_err(|e| format!("Serialization error: {}", e))?;
        let policy_hash = sha3_256_hex(&policy_json);

        // Check cache
        if let Some(ir_hash) = self.cache.get(&policy_hash) {
            return Ok((policy_hash.clone(), ir_hash.clone()));
        }

        // Cache miss - full compilation
        let (ph, ih) = compile_cold(yaml)?;
        self.cache.insert(ph.clone(), ih.clone());
        Ok((ph, ih))
    }

    /// Prime cache with policy
    fn prime(&mut self, yaml: &str) -> Result<(), String> {
        let _ = self.compile_warm(yaml)?;
        Ok(())
    }
}

/// Benchmark: Cold compilation
fn bench_compile_cold(c: &mut Criterion) {
    let yaml = EXAMPLE_POLICY_YAML;

    // Pre-validate that compilation works
    match compile_cold(yaml) {
        Ok(_) => {},
        Err(e) => {
            panic!("Pre-validation failed: {}", e);
        }
    }

    c.bench_function("compile_cold", |b| {
        b.iter(|| {
            let result = compile_cold(black_box(yaml));
            assert!(result.is_ok(), "Compilation should succeed: {:?}", result.err());
            result
        });
    });
}

/// Benchmark: Warm compilation (cache hit)
fn bench_compile_warm(c: &mut Criterion) {
    let yaml = EXAMPLE_POLICY_YAML;

    c.bench_function("compile_warm_hit", |b| {
        let mut cache = WarmCache::new();
        cache.prime(yaml).expect("Failed to prime cache");

        b.iter(|| {
            let result = cache.compile_warm(black_box(yaml));
            assert!(result.is_ok(), "Compilation should succeed");
            result
        });
    });
}

/// Benchmark: Warm compilation (cache miss)
fn bench_compile_warm_miss(c: &mut Criterion) {
    let yaml = EXAMPLE_POLICY_YAML;

    c.bench_function("compile_warm_miss", |b| {
        b.iter(|| {
            let mut cache = WarmCache::new();
            let result = cache.compile_warm(black_box(yaml));
            assert!(result.is_ok(), "Compilation should succeed");
            result
        });
    });
}

/// Benchmark: Policy parsing only
fn bench_parse_yaml(c: &mut Criterion) {
    let yaml = EXAMPLE_POLICY_YAML;

    c.bench_function("parse_yaml", |b| {
        b.iter(|| {
            let result = parse_yaml_str(black_box(yaml));
            assert!(result.is_ok(), "Parsing should succeed");
            result
        });
    });
}

/// Benchmark: Linting only
fn bench_lint(c: &mut Criterion) {
    let yaml = EXAMPLE_POLICY_YAML;
    let policy = parse_yaml_str(yaml).expect("Parse failed");

    c.bench_function("lint_strict", |b| {
        b.iter(|| {
            let diagnostics = lint(black_box(&policy), LintMode::Strict);
            diagnostics
        });
    });
}

/// Benchmark: IR generation only
fn bench_ir_generation(c: &mut Criterion) {
    let yaml = EXAMPLE_POLICY_YAML;
    let policy = parse_yaml_str(yaml).expect("Parse failed");
    let policy_json = serde_json::to_string(&policy).expect("Serialization failed");
    let policy_hash = sha3_256_hex(&policy_json);

    c.bench_function("ir_generation", |b| {
        b.iter(|| {
            let result = generate_ir(black_box(&policy), black_box(policy_hash.clone()));
            assert!(result.is_ok(), "IR generation should succeed");
            result
        });
    });
}

/// Benchmark: Canonicalization only
fn bench_canonicalize(c: &mut Criterion) {
    let yaml = EXAMPLE_POLICY_YAML;
    let policy = parse_yaml_str(yaml).expect("Parse failed");
    let policy_json = serde_json::to_string(&policy).expect("Serialization failed");
    let policy_hash = sha3_256_hex(&policy_json);
    let ir = generate_ir(&policy, policy_hash).expect("IR generation failed");

    c.bench_function("canonicalize_ir", |b| {
        b.iter(|| {
            let result = canonicalize(black_box(&ir));
            assert!(result.is_ok(), "Canonicalization should succeed");
            result
        });
    });
}

/// Benchmark: Hash computation only
fn bench_hash(c: &mut Criterion) {
    let data = "benchmark.policy.v1".repeat(100); // ~2KB

    c.bench_function("sha3_256_hash", |b| {
        b.iter(|| {
            let hash = sha3_256_hex(black_box(&data));
            hash
        });
    });
}

/// Benchmark group: Varying policy sizes
fn bench_policy_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("policy_sizes");

    for rule_count in [1, 5, 10, 20, 50].iter() {
        let yaml = generate_policy_with_rules(*rule_count);

        group.bench_with_input(
            BenchmarkId::from_parameter(rule_count),
            &yaml,
            |b, yaml| {
                b.iter(|| {
                    let result = compile_cold(black_box(yaml));
                    assert!(result.is_ok());
                    result
                });
            },
        );
    }

    group.finish();
}

/// Helper: Generate policy YAML with N rules
fn generate_policy_with_rules(count: usize) -> String {
    let mut yaml = String::from(
        r#"id: "bench.policy.v1"
version: "1.0"
legal_basis:
  law: "LkSG"
  jurisdiction: "DE"
inputs:
  - name: "supplier_hashes"
    type: "set<hash>"
rules:
"#,
    );

    for i in 0..count {
        yaml.push_str(&format!(
            r#"  - rule_id: "rule_{}"
    description: "Test rule {}"
    operator: "non_membership"
    args:
      set_var: "supplier_hashes"
      element:
        var: "empty_set"
"#,
            i, i
        ));
    }

    yaml
}

// Register benchmarks
criterion_group!(
    benches,
    bench_compile_cold,
    bench_compile_warm,
    bench_compile_warm_miss,
    bench_parse_yaml,
    bench_lint,
    bench_ir_generation,
    bench_canonicalize,
    bench_hash,
    bench_policy_sizes,
);

criterion_main!(benches);
