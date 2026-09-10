use regex::Regex;
use std::sync::LazyLock;

/// Default registry URL
pub const DEFAULT_REGISTRY: &str = "https://github.com/nazozokc/inx";

/// Default GitHub org used for package search
pub const DEFAULT_REGISTRY_ORG: &str = "nazozokc";

/// Maximum length of a package name
pub const PACKAGE_NAME_MAX_LENGTH: usize = 214;

/// Regex for validating package names.
/// Must start with alphanumeric, can contain alphanumeric and hyphens.
pub static PACKAGE_NAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9][a-zA-Z0-9-]*$").unwrap());

/// Regex for validating tag names.
/// Only alphanumeric, dots, underscores, and hyphens.
pub static TAG_NAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9._-]+$").unwrap());

/// Regex for validating version tags (e.g., v1.0.0, 1.0.0).
/// Only VERSION_TAG_REGEX-matched tags are passed to version comparison.
/// Pre-release identifiers (e.g. v1.0.0-rc.1) are excluded upstream.
pub static VERSION_TAG_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^v?\d+\.\d+\.\d+$").unwrap());

/// Validate a package name.
pub fn validate_package_name(name: &str) -> bool {
    PACKAGE_NAME_REGEX.is_match(name) && name.len() <= PACKAGE_NAME_MAX_LENGTH
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_name_regex_matches_valid_names() {
        assert!(PACKAGE_NAME_REGEX.is_match("foo"));
        assert!(PACKAGE_NAME_REGEX.is_match("foo-bar"));
        assert!(PACKAGE_NAME_REGEX.is_match("test123"));
    }

    #[test]
    fn package_name_regex_rejects_invalid_names() {
        assert!(!PACKAGE_NAME_REGEX.is_match("-foo"));
        assert!(!PACKAGE_NAME_REGEX.is_match("foo_bar"));
        assert!(!PACKAGE_NAME_REGEX.is_match(""));
    }

    #[test]
    fn tag_name_regex_matches_valid_tags() {
        assert!(TAG_NAME_REGEX.is_match("v1.0.0"));
        assert!(TAG_NAME_REGEX.is_match("1.0.0"));
        assert!(TAG_NAME_REGEX.is_match("release-1.0"));
        assert!(TAG_NAME_REGEX.is_match("test_tag"));
    }

    #[test]
    fn tag_name_regex_allows_double_dot() {
        // Regex itself allows `..`; validation is done in validate_tag_name.
        assert!(TAG_NAME_REGEX.is_match("v1..0.0"));
    }

    #[test]
    fn version_tag_regex_matches_valid_versions() {
        assert!(VERSION_TAG_REGEX.is_match("v1.0.0"));
        assert!(VERSION_TAG_REGEX.is_match("1.0.0"));
        assert!(VERSION_TAG_REGEX.is_match("v0.1.0"));
        assert!(VERSION_TAG_REGEX.is_match("10.20.30"));
    }

    #[test]
    fn version_tag_regex_rejects_invalid_versions() {
        assert!(!VERSION_TAG_REGEX.is_match("1.0"));
        assert!(!VERSION_TAG_REGEX.is_match("v1.0"));
        assert!(!VERSION_TAG_REGEX.is_match("release-1.0.0"));
        assert!(!VERSION_TAG_REGEX.is_match("1.0.0-beta"));
    }

    #[test]
    fn validate_package_name_accepts_valid_names() {
        assert!(validate_package_name("foo"));
        assert!(validate_package_name("foo-bar"));
        assert!(validate_package_name("test123"));
        assert!(validate_package_name("a"));
    }

    #[test]
    fn validate_package_name_rejects_hyphen_prefix() {
        assert!(!validate_package_name("-foo"));
    }

    #[test]
    fn validate_package_name_rejects_invalid_characters() {
        assert!(!validate_package_name("foo_bar"));
        assert!(!validate_package_name("foo.bar"));
        assert!(!validate_package_name("foo bar"));
    }

    #[test]
    fn validate_package_name_rejects_too_long_names() {
        let long_name = "a".repeat(215);
        assert!(!validate_package_name(&long_name));
    }

    #[test]
    fn validate_package_name_rejects_empty_string() {
        assert!(!validate_package_name(""));
    }
}