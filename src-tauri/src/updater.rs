pub fn parse_version(tag: &str) -> Option<(u64, u64, u64)> {
    let s = tag.strip_prefix('v').unwrap_or(tag);
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let major = parts[0].parse::<u64>().ok()?;
    let minor = parts[1].parse::<u64>().ok()?;
    let patch = parts[2].parse::<u64>().ok()?;
    Some((major, minor, patch))
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
}
