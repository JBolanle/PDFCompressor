// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
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
