//! Tauri command modules for Taurin Desktop App
//!
//! This module organizes all Tauri commands into logical groups.

pub mod project;
pub mod import;
pub mod commitments;
pub mod policy;
pub mod manifest;
pub mod proof;
pub mod export;
pub mod verify;
pub mod audit;
pub mod settings_cmd;
pub mod signing;

// Re-export all commands for easy registration
pub use project::{create_project, list_projects, get_project_status, read_file_content, list_all_projects, create_new_project, create_temp_project, create_project_in_folder};
pub use settings_cmd::{get_app_info, set_workspace_path, reset_workspace_path};
pub use import::import_csv;
pub use commitments::create_commitments;
pub use policy::load_policy;
pub use manifest::build_manifest;
pub use proof::build_proof;
pub use export::export_bundle;
pub use verify::{verify_bundle, get_bundle_info};
pub use audit::{get_audit_log, verify_audit_chain};
pub use signing::{generate_keys, list_keys, sign_project_manifest, verify_manifest_signature};
