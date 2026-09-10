use anyhow::Result;

use crate::utils::colors::{Colors, ICON_ERROR, ICON_SUCCESS, ICON_SYNC};
use crate::utils::constants::{TAG_NAME_REGEX, VERSION_TAG_REGEX, validate_package_name};
use crate::utils::packages::{get_available_packages, list_installed_packages};
use crate::utils::packages::install_package;

use crate::utils::registry::{update_registry, get_tags, checkout_tag};

/// Parse a version string like "v1.0.0" into numeric components.
fn parse_version(v: &str) -> Vec<u32> {
    v.replace('v', "")
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.')
        .collect::<String>()
        .split('.')
        .map(|p| p.parse().unwrap_or(0))
        .collect()
}

/// Compare two version tags (e.g. "v1.0.0", "1.2.3").
/// Only VERSION_TAG_REGEX-matched tags are passed in.
fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let a_parts = parse_version(a);
    let b_parts = parse_version(b);
    let len = a_parts.len().max(b_parts.len());

    for i in 0..len {
        let a_val = a_parts.get(i).copied().unwrap_or(0);
        let b_val = b_parts.get(i).copied().unwrap_or(0);
        match a_val.cmp(&b_val) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }
    std::cmp::Ordering::Equal
}

/// Run `inx upgrade`.
pub fn run() -> Result<()> {
    update_registry()?;

    // Validate tag names before use
    let invalid_tags = get_tags()?;
    for tag in &invalid_tags {
        if !TAG_NAME_REGEX.is_match(tag) || tag.contains("..") || tag.starts_with('-') {
            eprintln!(
                "{} Skipping invalid tag '{}'",
                Colors::warning("Warning:"),
                tag
            );
        }
    }

    let packages = list_installed_packages()?;
    if packages.is_empty() {
        println!("No packages installed");
        return Ok(());
    }

    let tags = get_tags()?;
    if tags.is_empty() {
        println!("No tags found in registry");
        return Ok(());
    }

    let valid_tags: Vec<&String> = tags.iter().filter(|t| VERSION_TAG_REGEX.is_match(t)).collect();
    if valid_tags.is_empty() {
        println!("No valid version tags found in registry");
        return Ok(());
    }

    let latest_tag = valid_tags
        .iter()
        .max_by(|a, b| compare_versions(a, b))
        .unwrap();
    println!(
        "{} {} {}",
        ICON_SYNC,
        Colors::bold("Checking out tag:"),
        Colors::accent(latest_tag)
    );
    checkout_tag(latest_tag)?;

    // Get available packages at the checked-out tag
    let available = get_available_packages()?;
    let installed_names: Vec<String> = packages.iter().map(|p| p.name.clone()).collect();
    let missing: Vec<&String> = installed_names.iter().filter(|n| !available.contains(n)).collect();
    if !missing.is_empty() {
        for name in &missing {
            eprintln!(
                "{} {} not found in tag {} — skipping",
                Colors::warning("Warning:"),
                Colors::accent(name),
                Colors::accent(latest_tag)
            );
        }
    }

    println!(
        "{} {} {}...",
        ICON_SYNC,
        Colors::bold("Upgrading"),
        Colors::dim(&format!("{} package(s)", packages.len()))
    );

    let mut failures = Vec::new();
    for pkg in &packages {
        // Validate package names before install
        if !validate_package_name(&pkg.name) {
            eprintln!("{} Invalid package name '{}' — skipping", Colors::warning("Warning:"), pkg.name);
            continue;
        }

        if let Err(error) = install_package(&pkg.name, true) {
            eprintln!("{}", error);
            failures.push(pkg.name.clone());
        }
    }

    if !failures.is_empty() {
        for name in &failures {
            eprintln!("{} Failed to upgrade {name}", ICON_ERROR);
        }
        println!("Upgrade completed with errors");
        std::process::exit(1);
    }

    // Rebuild the lock file for the new versions.
    crate::utils::lock::rebuild_lock()?;

    println!("{} Upgrade complete", ICON_SUCCESS);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::compare_versions;
    use std::cmp::Ordering;

    #[test]
    fn compare_versions_orders_major() {
        assert_eq!(compare_versions("v2.0.0", "v1.0.0"), Ordering::Greater);
        assert_eq!(compare_versions("v1.0.0", "v2.0.0"), Ordering::Less);
    }

    #[test]
    fn compare_versions_orders_minor() {
        assert_eq!(compare_versions("v1.2.0", "v1.1.0"), Ordering::Greater);
        assert_eq!(compare_versions("v1.1.0", "v1.2.0"), Ordering::Less);
    }

    #[test]
    fn compare_versions_orders_patch() {
        assert_eq!(compare_versions("v1.0.3", "v1.0.2"), Ordering::Greater);
        assert_eq!(compare_versions("1.0.2", "1.0.3"), Ordering::Less);
    }

    #[test]
    fn compare_versions_handles_equal() {
        assert_eq!(compare_versions("v1.0.0", "1.0.0"), Ordering::Equal);
    }

    #[test]
    fn compare_versions_pads_missing_components_with_zero() {
        // Missing components are treated as 0, matching the TypeScript behavior.
        assert_eq!(compare_versions("v1.0", "v1.0.0"), Ordering::Equal);
        assert_eq!(compare_versions("v1.0.1", "v1.0"), Ordering::Greater);
    }
}