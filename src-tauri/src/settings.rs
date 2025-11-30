//! Application settings management
//!
//! Persists user settings like workspace path to disk.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Custom workspace path (if set by user)
    pub workspace_path: Option<String>,

    /// Schema version for future migrations
    #[serde(default = "default_schema")]
    pub schema: String,
}

fn default_schema() -> String {
    "1.0".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            workspace_path: None,
            schema: default_schema(),
        }
    }
}

impl AppSettings {
    /// Gets the settings file path
    /// Uses ~/.config/cap-desktop-proofer/settings.json on macOS/Linux
    fn settings_path() -> Result<PathBuf, String> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| "Could not determine config directory".to_string())?;

        let app_config = config_dir.join("cap-desktop-proofer");

        // Create directory if needed
        if !app_config.exists() {
            fs::create_dir_all(&app_config)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        Ok(app_config.join("settings.json"))
    }

    /// Loads settings from disk, returns default if not found
    pub fn load() -> Self {
        match Self::settings_path() {
            Ok(path) => {
                if path.exists() {
                    match fs::read_to_string(&path) {
                        Ok(content) => {
                            serde_json::from_str(&content).unwrap_or_default()
                        }
                        Err(_) => Self::default(),
                    }
                } else {
                    Self::default()
                }
            }
            Err(_) => Self::default(),
        }
    }

    /// Saves settings to disk
    pub fn save(&self) -> Result<(), String> {
        let path = Self::settings_path()?;
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write settings: {}", e))?;
        Ok(())
    }

    /// Gets the effective workspace path
    /// Returns custom path if set, otherwise default ~/Documents/CAP-Proofs
    pub fn get_workspace_path(&self) -> Result<String, String> {
        if let Some(ref custom_path) = self.workspace_path {
            // Verify custom path still exists
            let path = std::path::Path::new(custom_path);
            if path.exists() {
                return Ok(custom_path.clone());
            }
            // Fall through to default if custom path no longer exists
        }

        // Default path
        let documents_dir = dirs::document_dir()
            .ok_or_else(|| "Could not determine Documents directory".to_string())?;

        let default_workspace = documents_dir.join("CAP-Proofs");

        // Create if doesn't exist
        if !default_workspace.exists() {
            fs::create_dir_all(&default_workspace)
                .map_err(|e| format!("Failed to create workspace: {}", e))?;
        }

        Ok(default_workspace.to_string_lossy().to_string())
    }

    /// Sets a custom workspace path
    pub fn set_workspace_path(&mut self, path: String) -> Result<(), String> {
        // Verify path exists
        let p = std::path::Path::new(&path);
        if !p.exists() {
            return Err("Path does not exist".to_string());
        }
        if !p.is_dir() {
            return Err("Path is not a directory".to_string());
        }

        self.workspace_path = Some(path);
        self.save()
    }

    /// Checks if this is the first run (no settings file exists)
    pub fn is_first_run() -> bool {
        match Self::settings_path() {
            Ok(path) => !path.exists(),
            Err(_) => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = AppSettings::default();
        assert!(settings.workspace_path.is_none());
    }
}
