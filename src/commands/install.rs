use anyhow::Result;

use crate::utils::constants::validate_package_name;
use crate::utils::packages::{get_available_packages, install_package};
use crate::utils::registry::init_registry;

/// Run `ox install <pkgname>`.
pub fn run(pkgname: &str) -> Result<()> {
    if !validate_package_name(pkgname) {
        anyhow::bail!(
            "Invalid package name '{pkgname}'. Use only letters, numbers, and hyphens."
        );
    }

    init_registry()?;
    let available = get_available_packages()?;
    if !available.iter().any(|p| p == pkgname) {
        eprintln!("Package '{pkgname}' not found in registry");
        println!("Available packages: {}", available.join(", "));
        std::process::exit(1);
    }
    install_package(pkgname, false)
}