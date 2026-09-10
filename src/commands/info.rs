use anyhow::{Context, Result};

use crate::utils::colors::{Colors, ICON_ERROR};
use crate::utils::config::{get_inx_dir, get_packages_dir};
use crate::utils::constants::validate_package_name;
use crate::utils::manifest::PackageManifest;

/// Run `inx info <pkgname>`.
/// Shows info for an installed package or a registry package.
pub fn run(pkgname: &str) -> Result<()> {
    if !validate_package_name(pkgname) {
        anyhow::bail!(
            "{} Invalid package name '{}'. Use only letters, numbers, and hyphens.",
            ICON_ERROR,
            pkgname
        );
    }

    // Try installed first
    let installed_dir = get_packages_dir()?.join(pkgname);
    if installed_dir.exists() {
        show_info(pkgname, &installed_dir, true)?;
        return Ok(());
    }

    // Try registry
    let registry_dir = get_inx_dir()?.join("registry").join("packages").join(pkgname);
    if registry_dir.exists() {
        show_info(pkgname, &registry_dir, false)?;
        return Ok(());
    }

    anyhow::bail!(
        "{} Package '{}' not found (not installed and not in registry)",
        ICON_ERROR,
        pkgname
    );
}

fn show_info(name: &str, dir: &std::path::Path, installed: bool) -> Result<()> {
    let manifest = PackageManifest::from_dir(dir)
        .with_context(|| format!("Failed to read package manifest for '{name}'"))?;

    println!("{}", Colors::bold(&format!("Package: {name}")));
    println!();

    if installed {
        println!(
            "  {} {}",
            Colors::dim("Status:"),
            Colors::success("installed")
        );
    } else {
        println!(
            "  {} {}",
            Colors::dim("Status:"),
            Colors::dim("available (not installed)")
        );
    }

    println!(
        "  {} {}",
        Colors::dim("Version:"),
        Colors::accent(&manifest.package.version)
    );

    if let Some(desc) = &manifest.package.description {
        println!("  {} {desc}", Colors::dim("Description:"));
    }

    if let Some(author) = &manifest.package.author {
        println!("  {} {author}", Colors::dim("Author:"));
    }

    if let Some(license) = &manifest.package.license {
        println!("  {} {license}", Colors::dim("License:"));
    }

    if let Some(tags) = &manifest.package.tags {
        if !tags.is_empty() {
            println!(
                "  {} {}",
                Colors::dim("Tags:"),
                tags.join(", ")
            );
        }
    }

    // Dependencies
    if !manifest.dependencies.is_empty() {
        println!();
        println!("  {}", Colors::bold("Dependencies:"));
        for (dep, constraint) in &manifest.dependencies {
            println!(
                "    {} {} {}",
                Colors::accent(dep),
                Colors::dim(constraint),
                ""
            );
        }
    }

    // Scripts
    if manifest.scripts.has_any() {
        println!();
        println!("  {}", Colors::bold("Scripts:"));
        if let Some(s) = &manifest.scripts.pre_install {
            println!("    {} {}", Colors::dim("pre-install:"), s);
        }
        if let Some(s) = &manifest.scripts.post_install {
            println!("    {} {}", Colors::dim("post-install:"), s);
        }
        if let Some(s) = &manifest.scripts.pre_uninstall {
            println!("    {} {}", Colors::dim("pre-uninstall:"), s);
        }
        if let Some(s) = &manifest.scripts.post_uninstall {
            println!("    {} {}", Colors::dim("post-uninstall:"), s);
        }
    }

    println!();
    println!(
        "  {} {}",
        Colors::dim("Path:"),
        dir.display()
    );

    Ok(())
}
