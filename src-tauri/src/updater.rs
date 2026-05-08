fn strip_v(tag: &str) -> &str {
    tag.strip_prefix('v').unwrap_or(tag)
}

pub fn parse_version(tag: &str) -> Option<(u64, u64, u64)> {
    let s = strip_v(tag);
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let major = parts[0].parse::<u64>().ok()?;
    let minor = parts[1].parse::<u64>().ok()?;
    let patch = parts[2].parse::<u64>().ok()?;
    Some((major, minor, patch))
}

pub fn is_newer(latest: &str, current: &str) -> bool {
    match (parse_version(latest), parse_version(current)) {
        (Some(l), Some(c)) => l > c,
        _ => false,
    }
}

#[derive(serde::Deserialize)]
struct GithubRelease {
    tag_name: String,
}

#[tauri::command]
pub async fn check_for_update() -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .user_agent("compress-pdf-updater")
        .build()
        .ok()?;

    let release: GithubRelease = client
        .get("https://api.github.com/repos/JBolanle/PDFCompressor/releases/latest")
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()?;

    if is_newer(&release.tag_name, env!("CARGO_PKG_VERSION")) {
        Some(strip_v(&release.tag_name).to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_version_handles_v_prefix() {
        assert_eq!(parse_version("v1.3.0"), Some((1, 3, 0)));
    }

    #[test]
    fn parse_version_handles_no_prefix() {
        assert_eq!(parse_version("1.3.0"), Some((1, 3, 0)));
    }

    #[test]
    fn parse_version_returns_none_for_empty() {
        assert_eq!(parse_version(""), None);
    }

    #[test]
    fn parse_version_returns_none_for_two_parts() {
        assert_eq!(parse_version("v1.0"), None);
    }

    #[test]
    fn parse_version_returns_none_for_prerelease() {
        // "v2.0.0-beta.1" splits into 4 parts on "." → None
        assert_eq!(parse_version("v2.0.0-beta.1"), None);
    }

    #[test]
    fn is_newer_returns_true_when_latest_is_newer() {
        assert!(is_newer("v1.4.0", "1.3.0"));
    }

    #[test]
    fn is_newer_returns_false_for_same_version() {
        assert!(!is_newer("v1.3.0", "1.3.0"));
    }

    #[test]
    fn is_newer_returns_false_when_older() {
        assert!(!is_newer("v1.2.0", "1.3.0"));
    }

    #[test]
    fn is_newer_returns_false_for_unparseable_latest() {
        assert!(!is_newer("v2.0.0-beta.1", "1.3.0"));
    }
}
