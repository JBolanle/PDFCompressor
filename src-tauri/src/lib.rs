// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod compress;
pub mod finder;
pub mod headless;
pub mod menu;
pub mod path_resolver;
pub mod settings;
pub mod updater;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Set true as soon as any file-open invocation (RunEvent::Opened or a
/// second-instance argv with PDFs) is observed during startup. The deferred
/// window-show task reads this to decide whether to surface the main window.
#[derive(Default)]
pub struct LaunchState {
    pub headless: Arc<AtomicBool>,
}

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
        Ok(_) => {
            let _ = std::fs::remove_file(&test_path);
            true
        }
        Err(_) => false,
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use crate::compress::compress_files;
    use crate::finder::reveal_in_finder;
    use crate::headless::compress_paths_headless;
    use crate::menu::{build_menu, set_menu_item_enabled};
    use crate::settings::{
        get_settings, load_settings_from_path, save_settings, settings_file_path,
    };
    use crate::updater::check_for_update;
    use std::path::PathBuf;
    use tauri::{Emitter, Manager, RunEvent};

    let launch_state = LaunchState::default();
    let headless_flag_for_single_instance = launch_state.headless.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(
            move |app, argv, _cwd| {
                // A second launch (e.g. user opens another PDF via "Open With"
                // while we're already running) forwards its argv here.
                let paths = argv_pdf_paths(&argv);
                if !paths.is_empty() {
                    headless_flag_for_single_instance.store(true, Ordering::Relaxed);
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        compress_paths_headless(app, paths).await;
                    });
                }
            },
        ))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .manage(launch_state)
        .setup(|app| {
            let (menu, registry, auto_update_item) = build_menu(app.handle())?;
            app.set_menu(menu)?;

            let saved =
                load_settings_from_path(&settings_file_path(app.handle())).unwrap_or_default();
            auto_update_item.set_checked(saved.auto_update_check).ok();

            app.handle().on_menu_event(|app, event| {
                let name = match event.id().as_ref() {
                    "add-files" => "menu:add-files",
                    "reveal-in-finder" => "menu:reveal-in-finder",
                    "clear-queue" => "menu:clear-queue",
                    "compress" => "menu:compress",
                    "reset-selected" => "menu:reset-selected",
                    "check-for-update" => "menu:check-for-update",
                    "check-for-update-auto" => "menu:check-for-update-auto",
                    _ => return,
                };
                let _ = app.emit(name, ());
            });
            app.manage(registry);

            // The window is configured with `visible: false`. If no file-open
            // event arrives within a short grace window, surface it. If one
            // does arrive, the Opened-event handler keeps the flag set and
            // we stay headless for this launch.
            let handle = app.handle().clone();
            let flag = handle.state::<LaunchState>().headless.clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(400)).await;
                if !flag.load(Ordering::Relaxed) {
                    if let Some(window) = handle.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            compress_files,
            get_settings,
            save_settings,
            reveal_in_finder,
            get_file_meta,
            validate_pdf,
            set_menu_item_enabled,
            check_for_update,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let RunEvent::Opened { urls } = event {
                let paths: Vec<PathBuf> =
                    urls.iter().filter_map(|u| u.to_file_path().ok()).collect();
                if paths.is_empty() {
                    return;
                }
                app.state::<LaunchState>()
                    .headless
                    .store(true, Ordering::Relaxed);
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    compress_paths_headless(app, paths).await;
                });
            }
        });
}

/// Extract PDF paths from a process argv vector, skipping the program name
/// and any non-`.pdf` tokens. Used by the single-instance forwarder.
pub fn argv_pdf_paths(argv: &[String]) -> Vec<std::path::PathBuf> {
    argv.iter()
        .skip(1)
        .filter(|s| s.to_ascii_lowercase().ends_with(".pdf"))
        .map(std::path::PathBuf::from)
        .collect()
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
        assert!(check_path_writable(
            std::env::temp_dir().to_str().unwrap().to_string()
        ));
    }

    #[test]
    fn check_path_writable_returns_false_for_nonexistent() {
        assert!(!check_path_writable(
            "/nonexistent/path/that/cannot/exist".to_string()
        ));
    }

    #[test]
    fn argv_pdf_paths_skips_program_name() {
        let argv = vec![
            "/Applications/compress[pdf].app/Contents/MacOS/pdf-compressor".to_string(),
            "/tmp/a.pdf".to_string(),
        ];
        assert_eq!(
            argv_pdf_paths(&argv),
            vec![std::path::PathBuf::from("/tmp/a.pdf")]
        );
    }

    #[test]
    fn argv_pdf_paths_filters_non_pdf_args() {
        let argv = vec![
            "binary".to_string(),
            "/tmp/a.pdf".to_string(),
            "--flag".to_string(),
            "/tmp/b.txt".to_string(),
            "/tmp/c.PDF".to_string(),
        ];
        assert_eq!(
            argv_pdf_paths(&argv),
            vec![
                std::path::PathBuf::from("/tmp/a.pdf"),
                std::path::PathBuf::from("/tmp/c.PDF"),
            ]
        );
    }

    #[test]
    fn argv_pdf_paths_empty_when_no_pdfs() {
        let argv = vec!["binary".to_string(), "--version".to_string()];
        assert!(argv_pdf_paths(&argv).is_empty());
    }
}
