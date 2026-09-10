use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};

use crate::utils::colors::{Colors, ICON_ERROR, ICON_INSTALL, ICON_SUCCESS};
use crate::utils::config::get_packages_dir;
use crate::utils::constants::validate_package_name;
use crate::utils::lock::rebuild_lock;
use crate::utils::manifest::PackageManifest;
use crate::utils::packages::{get_available_packages, install_package};
use crate::utils::registry::init_registry;
use crate::utils::resolver::{resolve_batched, ResolvedPackage};
use crate::utils::scripts::run_script;

/// Run `inx install <pkgname> [--force]`.
pub fn run(pkgname: &str, force: bool) -> Result<()> {
    if !validate_package_name(pkgname) {
        anyhow::bail!(
            "{} Invalid package name '{}'. Use only letters, numbers, and hyphens.",
            ICON_ERROR,
            pkgname
        );
    }

    init_registry()?;

    let registry_dir = crate::utils::config::get_registry_dir()?;
    let available = get_available_packages()?;
    if !available.iter().any(|p| p == pkgname) {
        eprintln!(
            "{} Package '{}' not found in registry",
            ICON_ERROR,
            Colors::accent(pkgname)
        );
        println!("Available packages: {}", available.join(", "));
        std::process::exit(1);
    }

    // Resolve the full dependency graph into install batches.
    println!(
        "{} {} {}",
        ICON_INSTALL,
        Colors::bold("Resolving dependencies for"),
        Colors::accent(pkgname)
    );

    let batches = resolve_batched(pkgname, &registry_dir)?;
    let total: usize = batches.iter().map(|b| b.len()).sum();

    println!(
        "{} {} {}",
        Colors::dim("→"),
        Colors::dim(&format!("{total} package(s) total")),
        Colors::dim(&format!("({} parallel batch(es))", batches.len()))
    );

    // Progress bar covering all packages.
    let pb = ProgressBar::new(total as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.cyan} [{elapsed_precise}] {pos}/{len} {msg}",
        )
        .unwrap_or_else(|_| ProgressStyle::default_bar())
        .progress_chars("▓▒░"),
    );

    let mut failures = Vec::new();

    for batch in &batches {
        if batch.len() > 1 {
            // Install siblings in parallel.
            pb.set_message(format!(
                "installing {} package(s) in parallel",
                batch.len()
            ));
            let results = std::thread::scope(|scope| {
                let handles: Vec<_> = batch
                    .iter()
                    .map(|pkg| {
                        let pkg = pkg.clone();
                        scope.spawn(move || {
                            let res = install_single(&pkg, force);
                            (pkg.name.clone(), res)
                        })
                    })
                    .collect();
                handles
                    .into_iter()
                    .map(|h| h.join().unwrap_or_else(|_| panic!("install thread panicked")))
                    .collect::<Vec<_>>()
            });

            for (name, res) in results {
                match res {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("  {} {e}", Colors::error("Failed:"));
                        failures.push(name);
                    }
                }
                pb.inc(1);
            }
        } else {
            let pkg = &batch[0];
            pb.set_message(format!("installing {}", pkg.name));
            if let Err(e) = install_single(pkg, force) {
                eprintln!("  {} {e}", Colors::error("Failed:"));
                failures.push(pkg.name.clone());
            }
            pb.inc(1);
        }
    }

    pb.finish_and_clear();

    if !failures.is_empty() {
        for name in &failures {
            eprintln!("{} Failed to install {name}", ICON_ERROR);
        }
        anyhow::bail!("Installation failed for {} package(s)", failures.len());
    }

    // Rebuild the lock file from the final installed state.
    rebuild_lock()?;

    println!(
        "{} {} {}",
        ICON_SUCCESS,
        Colors::success("Installed"),
        Colors::accent(pkgname)
    );
    Ok(())
}

/// Install a single package: pre-install hook, copy, post-install hook.
/// Returns `Ok(true)` if actually installed, `Ok(false)` if skipped.
fn install_single(pkg: &ResolvedPackage, force: bool) -> Result<bool> {
    let manifest = PackageManifest::from_dir(&pkg.source)
        .with_context(|| format!("Failed to read manifest for '{}'", pkg.name))?;

    // Detect skip early so script hooks don't run for already-installed packages.
    let dest_dir = get_packages_dir()?.join(&pkg.name);
    if dest_dir.exists() && !force {
        // Script hooks skipped; the copy step prints the "skipping" note.
        return install_package(&pkg.name, force);
    }

    // pre-install hook
    if let Some(script) = &manifest.scripts.pre_install {
        run_script(script, &pkg.name, "pre-install", &pkg.source)?;
    }

    // copy
    let installed = install_package(&pkg.name, force)?;

    // post-install hook (runs inside the installed directory)
    if installed {
        if let Some(script) = &manifest.scripts.post_install {
            run_script(script, &pkg.name, "post-install", &dest_dir)?;
        }
    }

    Ok(installed)
}