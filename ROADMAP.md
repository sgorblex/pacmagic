# PacMagic Roadmap

## Current Status: v0.1.0 (early development)

Completed:
- Core trait-based architecture (`PackageManager`, `Operation`, `OperationOptions`)
- Five package manager adapters: apt, pacman, paru, brew, winget
- TOML configuration system with custom aliases (single and multi-command)
- CLI interface (clap) with dynamic shell completions (bash, fish, zsh)
- Platform detection (OS, Linux distro, rolling-release detection)
- Package-name/search-query validation (injection prevention)
- Pre-update/upgrade policy for installs (`InstallPreUpdateBehavior`)

## Next up: design & performance refactor

The near-term priority is a refactor focused on **design and performance**, building on the "Current Characteristics" baseline documented in `ARCHITECTURE.md` (synchronous, one subprocess per operation, no caching, hardcoded adapter registry). Scope and specifics to be defined separately.

## Near-term, concretely scoped

Following the existing adapter pattern documented in `CONTRIBUTING.md`:

- [ ] DNF adapter (Fedora, RHEL)
- [ ] Zypper adapter (openSUSE)
- [ ] APK adapter (Alpine Linux)
- [ ] PPA support in the APT adapter (`PackageManagerExt::add_repository`/`remove_repository`)

## Contributing to the Roadmap

Have ideas? Open an issue with the `enhancement` label, or discuss in GitHub Discussions.
