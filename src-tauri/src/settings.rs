use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputMode {
    SameAsSource,
    CustomFolder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NamingMode {
    Suffix,
    Overwrite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub output_mode: OutputMode,
    pub output_folder: Option<String>,
    pub naming: NamingMode,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            output_mode: OutputMode::SameAsSource,
            output_folder: None,
            naming: NamingMode::Suffix,
        }
    }
}

pub fn settings_file_path(app: &tauri::AppHandle) -> PathBuf {
    app.path().app_data_dir()
        .expect("app data dir")
        .join("settings.json")
}

pub fn save_settings_to_path(settings: &Settings, path: &Path) -> Result<(), String> {
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

pub fn load_settings_from_path(path: &Path) -> Result<Settings, String> {
    let json = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&json).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_settings(app: tauri::AppHandle) -> Settings {
    let path = settings_file_path(&app);
    load_settings_from_path(&path).unwrap_or_default()
}

#[tauri::command]
pub fn save_settings(app: tauri::AppHandle, settings: Settings) -> Result<(), String> {
    let path = settings_file_path(&app);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    save_settings_to_path(&settings, &path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn settings_round_trip() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("settings.json");

        let original = Settings {
            output_mode: OutputMode::CustomFolder,
            output_folder: Some("/my/folder".into()),
            naming: NamingMode::Overwrite,
        };

        save_settings_to_path(&original, &path).unwrap();
        let loaded = load_settings_from_path(&path).unwrap();

        assert!(matches!(loaded.output_mode, OutputMode::CustomFolder));
        assert_eq!(loaded.output_folder.as_deref(), Some("/my/folder"));
        assert!(matches!(loaded.naming, NamingMode::Overwrite));
    }

    #[test]
    fn load_returns_error_for_missing_file() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("nonexistent.json");
        assert!(load_settings_from_path(&path).is_err());
    }

    #[test]
    fn default_settings_are_same_as_source_suffix() {
        let s = Settings::default();
        assert!(matches!(s.output_mode, OutputMode::SameAsSource));
        assert!(s.output_folder.is_none());
        assert!(matches!(s.naming, NamingMode::Suffix));
    }
}
