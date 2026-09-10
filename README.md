# ox

A simple package manager for local packages.

## Installation

```bash
cargo build --release
```

The binary is generated at `target/release/ox`. To use it as `ox`, add it to your `PATH`:

```bash
cp target/release/ox ~/.local/bin/ox
```

## Usage

```bash
# Install a package
ox install <pkgname>

# Update packages
ox update

# Upgrade packages
ox upgrade
```

## Storage Location

- **Unix:** `~/.ox`
- **Windows:** `C:\Users\<username>\.ox`

Packages are fetched from the remote repository at [https://github.com/nazozokc/ox/tree/main/packages](https://github.com/nazozokc/ox/tree/main/packages).

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

The registry URL can be changed via `~/.ox/config.json`:

```json
{
  "registry": "https://github.com/yourname/yourrepo"
}
```

Only `https:` and `ssh:` protocols are allowed.