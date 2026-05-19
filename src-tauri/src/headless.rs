use crate::compress::{compress_files_inner, CompressJob};
use crate::is_pdf;
use crate::settings::{load_settings_from_path, settings_file_path, NamingMode, Settings};
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

/// Right-click → Open With invocations always write to a `_compressed.pdf`
/// sibling, never overwrite. The in-app overwrite setting is a deliberate
/// choice made at the UI; Finder context has no preview/confirm, so we
/// refuse to destroy the original from here even if the user enabled
/// overwrite in the app.
pub fn settings_for_headless(mut s: Settings) -> Settings {
    s.naming = NamingMode::Suffix;
    s
}

pub fn filter_pdf_paths(paths: Vec<PathBuf>) -> Vec<String> {
    paths
        .into_iter()
        .filter_map(|p| {
            let s = p.to_string_lossy().into_owned();
            if Path::new(&s).is_file() && is_pdf(&s) {
                Some(s)
            } else {
                None
            }
        })
        .collect()
}

fn human_bytes(bytes: i64) -> String {
    let abs = bytes.unsigned_abs() as f64;
    let (val, unit) = if abs >= 1_073_741_824.0 {
        (abs / 1_073_741_824.0, "GB")
    } else if abs >= 1_048_576.0 {
        (abs / 1_048_576.0, "MB")
    } else if abs >= 1024.0 {
        (abs / 1024.0, "KB")
    } else {
        (abs, "B")
    };
    let sign = if bytes < 0 { "-" } else { "" };
    format!("{sign}{val:.1} {unit}")
}

pub fn summary_message(
    succeeded: usize,
    failed: usize,
    total_saved_bytes: i64,
) -> (String, String) {
    let title = match (succeeded, failed) {
        (0, _) => "Compression failed".to_string(),
        (1, 0) => "Compressed 1 PDF".to_string(),
        (n, 0) => format!("Compressed {n} PDFs"),
        (n, f) => format!("Compressed {n} of {} PDFs", n + f),
    };
    let body = if succeeded == 0 {
        format!("{failed} file(s) failed")
    } else if failed == 0 {
        format!("Saved {}", human_bytes(total_saved_bytes))
    } else {
        format!(
            "Saved {} \u{2022} {failed} failed",
            human_bytes(total_saved_bytes)
        )
    };
    (title, body)
}

pub async fn compress_paths_headless(app: AppHandle, paths: Vec<PathBuf>) {
    let pdfs = filter_pdf_paths(paths);
    if pdfs.is_empty() {
        return;
    }

    let settings = settings_for_headless(
        load_settings_from_path(&settings_file_path(&app)).unwrap_or_default(),
    );
    let preset = settings.default_preset;

    let jobs: Vec<CompressJob> = pdfs
        .into_iter()
        .map(|path| CompressJob {
            path,
            preset,
            dpi_override: None,
        })
        .collect();

    let outcomes = compress_files_inner(app.clone(), jobs, settings).await;

    let mut succeeded = 0usize;
    let mut failed = 0usize;
    let mut total_saved = 0i64;
    for o in &outcomes {
        if o.error.is_some() {
            failed += 1;
        } else {
            succeeded += 1;
            total_saved += o.saved_bytes.unwrap_or(0);
        }
    }

    let (title, body) = summary_message(succeeded, failed, total_saved);
    let _ = app.notification().builder().title(title).body(body).show();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn filter_pdf_paths_keeps_only_valid_pdfs() {
        let tmp = TempDir::new().unwrap();
        let pdf = tmp.path().join("a.pdf");
        let mut f = std::fs::File::create(&pdf).unwrap();
        f.write_all(b"%PDF-1.4 hello").unwrap();

        let fake = tmp.path().join("b.pdf");
        std::fs::write(&fake, b"not a pdf").unwrap();

        let txt = tmp.path().join("c.txt");
        std::fs::write(&txt, b"text").unwrap();

        let missing = tmp.path().join("ghost.pdf");

        let kept = filter_pdf_paths(vec![pdf.clone(), fake, txt, missing]);
        assert_eq!(kept, vec![pdf.to_string_lossy().into_owned()]);
    }

    #[test]
    fn filter_pdf_paths_empty_input_returns_empty() {
        assert!(filter_pdf_paths(vec![]).is_empty());
    }

    #[test]
    fn human_bytes_formats_units() {
        assert_eq!(human_bytes(0), "0.0 B");
        assert_eq!(human_bytes(2048), "2.0 KB");
        assert_eq!(human_bytes(5 * 1_048_576), "5.0 MB");
        assert_eq!(human_bytes(3 * 1_073_741_824), "3.0 GB");
    }

    #[test]
    fn human_bytes_handles_negative() {
        assert_eq!(human_bytes(-2048), "-2.0 KB");
    }

    #[test]
    fn summary_for_single_success() {
        let (title, body) = summary_message(1, 0, 1_048_576);
        assert_eq!(title, "Compressed 1 PDF");
        assert_eq!(body, "Saved 1.0 MB");
    }

    #[test]
    fn summary_for_multiple_all_succeed() {
        let (title, body) = summary_message(3, 0, 3 * 1_048_576);
        assert_eq!(title, "Compressed 3 PDFs");
        assert_eq!(body, "Saved 3.0 MB");
    }

    #[test]
    fn summary_for_partial_failure() {
        let (title, body) = summary_message(2, 1, 2 * 1_048_576);
        assert_eq!(title, "Compressed 2 of 3 PDFs");
        assert!(body.contains("Saved 2.0 MB"));
        assert!(body.contains("1 failed"));
    }

    #[test]
    fn summary_for_total_failure() {
        let (title, body) = summary_message(0, 2, 0);
        assert_eq!(title, "Compression failed");
        assert_eq!(body, "2 file(s) failed");
    }

    #[test]
    fn settings_for_headless_forces_suffix_naming() {
        let user_chose_overwrite = Settings {
            naming: NamingMode::Overwrite,
            ..Settings::default()
        };
        let forced = settings_for_headless(user_chose_overwrite);
        assert!(matches!(forced.naming, NamingMode::Suffix));
    }

    #[test]
    fn settings_for_headless_preserves_output_mode_and_preset() {
        use crate::compress::Preset;
        use crate::settings::OutputMode;
        let user = Settings {
            naming: NamingMode::Overwrite,
            output_mode: OutputMode::CustomFolder,
            output_folder: Some("/Users/me/out".into()),
            default_preset: Preset::Max,
            ..Settings::default()
        };
        let forced = settings_for_headless(user);
        assert!(matches!(forced.output_mode, OutputMode::CustomFolder));
        assert_eq!(forced.output_folder.as_deref(), Some("/Users/me/out"));
        assert_eq!(forced.default_preset, Preset::Max);
    }
}
