use cap_agent::policy_v2::*;
use std::fs;

#[test]
fn test_golden_ir_lksg_v1() {
    // Parse policy
    let policy = parse_yaml("examples/lksg_v1.policy.yml").expect("Failed to parse policy");

    // Lint (should pass)
    let diagnostics = lint(&policy, LintMode::Strict);
    assert!(
        !has_errors(&diagnostics),
        "Policy has lint errors: {:?}",
        diagnostics
    );

    // Compute policy hash
    let policy_json = serde_json::to_string(&policy).expect("Failed to serialize policy");
    let policy_hash = sha3_256_hex(&policy_json);

    // Generate IR
    let mut ir = generate_ir(&policy, policy_hash).expect("Failed to generate IR");

    // Compute IR hash
    let ir_canonical = canonicalize(&ir).expect("Failed to canonicalize IR");
    let ir_hash = sha3_256_hex(&ir_canonical);
    ir.ir_hash = ir_hash;

    // Serialize IR
    let ir_json = serde_json::to_string_pretty(&ir).expect("Failed to serialize IR");

    // Golden file path
    let golden_path = "examples/lksg_v1.ir.json";

    if std::env::var("UPDATE_GOLDEN").is_ok() {
        // Update golden file
        fs::write(golden_path, &ir_json).expect("Failed to write golden file");
        println!("✅ Updated golden file: {}", golden_path);
    } else {
        // Compare with golden file
        let golden = fs::read_to_string(golden_path)
            .expect("Golden file not found - run with UPDATE_GOLDEN=1 to create");

        assert_eq!(
            ir_json, golden,
            "IR does not match golden file. Run with UPDATE_GOLDEN=1 to update."
        );
    }
}

#[test]
fn test_ir_hash_determinism() {
    // This test ensures that generating the same IR twice produces the same hash
    let policy = parse_yaml("examples/lksg_v1.policy.yml").expect("Failed to parse policy");

    let policy_json = serde_json::to_string(&policy).unwrap();
    let policy_hash = sha3_256_hex(&policy_json);

    // Generate IR twice
    let ir1 = generate_ir(&policy, policy_hash.clone()).unwrap();
    let ir2 = generate_ir(&policy, policy_hash.clone()).unwrap();

    // Canonicalize both
    let canonical1 = canonicalize(&ir1).unwrap();
    let canonical2 = canonicalize(&ir2).unwrap();

    // Compute hashes
    let hash1 = sha3_256_hex(&canonical1);
    let hash2 = sha3_256_hex(&canonical2);

    assert_eq!(hash1, hash2, "IR hashes must be deterministic");
    assert_eq!(canonical1, canonical2, "Canonical IR must be identical");
}

#[test]
fn test_policy_hash_determinism() {
    // This test ensures that serializing the same policy twice produces the same hash
    let policy = parse_yaml("examples/lksg_v1.policy.yml").expect("Failed to parse policy");

    let json1 = serde_json::to_string(&policy).unwrap();
    let json2 = serde_json::to_string(&policy).unwrap();

    let hash1 = sha3_256_hex(&json1);
    let hash2 = sha3_256_hex(&json2);

    assert_eq!(hash1, hash2, "Policy hashes must be deterministic");
}
