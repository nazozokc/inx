use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::utils::config::{ensure_inx_dirs, get_packages_dir, get_registry_dir};
use crate::utils::constants::validate_package_name;

/// A package installed in the user's inx directory.
#[derive(Debug)]
#[allow(dead_code)] // part of the public API surface, used by future commands
pub struct InstalledPackage {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
}

/// Minimal package.json structure needed by inx.
#[derive(Debug, Deserialize)]
struct PackageJson {
    name: String,
    version: String,
}

impl PackageJson {
    fn from_dir(dir: &Path) -> Result<Self> {
        let pkg_json_path = dir.join("package.json");
        let data = fs::read_to_string(&pkg_json_path)
            .with_context(|| format!("Failed to read {}", pkg_json_path.display()))?;
        let pkg: PackageJson = serde_json::from_str(&data)
            .with_context(|| format!("Failed to parse {}", pkg_json_path.display()))?;
        if pkg.name.is_empty() || pkg.version.is_empty() {
            anyhow::bail!(
                "Invalid package.json in '{}': missing name or version",
                dir.display()
            );
        }
        Ok(pkg)
    }
}

/// List all installed packages by scanning ~/.inx/packages/*/package.json.
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
            match PackageJson::from_dir(&dir) {
                Ok(pkg) => packages.push(InstalledPackage {
                    name: pkg.name,
                    version: pkg.version,
                    path: dir,
                }),
                Err(error) => eprintln!("Warning: {}", error),
            }
        } else if !file_type.is_file() {
            eprintln!("Warning: Skipping non-regular file '{}'", entry.file_name().to_string_lossy());
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
pub fn install_package(name: &str, force: bool) -> Result<()> {
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

    ensure_inx_dirs()?;

    if dest_dir.exists() {
        if !force {
            println!("Package '{name}' is already installed");
            return Ok(());
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
            println!("Installed '{name}'");
            Ok(())
        }
        Err(error) => {
            let _ = fs::remove_dir_all(&dest_dir);
            Err(anyhow::anyhow!("Failed to install '{name}': {error}"))
        }
    }
}

/// Remove an installed package.
#[allow(dead_code)] // no CLI command yet, ported from the TypeScript API surface
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

    fs::remove_dir_all(&dest_dir)
        .with_context(|| format!("Failed to remove {}", dest_dir.display()))?;
    println!("Uninstalled '{name}'");
    Ok(())
}