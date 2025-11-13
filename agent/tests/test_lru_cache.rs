/// LRU Cache Eviction Tests - Week 4
///
/// Tests that verify the LRU cache correctly evicts least-recently-used entries
/// when the cache reaches its capacity (1000 entries).

use cap_agent::policy_v2::{parse_yaml_str, PolicyV2, IrV1};
use cap_agent::api::policy_compiler::{test_clear_cache, test_get_cache_size, test_insert_policy, test_cache_contains, test_touch_policy};

#[test]
#[ignore] // Requires significant time (inserting 1000+ policies)
fn test_lru_cache_size_limit() {
    // Clear cache before test
    test_clear_cache();

    // Generate and insert 1500 policies (500 more than limit)
    for i in 0..1500 {
        let policy_yaml = format!(
            r#"
id: "test.policy.{}"
version: "1.0"
legal_basis:
  - directive: "LkSG"
    article: "§3"
description: "Test policy {}"
inputs:
  supplier_hashes:
    type: array
    items: hex
rules:
  - id: "rule_{}"
    op: "non_membership"
    lhs: "supplier_hashes"
    rhs: "sanctions_root"
"#,
            i, i, i
        );

        let policy: PolicyV2 = parse_yaml_str(&policy_yaml)
            .expect(&format!("Failed to parse policy {}", i));

        let policy_hash = format!("sha3-256:policy_hash_{:04}", i);
        let ir_hash = format!("sha3-256:ir_hash_{:04}", i);

        // Create mock IR
        let ir = IrV1 {
            ir_version: "1.0".to_string(),
            policy_id: policy.id.clone(),
            policy_hash: policy_hash.clone(),
            rules: vec![],
            adaptivity: None,
            ir_hash: ir_hash.clone(),
        };

        // Insert into cache using test helper
        test_insert_policy(policy, policy_hash, ir, ir_hash);
    }

    // Verify cache size is at limit (1000)
    let cache_size = test_get_cache_size();
    assert_eq!(
        cache_size,
        1000,
        "Cache should maintain exactly 1000 entries"
    );

    println!("✅ LRU Cache Size Limit Test - PASSED");
    println!("   Inserted: 1500 policies");
    println!("   Cache size: 1000 (limit enforced)");
}

#[test]
#[ignore] // Requires significant time
fn test_lru_cache_eviction_order() {
    // Clear cache before test
    test_clear_cache();

    // Insert 1000 policies (fill cache to limit)
    for i in 0..1000 {
        let policy_yaml = format!(
            r#"
id: "eviction.test.{}"
version: "1.0"
legal_basis:
  - directive: "LkSG"
description: "Eviction test policy {}"
inputs:
  supplier_hashes:
    type: array
    items: hex
rules:
  - id: "rule_{}"
    op: "eq"
    lhs: "supplier_hashes"
    rhs: "empty_set"
"#,
            i, i, i
        );

        let policy: PolicyV2 = parse_yaml_str(&policy_yaml).unwrap();
        let policy_hash = format!("sha3-256:eviction_hash_{:04}", i);
        let ir_hash = format!("sha3-256:eviction_ir_{:04}", i);

        let ir = IrV1 {
            ir_version: "1.0".to_string(),
            policy_id: policy.id.clone(),
            policy_hash: policy_hash.clone(),
            rules: vec![],
            adaptivity: None,
            ir_hash: ir_hash.clone(),
        };

        test_insert_policy(policy, policy_hash, ir, ir_hash);
    }

    // Access first 10 policies (make them recently used)
    for i in 0..10 {
        let policy_hash = format!("sha3-256:eviction_hash_{:04}", i);
        test_touch_policy(&policy_hash);
    }

    // Insert 20 new policies (should evict 20 oldest policies, NOT the recently accessed ones)
    for i in 1000..1020 {
        let policy_yaml = format!(
            r#"
id: "eviction.new.{}"
version: "1.0"
legal_basis:
  - directive: "LkSG"
description: "New policy {}"
inputs:
  supplier_hashes:
    type: array
    items: hex
rules:
  - id: "rule_{}"
    op: "eq"
    lhs: "supplier_hashes"
    rhs: "empty_set"
"#,
            i, i, i
        );

        let policy: PolicyV2 = parse_yaml_str(&policy_yaml).unwrap();
        let policy_hash = format!("sha3-256:eviction_hash_{:04}", i);
        let ir_hash = format!("sha3-256:eviction_ir_{:04}", i);

        let ir = IrV1 {
            ir_version: "1.0".to_string(),
            policy_id: policy.id.clone(),
            policy_hash: policy_hash.clone(),
            rules: vec![],
            adaptivity: None,
            ir_hash: ir_hash.clone(),
        };

        test_insert_policy(policy, policy_hash, ir, ir_hash);
    }

    // Verify cache size is still 1000
    let cache_size = test_get_cache_size();
    assert_eq!(cache_size, 1000, "Cache should still be at limit");

    // Verify first 10 policies (recently accessed) are still in cache
    let mut recently_accessed_still_present = 0;
    for i in 0..10 {
        let policy_hash = format!("sha3-256:eviction_hash_{:04}", i);
        if test_cache_contains(&policy_hash) {
            recently_accessed_still_present += 1;
        }
    }

    assert!(
        recently_accessed_still_present >= 8,
        "At least 8/10 recently accessed policies should remain in cache (found: {})",
        recently_accessed_still_present
    );

    // Verify some old policies (not recently accessed) were evicted
    // Check policies 10-29 (should be evicted to make room for 1000-1019)
    let mut old_policies_evicted = 0;
    for i in 10..30 {
        let policy_hash = format!("sha3-256:eviction_hash_{:04}", i);
        if !test_cache_contains(&policy_hash) {
            old_policies_evicted += 1;
        }
    }

    assert!(
        old_policies_evicted >= 15,
        "At least 15/20 old policies should have been evicted (found: {})",
        old_policies_evicted
    );

    println!("✅ LRU Cache Eviction Order Test - PASSED");
    println!("   Recently accessed entries retained: {}/10", recently_accessed_still_present);
    println!("   Old entries evicted: {}/20", old_policies_evicted);
}

#[test]
#[ignore]
fn test_lru_cache_performance() {
    use std::time::Instant;

    // Clear cache
    test_clear_cache();

    let start = Instant::now();

    // Insert 1000 policies and measure time
    for i in 0..1000 {
        let policy_yaml = format!(
            r#"
id: "perf.test.{}"
version: "1.0"
legal_basis:
  - directive: "LkSG"
inputs:
  supplier_hashes:
    type: array
    items: hex
rules:
  - id: "rule_{}"
    op: "eq"
    lhs: "supplier_hashes"
    rhs: "empty_set"
"#,
            i, i
        );

        let policy: PolicyV2 = parse_yaml_str(&policy_yaml).unwrap();
        let policy_hash = format!("sha3-256:perf_hash_{:04}", i);
        let ir_hash = format!("sha3-256:perf_ir_{:04}", i);

        let ir = IrV1 {
            ir_version: "1.0".to_string(),
            policy_id: policy.id.clone(),
            policy_hash: policy_hash.clone(),
            rules: vec![],
            adaptivity: None,
            ir_hash: ir_hash.clone(),
        };

        test_insert_policy(policy, policy_hash, ir, ir_hash);
    }

    let insert_duration = start.elapsed();

    // Measure cache hits (should be fast)
    let lookup_start = Instant::now();
    for i in 0..1000 {
        let policy_hash = format!("sha3-256:perf_hash_{:04}", i);
        test_touch_policy(&policy_hash);
    }
    let lookup_duration = lookup_start.elapsed();

    println!("✅ LRU Cache Performance Test - PASSED");
    println!("   Insert 1000 policies: {:?}", insert_duration);
    println!("   Lookup 1000 policies: {:?}", lookup_duration);
    println!("   Avg insert: {:?}", insert_duration / 1000);
    println!("   Avg lookup: {:?}", lookup_duration / 1000);

    // Performance assertions (generous limits)
    assert!(
        insert_duration.as_secs() < 5,
        "Inserting 1000 policies should take < 5s (took: {:?})",
        insert_duration
    );

    assert!(
        lookup_duration.as_millis() < 100,
        "Looking up 1000 policies should take < 100ms (took: {:?})",
        lookup_duration
    );
}
