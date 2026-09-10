use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod utils;

use utils::config::ensure_inx_dirs;

/// A simple package manager for local packages.
#[derive(Parser)]
#[command(name = "inx", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install a package from the registry
    Install {
        /// Package name to install
        pkgname: String,
    },
    /// Update the registry (git pull)
    Update,
    /// Upgrade all installed packages to latest version
    Upgrade,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Mirror the TypeScript preAction hook: ensure directories exist for every command.
    ensure_inx_dirs()?;

    match cli.command {
        Commands::Install { pkgname } => commands::install::run(&pkgname),
        Commands::Update => commands::update::run(),
        Commands::Upgrade => commands::upgrade::run(),
    }
}