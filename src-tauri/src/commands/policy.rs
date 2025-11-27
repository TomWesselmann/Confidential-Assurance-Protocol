//! Policy loading commands
//!
//! Commands for loading and validating policy files.

use crate::audit_logger;
use crate::security::{
    sanitize_error_message, validate_file_size, validate_path_exists, validate_regular_file,
    MAX_POLICY_FILE_SIZE,
};
use crate::types::PolicyInfo;
use blake3::Hasher;
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Policy file structure (subset of full policy for validation)
#[derive(Debug, Deserialize)]
struct PolicyFile {
    name: String,
    version: String,
    #[serde(default)]
    rules: Vec<serde_yaml::Value>,
}

/// Loads and validates a policy file into a project
///
/// # Arguments
/// * `project` - Path to the project directory
/// * `policy_path` - Path to the policy YAML file
///
/// # Returns
/// PolicyInfo with name, version, hash, and rules count
///
/// # Security
/// - Validates file size (max 1 MB)
/// - Uses serde_yaml (safe from YAML bombs)
/// - Copies file to project (no modification of source)
#[tauri::command]
pub async fn load_policy(project: String, policy_path: String) -> Result<PolicyInfo, String> {
    let project_path = Path::new(&project);
    let source_path = Path::new(&policy_path);

    // 1. Validate project exists
    validate_path_exists(project_path)?;

    // 2. Validate source file
    validate_path_exists(source_path)?;
    validate_regular_file(source_path)?;
    validate_file_size(source_path, MAX_POLICY_FILE_SIZE)?;

    // 3. Read and parse policy
    let content = fs::read_to_string(source_path)
        .map_err(|e| sanitize_error_message(&format!("Failed to read policy file: {}", e)))?;

    let policy: PolicyFile = serde_yaml::from_str(&content)
        .map_err(|e| format!("Invalid policy YAML: {}", e))?;

    // 4. Validate policy has required fields
    if policy.name.is_empty() {
        return Err("Policy must have a 'name' field".to_string());
    }
    if policy.version.is_empty() {
        return Err("Policy must have a 'version' field".to_string());
    }

    // 5. Calculate hash of policy content
    let mut hasher = Hasher::new();
    hasher.update(content.as_bytes());
    let hash = format!("0x{}", hasher.finalize().to_hex());

    // 6. Copy to project input/
    let dest_path = project_path.join("input/policy.yml");
    fs::write(&dest_path, &content)
        .map_err(|e| sanitize_error_message(&format!("Failed to write policy: {}", e)))?;

    // 7. Log to audit trail
    let _ = audit_logger::events::policy_loaded(project_path, &policy.name, &policy.version, &hash);

    Ok(PolicyInfo {
        name: policy.name,
        version: policy.version,
        hash,
        rules_count: policy.rules.len(),
        path: dest_path.to_string_lossy().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::project::create_project;
    use tempfile::TempDir;

    fn create_test_policy(temp: &TempDir) -> String {
        let content = r#"
name: "LkSG Demo Policy"
version: "lksg.v1"
rules:
  - name: "has_suppliers"
    constraint: "supplier_count >= 1"
  - name: "has_ubos"
    constraint: "ubo_count >= 1"
"#;
        let path = temp.path().join("policy.yml");
        fs::write(&path, content).unwrap();
        path.to_string_lossy().to_string()
    }

    #[tokio::test]
    async fn test_load_policy_success() {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path().to_string_lossy().to_string();

        let project = create_project(workspace.clone(), "test".to_string())
            .await
            .unwrap();

        let policy_path = create_test_policy(&temp);

        let result = load_policy(project.path.clone(), policy_path).await;

        assert!(result.is_ok());
        let policy = result.unwrap();
        assert_eq!(policy.name, "LkSG Demo Policy");
        assert_eq!(policy.version, "lksg.v1");
        assert!(policy.hash.starts_with("0x"));
        assert_eq!(policy.rules_count, 2);
        assert!(Path::new(&project.path).join("input/policy.yml").exists());
    }

    #[tokio::test]
    async fn test_load_policy_invalid_yaml() {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path().to_string_lossy().to_string();

        let project = create_project(workspace.clone(), "test".to_string())
            .await
            .unwrap();

        let bad_policy = temp.path().join("bad.yml");
        fs::write(&bad_policy, "not: valid: yaml: [").unwrap();

        let result = load_policy(project.path, bad_policy.to_string_lossy().to_string()).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_load_policy_missing_name() {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path().to_string_lossy().to_string();

        let project = create_project(workspace.clone(), "test".to_string())
            .await
            .unwrap();

        let bad_policy = temp.path().join("noname.yml");
        fs::write(&bad_policy, "version: v1\nrules: []").unwrap();

        let result = load_policy(project.path, bad_policy.to_string_lossy().to_string()).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("name"));
    }
}
