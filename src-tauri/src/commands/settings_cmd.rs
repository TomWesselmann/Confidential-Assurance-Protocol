//! Settings-related Tauri commands

use crate::settings::AppSettings;

/// Response for app info endpoint
#[derive(serde::Serialize)]
pub struct AppInfo {
    /// Current workspace path
    pub workspace_path: String,
    /// Whether this is the first run
    pub is_first_run: bool,
    /// Whether a custom path is set
    pub has_custom_path: bool,
}

/// Gets current app info including workspace path
#[tauri::command]
pub async fn get_app_info() -> Result<AppInfo, String> {
    let settings = AppSettings::load();
    let workspace_path = settings.get_workspace_path()?;
    let is_first_run = AppSettings::is_first_run();
    let has_custom_path = settings.workspace_path.is_some();

    Ok(AppInfo {
        workspace_path,
        is_first_run,
        has_custom_path,
    })
}

/// Sets the workspace path
#[tauri::command]
pub async fn set_workspace_path(path: String) -> Result<String, String> {
    let mut settings = AppSettings::load();
    settings.set_workspace_path(path.clone())?;
    Ok(path)
}

/// Resets workspace to default (~/Documents/CAP-Proofs)
#[tauri::command]
pub async fn reset_workspace_path() -> Result<String, String> {
    let mut settings = AppSettings::load();
    settings.workspace_path = None;
    settings.save()?;
    settings.get_workspace_path()
}
