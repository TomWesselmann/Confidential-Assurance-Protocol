/// Policy API - REST Endpoints for Policy Management
///
/// Endpoints:
/// - POST /policy/compile - Compiles and validates a policy
/// - GET /policy/:id - Retrieves a policy by hash

use axum::{
    extract::Path,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, OnceLock};
use std::collections::HashMap;
use anyhow::{anyhow, Result};
use crate::policy::{Policy, PolicyInfo};

// ============================================================================
// In-Memory Policy Store (Thread-Safe)
// ============================================================================

static POLICY_STORE: OnceLock<Arc<Mutex<HashMap<String, Policy>>>> = OnceLock::new();

fn get_store() -> Arc<Mutex<HashMap<String, Policy>>> {
    POLICY_STORE.get_or_init(|| Arc::new(Mutex::new(HashMap::new()))).clone()
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
    /// Policy hash (SHA3-256)
    pub policy_hash: String,
    /// Policy info
    pub policy_info: PolicyInfo,
    /// Status message
    pub status: String,
}

/// Response for policy retrieval
#[derive(Debug, Serialize)]
pub struct PolicyGetResponse {
    /// Policy hash
    pub policy_hash: String,
    /// Full policy definition
    pub policy: Policy,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Computes SHA3-256 hash of a policy
fn compute_policy_hash(policy: &Policy) -> Result<String> {
    use sha3::{Digest, Sha3_256};

    let json = serde_json::to_string(policy)
        .map_err(|e| anyhow!("Failed to serialize policy: {}", e))?;

    let mut hasher = Sha3_256::new();
    hasher.update(json.as_bytes());
    let result = hasher.finalize();

    Ok(format!("0x{}", hex::encode(result)))
}

/// Stores a policy in the in-memory store
fn store_policy(hash: String, policy: Policy) -> Result<()> {
    let store = get_store();
    let mut map = store.lock()
        .map_err(|e| anyhow!("Failed to lock policy store: {}", e))?;

    map.insert(hash, policy);
    Ok(())
}

/// Retrieves a policy from the in-memory store
fn get_policy(hash: &str) -> Result<Option<Policy>> {
    let store = get_store();
    let map = store.lock()
        .map_err(|e| anyhow!("Failed to lock policy store: {}", e))?;

    Ok(map.get(hash).cloned())
}

// ============================================================================
// Handlers
// ============================================================================

/// Handles POST /policy/compile
///
/// Validates and compiles a policy, returns policy hash
pub async fn handle_policy_compile(
    Json(request): Json<PolicyCompileRequest>,
) -> Result<Json<PolicyCompileResponse>, StatusCode> {
    // Validate policy
    request.policy.validate()
        .map_err(|e| {
            tracing::warn!("Policy validation failed: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    // Compute policy hash
    let policy_hash = compute_policy_hash(&request.policy)
        .map_err(|e| {
            tracing::error!("Failed to compute policy hash: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Store policy
    store_policy(policy_hash.clone(), request.policy.clone())
        .map_err(|e| {
            tracing::error!("Failed to store policy: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Create policy info
    let policy_info = PolicyInfo {
        name: request.policy.name.clone(),
        version: request.policy.version.clone(),
        hash: policy_hash.clone(),
    };

    tracing::info!("Policy compiled successfully: {}", policy_hash);

    Ok(Json(PolicyCompileResponse {
        policy_hash,
        policy_info,
        status: "compiled".to_string(),
    }))
}

/// Handles GET /policy/:id
///
/// Retrieves a policy by its hash
pub async fn handle_policy_get(
    Path(policy_id): Path<String>,
) -> Result<Json<PolicyGetResponse>, StatusCode> {
    // Retrieve policy
    let policy = get_policy(&policy_id)
        .map_err(|e| {
            tracing::error!("Failed to retrieve policy: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    match policy {
        Some(policy) => {
            tracing::info!("Policy retrieved successfully: {}", policy_id);
            Ok(Json(PolicyGetResponse {
                policy_hash: policy_id,
                policy,
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
    use crate::policy::PolicyConstraints;

    #[test]
    fn test_compute_policy_hash() {
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
            notes: "".to_string(),
        };

        let hash = compute_policy_hash(&policy).unwrap();
        assert!(hash.starts_with("0x"));
        assert_eq!(hash.len(), 66); // 0x + 64 hex chars
    }

    #[test]
    fn test_store_and_get_policy() {
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
            notes: "".to_string(),
        };

        let hash = compute_policy_hash(&policy).unwrap();
        store_policy(hash.clone(), policy.clone()).unwrap();

        let retrieved = get_policy(&hash).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Policy");
    }
}
