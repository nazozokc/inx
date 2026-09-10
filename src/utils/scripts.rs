use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use crate::utils::colors::{Colors, ICON_WARNING};

/// Execute a shell script hook within the given package directory.
/// Returns Ok(()) even if the script fails (non-fatal).
pub fn run_script(script: &str, pkg_name: &str, hook_name: &str, pkg_dir: &Path) -> Result<()> {
    println!(
        "  {} {} {}...",
        Colors::dim("[script]"),
        Colors::accent(hook_name),
        Colors::dim(pkg_name)
    );

    let output = Command::new("sh")
        .arg("-c")
        .arg(script)
        .current_dir(pkg_dir)
        .output()
        .with_context(|| format!("Failed to execute {hook_name} script for '{pkg_name}'"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!(
            "{} {} {} returned non-zero exit code: {}",
            ICON_WARNING,
            Colors::warning("Warning:"),
            hook_name,
            stderr.trim()
        );
    }

    Ok(())
}
