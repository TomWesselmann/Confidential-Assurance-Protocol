//! # SAP Adapter Pilot Integration Tests (Week 6 - C2)
//!
//! Integration tests for SAP Adapter E2E flow:
//! - OData Pull → Verify → Writeback
//! - Idempotency verification
//! - Audit trail validation
//!
//! **IMPORTANT**: These tests require a live SAP S/4HANA system and are marked with `#[ignore]`.
//! Run with: `cargo test --test adapter_pilot -- --ignored --nocapture`
//!
//! ## Prerequisites
//!
//! 1. SAP System Configuration:
//!    - Z_CAP_SUPPLIERS table populated with test data
//!    - Z_CAP_SUPPLIER_STATUS table exists (will be written to)
//!    - OData services: Z_CAP_SUPPLIERS_SRV, Z_CAP_STATUS_SRV
//!
//! 2. Environment Variables:
//!    ```bash
//!    export SAP_URL="https://sap-staging.example.com:8443/sap/opu/odata/sap/Z_CAP_SUPPLIERS_SRV"
//!    export SAP_CLIENT="100"
//!    export SAP_USER="CAP_ADAPTER_TEST"
//!    export SAP_PASSWORD="<password>"
//!    export CAP_API_BASE="https://cap-verifier-staging.example.com/api/v1"
//!    export CAP_API_TOKEN="<oauth2-jwt-token>"
//!    ```
//!
//! 3. CAP Adapter Binary:
//!    - Must be built and available in PATH or target/release/cap-adapter
//!
//! ## Test Data
//!
//! Tests expect at least 10 supplier records in SAP with TIER <= 2.
//! Recommended test dataset:
//! - 10 suppliers (S_TEST_001 through S_TEST_010)
//! - All with TIER = 1 or 2
//! - No sanctioned suppliers (SANCTION_FLAG = '')

use serde_json::Value;
use std::env;
use std::fs;
use std::process::Command;

/// Helper: Get required environment variable or panic with clear message
fn get_env_or_panic(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| {
        panic!(
            "Missing required environment variable: {}. \
             Please see test file documentation for setup instructions.",
            key
        )
    })
}

/// Helper: Get optional environment variable with default
fn get_env_or_default(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Helper: Execute cap-adapter command and return output
fn run_cap_adapter(args: &[&str]) -> Result<String, String> {
    let adapter_path = get_env_or_default("CAP_ADAPTER_BIN", "cap-adapter");

    let output = Command::new(&adapter_path)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute cap-adapter: {}. Ensure cap-adapter is in PATH or set CAP_ADAPTER_BIN.", e))?;

    if !output.status.success() {
        return Err(format!(
            "cap-adapter exited with non-zero status: {}\nStdout: {}\nStderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Helper: Execute curl command for CAP API verification
fn verify_via_api(context_json_path: &str) -> Result<Value, String> {
    let api_base = get_env_or_panic("CAP_API_BASE");
    let api_token = get_env_or_panic("CAP_API_TOKEN");
    let verify_url = format!("{}/verify", api_base);

    let context_data = fs::read_to_string(context_json_path)
        .map_err(|e| format!("Failed to read context file: {}", e))?;

    let output = Command::new("curl")
        .args([
            "-s",
            "-k", // Silent, insecure (for staging TLS)
            "-H",
            &format!("Authorization: Bearer {}", api_token),
            "-H",
            "Content-Type: application/json",
            "-d",
            &context_data,
            &verify_url,
        ])
        .output()
        .map_err(|e| format!("Failed to execute curl: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "curl exited with non-zero status: {}\nStderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let response_text = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&response_text).map_err(|e| {
        format!(
            "Failed to parse API response as JSON: {}\nResponse: {}",
            e, response_text
        )
    })
}

/// Helper: Query SAP table via OData (for idempotency validation)
fn query_sap_table(table: &str, filter: &str) -> Result<Value, String> {
    let sap_url = get_env_or_panic("SAP_URL");
    let sap_client = get_env_or_panic("SAP_CLIENT");
    let sap_user = get_env_or_panic("SAP_USER");
    let sap_password = get_env_or_panic("SAP_PASSWORD");

    // Construct OData URL (assuming table is exposed as EntitySet)
    let query_url = format!(
        "{}{}Set?$filter={}&$format=json&sap-client={}",
        sap_url, table, filter, sap_client
    );

    let output = Command::new("curl")
        .args([
            "-s",
            "-k", // Silent, insecure
            "-u",
            &format!("{}:{}", sap_user, sap_password),
            &query_url,
        ])
        .output()
        .map_err(|e| format!("Failed to execute curl: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "curl exited with non-zero status: {}\nStderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let response_text = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&response_text).map_err(|e| {
        format!(
            "Failed to parse OData response as JSON: {}\nResponse: {}",
            e, response_text
        )
    })
}

#[test]
#[ignore] // Requires live SAP system
fn test_adapter_e2e_pull_verify_writeback() {
    println!("🧪 SAP Adapter E2E Test: Pull → Verify → Writeback");

    // Test configuration
    let sap_url = get_env_or_panic("SAP_URL");
    let sap_client = get_env_or_panic("SAP_CLIENT");
    let sap_user = get_env_or_panic("SAP_USER");
    let run_id = format!(
        "TEST_RUN_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );

    println!("📋 Test Configuration:");
    println!("  SAP URL: {}", sap_url);
    println!("  SAP Client: {}", sap_client);
    println!("  SAP User: {}", sap_user);
    println!("  Run ID: {}", run_id);

    // Phase 1: Pull from SAP
    println!("\n📥 Phase 1: Pulling data from SAP...");
    let context_json_path = "/tmp/adapter_test_context.json";

    let pull_result = run_cap_adapter(&[
        "pull",
        "--odata",
        &sap_url,
        "--client",
        &sap_client,
        "--user",
        &sap_user,
        "--password-env",
        "SAP_PASSWORD",
        "--filter",
        "TIER le 2",
        "--out",
        context_json_path,
    ]);

    assert!(pull_result.is_ok(), "Pull failed: {:?}", pull_result.err());
    println!("✅ Pull successful");

    // Validate context.json
    let context_data: Value = serde_json::from_str(
        &fs::read_to_string(context_json_path).expect("Failed to read context.json"),
    )
    .expect("Failed to parse context.json");

    assert_eq!(context_data["policy_id"], "lksg.v1", "Policy ID mismatch");
    assert!(
        context_data["context"]["supplier_hashes"].is_array(),
        "supplier_hashes not an array"
    );

    let supplier_count = context_data["context"]["supplier_hashes"]
        .as_array()
        .unwrap()
        .len();
    assert!(
        supplier_count >= 10,
        "Expected at least 10 suppliers, got {}",
        supplier_count
    );
    println!("📊 Pulled {} supplier records", supplier_count);

    // Phase 2: Verify via CAP API
    println!("\n🔍 Phase 2: Verifying via CAP API...");
    let verify_result = verify_via_api(context_json_path);

    assert!(
        verify_result.is_ok(),
        "Verification failed: {:?}",
        verify_result.err()
    );
    let verify_response = verify_result.unwrap();

    assert_eq!(
        verify_response["result"], "ok",
        "Verification result not ok: {:?}",
        verify_response
    );
    assert!(
        verify_response["manifest_hash"].is_string(),
        "manifest_hash missing"
    );
    assert!(
        verify_response["proof_hash"].is_string(),
        "proof_hash missing"
    );

    let manifest_hash = verify_response["manifest_hash"].as_str().unwrap();
    let proof_hash = verify_response["proof_hash"].as_str().unwrap();
    println!("✅ Verification successful");
    println!("  Manifest Hash: {}", manifest_hash);
    println!("  Proof Hash: {}", proof_hash);

    // Save verify response for writeback
    let verify_json_path = "/tmp/adapter_test_verify.json";
    fs::write(verify_json_path, verify_response.to_string()).expect("Failed to write verify.json");

    // Phase 3: Writeback to SAP
    println!("\n📝 Phase 3: Writing back to SAP...");
    let writeback_result = run_cap_adapter(&[
        "writeback",
        "--in",
        verify_json_path,
        "--odata",
        &sap_url,
        "--table",
        "Z_CAP_SUPPLIER_STATUS",
        "--idempotency",
        &run_id,
        "--user",
        &sap_user,
        "--password-env",
        "SAP_PASSWORD",
        "--batch-size",
        "10",
    ]);

    assert!(
        writeback_result.is_ok(),
        "Writeback failed: {:?}",
        writeback_result.err()
    );
    println!("✅ Writeback successful");

    // Phase 4: Validate writeback via OData query
    println!("\n✅ Phase 4: Validating writeback...");
    let query_filter = format!("RUN_ID eq '{}'", run_id);
    let query_result = query_sap_table("Z_CAP_SUPPLIER_STATUS", &query_filter);

    assert!(
        query_result.is_ok(),
        "Query failed: {:?}",
        query_result.err()
    );
    let query_response = query_result.unwrap();

    let results = query_response["d"]["results"]
        .as_array()
        .expect("Results not an array");
    assert_eq!(
        results.len(),
        supplier_count,
        "Record count mismatch: expected {}, got {}",
        supplier_count,
        results.len()
    );
    println!("📊 Validated {} records in SAP", results.len());

    // Validate audit trail fields
    for record in results {
        assert_eq!(record["RUN_ID"], run_id, "RUN_ID mismatch");
        assert_eq!(
            record["MANIFEST_HASH"], manifest_hash,
            "MANIFEST_HASH mismatch"
        );
        // Note: POLICY_HASH and IR_HASH validation would require parsing manifest
        assert!(
            record["VERDICT"].as_str().unwrap() == "ok",
            "Verdict not ok"
        );
    }
    println!("✅ Audit trail validated");

    println!("\n🎉 E2E Test PASSED");
}

#[test]
#[ignore] // Requires live SAP system
fn test_adapter_idempotency() {
    println!("🧪 SAP Adapter Idempotency Test");

    let sap_url = get_env_or_panic("SAP_URL");
    let sap_client = get_env_or_panic("SAP_CLIENT");
    let sap_user = get_env_or_panic("SAP_USER");
    let run_id = format!(
        "IDEMPOTENCY_TEST_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );

    println!("📋 Run ID: {}", run_id);

    // Setup: Pull and verify
    println!("\n📥 Setup: Pull and verify...");
    let context_json_path = "/tmp/adapter_idem_context.json";
    let verify_json_path = "/tmp/adapter_idem_verify.json";

    run_cap_adapter(&[
        "pull",
        "--odata",
        &sap_url,
        "--client",
        &sap_client,
        "--user",
        &sap_user,
        "--password-env",
        "SAP_PASSWORD",
        "--filter",
        "TIER le 2",
        "--out",
        context_json_path,
    ])
    .expect("Pull failed");

    let verify_response = verify_via_api(context_json_path).expect("Verify failed");
    fs::write(verify_json_path, verify_response.to_string()).expect("Failed to write verify.json");

    let supplier_count = verify_response["context"]["supplier_hashes"]
        .as_array()
        .unwrap()
        .len();
    println!("📊 Using {} suppliers for test", supplier_count);

    // First writeback
    println!("\n📝 First writeback (Run ID: {})...", run_id);
    run_cap_adapter(&[
        "writeback",
        "--in",
        verify_json_path,
        "--odata",
        &sap_url,
        "--table",
        "Z_CAP_SUPPLIER_STATUS",
        "--idempotency",
        &run_id,
        "--user",
        &sap_user,
        "--password-env",
        "SAP_PASSWORD",
        "--batch-size",
        "10",
    ])
    .expect("First writeback failed");

    // Query SAP after first writeback
    let query_filter_1 = format!("RUN_ID eq '{}'", run_id);
    let query_result_1 =
        query_sap_table("Z_CAP_SUPPLIER_STATUS", &query_filter_1).expect("Query 1 failed");
    let count_1 = query_result_1["d"]["results"].as_array().unwrap().len();

    assert_eq!(
        count_1, supplier_count,
        "First writeback: expected {} records, got {}",
        supplier_count, count_1
    );
    println!("✅ First writeback: {} records", count_1);

    // Second writeback (same RUN_ID)
    println!("\n📝 Second writeback (SAME Run ID: {})...", run_id);
    run_cap_adapter(&[
        "writeback",
        "--in",
        verify_json_path,
        "--odata",
        &sap_url,
        "--table",
        "Z_CAP_SUPPLIER_STATUS",
        "--idempotency",
        &run_id, // SAME RUN_ID
        "--user",
        &sap_user,
        "--password-env",
        "SAP_PASSWORD",
        "--batch-size",
        "10",
    ])
    .expect("Second writeback failed");

    // Query SAP after second writeback
    let query_filter_2 = format!("RUN_ID eq '{}'", run_id);
    let query_result_2 =
        query_sap_table("Z_CAP_SUPPLIER_STATUS", &query_filter_2).expect("Query 2 failed");
    let count_2 = query_result_2["d"]["results"].as_array().unwrap().len();

    // CRITICAL: Count should still be supplier_count, NOT 2 * supplier_count
    assert_eq!(
        count_2, supplier_count,
        "Idempotency FAILED: expected {} records (no duplicates), got {}",
        supplier_count, count_2
    );
    println!("✅ Second writeback: {} records (no duplicates)", count_2);

    println!("\n🎉 Idempotency Test PASSED");
}

#[test]
#[ignore] // Requires live SAP system
fn test_adapter_rate_limiting() {
    println!("🧪 SAP Adapter Rate Limiting Test");

    let sap_url = get_env_or_panic("SAP_URL");
    let sap_client = get_env_or_panic("SAP_CLIENT");
    let sap_user = get_env_or_panic("SAP_USER");

    println!("📋 Testing rate limiting behavior...");
    println!("  This test deliberately triggers rate limits to verify retry logic.");

    // Configure adapter with very low rate limit
    env::set_var("ADAPTER_RATE_LIMIT", "1"); // 1 req/s
    env::set_var("ADAPTER_RETRY_MAX", "3");

    // Pull multiple times rapidly
    println!("\n📥 Executing 5 rapid pulls (rate limit: 1 req/s)...");
    let mut success_count = 0;
    let mut retry_count = 0;

    for i in 1..=5 {
        println!("  Pull {}/5...", i);
        let context_json_path = format!("/tmp/adapter_rate_test_{}.json", i);

        let pull_result = run_cap_adapter(&[
            "pull",
            "--odata",
            &sap_url,
            "--client",
            &sap_client,
            "--user",
            &sap_user,
            "--password-env",
            "SAP_PASSWORD",
            "--filter",
            "TIER le 2",
            "--out",
            &context_json_path,
        ]);

        match pull_result {
            Ok(output) => {
                success_count += 1;
                if output.contains("retry") || output.contains("429") {
                    retry_count += 1;
                }
            }
            Err(e) => {
                eprintln!("  Pull {} failed: {}", i, e);
                // Check if failure is due to rate limiting (expected)
                if e.contains("429") || e.contains("rate limit") {
                    retry_count += 1;
                    // Expected failure, continue
                } else {
                    // Unexpected failure
                    panic!("Unexpected failure on pull {}: {}", i, e);
                }
            }
        }

        // Small delay between requests to simulate real-world scenario
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    println!("\n📊 Results:");
    println!("  Successful pulls: {}/5", success_count);
    println!("  Retries/Rate limits: {}", retry_count);

    // We expect at least some retries due to rate limiting
    assert!(
        retry_count > 0,
        "Expected rate limiting to be triggered, but no retries detected"
    );

    // But all requests should eventually succeed (with retries)
    assert!(
        success_count >= 3,
        "Expected at least 3 successful pulls with retry logic, got {}",
        success_count
    );

    println!("✅ Rate limiting test PASSED (adapter respects rate limits and retries)");
}

#[test]
#[ignore] // Requires live SAP system
fn test_adapter_audit_trail() {
    println!("🧪 SAP Adapter Audit Trail Test");

    let sap_url = get_env_or_panic("SAP_URL");
    let sap_client = get_env_or_panic("SAP_CLIENT");
    let sap_user = get_env_or_panic("SAP_USER");
    let run_id = format!(
        "AUDIT_TEST_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let audit_log_path = "/tmp/adapter_audit_test.jsonl";

    println!("📋 Run ID: {}", run_id);
    println!("📝 Audit log: {}", audit_log_path);

    // Clean up any existing audit log
    let _ = fs::remove_file(audit_log_path);

    // Execute full E2E with audit logging
    let context_json_path = "/tmp/adapter_audit_context.json";
    let verify_json_path = "/tmp/adapter_audit_verify.json";

    println!("\n📥 Phase 1: Pull with audit logging...");
    run_cap_adapter(&[
        "pull",
        "--odata",
        &sap_url,
        "--client",
        &sap_client,
        "--user",
        &sap_user,
        "--password-env",
        "SAP_PASSWORD",
        "--filter",
        "TIER le 2",
        "--out",
        context_json_path,
        "--audit",
        audit_log_path,
    ])
    .expect("Pull failed");

    println!("\n🔍 Phase 2: Verify...");
    let verify_response = verify_via_api(context_json_path).expect("Verify failed");
    fs::write(verify_json_path, verify_response.to_string()).expect("Failed to write verify.json");

    println!("\n📝 Phase 3: Writeback with audit logging...");
    run_cap_adapter(&[
        "writeback",
        "--in",
        verify_json_path,
        "--odata",
        &sap_url,
        "--table",
        "Z_CAP_SUPPLIER_STATUS",
        "--idempotency",
        &run_id,
        "--user",
        &sap_user,
        "--password-env",
        "SAP_PASSWORD",
        "--batch-size",
        "10",
        "--audit",
        audit_log_path,
    ])
    .expect("Writeback failed");

    // Read and validate audit log
    println!("\n🔍 Validating audit log...");
    let audit_content = fs::read_to_string(audit_log_path).expect("Failed to read audit log");
    let audit_lines: Vec<&str> = audit_content.lines().collect();

    assert!(
        audit_lines.len() >= 5,
        "Expected at least 5 audit events, got {}",
        audit_lines.len()
    );
    println!("📊 Audit log contains {} events", audit_lines.len());

    // Parse and validate audit events
    let mut pull_start_found = false;
    let mut pull_complete_found = false;
    let mut verify_request_found = false;
    let mut verify_response_found = false;
    let mut writeback_complete_found = false;

    for (i, line) in audit_lines.iter().enumerate() {
        let event: Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("Failed to parse audit line {}: {}\nLine: {}", i, e, line));

        // Validate common fields
        assert!(
            event["timestamp"].is_string(),
            "Audit line {}: timestamp missing",
            i
        );
        assert!(
            event["event"].is_string(),
            "Audit line {}: event type missing",
            i
        );
        assert!(
            event["payload"].is_object(),
            "Audit line {}: payload missing",
            i
        );

        // Track event types
        match event["event"].as_str().unwrap() {
            "pull_start" => pull_start_found = true,
            "pull_complete" => {
                pull_complete_found = true;
                assert!(
                    event["payload"]["record_count"].is_number(),
                    "pull_complete missing record_count"
                );
            }
            "verify_request" => verify_request_found = true,
            "verify_response" => {
                verify_response_found = true;
                assert!(
                    event["payload"]["manifest_hash"].is_string(),
                    "verify_response missing manifest_hash"
                );
                assert!(
                    event["payload"]["proof_hash"].is_string(),
                    "verify_response missing proof_hash"
                );
            }
            "writeback_complete" => {
                writeback_complete_found = true;
                assert!(
                    event["payload"]["total_written"].is_number(),
                    "writeback_complete missing total_written"
                );
            }
            _ => {}
        }
    }

    // Validate that all critical events are present
    assert!(pull_start_found, "Audit log missing pull_start event");
    assert!(pull_complete_found, "Audit log missing pull_complete event");
    assert!(
        verify_request_found,
        "Audit log missing verify_request event"
    );
    assert!(
        verify_response_found,
        "Audit log missing verify_response event"
    );
    assert!(
        writeback_complete_found,
        "Audit log missing writeback_complete event"
    );

    println!("✅ All critical audit events present");
    println!("  - pull_start: ✓");
    println!("  - pull_complete: ✓");
    println!("  - verify_request: ✓");
    println!("  - verify_response: ✓");
    println!("  - writeback_complete: ✓");

    println!("\n🎉 Audit Trail Test PASSED");
}

#[test]
#[ignore] // Requires live SAP system
fn test_adapter_error_handling() {
    println!("🧪 SAP Adapter Error Handling Test");

    let sap_url = get_env_or_panic("SAP_URL");
    let sap_client = get_env_or_panic("SAP_CLIENT");

    // Test 1: Invalid credentials (401 Unauthorized)
    println!("\n🔐 Test 1: Invalid credentials...");
    let pull_result = run_cap_adapter(&[
        "pull",
        "--odata",
        &sap_url,
        "--client",
        &sap_client,
        "--user",
        "INVALID_USER",
        "--password-env",
        "NONEXISTENT_PASSWORD",
        "--filter",
        "TIER le 2",
        "--out",
        "/tmp/adapter_error_test.json",
    ]);

    assert!(
        pull_result.is_err(),
        "Expected pull to fail with invalid credentials"
    );
    let error_msg = pull_result.err().unwrap();
    assert!(
        error_msg.contains("401") || error_msg.contains("Unauthorized"),
        "Expected 401 error, got: {}",
        error_msg
    );
    println!("✅ Invalid credentials correctly rejected");

    // Test 2: Invalid OData URL (connection error)
    println!("\n🌐 Test 2: Invalid OData URL...");
    let pull_result_2 = run_cap_adapter(&[
        "pull",
        "--odata",
        "https://invalid-sap-url.example.com/invalid",
        "--client",
        &sap_client,
        "--user",
        "CAP_ADAPTER",
        "--password-env",
        "SAP_PASSWORD",
        "--filter",
        "TIER le 2",
        "--out",
        "/tmp/adapter_error_test2.json",
    ]);

    assert!(
        pull_result_2.is_err(),
        "Expected pull to fail with invalid URL"
    );
    let error_msg_2 = pull_result_2.err().unwrap();
    assert!(
        error_msg_2.contains("connection")
            || error_msg_2.contains("timeout")
            || error_msg_2.contains("resolve"),
        "Expected connection error, got: {}",
        error_msg_2
    );
    println!("✅ Connection error correctly handled");

    // Test 3: Invalid filter syntax (400 Bad Request)
    println!("\n🔍 Test 3: Invalid OData filter...");
    let sap_user = get_env_or_panic("SAP_USER");
    let pull_result_3 = run_cap_adapter(&[
        "pull",
        "--odata",
        &sap_url,
        "--client",
        &sap_client,
        "--user",
        &sap_user,
        "--password-env",
        "SAP_PASSWORD",
        "--filter",
        "INVALID FILTER SYNTAX !!!",
        "--out",
        "/tmp/adapter_error_test3.json",
    ]);

    assert!(
        pull_result_3.is_err(),
        "Expected pull to fail with invalid filter"
    );
    let error_msg_3 = pull_result_3.err().unwrap();
    assert!(
        error_msg_3.contains("400")
            || error_msg_3.contains("Bad Request")
            || error_msg_3.contains("filter"),
        "Expected 400 error, got: {}",
        error_msg_3
    );
    println!("✅ Invalid filter correctly rejected");

    println!("\n🎉 Error Handling Test PASSED");
}
