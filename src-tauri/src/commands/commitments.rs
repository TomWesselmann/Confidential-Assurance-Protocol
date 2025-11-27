//! Commitment creation commands
//!
//! Commands for creating cryptographic commitments from CSV data.

use crate::audit_logger;
use crate::security::{sanitize_error_message, validate_path_exists};
use crate::types::CommitmentsResult;
use std::fs;
use std::path::Path;

// Import CAP-Agent library functions
use cap_agent::commitment::{compute_company_root, compute_supplier_root, compute_ubo_root};
use cap_agent::io::{read_suppliers_csv, read_ubos_csv};

/// Creates commitments (Merkle roots) from imported CSV files
///
/// # Arguments
/// * `project` - Path to the project directory
///
/// # Returns
/// CommitmentsResult with supplier_root, ubo_root, and company_root
///
/// # Security
/// - Uses BLAKE3 for hashing (deterministic)
/// - No PII in output (only hashes)
#[tauri::command]
pub async fn create_commitments(project: String) -> Result<CommitmentsResult, String> {
    let project_path = Path::new(&project);
    validate_path_exists(project_path)?;

    // 1. Check required CSV files exist
    let suppliers_path = project_path.join("input/suppliers.csv");
    let ubos_path = project_path.join("input/ubos.csv");

    if !suppliers_path.exists() {
        return Err("suppliers.csv not found in input/ - please import first".to_string());
    }
    if !ubos_path.exists() {
        return Err("ubos.csv not found in input/ - please import first".to_string());
    }

    // 2. Read CSV files using CAP-Agent library
    let suppliers = read_suppliers_csv(&suppliers_path)
        .map_err(|e| sanitize_error_message(&format!("Failed to read suppliers CSV: {}", e)))?;

    let ubos = read_ubos_csv(&ubos_path)
        .map_err(|e| sanitize_error_message(&format!("Failed to read UBOs CSV: {}", e)))?;

    // 3. Compute Merkle roots
    let supplier_root = compute_supplier_root(&suppliers)
        .map_err(|e| format!("Failed to compute supplier root: {}", e))?;

    let ubo_root = compute_ubo_root(&ubos)
        .map_err(|e| format!("Failed to compute UBO root: {}", e))?;

    let company_root = compute_company_root(&supplier_root, &ubo_root);

    // 4. Save commitments to build/
    let commitments_path = project_path.join("build/commitments.json");
    let commitments = serde_json::json!({
        "supplier_root": supplier_root,
        "ubo_root": ubo_root,
        "company_root": company_root,
        "supplier_count": suppliers.len(),
        "ubo_count": ubos.len(),
        "created_at": chrono::Utc::now().to_rfc3339(),
    });

    let commitments_json = serde_json::to_string_pretty(&commitments)
        .map_err(|e| format!("Failed to serialize commitments: {}", e))?;

    fs::write(&commitments_path, &commitments_json)
        .map_err(|e| sanitize_error_message(&format!("Failed to write commitments: {}", e)))?;

    // 5. Log to audit trail
    let _ = audit_logger::events::commitments_created(project_path, &supplier_root, &ubo_root);

    Ok(CommitmentsResult {
        supplier_root,
        ubo_root,
        company_root,
        path: commitments_path.to_string_lossy().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::import::{import_csv};
    use crate::commands::project::create_project;
    use crate::types::CsvType;
    use tempfile::TempDir;

    fn create_test_csv(temp: &TempDir, name: &str, content: &str) -> String {
        let path = temp.path().join(name);
        std::fs::write(&path, content).unwrap();
        path.to_string_lossy().to_string()
    }

    #[tokio::test]
    async fn test_create_commitments_success() {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path().to_string_lossy().to_string();

        // Create project
        let project = create_project(workspace.clone(), "test".to_string())
            .await
            .unwrap();

        // Import CSVs
        let suppliers_csv = create_test_csv(
            &temp,
            "suppliers.csv",
            "name,jurisdiction,tier\nAcme GmbH,DE,1\n",
        );
        let ubos_csv = create_test_csv(
            &temp,
            "ubos.csv",
            "name,birthdate,citizenship\nMax Mustermann,1980-01-01,DE\n",
        );

        import_csv(project.path.clone(), CsvType::Suppliers, suppliers_csv)
            .await
            .unwrap();
        import_csv(project.path.clone(), CsvType::Ubos, ubos_csv)
            .await
            .unwrap();

        // Create commitments
        let result = create_commitments(project.path.clone()).await;

        assert!(result.is_ok());
        let commitments = result.unwrap();
        assert!(commitments.supplier_root.starts_with("0x"));
        assert!(commitments.ubo_root.starts_with("0x"));
        assert!(commitments.company_root.starts_with("0x"));
        assert!(Path::new(&project.path)
            .join("build/commitments.json")
            .exists());
    }

    #[tokio::test]
    async fn test_create_commitments_deterministic() {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path().to_string_lossy().to_string();

        let project = create_project(workspace.clone(), "test".to_string())
            .await
            .unwrap();

        let suppliers_csv = create_test_csv(
            &temp,
            "suppliers.csv",
            "name,jurisdiction,tier\nAcme GmbH,DE,1\n",
        );
        let ubos_csv = create_test_csv(
            &temp,
            "ubos.csv",
            "name,birthdate,citizenship\nMax Mustermann,1980-01-01,DE\n",
        );

        import_csv(project.path.clone(), CsvType::Suppliers, suppliers_csv)
            .await
            .unwrap();
        import_csv(project.path.clone(), CsvType::Ubos, ubos_csv)
            .await
            .unwrap();

        // Call twice
        let result1 = create_commitments(project.path.clone()).await.unwrap();
        let result2 = create_commitments(project.path.clone()).await.unwrap();

        // Hashes should be identical
        assert_eq!(result1.supplier_root, result2.supplier_root);
        assert_eq!(result1.ubo_root, result2.ubo_root);
        assert_eq!(result1.company_root, result2.company_root);
    }

    #[tokio::test]
    async fn test_create_commitments_missing_csv() {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path().to_string_lossy().to_string();

        let project = create_project(workspace.clone(), "test".to_string())
            .await
            .unwrap();

        // Don't import any CSVs
        let result = create_commitments(project.path).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("suppliers.csv not found"));
    }
}
