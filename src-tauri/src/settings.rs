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
    #[serde(default)]
    pub auto_update_check: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            output_mode: OutputMode::SameAsSource,
            output_folder: None,
            naming: NamingMode::Suffix,
            auto_update_check: false,
        }
    }
}

pub fn settings_file_path(app: &tauri::AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .expect("app data dir")
        .join("settings.json")
}

pub fn validate_settings(settings: &Settings) -> Result<(), String> {
    if let Some(folder) = settings.output_folder.as_deref() {
        if folder.chars().any(|c| c.is_control()) {
            return Err("output_folder contains control characters".into());
        }
        let path = Path::new(folder);
        if !path.is_absolute() {
            return Err("output_folder must be an absolute path".into());
        }
        if !path.is_dir() {
            return Err("output_folder must be an existing directory".into());
        }
    }
    Ok(())
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
    validate_settings(&settings)?;
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
    fn validate_accepts_none_output_folder() {
        let s = Settings {
            output_mode: OutputMode::SameAsSource,
            output_folder: None,
            naming: NamingMode::Suffix,
            auto_update_check: false,
        };
        assert!(validate_settings(&s).is_ok());
    }

    #[test]
    fn validate_accepts_existing_absolute_directory() {
        let tmp = TempDir::new().unwrap();
        let s = Settings {
            output_mode: OutputMode::CustomFolder,
            output_folder: Some(tmp.path().to_string_lossy().into_owned()),
            naming: NamingMode::Suffix,
            auto_update_check: false,
        };
        assert!(validate_settings(&s).is_ok());
    }

    #[test]
    fn validate_rejects_relative_output_folder() {
        let s = Settings {
            output_mode: OutputMode::CustomFolder,
            output_folder: Some("relative/path".into()),
            naming: NamingMode::Suffix,
            auto_update_check: false,
        };
        assert!(validate_settings(&s).is_err());
    }

    #[test]
    fn validate_rejects_nonexistent_output_folder() {
        let s = Settings {
            output_mode: OutputMode::CustomFolder,
            output_folder: Some("/nonexistent/path/that/should/not/exist/xyz".into()),
            naming: NamingMode::Suffix,
            auto_update_check: false,
        };
        assert!(validate_settings(&s).is_err());
    }

    #[test]
    fn validate_rejects_output_folder_pointing_at_file() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("a.txt");
        std::fs::write(&file, "x").unwrap();
        let s = Settings {
            output_mode: OutputMode::CustomFolder,
            output_folder: Some(file.to_string_lossy().into_owned()),
            naming: NamingMode::Suffix,
            auto_update_check: false,
        };
        assert!(validate_settings(&s).is_err());
    }

    #[test]
    fn validate_rejects_output_folder_with_control_chars() {
        let s = Settings {
            output_mode: OutputMode::CustomFolder,
            output_folder: Some("/tmp/has\nnewline".into()),
            naming: NamingMode::Suffix,
            auto_update_check: false,
        };
        assert!(validate_settings(&s).is_err());
    }

    #[test]
    fn settings_round_trip() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("settings.json");

        let original = Settings {
            output_mode: OutputMode::CustomFolder,
            output_folder: Some("/my/folder".into()),
            naming: NamingMode::Overwrite,
            auto_update_check: false,
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

    #[test]
    fn auto_update_check_defaults_to_false() {
        let s = Settings::default();
        assert!(!s.auto_update_check);
    }

    #[test]
    fn auto_update_check_round_trips_as_true() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("settings.json");
        let original = Settings {
            auto_update_check: true,
            ..Settings::default()
        };
        save_settings_to_path(&original, &path).unwrap();
        let loaded = load_settings_from_path(&path).unwrap();
        assert!(loaded.auto_update_check);
    }

    #[test]
    fn auto_update_check_defaults_when_absent_from_json() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("settings.json");
        std::fs::write(&path, r#"{"output_mode":"same_as_source","naming":"suffix"}"#).unwrap();
        let loaded = load_settings_from_path(&path).unwrap();
        assert!(!loaded.auto_update_check);
    }
}
