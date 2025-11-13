use anyhow::Result;
/// Policy Compiler API (Week 3) - PolicyV2 with IR Generation
///
/// Endpoints:
/// - POST /policy/compile - Compiles PolicyV2 YAML to IR v1 with linting
/// - GET /policy/:id - Retrieves policy and IR by hash (with ETag support)
use axum::{
    extract::Path,
    http::{HeaderMap, StatusCode},
    Json,
};
use base64::{engine::general_purpose, Engine as _};
use lru::LruCache;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex, OnceLock};

use crate::policy_v2::{
    canonicalize, generate_ir, has_errors, http_status_from_diagnostics, lint, parse_yaml_str,
    sha3_256_hex, IrV1, LintDiagnostic, LintMode, PolicyV2,
};

// ============================================================================
// LRU Cache for IR (Week 4)
// ============================================================================

/// Cache entry containing policy and compiled IR
#[derive(Debug, Clone)]
pub(crate) struct PolicyEntry {
    pub(crate) policy: PolicyV2,
    pub(crate) policy_hash: String,
    pub(crate) ir: IrV1,
    pub(crate) ir_hash: String,
}

/// LRU Cache for policy_hash → IR mapping
/// Key: policy_hash (SHA3-256)
/// Size: 1000 entries (Week 4 spec)
#[allow(clippy::type_complexity)]
static POLICY_IR_CACHE: OnceLock<Arc<Mutex<LruCache<String, Arc<PolicyEntry>>>>> = OnceLock::new();

/// Policy ID → policy_hash index for lookups
static POLICY_ID_INDEX: OnceLock<Arc<Mutex<HashMap<String, String>>>> = OnceLock::new();

pub(crate) fn get_cache() -> Arc<Mutex<LruCache<String, Arc<PolicyEntry>>>> {
    POLICY_IR_CACHE
        .get_or_init(|| {
            let cache = LruCache::new(NonZeroUsize::new(1000).unwrap());
            Arc::new(Mutex::new(cache))
        })
        .clone()
}

pub(crate) fn get_id_index() -> Arc<Mutex<HashMap<String, String>>> {
    POLICY_ID_INDEX
        .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
        .clone()
}

// ============================================================================
// Request/Response Structures (Week 3 Spec)
// ============================================================================

/// Request for compiling a PolicyV2
#[derive(Debug, Deserialize)]
pub struct PolicyV2CompileRequest {
    /// Policy YAML (base64-encoded) or direct PolicyV2 JSON
    #[serde(default)]
    pub policy_yaml: Option<String>,

    /// Direct PolicyV2 JSON (alternative to policy_yaml)
    #[serde(default)]
    pub policy: Option<PolicyV2>,

    /// Lint mode (strict or relaxed)
    #[serde(default = "default_lint_mode")]
    pub lint_mode: String,

    /// Persist policy and IR in store
    #[serde(default)]
    pub persist: bool,
}

fn default_lint_mode() -> String {
    "strict".to_string()
}

/// Response after PolicyV2 compilation
#[derive(Debug, Serialize)]
pub struct PolicyV2CompileResponse {
    /// Policy ID from PolicyV2.id
    pub policy_id: String,

    /// Policy hash (SHA3-256)
    pub policy_hash: String,

    /// Compiled IR v1
    pub ir: IrV1,

    /// IR hash (SHA3-256)
    pub ir_hash: String,

    /// Lint diagnostics (warnings and errors)
    pub lints: Vec<LintDiagnostic>,

    /// Whether policy was stored
    pub stored: bool,

    /// ETag for caching (format: "ir:sha3-256:...")
    pub etag: String,
}

/// Response for policy retrieval with ETag
#[derive(Debug, Serialize)]
pub struct PolicyV2GetResponse {
    /// Policy ID
    pub policy_id: String,

    /// Policy version
    pub version: String,

    /// Policy hash
    pub policy_hash: String,

    /// Compiled IR v1
    pub ir: IrV1,

    /// IR hash
    pub ir_hash: String,

    /// ETag
    pub etag: String,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Parse lint_mode string to LintMode enum
fn parse_lint_mode(mode: &str) -> LintMode {
    match mode.to_lowercase().as_str() {
        "relaxed" => LintMode::Relaxed,
        _ => LintMode::Strict,
    }
}

/// Generate ETag from IR hash
fn generate_etag(ir_hash: &str) -> String {
    format!("\"ir:{}\"", ir_hash)
}

// ============================================================================
// Handlers
// ============================================================================

/// Handles POST /policy/compile (Week 3 Spec)
///
/// Compiles PolicyV2 YAML → IR v1 with linting
pub async fn handle_policy_v2_compile(
    Json(request): Json<PolicyV2CompileRequest>,
) -> Result<(StatusCode, Json<PolicyV2CompileResponse>), (StatusCode, String)> {
    // Parse policy from YAML or JSON
    let policy = if let Some(yaml_b64) = request.policy_yaml {
        // Decode base64
        let yaml_bytes = general_purpose::STANDARD
            .decode(&yaml_b64)
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid base64: {}", e)))?;

        let yaml_str = String::from_utf8(yaml_bytes)
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid UTF-8: {}", e)))?;

        // Parse YAML
        parse_yaml_str(&yaml_str)
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("YAML parse error: {}", e)))?
    } else if let Some(policy) = request.policy {
        policy
    } else {
        return Err((
            StatusCode::BAD_REQUEST,
            "Missing policy_yaml or policy".to_string(),
        ));
    };

    // Parse lint mode
    let lint_mode = parse_lint_mode(&request.lint_mode);

    // Run linter
    let diagnostics = lint(&policy, lint_mode);

    // Check for errors in strict mode
    if has_errors(&diagnostics) {
        let status = http_status_from_diagnostics(&diagnostics);
        tracing::warn!(
            "Policy compilation failed with {} lint errors",
            diagnostics.len()
        );

        // Return 422 with diagnostics
        return Ok((
            StatusCode::from_u16(status).unwrap(),
            Json(PolicyV2CompileResponse {
                policy_id: policy.id.clone(),
                policy_hash: String::new(),
                ir: IrV1 {
                    ir_version: "1.0".to_string(),
                    policy_id: policy.id.clone(),
                    policy_hash: String::new(),
                    rules: vec![],
                    adaptivity: None,
                    ir_hash: String::new(),
                },
                ir_hash: String::new(),
                lints: diagnostics,
                stored: false,
                etag: String::new(),
            }),
        ));
    }

    // Compute policy hash
    let policy_json = serde_json::to_string(&policy).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Serialization error: {}", e),
        )
    })?;

    let policy_hash = sha3_256_hex(&policy_json);

    // Check for conflicts (409) if persist=true
    if request.persist {
        let id_index = get_id_index();
        let index = id_index.lock().map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Lock error: {}", e),
            )
        })?;

        if let Some(existing_hash) = index.get(&policy.id) {
            if existing_hash != &policy_hash {
                tracing::warn!("Policy conflict: {} exists with different hash", policy.id);
                return Err((
                    StatusCode::CONFLICT,
                    format!("Policy {} already exists with different hash", policy.id),
                ));
            }
        }
    }

    // Generate IR v1
    let mut ir = generate_ir(&policy, policy_hash.clone()).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("IR generation error: {}", e),
        )
    })?;

    // Canonicalize and compute IR hash
    let ir_canonical = canonicalize(&ir).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Canonicalization error: {}", e),
        )
    })?;

    let ir_hash = sha3_256_hex(&ir_canonical);
    ir.ir_hash = ir_hash.clone();

    // Generate ETag
    let etag = generate_etag(&ir_hash);

    // Persist if requested (Week 4: LRU Cache)
    let stored = if request.persist {
        let cache = get_cache();
        let mut lru = cache.lock().map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Lock error: {}", e),
            )
        })?;

        let entry = Arc::new(PolicyEntry {
            policy: policy.clone(),
            policy_hash: policy_hash.clone(),
            ir: ir.clone(),
            ir_hash: ir_hash.clone(),
        });

        // Insert into LRU cache (key = policy_hash)
        lru.put(policy_hash.clone(), entry);

        // Update policy_id → policy_hash index
        let id_index = get_id_index();
        let mut index = id_index.lock().map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Lock error: {}", e),
            )
        })?;
        index.insert(policy.id.clone(), policy_hash.clone());

        tracing::info!(
            "Policy cached: {} → {} (LRU size: {})",
            policy.id,
            policy_hash,
            lru.len()
        );
        true
    } else {
        false
    };

    // Determine HTTP status
    let status = http_status_from_diagnostics(&diagnostics);

    tracing::info!("Policy compiled: {} → IR hash: {}", policy.id, ir_hash);

    Ok((
        StatusCode::from_u16(status).unwrap(),
        Json(PolicyV2CompileResponse {
            policy_id: policy.id.clone(),
            policy_hash,
            ir,
            ir_hash,
            lints: diagnostics,
            stored,
            etag,
        }),
    ))
}

/// Handles GET /policy/:id (with ETag support)
///
/// Retrieves policy and IR by ID, supports If-None-Match for 304
/// Week 4: Uses LRU cache with policy_hash lookup
pub async fn handle_policy_v2_get(
    Path(policy_id): Path<String>,
    headers: HeaderMap,
) -> Result<(StatusCode, HeaderMap, Json<PolicyV2GetResponse>), (StatusCode, String)> {
    // Lookup policy_hash from ID index
    let id_index = get_id_index();
    let index = id_index.lock().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Lock error: {}", e),
        )
    })?;

    let policy_hash = index
        .get(&policy_id)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("Policy not found: {}", policy_id),
            )
        })?
        .clone();

    drop(index); // Release lock early

    // Retrieve from LRU cache
    let cache = get_cache();
    let mut lru = cache.lock().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Lock error: {}", e),
        )
    })?;

    let entry = lru
        .get(&policy_hash)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("Policy not in cache: {}", policy_id),
            )
        })?
        .clone();

    drop(lru); // Release lock early

    // Generate ETag
    let etag = generate_etag(&entry.ir_hash);

    // Check If-None-Match for 304
    if let Some(if_none_match) = headers.get("if-none-match") {
        if let Ok(if_none_match_str) = if_none_match.to_str() {
            if if_none_match_str == etag {
                tracing::debug!("ETag match, returning 304: {}", policy_id);

                let mut response_headers = HeaderMap::new();
                response_headers.insert("etag", etag.parse().unwrap());

                return Err((StatusCode::NOT_MODIFIED, String::new()));
            }
        }
    }

    // Prepare response headers
    let mut response_headers = HeaderMap::new();
    response_headers.insert("etag", etag.parse().unwrap());
    response_headers.insert("cache-control", "private, max-age=3600".parse().unwrap());

    tracing::info!("Policy retrieved: {}", policy_id);

    Ok((
        StatusCode::OK,
        response_headers,
        Json(PolicyV2GetResponse {
            policy_id: entry.policy.id.clone(),
            version: entry.policy.version.clone(),
            policy_hash: entry.policy_hash.clone(),
            ir: entry.ir.clone(),
            ir_hash: entry.ir_hash.clone(),
            etag,
        }),
    ))
}

// ============================================================================
// Test Helpers (for integration tests)
// ============================================================================

/// Clear the LRU cache and ID index (for testing)
pub fn test_clear_cache() {
    let cache = get_cache();
    let mut lru = cache.lock().expect("Failed to lock cache");
    lru.clear();

    let id_index = get_id_index();
    let mut index = id_index.lock().expect("Failed to lock index");
    index.clear();
}

/// Get current cache size (for testing)
pub fn test_get_cache_size() -> usize {
    let cache = get_cache();
    let lru = cache.lock().expect("Failed to lock cache");
    lru.len()
}

/// Check if a policy_hash exists in cache (for testing)
pub fn test_cache_contains(policy_hash: &str) -> bool {
    let cache = get_cache();
    let lru = cache.lock().expect("Failed to lock cache");
    lru.peek(policy_hash).is_some()
}

/// Insert a test policy into cache (for testing)
pub fn test_insert_policy(policy: PolicyV2, policy_hash: String, ir: IrV1, ir_hash: String) {
    let cache = get_cache();
    let mut lru = cache.lock().expect("Failed to lock cache");

    let entry = Arc::new(PolicyEntry {
        policy: policy.clone(),
        policy_hash: policy_hash.clone(),
        ir,
        ir_hash,
    });

    lru.put(policy_hash.clone(), entry);

    // Update ID index
    let id_index = get_id_index();
    let mut index = id_index.lock().expect("Failed to lock index");
    index.insert(policy.id.clone(), policy_hash);
}

/// Access (touch) a policy in cache to update LRU order (for testing)
pub fn test_touch_policy(policy_hash: &str) -> bool {
    let cache = get_cache();
    let mut lru = cache.lock().expect("Failed to lock cache");
    lru.get(policy_hash).is_some()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lint_mode() {
        assert!(matches!(parse_lint_mode("strict"), LintMode::Strict));
        assert!(matches!(parse_lint_mode("STRICT"), LintMode::Strict));
        assert!(matches!(parse_lint_mode("relaxed"), LintMode::Relaxed));
        assert!(matches!(parse_lint_mode("RELAXED"), LintMode::Relaxed));
        assert!(matches!(parse_lint_mode("invalid"), LintMode::Strict)); // default
    }

    #[test]
    fn test_generate_etag() {
        let ir_hash = "sha3-256:abc123";
        let etag = generate_etag(ir_hash);
        assert_eq!(etag, "\"ir:sha3-256:abc123\"");
    }

    #[test]
    fn test_base64_decode() {
        let yaml = "id: test\nversion: \"1.0\"\n";
        let encoded = general_purpose::STANDARD.encode(yaml);
        let decoded = general_purpose::STANDARD.decode(&encoded).unwrap();
        let decoded_str = String::from_utf8(decoded).unwrap();
        assert_eq!(decoded_str, yaml);
    }
}
