use anyhow::Result;

use crate::utils::colors::{Colors, ICON_ERROR, ICON_REMOVE};
use crate::utils::constants::validate_package_name;
use crate::utils::lock::remove_from_lock;
use crate::utils::packages::uninstall_package;

/// Run `inx remove <pkgname>`.
pub fn run(pkgname: &str) -> Result<()> {
    if !validate_package_name(pkgname) {
        anyhow::bail!(
            "{} Invalid package name '{}'. Use only letters, numbers, and hyphens.",
            ICON_ERROR,
            pkgname
        );
    }

    uninstall_package(pkgname)?;

    // Update lock file
    if let Err(e) = remove_from_lock(pkgname) {
        eprintln!(
            "{} {}",
            Colors::warning(&format!("Lock file warning: {e}")),
            Colors::dim("(package was removed)")
        );
    }

    println!(
        "{} {} {}",
        ICON_REMOVE,
        Colors::success("Removed"),
        Colors::accent(pkgname)
    );

    Ok(())
}
