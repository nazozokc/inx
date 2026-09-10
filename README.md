# inx

A simple, modern package manager for local packages.

## Installation

```bash
cargo build --release
```

The binary is generated at `target/release/inx`. To use it as `inx`, add it to your `PATH`:

```bash
cp target/release/inx ~/.local/bin/inx
```

## Usage

```bash
# Install a package and its dependencies
inx install <pkgname>

# Force reinstall even if already installed
inx install <pkgname> --force

# Update the registry (git pull)
inx update

# Upgrade all packages to the latest version tag
inx upgrade

# List installed packages
inx list

# Show package details
inx info <pkgname>

# Remove a package
inx remove <pkgname>

# Search for packages on GitHub
inx search <query>

# Verify installed packages against the lock file
inx verify
```

## Storage Location

- **Unix:** `~/.inx`
- **Windows:** `C:\Users\<username>\.inx`

Packages are fetched from the remote repository at [https://github.com/nazozokc/inx/tree/main/packages](https://github.com/nazozokc/inx/tree/main/packages).

## Package Manifest (inx.toml)

Each package is a directory under `packages/` containing an `inx.toml`:

```toml
[package]
name = "my-package"
version = "1.0.0"
description = "A cool package"
author = "nazozokc"
license = "MIT"
tags = ["cli", "tool"]

[dependencies]
some-dep = "1.0.0"
another-dep = "^2.0.0"

[scripts]
pre-install = "echo preparing"
post-install = "echo done"
pre-uninstall = "echo removing"
post-uninstall = "echo cleaned"
```

- **Dependencies** are resolved automatically (topological order, parallel batches).
- Circular dependencies are detected and rejected.
- **Script hooks** run in the package directory; failures are warnings, not errors.
- Legacy `package.json` files are still supported (backward compatible).

## Lock File

Installed state is recorded in `~/.inx/lock/inx.lock` with SHA-256 checksums.
`inx verify` compares the lock against `~/.inx/packages/` and reports missing,
modified, or extra packages.

## Search

`inx search` queries the GitHub API for repositories in the `nazozokc` org.
Set `GITHUB_TOKEN` to raise the API rate limit.

## Configuration

The registry URL can be changed via `~/.inx/config.json`:

```json
{
  "registry": "https://github.com/yourname/yourrepo"
}
```

Only `https:` and `ssh:` protocols are allowed.

## Development

```bash
# Build
cargo build

# Run in development mode
cargo run

# Test
cargo test
```

## Tech Stack

- Rust
- clap (CLI)
- git2 (Git operations)
- toml (manifest parsing)
- sha2 (checksums)
- indicatif (progress bars)