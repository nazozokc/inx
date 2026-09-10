use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod utils;

use utils::config::ensure_inx_dirs;
use utils::constants::DEFAULT_REGISTRY_ORG;

/// A simple package manager for local packages.
#[derive(Parser)]
#[command(name = "inx", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install a package and its dependencies
    Install {
        /// Package name to install
        pkgname: String,

        /// Force reinstall even if already installed
        #[arg(long, short = 'f')]
        force: bool,
    },
    /// Update the registry (git pull)
    Update,
    /// Upgrade all installed packages to latest version
    Upgrade,
    /// List installed packages
    List {
        /// Show full paths
        #[arg(long, short = 'v')]
        verbose: bool,
    },
    /// Remove an installed package
    #[command(alias = "uninstall", alias = "rm")]
    Remove {
        /// Package name to remove
        pkgname: String,
    },
    /// Show package information
    Info {
        /// Package name
        pkgname: String,
    },
    /// Search for packages on GitHub
    Search {
        /// Search query
        query: String,
    },
    /// Verify installed packages against the lock file
    Verify,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Ensure directories exist for every command.
    ensure_inx_dirs()?;

    match cli.command {
        Commands::Install { pkgname, force } => commands::install::run(&pkgname, force),
        Commands::Update => commands::update::run(),
        Commands::Upgrade => commands::upgrade::run(),
        Commands::List { verbose } => commands::list::run(verbose),
        Commands::Remove { pkgname } => commands::remove::run(&pkgname),
        Commands::Info { pkgname } => commands::info::run(&pkgname),
        Commands::Search { query } => commands::search::run(&query, DEFAULT_REGISTRY_ORG),
        Commands::Verify => commands::verify::run(),
    }
}