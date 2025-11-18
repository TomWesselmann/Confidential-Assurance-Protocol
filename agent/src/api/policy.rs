use crate::policy::{Policy, PolicyStore, PolicyMetadata};
/// Policy API - REST Endpoints for Policy Management
///
/// Endpoints:
/// - POST /policy/compile - Compiles and validates a policy
/// - GET /policy/:id - Retrieves a policy by hash or UUID
use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// Application State
// ============================================================================

/// Shared application state containing the policy store
#[derive(Clone)]
pub struct PolicyState {
    pub store: Arc<RwLock<Box<dyn PolicyStore + Send + Sync>>>,
}

impl PolicyState {
    pub fn new<S: PolicyStore + Send + Sync + 'static>(store: S) -> Self {
        Self {
            store: Arc::new(RwLock::new(Box::new(store))),
        }
    }
}

// ============================================================================
// Request/Response Structures
// ============================================================================

/// Request for compiling a policy
#[derive(Debug, Deserialize)]
pub struct PolicyCompileRequest {
    /// Policy definition (YAML or JSON as string)
    pub policy: Policy,
}

/// Response after policy compilation
#[derive(Debug, Serialize)]
pub struct PolicyCompileResponse {
    /// Policy UUID
    pub policy_id: String,
    /// Policy hash (SHA3-256)
    pub policy_hash: String,
    /// Policy metadata
    pub metadata: PolicyMetadata,
    /// Status message
    pub status: String,
}

/// Response for policy retrieval
#[derive(Debug, Serialize)]
pub struct PolicyGetResponse {
    /// Policy metadata
    pub metadata: PolicyMetadata,
    /// Full policy definition
    pub policy: Policy,
}

// ============================================================================
// Handlers
// ============================================================================

/// Handles POST /policy/compile
///
/// Validates and compiles a policy, returns policy hash and metadata
pub async fn handle_policy_compile(
    State(state): State<PolicyState>,
    Json(request): Json<PolicyCompileRequest>,
) -> Result<Json<PolicyCompileResponse>, StatusCode> {
    // Validate policy
    request.policy.validate().map_err(|e| {
        tracing::warn!("Policy validation failed: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    // Save policy (this computes hash and creates metadata)
    let store = state.store.write().await;
    let metadata = store.save(request.policy.clone()).await.map_err(|e| {
        tracing::error!("Failed to save policy: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!("Policy compiled successfully: {} (hash: {})", metadata.id, metadata.hash);

    Ok(Json(PolicyCompileResponse {
        policy_id: metadata.id.to_string(),
        policy_hash: metadata.hash.clone(),
        metadata,
        status: "compiled".to_string(),
    }))
}

/// Handles GET /policy/:id
///
/// Retrieves a policy by its UUID or hash
pub async fn handle_policy_get(
    State(state): State<PolicyState>,
    Path(policy_id): Path<String>,
) -> Result<Json<PolicyGetResponse>, StatusCode> {
    let store = state.store.read().await;

    // Try to get by UUID first, then by hash
    let compiled_policy = if policy_id.starts_with("0x") {
        // Hash lookup
        store.get_by_hash(&policy_id).await.map_err(|e| {
            tracing::error!("Failed to retrieve policy by hash: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
    } else {
        // UUID lookup
        store.get(&policy_id).await.map_err(|e| {
            tracing::error!("Failed to retrieve policy by ID: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
    };

    match compiled_policy {
        Some(compiled) => {
            tracing::info!("Policy retrieved successfully: {}", policy_id);
            Ok(Json(PolicyGetResponse {
                metadata: compiled.metadata,
                policy: compiled.policy,
            }))
        }
        None => {
            tracing::warn!("Policy not found: {}", policy_id);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::{PolicyConstraints, InMemoryPolicyStore};

    #[tokio::test]
    async fn test_policy_compile_and_get() {
        let policy = Policy {
            version: "lksg.v1".to_string(),
            name: "Test Policy".to_string(),
            created_at: "2025-11-06T10:00:00Z".to_string(),
            constraints: PolicyConstraints {
                require_at_least_one_ubo: true,
                supplier_count_max: 10,
                ubo_count_min: None,
                require_statement_roots: None,
            },
            notes: "Test policy".to_string(),
        };

        // Create state with in-memory store
        let state = PolicyState::new(InMemoryPolicyStore::new());

        // Compile policy
        let compile_req = PolicyCompileRequest {
            policy: policy.clone(),
        };
        let compile_resp = handle_policy_compile(
            State(state.clone()),
            Json(compile_req),
        )
        .await
        .expect("Compilation should succeed");

        let resp_data = compile_resp.0;
        assert_eq!(resp_data.status, "compiled");
        assert!(resp_data.policy_hash.starts_with("0x"));
        assert_eq!(resp_data.policy_hash.len(), 66); // 0x + 64 hex chars

        // Retrieve by UUID
        let get_resp = handle_policy_get(
            State(state.clone()),
            Path(resp_data.policy_id.clone()),
        )
        .await
        .expect("Get by UUID should succeed");

        assert_eq!(get_resp.0.policy.name, "Test Policy");

        // Retrieve by hash
        let get_resp_hash = handle_policy_get(
            State(state.clone()),
            Path(resp_data.policy_hash.clone()),
        )
        .await
        .expect("Get by hash should succeed");

        assert_eq!(get_resp_hash.0.policy.name, "Test Policy");
    }

    #[tokio::test]
    async fn test_policy_not_found() {
        let state = PolicyState::new(InMemoryPolicyStore::new());

        let result = handle_policy_get(
            State(state),
            Path("nonexistent-id".to_string()),
        )
        .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);
    }
}
