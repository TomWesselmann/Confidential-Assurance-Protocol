/// Integration Tests for Week 4 - HTTP Flows (IT-01 to IT-09)
///
/// Tests the complete REST API with PolicyV2 compiler and dual-mode verification.
/// Requires the REST API server to be running.

use serde_json::{json, Value};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

/// Helper: Start REST API server in background
fn start_test_server() -> std::process::Child {
    let child = Command::new("cargo")
        .args(&["run", "--bin", "cap-verifier-api"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start REST API server");

    // Wait for server to be ready
    thread::sleep(Duration::from_secs(3));

    child
}

/// Helper: Generate mock OAuth2 token
fn generate_mock_token() -> String {
    // For testing, we'll use a simple mock token
    // In production, this would come from an OAuth2 provider
    let output = Command::new("cargo")
        .args(&["run", "--example", "generate_mock_token"])
        .output()
        .expect("Failed to generate mock token");

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines()
        .find(|line| line.starts_with("eyJ"))
        .unwrap_or("")
        .to_string()
}

/// Helper: HTTP POST request
fn http_post(url: &str, token: &str, body: Value) -> (u16, Value) {
    let client = reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true) // For self-signed certs in testing
        .build()
        .expect("Failed to create HTTP client");

    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .expect("HTTP request failed");

    let status = response.status().as_u16();
    let json = response.json::<Value>().unwrap_or(json!({}));

    (status, json)
}

/// Helper: HTTP GET request
fn http_get(url: &str, token: &str, if_none_match: Option<&str>) -> (u16, Value) {
    let client = reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("Failed to create HTTP client");

    let mut req = client
        .get(url)
        .header("Authorization", format!("Bearer {}", token));

    if let Some(etag) = if_none_match {
        req = req.header("If-None-Match", etag);
    }

    let response = req.send().expect("HTTP request failed");
    let status = response.status().as_u16();
    let json = response.json::<Value>().unwrap_or(json!({}));

    (status, json)
}

/// Helper: Create valid PolicyV2 for testing
fn create_valid_policy() -> Value {
    json!({
        "id": "test.policy.v1",
        "version": "1.0",
        "legal_basis": {
            "law": "LkSG",
            "jurisdiction": "DE"
        },
        "inputs": [],
        "rules": [
            {
                "rule_id": "require_at_least_one_ubo",
                "description": "At least one UBO required",
                "operator": "non_membership",
                "args": {
                    "set_var": "ubo_hashes",
                    "element": {"var": "empty_set"}
                }
            }
        ]
    })
}

/// Helper: Create invalid PolicyV2 (missing legal_basis)
fn create_invalid_policy() -> Value {
    json!({
        "id": "test.policy.invalid",
        "version": "1.0",
        // Missing legal_basis - should trigger E1002
        "inputs": [],
        "rules": []
    })
}

/// Helper: Create valid verification context
fn create_valid_context() -> Value {
    json!({
        "supplier_hashes": ["0x1234567890123456789012345678901234567890123456789012345678901234"],
        "ubo_hashes": ["0xabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd"],
        "company_commitment_root": null,
        "sanctions_root": null,
        "jurisdiction_root": null
    })
}

#[test]
#[ignore] // Run with: cargo test --test test_integration_http -- --ignored
fn it_01_policy_compile_valid_strict() {
    let mut _server = start_test_server();
    let token = generate_mock_token();

    let policy = create_valid_policy();
    let request = json!({
        "policy": policy,
        "lint_mode": "strict",
        "persist": true
    });

    let (status, response) = http_post("http://localhost:8080/policy/compile", &token, request);

    // Assertions
    assert_eq!(status, 200, "Expected 200 OK");
    assert!(response.get("policy_hash").is_some(), "Expected policy_hash");
    assert!(response.get("ir").is_some(), "Expected ir");
    assert!(response.get("ir_hash").is_some(), "Expected ir_hash");
    assert!(response.get("etag").is_some(), "Expected etag");
    assert_eq!(response["stored"], true, "Expected stored=true");

    println!("✅ IT-01: POST /policy/compile (valid, strict) - PASSED");
}

#[test]
#[ignore]
fn it_02_policy_compile_missing_legal_basis() {
    let mut _server = start_test_server();
    let token = generate_mock_token();

    let policy = create_invalid_policy();
    let request = json!({
        "policy": policy,
        "lint_mode": "strict",
        "persist": false
    });

    let (status, response) = http_post("http://localhost:8080/policy/compile", &token, request);

    // Assertions
    assert_eq!(status, 422, "Expected 422 Unprocessable Entity");
    assert!(response.get("lints").is_some(), "Expected lints array");

    let lints = response["lints"].as_array().expect("lints should be array");
    let has_e1002 = lints.iter().any(|lint| {
        lint["code"].as_str() == Some("E1002")
    });

    assert!(has_e1002, "Expected E1002 lint error for missing legal_basis");

    println!("✅ IT-02: POST /policy/compile (missing legal_basis) - PASSED");
}

#[test]
#[ignore]
fn it_03_verify_policy_mode_ok() {
    let mut _server = start_test_server();
    let token = generate_mock_token();

    // First, compile and persist policy
    let policy = create_valid_policy();
    let compile_request = json!({
        "policy": policy,
        "lint_mode": "strict",
        "persist": true
    });

    let (status, _response) = http_post("http://localhost:8080/policy/compile", &token, compile_request);
    assert_eq!(status, 200, "Policy compilation failed");

    // Now verify with policy_id (Mode A)
    let verify_request = json!({
        "policy_id": "test.policy.v1",
        "context": create_valid_context(),
        "backend": "mock",
        "options": {"adaptive": false}
    });

    let (status, response) = http_post("http://localhost:8080/verify", &token, verify_request);

    // Assertions
    assert_eq!(status, 200, "Expected 200 OK");
    assert_eq!(response["result"], "OK", "Expected result=OK");
    assert!(response.get("trace").is_some(), "Expected trace");
    assert!(response.get("manifest_hash").is_some(), "Expected manifest_hash");
    assert!(response.get("proof_hash").is_some(), "Expected proof_hash");

    println!("✅ IT-03: POST /verify (Policy mode, OK) - PASSED");
}

#[test]
#[ignore]
fn it_04_verify_embedded_ir_ok() {
    let mut _server = start_test_server();
    let token = generate_mock_token();

    // First, compile policy to get IR
    let policy = create_valid_policy();
    let compile_request = json!({
        "policy": policy,
        "lint_mode": "strict",
        "persist": true
    });

    let (status, compile_response) = http_post("http://localhost:8080/policy/compile", &token, compile_request);
    assert_eq!(status, 200, "Policy compilation failed");

    let ir = compile_response["ir"].clone();

    // Now verify with embedded IR (Mode B)
    let verify_request = json!({
        "ir": ir,
        "context": create_valid_context(),
        "backend": "mock",
        "options": {"adaptive": false}
    });

    let (status, response) = http_post("http://localhost:8080/verify", &token, verify_request);

    // Assertions
    assert_eq!(status, 200, "Expected 200 OK");
    assert_eq!(response["result"], "OK", "Expected result=OK");
    assert!(response.get("trace").is_some(), "Expected trace");
    assert!(response.get("manifest_hash").is_some(), "Expected manifest_hash");

    println!("✅ IT-04: POST /verify (Embedded IR, OK) - PASSED");
}

#[test]
#[ignore]
fn it_05_verify_mode_ab_equivalence() {
    let mut _server = start_test_server();
    let token = generate_mock_token();

    // Compile policy
    let policy = create_valid_policy();
    let compile_request = json!({
        "policy": policy,
        "lint_mode": "strict",
        "persist": true
    });

    let (_, compile_response) = http_post("http://localhost:8080/policy/compile", &token, compile_request);
    let ir = compile_response["ir"].clone();

    let context = create_valid_context();

    // Mode A: policy_id
    let verify_a = json!({
        "policy_id": "test.policy.v1",
        "context": context,
        "backend": "mock",
        "options": {"adaptive": false}
    });

    let (_, response_a) = http_post("http://localhost:8080/verify", &token, verify_a);

    // Mode B: embedded IR
    let verify_b = json!({
        "ir": ir,
        "context": context,
        "backend": "mock",
        "options": {"adaptive": false}
    });

    let (_, response_b) = http_post("http://localhost:8080/verify", &token, verify_b);

    // Equivalence check
    assert_eq!(response_a["result"], response_b["result"], "Results should match");
    assert_eq!(response_a["manifest_hash"], response_b["manifest_hash"], "Manifest hashes should match");
    assert_eq!(response_a["proof_hash"], response_b["proof_hash"], "Proof hashes should match");

    println!("✅ IT-05: Mode A/B Equivalence - PASSED");
}

#[test]
#[ignore]
fn it_06_policy_get_with_etag_304() {
    let mut _server = start_test_server();
    let token = generate_mock_token();

    // Compile and persist policy
    let policy = create_valid_policy();
    let compile_request = json!({
        "policy": policy,
        "lint_mode": "strict",
        "persist": true
    });

    let (_, compile_response) = http_post("http://localhost:8080/policy/compile", &token, compile_request);
    let etag = compile_response["etag"].as_str().expect("Expected etag");

    // GET with If-None-Match
    let (status, _) = http_get("http://localhost:8080/policy/test.policy.v1", &token, Some(etag));

    // Assertion
    assert_eq!(status, 304, "Expected 304 Not Modified");

    println!("✅ IT-06: GET /policy/:id with ETag (304) - PASSED");
}

#[test]
#[ignore]
fn it_07_verify_without_auth_401() {
    let mut _server = start_test_server();

    let verify_request = json!({
        "policy_id": "test.policy.v1",
        "context": create_valid_context(),
        "backend": "mock"
    });

    // No token provided
    let client = reqwest::blocking::Client::new();
    let response = client
        .post("http://localhost:8080/verify")
        .json(&verify_request)
        .send()
        .expect("HTTP request failed");

    let status = response.status().as_u16();

    // Assertion
    assert_eq!(status, 401, "Expected 401 Unauthorized");

    println!("✅ IT-07: POST /verify without OAuth2 (401) - PASSED");
}

#[test]
#[ignore]
fn it_08_verify_invalid_scope_403() {
    let mut _server = start_test_server();

    // Use invalid token (expired or wrong scope)
    let invalid_token = "invalid.jwt.token";

    let verify_request = json!({
        "policy_id": "test.policy.v1",
        "context": create_valid_context(),
        "backend": "mock"
    });

    let client = reqwest::blocking::Client::new();
    let response = client
        .post("http://localhost:8080/verify")
        .header("Authorization", format!("Bearer {}", invalid_token))
        .json(&verify_request)
        .send()
        .expect("HTTP request failed");

    let status = response.status().as_u16();

    // Assertion (could be 401 or 403 depending on implementation)
    assert!(status == 401 || status == 403, "Expected 401 or 403");

    println!("✅ IT-08: POST /verify with invalid token (401/403) - PASSED");
}

#[test]
#[ignore]
fn it_09_policy_conflict_409() {
    let mut _server = start_test_server();
    let token = generate_mock_token();

    // Compile and persist policy
    let policy1 = create_valid_policy();
    let compile_request1 = json!({
        "policy": policy1,
        "lint_mode": "strict",
        "persist": true
    });

    let (status1, _) = http_post("http://localhost:8080/policy/compile", &token, compile_request1);
    assert_eq!(status1, 200, "First compilation should succeed");

    // Try to compile different policy with same ID (should conflict)
    let mut policy2 = create_valid_policy();
    policy2["rules"] = json!([]); // Different content, same ID

    let compile_request2 = json!({
        "policy": policy2,
        "lint_mode": "strict",
        "persist": true
    });

    let (status2, _) = http_post("http://localhost:8080/policy/compile", &token, compile_request2);

    // Assertion
    assert_eq!(status2, 409, "Expected 409 Conflict");

    println!("✅ IT-09: POST /policy/compile with hash conflict (409) - PASSED");
}

#[test]
#[ignore]
fn test_healthz_public() {
    let mut _server = start_test_server();

    let client = reqwest::blocking::Client::new();
    let response = client
        .get("http://localhost:8080/healthz")
        .send()
        .expect("HTTP request failed");

    let status = response.status().as_u16();
    let json = response.json::<Value>().expect("JSON parse failed");

    assert_eq!(status, 200, "Expected 200 OK");
    assert_eq!(json["status"], "OK", "Expected status=OK");

    println!("✅ GET /healthz (public) - PASSED");
}

#[test]
#[ignore]
fn test_readyz_public() {
    let mut _server = start_test_server();

    let client = reqwest::blocking::Client::new();
    let response = client
        .get("http://localhost:8080/readyz")
        .send()
        .expect("HTTP request failed");

    let status = response.status().as_u16();
    let json = response.json::<Value>().expect("JSON parse failed");

    assert_eq!(status, 200, "Expected 200 OK");
    assert_eq!(json["status"], "OK", "Expected status=OK");

    println!("✅ GET /readyz (public) - PASSED");
}
