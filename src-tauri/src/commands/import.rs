//! CSV import commands
//!
//! Commands for importing CSV files into a Taurin project.

use crate::audit_logger;
use crate::security::{
    sanitize_error_message, validate_file_size, validate_path_exists, validate_regular_file,
    MAX_CSV_FIELD_LENGTH, MAX_CSV_FILE_SIZE,
};
use crate::types::{CsvType, ImportResult};
use blake3::Hasher;
use std::fs;
use std::path::Path;

/// Imports a CSV file into a project
///
/// # Arguments
/// * `project` - Path to the project directory
/// * `csv_type` - Type of CSV (suppliers, ubos, etc.)
/// * `file_path` - Path to the source CSV file
///
/// # Returns
/// ImportResult with record count and hash
///
/// # Security
/// - Validates file is regular file (no symlinks)
/// - Validates file size limits
/// - Validates CSV field lengths
/// - Copies file to project (no modification of source)
#[tauri::command]
pub async fn import_csv(
    project: String,
    csv_type: CsvType,
    file_path: String,
) -> Result<ImportResult, String> {
    let project_path = Path::new(&project);
    let source_path = Path::new(&file_path);

    // 1. Validate project exists
    validate_path_exists(project_path)?;

    // 2. Validate source file
    validate_path_exists(source_path)?;
    validate_regular_file(source_path)?;
    validate_file_size(source_path, MAX_CSV_FILE_SIZE)?;

    // 3. Determine destination filename
    let dest_filename = match csv_type {
        CsvType::Suppliers => "suppliers.csv",
        CsvType::Ubos => "ubos.csv",
        CsvType::Sanctions => "sanctions.csv",
        CsvType::Jurisdictions => "jurisdictions.csv",
    };
    let dest_path = project_path.join("input").join(dest_filename);

    // 4. Read and validate CSV content
    let content = fs::read_to_string(source_path)
        .map_err(|e| sanitize_error_message(&format!("Failed to read CSV: {}", e)))?;

    let (record_count, validated_content) = validate_and_count_csv(&content, &csv_type)?;

    // 5. Calculate hash of content
    let mut hasher = Hasher::new();
    hasher.update(validated_content.as_bytes());
    let hash = format!("0x{}", hasher.finalize().to_hex());

    // 6. Write to destination
    fs::write(&dest_path, &validated_content)
        .map_err(|e| sanitize_error_message(&format!("Failed to write CSV: {}", e)))?;

    // 7. Log to audit trail
    let csv_type_str = format!("{:?}", csv_type).to_lowercase();
    let _ = audit_logger::events::csv_imported(project_path, &csv_type_str, record_count, &hash);

    Ok(ImportResult {
        csv_type: csv_type_str,
        record_count,
        hash,
        destination: dest_path.to_string_lossy().to_string(),
    })
}

/// Validates CSV content and counts records
fn validate_and_count_csv(content: &str, csv_type: &CsvType) -> Result<(usize, String), String> {
    let mut reader = csv::Reader::from_reader(content.as_bytes());

    // Validate headers
    let headers = reader
        .headers()
        .map_err(|e| format!("Invalid CSV headers: {}", e))?;

    let expected_headers = match csv_type {
        CsvType::Suppliers => vec!["name", "jurisdiction", "tier"],
        CsvType::Ubos => vec!["name", "birthdate", "citizenship"],
        CsvType::Sanctions => vec!["name", "type", "source"],
        CsvType::Jurisdictions => vec!["country", "risk_level", "category"],
    };

    // Check required headers exist (case-insensitive)
    let header_lower: Vec<String> = headers.iter().map(|h| h.to_lowercase()).collect();
    for expected in &expected_headers {
        if !header_lower.contains(&expected.to_string()) {
            return Err(format!(
                "Missing required column '{}' for {:?} CSV",
                expected, csv_type
            ));
        }
    }

    // Count and validate records
    let mut record_count = 0;
    for result in reader.records() {
        let record = result.map_err(|e| format!("Invalid CSV record at line {}: {}", record_count + 2, e))?;

        // Validate field lengths
        for (i, field) in record.iter().enumerate() {
            if field.len() > MAX_CSV_FIELD_LENGTH {
                return Err(format!(
                    "Field {} at line {} exceeds maximum length ({} > {})",
                    i + 1,
                    record_count + 2,
                    field.len(),
                    MAX_CSV_FIELD_LENGTH
                ));
            }
        }

        record_count += 1;
    }

    if record_count == 0 {
        return Err("CSV file contains no data records".to_string());
    }

    Ok((record_count, content.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::project::create_project;
    use tempfile::TempDir;

    fn create_test_csv(temp: &TempDir, name: &str, content: &str) -> String {
        let path = temp.path().join(name);
        fs::write(&path, content).unwrap();
        path.to_string_lossy().to_string()
    }

    #[tokio::test]
    async fn test_import_csv_suppliers_success() {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path().to_string_lossy().to_string();

        let project = create_project(workspace.clone(), "test".to_string())
            .await
            .unwrap();

        let csv_content = "name,jurisdiction,tier\nAcme GmbH,DE,1\nGlobex AG,PL,2\n";
        let csv_path = create_test_csv(&temp, "suppliers.csv", csv_content);

        let result = import_csv(project.path.clone(), CsvType::Suppliers, csv_path).await;

        assert!(result.is_ok());
        let import = result.unwrap();
        assert_eq!(import.record_count, 2);
        assert!(import.hash.starts_with("0x"));
        assert!(Path::new(&project.path).join("input/suppliers.csv").exists());
    }

    #[tokio::test]
    async fn test_import_csv_ubos_success() {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path().to_string_lossy().to_string();

        let project = create_project(workspace.clone(), "test".to_string())
            .await
            .unwrap();

        let csv_content = "name,birthdate,citizenship\nMax Mustermann,1980-01-01,DE\n";
        let csv_path = create_test_csv(&temp, "ubos.csv", csv_content);

        let result = import_csv(project.path.clone(), CsvType::Ubos, csv_path).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().record_count, 1);
    }

    #[tokio::test]
    async fn test_import_csv_missing_column() {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path().to_string_lossy().to_string();

        let project = create_project(workspace.clone(), "test".to_string())
            .await
            .unwrap();

        let csv_content = "invalid,columns,here\ndata,data,data\n";
        let csv_path = create_test_csv(&temp, "bad.csv", csv_content);

        let result = import_csv(project.path.clone(), CsvType::Suppliers, csv_path).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required column"));
    }

    #[tokio::test]
    async fn test_import_csv_empty() {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path().to_string_lossy().to_string();

        let project = create_project(workspace.clone(), "test".to_string())
            .await
            .unwrap();

        let csv_content = "name,jurisdiction,tier\n"; // Header only
        let csv_path = create_test_csv(&temp, "empty.csv", csv_content);

        let result = import_csv(project.path.clone(), CsvType::Suppliers, csv_path).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no data records"));
    }
}
