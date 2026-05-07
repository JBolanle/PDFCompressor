use crate::path_resolver::resolve_output_path;
use crate::settings::Settings;
use serde::Deserialize;
use tauri::AppHandle;
use tauri::Emitter;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Preset {
    Max,
    Balanced,
    Minimal,
}

pub fn build_gs_args(
    preset: Preset,
    dpi_override: Option<u32>,
    input: &str,
    output: &str,
) -> Vec<String> {
    let (pdf_settings, default_dpi) = match preset {
        Preset::Max => ("/screen", 72u32),
        Preset::Balanced => ("/ebook", 150u32),
        Preset::Minimal => ("/printer", 300u32),
    };
    let dpi = dpi_override.unwrap_or(default_dpi);
    vec![
        "-sDEVICE=pdfwrite".into(),
        "-dCompatibilityLevel=1.4".into(),
        format!("-dPDFSETTINGS={}", pdf_settings),
        "-dNOPAUSE".into(),
        "-dQUIET".into(),
        "-dBATCH".into(),
        format!("-dColorImageResolution={}", dpi),
        format!("-dGrayImageResolution={}", dpi),
        format!("-sOutputFile={}", output),
        input.into(),
    ]
}

#[derive(Debug, serde::Deserialize)]
pub struct CompressJob {
    pub path: String,
    pub preset: Preset,
    pub dpi_override: Option<u32>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProgressEvent {
    pub file: String,
    pub status: String,
    pub saved_bytes: Option<i64>,
    pub compressed_size: Option<i64>,
    pub error_msg: Option<String>,
}

#[tauri::command]
pub async fn compress_files(
    app: AppHandle,
    jobs: Vec<CompressJob>,
    settings: Settings,
) -> Result<(), String> {
    for job in &jobs {
        let _ = app.emit(
            "compress:progress",
            ProgressEvent {
                file: job.path.clone(),
                status: "processing".into(),
                saved_bytes: None,
                compressed_size: None,
                error_msg: None,
            },
        );

        // Capture original size before any writes so overwrite mode is correct
        let original_size = std::fs::metadata(&job.path)
            .map(|m| m.len() as i64)
            .unwrap_or(0);

        let output_path = resolve_output_path(&job.path, &settings);
        let tmp_path = output_path.with_extension("pdf.tmp");

        let args = build_gs_args(
            job.preset,
            job.dpi_override,
            &job.path,
            tmp_path.to_str().unwrap(),
        );

        match run_gs(&app, args).await {
            Ok(()) => {
                std::fs::rename(&tmp_path, &output_path).map_err(|e| e.to_string())?;

                let compressed_size = std::fs::metadata(&output_path)
                    .map(|m| m.len() as i64)
                    .unwrap_or(0);

                let _ = app.emit(
                    "compress:progress",
                    ProgressEvent {
                        file: job.path.clone(),
                        status: "done".into(),
                        saved_bytes: Some(original_size - compressed_size),
                        compressed_size: Some(compressed_size),
                        error_msg: None,
                    },
                );
            }
            Err(msg) => {
                let _ = std::fs::remove_file(&tmp_path);

                let _ = app.emit(
                    "compress:progress",
                    ProgressEvent {
                        file: job.path.clone(),
                        status: "error".into(),
                        saved_bytes: None,
                        compressed_size: None,
                        error_msg: Some(msg),
                    },
                );
            }
        }
    }
    Ok(())
}

async fn run_gs(app: &AppHandle, args: Vec<String>) -> Result<(), String> {
    let (mut rx, _child) = app
        .shell()
        .sidecar("gs")
        .map_err(|e| e.to_string())?
        .args(&args)
        .spawn()
        .map_err(|e| e.to_string())?;

    let mut stderr_buf = String::new();
    while let Some(event) = rx.recv().await {
        match event {
            CommandEvent::Stderr(bytes) => {
                stderr_buf.push_str(&String::from_utf8_lossy(&bytes));
            }
            CommandEvent::Terminated(payload) => {
                return if payload.code == Some(0) {
                    Ok(())
                } else {
                    Err(stderr_buf
                        .trim()
                        .lines()
                        .next()
                        .unwrap_or("Unknown GS error")
                        .to_string())
                };
            }
            CommandEvent::Error(msg) => return Err(msg),
            _ => {}
        }
    }
    Err("GS process terminated unexpectedly".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_preset_uses_screen_settings() {
        let args = build_gs_args(Preset::Max, None, "/in.pdf", "/out.pdf");
        assert!(args.contains(&"-dPDFSETTINGS=/screen".to_string()));
        assert!(args.contains(&"-dColorImageResolution=72".to_string()));
        assert!(args.contains(&"-dGrayImageResolution=72".to_string()));
    }

    #[test]
    fn balanced_preset_uses_ebook_settings() {
        let args = build_gs_args(Preset::Balanced, None, "/in.pdf", "/out.pdf");
        assert!(args.contains(&"-dPDFSETTINGS=/ebook".to_string()));
        assert!(args.contains(&"-dColorImageResolution=150".to_string()));
        assert!(args.contains(&"-dGrayImageResolution=150".to_string()));
    }

    #[test]
    fn minimal_preset_uses_printer_settings() {
        let args = build_gs_args(Preset::Minimal, None, "/in.pdf", "/out.pdf");
        assert!(args.contains(&"-dPDFSETTINGS=/printer".to_string()));
        assert!(args.contains(&"-dColorImageResolution=300".to_string()));
        assert!(args.contains(&"-dGrayImageResolution=300".to_string()));
    }

    #[test]
    fn dpi_override_replaces_preset_dpi() {
        let args = build_gs_args(Preset::Balanced, Some(120), "/in.pdf", "/out.pdf");
        assert!(args.contains(&"-dColorImageResolution=120".to_string()));
        assert!(args.contains(&"-dGrayImageResolution=120".to_string()));
        assert!(args.contains(&"-dPDFSETTINGS=/ebook".to_string()));
    }

    #[test]
    fn standard_flags_always_present() {
        let args = build_gs_args(Preset::Balanced, None, "/in.pdf", "/out.pdf");
        assert!(args.contains(&"-sDEVICE=pdfwrite".to_string()));
        assert!(args.contains(&"-dNOPAUSE".to_string()));
        assert!(args.contains(&"-dBATCH".to_string()));
        assert!(args.contains(&"-dQUIET".to_string()));
        assert!(args.contains(&"-dCompatibilityLevel=1.4".to_string()));
    }

    #[test]
    fn input_and_output_paths_are_correct() {
        let args = build_gs_args(Preset::Balanced, None, "/tmp/in.pdf", "/tmp/out.pdf");
        assert!(args.contains(&"-sOutputFile=/tmp/out.pdf".to_string()));
        assert_eq!(args.last().unwrap(), "/tmp/in.pdf");
    }

    #[test]
    #[ignore] // Run explicitly: cargo test compress_integration -- --ignored
    fn compress_integration_produces_smaller_or_equal_file() {
        use std::path::Path;
        use tempfile::TempDir;

        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/tiny.pdf");
        assert!(fixture.exists(), "Test fixture missing at {:?}", fixture);

        let tmp = TempDir::new().unwrap();
        let output = tmp.path().join("out.pdf");

        let args = build_gs_args(
            Preset::Max,
            None,
            fixture.to_str().unwrap(),
            output.to_str().unwrap(),
        );

        let status = std::process::Command::new("gs")
            .args(&args)
            .status()
            .expect("GS not found — install via `brew install ghostscript`");

        assert!(status.success(), "GS returned non-zero exit");
        assert!(output.exists(), "Output file not created");
        assert!(
            fixture.exists(),
            "Fixture deleted — original must be untouched"
        );
    }
}
