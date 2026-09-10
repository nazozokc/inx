use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::utils::config::get_inx_dir;

/// Lock file format (version 1).
#[derive(Debug, Serialize, Deserialize)]
pub struct LockFile {
    pub version: u32,

    #[serde(default)]
    pub packages: Vec<LockPackage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockPackage {
    pub name: String,
    pub version: String,
    pub checksum: String,
}

impl LockFile {
    pub fn new() -> Self {
        Self {
            version: 1,
            packages: Vec::new(),
        }
    }
}

impl Default for LockFile {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the lock file path (~/.inx/lock/inx.lock).
fn get_lock_path() -> Result<PathBuf> {
    let lock_dir = get_inx_dir()?.join("lock");
    fs::create_dir_all(&lock_dir)?;
    Ok(lock_dir.join("inx.lock"))
}

/// Load the lock file from disk. Returns a default empty lock if not found.
pub fn load_lock() -> Result<LockFile> {
    let path = get_lock_path()?;
    if !path.exists() {
        return Ok(LockFile::new());
    }
    let data = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read lock file: {}", path.display()))?;
    let lock: LockFile =
        toml::from_str(&data).with_context(|| format!("Failed to parse lock file: {}", path.display()))?;
    Ok(lock)
}

/// Save the lock file to disk.
pub fn save_lock(lock: &LockFile) -> Result<()> {
    let path = get_lock_path()?;
    let data = toml::to_string_pretty(lock).context("Failed to serialize lock file")?;
    fs::write(&path, data)
        .with_context(|| format!("Failed to write lock file: {}", path.display()))?;
    Ok(())
}

/// Compute SHA-256 checksum of a directory.
pub fn compute_checksum(dir: &PathBuf) -> Result<String> {
    let mut hasher = Sha256::new();

    let mut entries: Vec<_> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .collect();
    entries.sort_by_key(|e| e.path().to_path_buf());

    for entry in &entries {
        let data = fs::read(entry.path())
            .with_context(|| format!("Failed to read {}", entry.path().display()))?;
        hasher.update(entry.path().to_string_lossy().as_bytes());
        hasher.update(&data);
    }

    let hash = hasher.finalize();
    Ok(format!("sha256:{:x}", hash))
}

/// Remove a package from the lock file.
pub fn remove_from_lock(name: &str) -> Result<()> {
    let mut lock = load_lock()?;
    let before = lock.packages.len();
    lock.packages.retain(|p| p.name != name);

    if lock.packages.len() == before {
        anyhow::bail!("Package '{}' not found in lock file", name);
    }

    save_lock(&lock)
}

/// Verify installed packages against the lock file.
/// Returns a list of (package_name, status) tuples.
pub fn verify_lock(installed_dir: &PathBuf) -> Result<Vec<(String, VerifyStatus)>> {
    let lock = load_lock()?;
    let mut results = Vec::new();

    for pkg in &lock.packages {
        let pkg_dir = installed_dir.join(&pkg.name);
        if !pkg_dir.exists() {
            results.push((pkg.name.clone(), VerifyStatus::Missing));
            continue;
        }

        match compute_checksum(&pkg_dir) {
            Ok(checksum) => {
                if checksum == pkg.checksum {
                    results.push((pkg.name.clone(), VerifyStatus::Ok));
                } else {
                    results.push((pkg.name.clone(), VerifyStatus::Mismatch));
                }
            }
            Err(_) => {
                results.push((pkg.name.clone(), VerifyStatus::Error));
            }
        }
    }

    Ok(results)
}

/// Rebuild the lock file from the currently installed packages.
/// Scans ~/.inx/packages/*, computes checksums, and writes a fresh lock.
pub fn rebuild_lock() -> Result<()> {
    let installed_dir = crate::utils::config::get_packages_dir()?;
    let mut lock = LockFile::new();

    let entries = match fs::read_dir(&installed_dir) {
        Ok(entries) => entries,
        Err(_) => {
            // No packages dir: write an empty lock.
            return save_lock(&lock);
        }
    };

    for entry in entries.flatten() {
        if !entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
            continue;
        }
        let pkg_dir = entry.path();
        let manifest = crate::utils::manifest::PackageManifest::from_dir(&pkg_dir);
        let name = entry.file_name().to_string_lossy().to_string();

        let (version, checksum) = match (&manifest, compute_checksum(&pkg_dir)) {
            (Ok(m), Ok(csum)) => (m.package.version.clone(), csum),
            (Ok(m), Err(_)) => (m.package.version.clone(), String::from("unknown")),
            (Err(_), Ok(csum)) => (String::from("unknown"), csum),
            (Err(_), Err(_)) => (String::from("unknown"), String::from("unknown")),
        };

        lock.packages.push(LockPackage {
            name,
            version,
            checksum,
        });
    }

    lock.packages.sort_by(|a, b| a.name.cmp(&b.name));
    save_lock(&lock)
}

#[derive(Debug, PartialEq)]
pub enum VerifyStatus {
    Ok,
    Missing,
    Mismatch,
    Error,
}
