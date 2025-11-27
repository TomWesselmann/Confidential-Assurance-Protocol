//! Project management commands
//!
//! Commands for creating and managing Taurin projects.

use crate::audit_logger;
use crate::security::{sanitize_error_message, validate_project_name, validate_path_exists};
use crate::types::{ProjectInfo, ProjectMeta, ProjectStatus};
use chrono::Utc;
use std::fs;
use std::path::Path;

/// Creates a new Taurin project with standard directory structure
///
/// # Arguments
/// * `workspace` - Path to the workspace directory
/// * `name` - Name of the new project
///
/// # Returns
/// ProjectInfo with path and metadata
///
/// # Security
/// - Validates project name for path traversal
/// - Creates isolated directory structure
#[tauri::command]
pub async fn create_project(workspace: String, name: String) -> Result<ProjectInfo, String> {
    // 1. Validate project name
    validate_project_name(&name)?;

    // 2. Validate workspace exists
    let workspace_path = Path::new(&workspace);
    validate_path_exists(workspace_path)?;

    // 3. Check project doesn't already exist
    let project_path = workspace_path.join(&name);
    if project_path.exists() {
        return Err(format!("Project '{}' already exists", name));
    }

    // 4. Create project directory structure
    let dirs = ["input", "build", "audit", "export"];
    for dir in &dirs {
        let dir_path = project_path.join(dir);
        fs::create_dir_all(&dir_path).map_err(|e| {
            sanitize_error_message(&format!("Failed to create directory '{}': {}", dir, e))
        })?;
    }

    // 5. Create project metadata file
    let created_at = Utc::now().to_rfc3339();
    let meta = ProjectMeta {
        schema: ProjectMeta::SCHEMA_VERSION.to_string(),
        name: name.clone(),
        created_at: created_at.clone(),
        description: None,
        cap_version: Some(env!("CARGO_PKG_VERSION").to_string()),
    };

    let meta_path = project_path.join("taurin.project.json");
    let meta_json = serde_json::to_string_pretty(&meta)
        .map_err(|e| format!("Failed to serialize project metadata: {}", e))?;
    fs::write(&meta_path, meta_json)
        .map_err(|e| sanitize_error_message(&format!("Failed to write project file: {}", e)))?;

    // 6. Log project creation to audit log
    let _ = audit_logger::events::project_created(&project_path, &name);

    Ok(ProjectInfo {
        path: project_path.to_string_lossy().to_string(),
        name,
        created_at,
    })
}

/// Lists all projects in a workspace
///
/// # Arguments
/// * `workspace` - Path to the workspace directory
///
/// # Returns
/// Vector of ProjectInfo for each valid project found
#[tauri::command]
pub async fn list_projects(workspace: String) -> Result<Vec<ProjectInfo>, String> {
    let workspace_path = Path::new(&workspace);
    validate_path_exists(workspace_path)?;

    let mut projects = Vec::new();

    let entries = fs::read_dir(workspace_path)
        .map_err(|e| sanitize_error_message(&format!("Failed to read workspace: {}", e)))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let meta_path = path.join("taurin.project.json");
            if meta_path.exists() {
                if let Ok(content) = fs::read_to_string(&meta_path) {
                    if let Ok(meta) = serde_json::from_str::<ProjectMeta>(&content) {
                        projects.push(ProjectInfo {
                            path: path.to_string_lossy().to_string(),
                            name: meta.name,
                            created_at: meta.created_at,
                        });
                    }
                }
            }
        }
    }

    // Sort by created_at descending (newest first)
    projects.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(projects)
}

/// Reads file content from within a project directory
///
/// # Arguments
/// * `project` - Path to the project directory
/// * `relative_path` - Relative path within the project (e.g., "input/policy.yml")
///
/// # Returns
/// File content as string
///
/// # Security
/// - Only allows reading from within project directory
/// - Validates path doesn't escape project bounds
/// - Maximum file size: 10 MB
#[tauri::command]
pub async fn read_file_content(project: String, relative_path: String) -> Result<String, String> {
    let project_path = Path::new(&project);
    validate_path_exists(project_path)?;

    // Security: Prevent path traversal
    if relative_path.contains("..") || relative_path.starts_with('/') || relative_path.starts_with('\\') {
        return Err("Invalid path: path traversal not allowed".to_string());
    }

    let file_path = project_path.join(&relative_path);

    // Verify the resolved path is still within project
    let canonical_project = project_path.canonicalize()
        .map_err(|e| sanitize_error_message(&format!("Failed to resolve project path: {}", e)))?;
    let canonical_file = file_path.canonicalize()
        .map_err(|_| format!("File not found: {}", relative_path))?;

    if !canonical_file.starts_with(&canonical_project) {
        return Err("Invalid path: file must be within project directory".to_string());
    }

    // Check file size (max 10 MB)
    let metadata = fs::metadata(&canonical_file)
        .map_err(|e| sanitize_error_message(&format!("Failed to read file metadata: {}", e)))?;

    if metadata.len() > 10 * 1024 * 1024 {
        return Err("File too large (max 10 MB)".to_string());
    }

    // Read and return content
    fs::read_to_string(&canonical_file)
        .map_err(|e| sanitize_error_message(&format!("Failed to read file: {}", e)))
}

/// Gets the current status of a project
///
/// # Arguments
/// * `project` - Path to the project directory
///
/// # Returns
/// ProjectStatus with workflow state
#[tauri::command]
pub async fn get_project_status(project: String) -> Result<ProjectStatus, String> {
    let project_path = Path::new(&project);
    validate_path_exists(project_path)?;

    // Read project metadata
    let meta_path = project_path.join("taurin.project.json");
    let meta_content = fs::read_to_string(&meta_path)
        .map_err(|_| "Not a valid Taurin project (missing taurin.project.json)".to_string())?;
    let meta: ProjectMeta = serde_json::from_str(&meta_content)
        .map_err(|_| "Invalid project metadata".to_string())?;

    // Check which files exist
    let has_suppliers_csv = project_path.join("input/suppliers.csv").exists();
    let has_ubos_csv = project_path.join("input/ubos.csv").exists();
    let has_policy = project_path.join("input/policy.yml").exists()
        || project_path.join("input/policy.yaml").exists();
    let has_commitments = project_path.join("build/commitments.json").exists();
    let has_manifest = project_path.join("build/manifest.json").exists();
    let has_proof = project_path.join("build/proof.capz").exists()
        || project_path.join("build/proof.dat").exists();

    // Determine current step
    let current_step = if has_proof {
        "export"
    } else if has_manifest {
        "proof"
    } else if has_commitments && has_policy {
        "manifest"
    } else if has_suppliers_csv && has_ubos_csv {
        "commitments"
    } else if has_policy {
        "import"
    } else {
        "import"
    };

    Ok(ProjectStatus {
        info: ProjectInfo {
            path: project.clone(),
            name: meta.name,
            created_at: meta.created_at,
        },
        has_suppliers_csv,
        has_ubos_csv,
        has_policy,
        has_commitments,
        has_manifest,
        has_proof,
        current_step: current_step.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_create_project_success() {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path().to_string_lossy().to_string();

        let result = create_project(workspace.clone(), "test-project".to_string()).await;

        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.name, "test-project");
        assert!(Path::new(&info.path).exists());
        assert!(Path::new(&info.path).join("input").exists());
        assert!(Path::new(&info.path).join("build").exists());
        assert!(Path::new(&info.path).join("audit").exists());
        assert!(Path::new(&info.path).join("export").exists());
        assert!(Path::new(&info.path).join("taurin.project.json").exists());
    }

    #[tokio::test]
    async fn test_create_project_already_exists() {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path().to_string_lossy().to_string();

        create_project(workspace.clone(), "existing".to_string())
            .await
            .unwrap();
        let result = create_project(workspace, "existing".to_string()).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));
    }

    #[tokio::test]
    async fn test_create_project_invalid_name() {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path().to_string_lossy().to_string();

        let result = create_project(workspace, "../traversal".to_string()).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_projects() {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path().to_string_lossy().to_string();

        create_project(workspace.clone(), "project1".to_string())
            .await
            .unwrap();
        create_project(workspace.clone(), "project2".to_string())
            .await
            .unwrap();

        let result = list_projects(workspace).await;

        assert!(result.is_ok());
        let projects = result.unwrap();
        assert_eq!(projects.len(), 2);
    }
}
