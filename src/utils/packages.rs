use anyhow::{Context, Result};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::utils::config::{get_inx_dir, get_packages_dir, get_registry_dir};
use crate::utils::constants::validate_package_name;
use crate::utils::manifest::PackageManifest;
use crate::utils::scripts::run_script;

/// A package installed in the user's inx directory.
#[derive(Debug)]
#[allow(dead_code)] // part of the public API surface, used by future commands
pub struct InstalledPackage {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
}

/// List all installed packages by scanning ~/.inx/packages/*/manifest.
pub fn list_installed_packages() -> Result<Vec<InstalledPackage>> {
    let packages_dir = get_packages_dir()?;
    let mut packages = Vec::new();

    let entries = match fs::read_dir(&packages_dir) {
        Ok(entries) => entries,
        Err(_) => return Ok(packages), // No packages installed yet
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };
        if file_type.is_dir() {
            let dir = entry.path();
            match PackageManifest::from_dir(&dir) {
                Ok(manifest) => packages.push(InstalledPackage {
                    name: manifest.package.name,
                    version: manifest.package.version,
                    path: dir,
                }),
                Err(error) => eprintln!(
                    "{} {}",
                    crate::utils::colors::Colors::warning("Warning:"),
                    error
                ),
            }
        } else if !file_type.is_file() {
            eprintln!(
                "{} Skipping non-regular file '{}'",
                crate::utils::colors::Colors::warning("Warning:"),
                entry.file_name().to_string_lossy()
            );
        }
    }

    Ok(packages)
}

/// List the package names available in the registry's packages/ directory.
pub fn get_available_packages() -> Result<Vec<String>> {
    let packages_dir = get_registry_dir()?.join("packages");
    let mut packages = Vec::new();

    let entries = match fs::read_dir(&packages_dir) {
        Ok(entries) => entries,
        Err(_) => return Ok(packages), // Registry not initialized
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };
        if file_type.is_dir() {
            packages.push(entry.file_name().to_string_lossy().into_owned());
        }
    }

    Ok(packages)
}

/// Recursively verify that every symlink in the tree resolves to a path
/// strictly inside the base directory (prevents directory traversal via symlinks).
fn validate_no_symlinks(dir: &Path) -> io::Result<bool> {
    let resolved_base = fs::canonicalize(dir)?;
    let base_with_sep = {
        let mut p = resolved_base.as_os_str().to_owned();
        p.push(std::path::MAIN_SEPARATOR.to_string());
        PathBuf::from(p)
    };

    let entries = fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let entry_path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_symlink() {
            let resolved = fs::canonicalize(&entry_path)?;
            if !resolved.starts_with(&base_with_sep) {
                return Ok(false);
            }
        } else if file_type.is_dir() {
            if !validate_no_symlinks(&entry_path)? {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

/// Recursively copy a directory tree, preserving structure and permissions.
fn copy_recursively(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let file_type = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());

        if file_type.is_symlink() {
            let target = fs::read_link(&src_path)?;
            #[cfg(unix)]
            std::os::unix::fs::symlink(&target, &dst_path)?;
            #[cfg(windows)]
            {
                if src_path.is_dir() {
                    std::os::windows::fs::symlink_dir(&target, &dst_path)?;
                } else {
                    std::os::windows::fs::symlink_file(&target, &dst_path)?;
                }
            }
        } else if file_type.is_dir() {
            copy_recursively(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Install a package from the registry to ~/.inx/packages/.
///
/// - Validates the package name and presence in the registry.
/// - Verifies no malicious symlinks exist in the source (and again after copy).
/// - When `force` is true, removes an already-installed copy beforehand.
/// - Cleans up the destination on any failure.
///
/// Returns `Ok(true)` if the package was actually installed and
/// `Ok(false)` if it was already installed and skipped.
pub fn install_package(name: &str, force: bool) -> Result<bool> {
    if !validate_package_name(name) {
        anyhow::bail!(
            "Invalid package name '{name}'. Use only letters, numbers, and hyphens."
        );
    }

    let source_dir = get_registry_dir()?.join("packages").join(name);
    let dest_dir = get_packages_dir()?.join(name);

    if !source_dir.exists() {
        anyhow::bail!("Package '{name}' not found in registry");
    }

    if !validate_no_symlinks(&source_dir)
        .context("Failed to validate symlinks")?
    {
        anyhow::bail!("Package '{name}' contains invalid symbolic links");
    }

    fs::create_dir_all(get_packages_dir()?)?;

    if dest_dir.exists() {
        if !force {
            println!(
                "  {} {} {}",
                crate::utils::colors::Colors::dim("→"),
                crate::utils::colors::Colors::accent(name),
                crate::utils::colors::Colors::dim("(already installed, skipping)")
            );
            return Ok(false);
        }
        fs::remove_dir_all(&dest_dir)
            .with_context(|| format!("Failed to remove {}", dest_dir.display()))?;
    }

    match copy_recursively(&source_dir, &dest_dir) {
        Ok(()) => {
            if !validate_no_symlinks(&dest_dir)
                .context("Failed to validate symlinks")?
            {
                let _ = fs::remove_dir_all(&dest_dir);
                anyhow::bail!("Installed '{name}' contains invalid symbolic links");
            }
            println!(
                "  {} {} {} v{}",
                crate::utils::colors::Colors::dim("→"),
                crate::utils::colors::Colors::accent(name),
                crate::utils::colors::Colors::success("installed"),
                crate::utils::colors::Colors::accent(
                    &PackageManifest::from_dir(&source_dir)
                        .map(|m| m.package.version)
                        .unwrap_or_default()
                )
            );
            Ok(true)
        }
        Err(error) => {
            let _ = fs::remove_dir_all(&dest_dir);
            Err(anyhow::anyhow!("Failed to install '{name}': {error}"))
        }
    }
}

/// Remove an installed package.
pub fn uninstall_package(name: &str) -> Result<()> {
    if !validate_package_name(name) {
        anyhow::bail!(
            "Invalid package name '{name}'. Use only letters, numbers, and hyphens."
        );
    }

    let dest_dir = get_packages_dir()?.join(name);

    if !dest_dir.exists() {
        anyhow::bail!("Package '{name}' is not installed");
    }

    // Run pre-uninstall hook
    let manifest = PackageManifest::from_dir(&dest_dir);
    if let Ok(m) = &manifest {
        if let Some(script) = &m.scripts.pre_uninstall {
            run_script(script, name, "pre-uninstall", &dest_dir)?;
        }
    }

    fs::remove_dir_all(&dest_dir)
        .with_context(|| format!("Failed to remove {}", dest_dir.display()))?;

    // Run post-uninstall hook from the cached manifest
    if let Ok(m) = manifest {
        if let Some(script) = &m.scripts.post_uninstall {
            // No dir exists anymore; run in the packages dir
            let parent = get_packages_dir()?;
            run_script(script, name, "post-uninstall", &parent)?;
        }
    }

    Ok(())
}

/// Get the inx directory, mainly for the registry clone.
#[allow(dead_code)]
pub fn get_inx_dir_public() -> Result<PathBuf> {
    get_inx_dir()
}