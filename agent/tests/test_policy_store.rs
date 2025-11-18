/// Integration Tests for Policy Store System
///
/// Tests both InMemoryPolicyStore and SqlitePolicyStore implementations
use cap_agent::policy::{
    InMemoryPolicyStore, Policy, PolicyConstraints, PolicyStatus, PolicyStore,
    SqlitePolicyStore,
};
use tempfile::tempdir;

/// Helper function to create a test policy
fn create_test_policy(name: &str, version: &str) -> Policy {
    Policy {
        version: version.to_string(),
        name: name.to_string(),
        created_at: "2025-11-18T10:00:00Z".to_string(),
        constraints: PolicyConstraints {
            require_at_least_one_ubo: true,
            supplier_count_max: 10,
            ubo_count_min: Some(1),
            require_statement_roots: None,
        },
        notes: format!("Test policy: {}", name),
    }
}

// ============================================================================
// InMemoryPolicyStore Tests
// ============================================================================

#[tokio::test]
async fn test_inmemory_save_and_get() {
    let store = InMemoryPolicyStore::new();
    let policy = create_test_policy("Test Policy 1", "lksg.v1");

    // Save policy
    let metadata = store.save(policy.clone()).await.unwrap();

    // Verify metadata
    assert_eq!(metadata.name, "Test Policy 1");
    assert_eq!(metadata.version, "lksg.v1");
    assert_eq!(metadata.status, PolicyStatus::Active);
    assert!(metadata.hash.starts_with("0x"));
    assert_eq!(metadata.hash.len(), 66); // 0x + 64 hex chars

    // Get by ID
    let retrieved = store.get(&metadata.id.to_string()).await.unwrap();
    assert!(retrieved.is_some());
    let compiled = retrieved.unwrap();
    assert_eq!(compiled.policy.name, "Test Policy 1");
    assert_eq!(compiled.metadata.id, metadata.id);
}

#[tokio::test]
async fn test_inmemory_get_by_hash() {
    let store = InMemoryPolicyStore::new();
    let policy = create_test_policy("Test Policy 2", "lksg.v1");

    // Save policy
    let metadata = store.save(policy.clone()).await.unwrap();

    // Get by hash
    let retrieved = store.get_by_hash(&metadata.hash).await.unwrap();
    assert!(retrieved.is_some());
    let compiled = retrieved.unwrap();
    assert_eq!(compiled.policy.name, "Test Policy 2");
    assert_eq!(compiled.metadata.hash, metadata.hash);
}

#[tokio::test]
async fn test_inmemory_deduplication() {
    let store = InMemoryPolicyStore::new();
    let policy = create_test_policy("Test Policy 3", "lksg.v1");

    // Save same policy twice
    let metadata1 = store.save(policy.clone()).await.unwrap();
    let metadata2 = store.save(policy.clone()).await.unwrap();

    // Should return same ID (deduplication by hash)
    assert_eq!(metadata1.id, metadata2.id);
    assert_eq!(metadata1.hash, metadata2.hash);
}

#[tokio::test]
async fn test_inmemory_list() {
    let store = InMemoryPolicyStore::new();

    // Save multiple policies
    let policy1 = create_test_policy("Policy A", "lksg.v1");
    let policy2 = create_test_policy("Policy B", "lksg.v1");
    let policy3 = create_test_policy("Policy C", "lksg.v1");

    store.save(policy1).await.unwrap();
    store.save(policy2).await.unwrap();
    let metadata3 = store.save(policy3).await.unwrap();

    // List all
    let all = store.list(None).await.unwrap();
    assert_eq!(all.len(), 3);

    // Set one policy to deprecated
    store
        .set_status(&metadata3.id.to_string(), PolicyStatus::Deprecated)
        .await
        .unwrap();

    // List only active
    let active = store.list(Some(PolicyStatus::Active)).await.unwrap();
    assert_eq!(active.len(), 2);

    // List only deprecated
    let deprecated = store.list(Some(PolicyStatus::Deprecated)).await.unwrap();
    assert_eq!(deprecated.len(), 1);
    assert_eq!(deprecated[0].name, "Policy C");
}

#[tokio::test]
async fn test_inmemory_set_status() {
    let store = InMemoryPolicyStore::new();
    let policy = create_test_policy("Test Policy 4", "lksg.v1");

    let metadata = store.save(policy).await.unwrap();
    assert_eq!(metadata.status, PolicyStatus::Active);

    // Change to Draft
    store
        .set_status(&metadata.id.to_string(), PolicyStatus::Draft)
        .await
        .unwrap();

    let retrieved = store.get(&metadata.id.to_string()).await.unwrap().unwrap();
    assert_eq!(retrieved.metadata.status, PolicyStatus::Draft);

    // Change to Deprecated
    store
        .set_status(&metadata.id.to_string(), PolicyStatus::Deprecated)
        .await
        .unwrap();

    let retrieved = store.get(&metadata.id.to_string()).await.unwrap().unwrap();
    assert_eq!(retrieved.metadata.status, PolicyStatus::Deprecated);
}

#[tokio::test]
async fn test_inmemory_not_found() {
    let store = InMemoryPolicyStore::new();

    // Get non-existent ID
    let result = store.get("nonexistent-id").await.unwrap();
    assert!(result.is_none());

    // Get by non-existent hash
    let result = store.get_by_hash("0x0000000000000000000000000000000000000000000000000000000000000000").await.unwrap();
    assert!(result.is_none());

    // Set status on non-existent ID
    let result = store
        .set_status("nonexistent-id", PolicyStatus::Deprecated)
        .await;
    assert!(result.is_err());
}

// ============================================================================
// SqlitePolicyStore Tests
// ============================================================================

#[tokio::test]
async fn test_sqlite_save_and_get() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let store = SqlitePolicyStore::new(db_path.to_str().unwrap()).unwrap();

    let policy = create_test_policy("SQLite Test Policy 1", "lksg.v1");

    // Save policy
    let metadata = store.save(policy.clone()).await.unwrap();

    // Verify metadata
    assert_eq!(metadata.name, "SQLite Test Policy 1");
    assert_eq!(metadata.version, "lksg.v1");
    assert_eq!(metadata.status, PolicyStatus::Active);
    assert!(metadata.hash.starts_with("0x"));

    // Get by ID
    let retrieved = store.get(&metadata.id.to_string()).await.unwrap();
    assert!(retrieved.is_some());
    let compiled = retrieved.unwrap();
    assert_eq!(compiled.policy.name, "SQLite Test Policy 1");
}

#[tokio::test]
async fn test_sqlite_get_by_hash() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let store = SqlitePolicyStore::new(db_path.to_str().unwrap()).unwrap();

    let policy = create_test_policy("SQLite Test Policy 2", "lksg.v1");

    let metadata = store.save(policy).await.unwrap();

    // Get by hash
    let retrieved = store.get_by_hash(&metadata.hash).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().metadata.hash, metadata.hash);
}

#[tokio::test]
async fn test_sqlite_deduplication() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let store = SqlitePolicyStore::new(db_path.to_str().unwrap()).unwrap();

    let policy = create_test_policy("SQLite Test Policy 3", "lksg.v1");

    // Save same policy twice
    let metadata1 = store.save(policy.clone()).await.unwrap();
    let metadata2 = store.save(policy.clone()).await.unwrap();

    // Should return same ID (deduplication by hash)
    assert_eq!(metadata1.id, metadata2.id);
    assert_eq!(metadata1.hash, metadata2.hash);

    // Verify only one entry in database
    let all = store.list(None).await.unwrap();
    assert_eq!(all.len(), 1);
}

#[tokio::test]
async fn test_sqlite_list() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let store = SqlitePolicyStore::new(db_path.to_str().unwrap()).unwrap();

    // Save multiple policies
    let policy1 = create_test_policy("SQLite Policy A", "lksg.v1");
    let policy2 = create_test_policy("SQLite Policy B", "lksg.v1");
    let policy3 = create_test_policy("SQLite Policy C", "lksg.v1");

    store.save(policy1).await.unwrap();
    store.save(policy2).await.unwrap();
    let metadata3 = store.save(policy3).await.unwrap();

    // List all
    let all = store.list(None).await.unwrap();
    assert_eq!(all.len(), 3);

    // Set one policy to deprecated
    store
        .set_status(&metadata3.id.to_string(), PolicyStatus::Deprecated)
        .await
        .unwrap();

    // List only active
    let active = store.list(Some(PolicyStatus::Active)).await.unwrap();
    assert_eq!(active.len(), 2);

    // List only deprecated
    let deprecated = store.list(Some(PolicyStatus::Deprecated)).await.unwrap();
    assert_eq!(deprecated.len(), 1);
}

#[tokio::test]
async fn test_sqlite_set_status() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let store = SqlitePolicyStore::new(db_path.to_str().unwrap()).unwrap();

    let policy = create_test_policy("SQLite Test Policy 4", "lksg.v1");

    let metadata = store.save(policy).await.unwrap();

    // Change to Draft
    store
        .set_status(&metadata.id.to_string(), PolicyStatus::Draft)
        .await
        .unwrap();

    let retrieved = store.get(&metadata.id.to_string()).await.unwrap().unwrap();
    assert_eq!(retrieved.metadata.status, PolicyStatus::Draft);
}

#[tokio::test]
async fn test_sqlite_persistence() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let policy = create_test_policy("Persistent Policy", "lksg.v1");

    // Save with first store instance
    let metadata = {
        let store = SqlitePolicyStore::new(db_path.to_str().unwrap()).unwrap();
        store.save(policy).await.unwrap()
    };

    // Retrieve with new store instance (tests persistence)
    {
        let store = SqlitePolicyStore::new(db_path.to_str().unwrap()).unwrap();
        let retrieved = store.get(&metadata.id.to_string()).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().policy.name, "Persistent Policy");
    }
}

#[tokio::test]
async fn test_sqlite_not_found() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let store = SqlitePolicyStore::new(db_path.to_str().unwrap()).unwrap();

    // Get non-existent ID
    let result = store.get("nonexistent-id").await.unwrap();
    assert!(result.is_none());

    // Get by non-existent hash
    let result = store
        .get_by_hash("0x0000000000000000000000000000000000000000000000000000000000000000")
        .await
        .unwrap();
    assert!(result.is_none());

    // Set status on non-existent ID
    let result = store
        .set_status("nonexistent-id", PolicyStatus::Deprecated)
        .await;
    assert!(result.is_err());
}

// ============================================================================
// API Integration Tests
// ============================================================================

#[tokio::test]
async fn test_api_policy_compile_and_get() {
    use cap_agent::api::policy::{PolicyCompileRequest, PolicyState};
    use cap_agent::api::policy::{handle_policy_compile, handle_policy_get};
    use axum::extract::{Path, State};
    use axum::Json;

    let state = PolicyState::new(InMemoryPolicyStore::new());
    let policy = create_test_policy("API Test Policy", "lksg.v1");

    // Compile policy
    let req = PolicyCompileRequest { policy };
    let response = handle_policy_compile(State(state.clone()), Json(req))
        .await
        .expect("Compile should succeed");

    let compile_data = response.0;
    assert_eq!(compile_data.status, "compiled");
    assert!(compile_data.policy_hash.starts_with("0x"));

    // Get by UUID
    let get_response = handle_policy_get(State(state.clone()), Path(compile_data.policy_id.clone()))
        .await
        .expect("Get by UUID should succeed");

    assert_eq!(get_response.0.policy.name, "API Test Policy");

    // Get by hash
    let get_response_hash =
        handle_policy_get(State(state), Path(compile_data.policy_hash.clone()))
            .await
            .expect("Get by hash should succeed");

    assert_eq!(get_response_hash.0.policy.name, "API Test Policy");
}

#[tokio::test]
async fn test_api_policy_not_found() {
    use cap_agent::api::policy::{handle_policy_get, PolicyState};
    use axum::extract::{Path, State};
    use axum::http::StatusCode;

    let state = PolicyState::new(InMemoryPolicyStore::new());

    // Try to get non-existent policy by UUID
    let result = handle_policy_get(State(state.clone()), Path("nonexistent-uuid".to_string())).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);

    // Try to get non-existent policy by hash
    let result = handle_policy_get(
        State(state),
        Path("0x0000000000000000000000000000000000000000000000000000000000000000".to_string()),
    )
    .await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_api_policy_invalid_policy() {
    use cap_agent::api::policy::{handle_policy_compile, PolicyCompileRequest, PolicyState};
    use axum::extract::State;
    use axum::http::StatusCode;
    use axum::Json;

    let state = PolicyState::new(InMemoryPolicyStore::new());

    // Create invalid policy (empty name)
    let mut invalid_policy = create_test_policy("", "lksg.v1");
    invalid_policy.name = "".to_string();

    let req = PolicyCompileRequest {
        policy: invalid_policy,
    };

    let result = handle_policy_compile(State(state), Json(req)).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_api_policy_deduplication() {
    use cap_agent::api::policy::{handle_policy_compile, PolicyCompileRequest, PolicyState};
    use axum::extract::State;
    use axum::Json;

    let state = PolicyState::new(InMemoryPolicyStore::new());
    let policy = create_test_policy("Dedup Test Policy", "lksg.v1");

    // Compile same policy twice
    let req1 = PolicyCompileRequest {
        policy: policy.clone(),
    };
    let response1 = handle_policy_compile(State(state.clone()), Json(req1))
        .await
        .expect("First compile should succeed");

    let req2 = PolicyCompileRequest { policy };
    let response2 = handle_policy_compile(State(state), Json(req2))
        .await
        .expect("Second compile should succeed");

    // Should return same policy_id and hash (deduplication)
    assert_eq!(response1.0.policy_id, response2.0.policy_id);
    assert_eq!(response1.0.policy_hash, response2.0.policy_hash);
}

#[tokio::test]
async fn test_api_sqlite_backend() {
    use cap_agent::api::policy::{handle_policy_compile, handle_policy_get, PolicyCompileRequest, PolicyState};
    use axum::extract::{Path, State};
    use axum::Json;

    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("api_test.db");
    let store = SqlitePolicyStore::new(db_path.to_str().unwrap()).unwrap();
    let state = PolicyState::new(store);

    let policy = create_test_policy("SQLite API Test", "lksg.v1");

    // Compile policy
    let req = PolicyCompileRequest { policy };
    let response = handle_policy_compile(State(state.clone()), Json(req))
        .await
        .expect("Compile should succeed");

    let compile_data = response.0;

    // Get by UUID
    let get_response = handle_policy_get(State(state.clone()), Path(compile_data.policy_id.clone()))
        .await
        .expect("Get by UUID should succeed");

    assert_eq!(get_response.0.policy.name, "SQLite API Test");

    // Get by hash
    let get_response_hash = handle_policy_get(State(state), Path(compile_data.policy_hash.clone()))
        .await
        .expect("Get by hash should succeed");

    assert_eq!(get_response_hash.0.policy.name, "SQLite API Test");
}

#[tokio::test]
async fn test_api_concurrent_access() {
    use cap_agent::api::policy::{handle_policy_compile, PolicyCompileRequest, PolicyState};
    use axum::extract::State;
    use axum::Json;
    use std::sync::Arc;
    use tokio::task::JoinSet;

    let state = Arc::new(PolicyState::new(InMemoryPolicyStore::new()));
    let mut tasks = JoinSet::new();

    // Spawn 10 concurrent compile requests
    for i in 0..10 {
        let state_clone = state.clone();
        tasks.spawn(async move {
            let policy = create_test_policy(&format!("Concurrent Policy {}", i), "lksg.v1");
            let req = PolicyCompileRequest { policy };
            handle_policy_compile(State((*state_clone).clone()), Json(req)).await
        });
    }

    // Wait for all tasks to complete
    let mut results = Vec::new();
    while let Some(result) = tasks.join_next().await {
        results.push(result.unwrap());
    }

    // All should succeed
    assert_eq!(results.len(), 10);
    for result in results {
        assert!(result.is_ok());
    }
}
