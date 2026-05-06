// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod compress;
pub mod finder;
pub mod path_resolver;
pub mod settings;


#[derive(serde::Serialize)]
struct FileMeta {
    size: u64,
}

#[tauri::command]
fn get_file_meta(path: String) -> Result<FileMeta, String> {
    std::fs::metadata(&path)
        .map(|m| FileMeta { size: m.len() })
        .map_err(|e| e.to_string())
}

pub fn is_pdf(path: &str) -> bool {
    use std::io::Read;
    let mut file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic).is_ok() && &magic == b"%PDF"
}

#[tauri::command]
fn validate_pdf(path: String) -> bool {
    is_pdf(&path)
}

pub fn check_path_writable(path: String) -> bool {
    let dir = std::path::Path::new(&path);
    let test_path = dir.join(".pdf_compressor_write_test");
    match std::fs::File::create(&test_path) {
        Ok(_) => { let _ = std::fs::remove_file(&test_path); true }
        Err(_) => false,
    }
}

#[tauri::command]
fn check_path_writable_cmd(path: String) -> bool {
    check_path_writable(path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use crate::compress::compress_files;
    use crate::settings::{get_settings, save_settings};
    use crate::finder::reveal_in_finder;

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            compress_files,
            get_settings,
            save_settings,
            reveal_in_finder,
            get_file_meta,
            validate_pdf,
            check_path_writable_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod sidecar_tests {
    #[test]
    fn gs_binary_exists_for_current_arch() {
        use std::path::PathBuf;

        let output = std::process::Command::new("rustc")
            .args(["-vV"])
            .output()
            .expect("rustc not found");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let target = stdout
            .lines()
            .find(|l| l.starts_with("host:"))
            .map(|l| l.trim_start_matches("host:").trim().to_string())
            .unwrap_or_default();

        let binary = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("binaries")
            .join(format!("gs-{}", target));

        assert!(binary.exists(), "GS sidecar not found at {:?}", binary);

        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&binary).unwrap().permissions().mode();
        assert!(mode & 0o111 != 0, "GS binary is not executable");
    }
}

#[cfg(test)]
mod lib_tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn is_pdf_returns_true_for_valid_pdf_magic() {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"%PDF-1.4 rest of content").unwrap();
        assert!(is_pdf(f.path().to_str().unwrap()));
    }

    #[test]
    fn is_pdf_returns_false_for_non_pdf() {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"PK\x03\x04 zip file").unwrap();
        assert!(!is_pdf(f.path().to_str().unwrap()));
    }

    #[test]
    fn is_pdf_returns_false_for_missing_file() {
        assert!(!is_pdf("/nonexistent/path/file.pdf"));
    }

    #[test]
    fn check_path_writable_returns_true_for_tmp() {
        assert!(check_path_writable(std::env::temp_dir().to_str().unwrap().to_string()));
    }

    #[test]
    fn check_path_writable_returns_false_for_nonexistent() {
        assert!(!check_path_writable("/nonexistent/path/that/cannot/exist".to_string()));
    }
}
