use std::path::PathBuf;
use crate::settings::{Settings, OutputMode, NamingMode};

pub fn resolve_output_path(input: &str, settings: &Settings) -> PathBuf {
    use std::path::Path;
    let input_path = Path::new(input);
    let stem = input_path.file_stem().unwrap_or_default().to_string_lossy();
    let ext = input_path.extension().unwrap_or_default().to_string_lossy();

    let dir = match &settings.output_mode {
        OutputMode::SameAsSource => input_path.parent().unwrap_or(Path::new(".")).to_path_buf(),
        OutputMode::CustomFolder => PathBuf::from(settings.output_folder.as_deref().unwrap_or(".")),
    };

    match &settings.naming {
        NamingMode::Overwrite => dir.join(format!("{}.{}", stem, ext)),
        NamingMode::Suffix    => dir.join(format!("{}_compressed.{}", stem, ext)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn same_source_suffix() -> Settings {
        Settings {
            output_mode: OutputMode::SameAsSource,
            output_folder: None,
            naming: NamingMode::Suffix,
        }
    }

    #[test]
    fn same_source_suffix_appends_compressed() {
        let result = resolve_output_path("/home/user/docs/report.pdf", &same_source_suffix());
        assert_eq!(result, PathBuf::from("/home/user/docs/report_compressed.pdf"));
    }

    #[test]
    fn custom_folder_suffix_uses_custom_dir() {
        let settings = Settings {
            output_mode: OutputMode::CustomFolder,
            output_folder: Some("/home/user/output".into()),
            naming: NamingMode::Suffix,
        };
        let result = resolve_output_path("/home/user/docs/report.pdf", &settings);
        assert_eq!(result, PathBuf::from("/home/user/output/report_compressed.pdf"));
    }

    #[test]
    fn overwrite_mode_returns_same_path() {
        let settings = Settings {
            output_mode: OutputMode::SameAsSource,
            output_folder: None,
            naming: NamingMode::Overwrite,
        };
        let result = resolve_output_path("/home/user/docs/report.pdf", &settings);
        assert_eq!(result, PathBuf::from("/home/user/docs/report.pdf"));
    }

    #[test]
    fn custom_folder_overwrite_uses_custom_dir_original_name() {
        let settings = Settings {
            output_mode: OutputMode::CustomFolder,
            output_folder: Some("/out".into()),
            naming: NamingMode::Overwrite,
        };
        let result = resolve_output_path("/home/user/docs/report.pdf", &settings);
        assert_eq!(result, PathBuf::from("/out/report.pdf"));
    }
}
