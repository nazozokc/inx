# Changelog

## v2.0.0 (2026-09-10)

### ✨ Features

- Add `inx list` — show installed packages with versions
- Add `inx remove` — uninstall a package (alias: `uninstall`, `rm`)
- Add `inx info` — show package details (manifest, deps, scripts)
- Add `inx search` — GitHub API repository search
- Add `inx verify` — checksum verification against the lock file
- Add `inx.toml` manifest format with `[package]`, `[dependencies]`, `[scripts]`
- Automatic dependency resolution (topological order, cycle detection)
- Parallel installation of independent sibling packages
- Pre/post install & uninstall script hooks
- Lock file with SHA-256 checksums (`~/.inx/lock/inx.lock`)
- Retry with exponential backoff for network operations (clone/fetch/API)
- Progress bar during installation
- Colored terminal output with icons
- `--force` flag for `inx install`
- Backward compatibility with legacy `package.json` manifests

### 🐛 Bug Fixes

- Checkout annotated git tags correctly during `upgrade` (peel to commit)
- Skip script hooks when a package is already installed

## v1.0.0 (2026-03-26)

### ✨ Features

- Add English README ([8bceba1](https://github.com/nazozokc/inx/commit/8bceba1))

### 🐛 Bug Fixes

- Change shebang to node and chmod dist/index.js after build ([822eb14](https://github.com/nazozokc/inx/commit/822eb14))
- Force reinstall on upgrade to bypass already-installed guard ([ef56e28](https://github.com/nazozokc/inx/commit/ef56e28))
- Use registryDir as simpleGit base in initRegistry ([1913c29](https://github.com/nazozokc/inx/commit/1913c29))

### 📦 Full Changelog

**Full Changelog**: https://github.com/nazozokc/inx/commits/v1.0.0