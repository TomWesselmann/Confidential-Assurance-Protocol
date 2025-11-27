//! Security helpers for Taurin Desktop App
//!
//! This module provides validation and sanitization functions
//! to prevent common security vulnerabilities.

use std::path::Path;

// ============================================================================
// Constants
// ============================================================================

/// Maximum length for project names
pub const MAX_PROJECT_NAME_LENGTH: usize = 128;

/// Maximum file size for CSV files (100 MB)
pub const MAX_CSV_FILE_SIZE: u64 = 100 * 1024 * 1024;

/// Maximum file size for policy files (1 MB)
pub const MAX_POLICY_FILE_SIZE: u64 = 1024 * 1024;

/// Maximum field length in CSV
pub const MAX_CSV_FIELD_LENGTH: usize = 1000;

// ============================================================================
// Error Sanitization
// ============================================================================

/// Sanitizes error messages to prevent path leaks (REQ-13)
///
/// Removes absolute paths from error messages to prevent information disclosure.
pub fn sanitize_error_message(err: &str) -> String {
    err.replace("/Users/", "[USER]/")
        .replace("/home/", "[USER]/")
        .replace("C:\\", "[DRIVE]\\")
        .replace("D:\\", "[DRIVE]\\")
        .replace("\\Users\\", "\\[USER]\\")
        .replace("\\home\\", "\\[USER]\\")
}

// ============================================================================
// Path Validation
// ============================================================================

/// Validates a project name for safety
///
/// # Security
/// - Prevents path traversal via ".."
/// - Prevents absolute paths via "/" or "\"
/// - Limits length to prevent filesystem issues
pub fn validate_project_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Project name cannot be empty".to_string());
    }

    if name.len() > MAX_PROJECT_NAME_LENGTH {
        return Err(format!(
            "Project name too long (max {} characters)",
            MAX_PROJECT_NAME_LENGTH
        ));
    }

    if name.contains("..") {
        return Err("Project name cannot contain '..'".to_string());
    }

    if name.contains('/') || name.contains('\\') {
        return Err("Project name cannot contain path separators".to_string());
    }

    // Check for invalid filesystem characters
    let invalid_chars = ['<', '>', ':', '"', '|', '?', '*', '\0'];
    for c in invalid_chars {
        if name.contains(c) {
            return Err(format!("Project name cannot contain '{}'", c));
        }
    }

    // Check for reserved names on Windows
    let reserved = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    let upper = name.to_uppercase();
    if reserved.contains(&upper.as_str()) {
        return Err("Project name is a reserved system name".to_string());
    }

    Ok(())
}

/// Validates that a target path is within a project directory
///
/// # Security
/// - Prevents path traversal attacks
/// - Uses canonicalization to resolve symlinks
pub fn validate_path_within_project(project_path: &Path, target: &Path) -> Result<(), String> {
    // Canonicalize both paths to resolve symlinks and ".."
    let canonical_project = project_path
        .canonicalize()
        .map_err(|_| "Invalid project path".to_string())?;

    let canonical_target = target
        .canonicalize()
        .map_err(|_| "Invalid target path".to_string())?;

    if !canonical_target.starts_with(&canonical_project) {
        return Err("Path traversal detected: target is outside project".to_string());
    }

    Ok(())
}

/// Validates that a file is a regular file (not symlink, directory, etc.)
///
/// # Security
/// - Prevents symlink attacks
pub fn validate_regular_file(path: &Path) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(path).map_err(|e| {
        sanitize_error_message(&format!("Cannot access file: {}", e))
    })?;

    if metadata.file_type().is_symlink() {
        return Err("Symlinks are not allowed".to_string());
    }

    if !metadata.is_file() {
        return Err("Not a regular file".to_string());
    }

    Ok(())
}

/// Validates file size is within limits
pub fn validate_file_size(path: &Path, max_size: u64) -> Result<u64, String> {
    let metadata = std::fs::metadata(path).map_err(|e| {
        sanitize_error_message(&format!("Cannot read file metadata: {}", e))
    })?;

    let size = metadata.len();
    if size > max_size {
        return Err(format!(
            "File too large: {} bytes (max {} bytes)",
            size, max_size
        ));
    }

    Ok(size)
}

/// Validates that a path exists
pub fn validate_path_exists(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err("Path not found".to_string());
    }
    Ok(())
}

/// Validates that a path does NOT exist (for creation)
pub fn validate_path_not_exists(path: &Path) -> Result<(), String> {
    if path.exists() {
        return Err("Path already exists".to_string());
    }
    Ok(())
}

// ============================================================================
// CSV Validation
// ============================================================================

/// Validates CSV record field lengths
pub fn validate_csv_field(field: &str) -> Result<(), String> {
    if field.len() > MAX_CSV_FIELD_LENGTH {
        return Err(format!(
            "CSV field too long: {} characters (max {})",
            field.len(),
            MAX_CSV_FIELD_LENGTH
        ));
    }
    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_project_name_valid() {
        assert!(validate_project_name("my-project").is_ok());
        assert!(validate_project_name("Project_2024").is_ok());
        assert!(validate_project_name("test").is_ok());
    }

    #[test]
    fn test_validate_project_name_empty() {
        assert!(validate_project_name("").is_err());
    }

    #[test]
    fn test_validate_project_name_traversal() {
        assert!(validate_project_name("../etc").is_err());
        assert!(validate_project_name("foo/../bar").is_err());
        assert!(validate_project_name("..").is_err());
    }

    #[test]
    fn test_validate_project_name_path_sep() {
        assert!(validate_project_name("foo/bar").is_err());
        assert!(validate_project_name("foo\\bar").is_err());
    }

    #[test]
    fn test_validate_project_name_too_long() {
        let long_name = "a".repeat(MAX_PROJECT_NAME_LENGTH + 1);
        assert!(validate_project_name(&long_name).is_err());
    }

    #[test]
    fn test_sanitize_error_message() {
        let err = "File not found: /Users/john/secret.txt";
        let sanitized = sanitize_error_message(err);
        assert!(!sanitized.contains("/Users/john"));
        assert!(sanitized.contains("[USER]"));
    }
}
