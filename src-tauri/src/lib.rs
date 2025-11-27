//! Taurin Desktop App - Tauri Backend
//!
//! This is the main entry point for the Tauri application.
//! It registers all commands and initializes plugins.
//!
//! # Architecture
//! - `commands/` - Tauri command handlers
//! - `types.rs` - Shared types for request/response
//! - `security.rs` - Security validation helpers
//!
//! # Commands
//! ## Project Management
//! - `create_project` - Create new project with standard structure
//! - `list_projects` - List all projects in workspace
//! - `get_project_status` - Get current workflow status
//!
//! ## Proofer Workflow
//! - `import_csv` - Import CSV files (suppliers, ubos)
//! - `create_commitments` - Generate Merkle roots from CSVs
//! - `load_policy` - Load and validate policy YAML
//! - `build_manifest` - Create manifest from commitments + policy
//! - `build_proof` - Generate proof (with progress events)
//! - `export_bundle` - Export as ZIP bundle
//!
//! ## Verifier
//! - `verify_bundle` - Verify a proof bundle
//! - `get_bundle_info` - Get bundle metadata

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audit_logger;
mod commands;
mod security;
mod types;

use commands::{
    // Project management
    create_project,
    list_projects,
    get_project_status,
    read_file_content,
    // Proofer workflow
    import_csv,
    create_commitments,
    load_policy,
    build_manifest,
    build_proof,
    export_bundle,
    // Verifier
    verify_bundle,
    get_bundle_info,
    // Audit
    get_audit_log,
    verify_audit_chain,
};

// ============================================================================
// Main Entry Point
// ============================================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            // Project management
            create_project,
            list_projects,
            get_project_status,
            read_file_content,
            // Proofer workflow
            import_csv,
            create_commitments,
            load_policy,
            build_manifest,
            build_proof,
            export_bundle,
            // Verifier
            verify_bundle,
            get_bundle_info,
            // Audit
            get_audit_log,
            verify_audit_chain,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
