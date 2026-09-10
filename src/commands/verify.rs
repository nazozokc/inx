use anyhow::Result;

use crate::utils::colors::{Colors, ICON_CHECK, ICON_ERROR, ICON_WARNING};
use crate::utils::config::get_packages_dir;
use crate::utils::lock::{verify_lock, VerifyStatus};

/// Run `inx verify`.
pub fn run() -> Result<()> {
    let installed_dir = get_packages_dir()?;
    let results = verify_lock(&installed_dir)?;

    if results.is_empty() {
        println!(
            "{} No packages in lock file. Nothing to verify.",
            ICON_WARNING
        );
        return Ok(());
    }

    let mut ok_count = 0;
    let mut warn_count = 0;

    for (name, status) in &results {
        match status {
            VerifyStatus::Ok => {
                ok_count += 1;
                println!(
                    "  {} {} {}",
                    ICON_CHECK,
                    Colors::accent(name),
                    Colors::success("verified")
                );
            }
            VerifyStatus::Missing => {
                warn_count += 1;
                eprintln!(
                    "  {} {} {}",
                    ICON_WARNING,
                    Colors::accent(name),
                    Colors::warning("missing (in lock but not installed)")
                );
            }
            VerifyStatus::Mismatch => {
                warn_count += 1;
                eprintln!(
                    "  {} {} {}",
                    ICON_ERROR,
                    Colors::accent(name),
                    Colors::error("checksum mismatch (modified?)")
                );
            }
            VerifyStatus::Error => {
                warn_count += 1;
                eprintln!(
                    "  {} {} {}",
                    ICON_ERROR,
                    Colors::accent(name),
                    Colors::error("failed to compute checksum")
                );
            }
        }
    }

    println!();
    println!(
        "{} {}",
        Colors::bold("Verification complete:"),
        Colors::dim(&format!("{ok_count} OK, {warn_count} warnings"))
    );

    if warn_count > 0 {
        std::process::exit(1);
    }

    Ok(())
}
