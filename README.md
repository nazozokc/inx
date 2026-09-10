# inx

A simple package manager for local packages.

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
# Install a package
inx install <pkgname>

# Update packages
inx update

# Upgrade packages
inx upgrade
```

## Storage Location

- **Unix:** `~/.inx`
- **Windows:** `C:\Users\<username>\.inx`

Packages are fetched from the remote repository at [https://github.com/nazozokc/inx/tree/main/packages](https://github.com/nazozokc/inx/tree/main/packages).

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

## Configuration

The registry URL can be changed via `~/.inx/config.json`:

```json
{
  "registry": "https://github.com/yourname/yourrepo"
}
```

Only `https:` and `ssh:` protocols are allowed.