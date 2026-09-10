use anyhow::Result;

use crate::utils::colors::{Colors, ICON_INFO};
use crate::utils::packages::list_installed_packages;

/// Run `inx list`.
pub fn run(verbose: bool) -> Result<()> {
    let packages = list_installed_packages()?;

    if packages.is_empty() {
        println!("{} No packages installed", ICON_INFO);
        return Ok(());
    }

    println!(
        "{} {}",
        Colors::bold("Installed packages:"),
        Colors::dim(&format!("({})", packages.len()))
    );
    println!();

    for pkg in &packages {
        if verbose {
            println!(
                "  {} {} {} {}",
                Colors::accent(&pkg.name),
                Colors::dim("@"),
                Colors::success(&pkg.version),
                Colors::dim(&format!("({})", pkg.path.display()))
            );
        } else {
            println!(
                "  {} {} {}",
                Colors::accent(&pkg.name),
                Colors::dim("@"),
                Colors::success(&pkg.version)
            );
        }
    }

    Ok(())
}
