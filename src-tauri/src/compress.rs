use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Preset {
    Max,
    Balanced,
    Minimal,
}

pub fn build_gs_args(preset: Preset, dpi_override: Option<u32>, input: &str, output: &str) -> Vec<String> {
    let (pdf_settings, default_dpi) = match preset {
        Preset::Max      => ("/screen",  72u32),
        Preset::Balanced => ("/ebook",  150u32),
        Preset::Minimal  => ("/printer", 300u32),
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
}
