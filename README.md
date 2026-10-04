# PacMagic 🪄

A universal, extensible package manager wrapper written in Rust.

## Vision

PacMagic aims to provide a unified interface for package management across all platforms and package managers, with:

- **Universal Support**: Works with brew, apt, pacman, dnf, winget, pkg, apk, and more
- **Extensible Architecture**: Easy to add new package managers via trait-based plugins
- **Flexible Paradigms**: Supports different package management paradigms (AUR, PPAs, dist-updates, etc.)
- **Customizable**: Configure your own command aliases and workflows
- **Clean API**: Use as a library or CLI tool

## Architecture

PacMagic uses a trait-based plugin architecture:

```
┌─────────────────────────────────────────────┐
│           CLI / API Interface               │
├─────────────────────────────────────────────┤
│         Command Router & Config             │
├─────────────────────────────────────────────┤
│         Core Traits & Abstractions          │
├──────────┬──────────┬──────────┬──────────┬────────────┤
│   apt    │  pacman  │   paru   │   brew   │   winget   │
│ adapter  │ adapter  │ adapter  │  adapter │  adapter   │
└──────────┴──────────┴──────────┴──────────┴────────────┘
```

## Status

🚧 **Early Development** - This project is in the design and proof-of-concept phase.

### Supported Package Managers (current)

- **APT** (Debian/Ubuntu)
- **Pacman** (Arch Linux)
- **Paru** (Arch Linux AUR helper)
- **Homebrew** (macOS)
- **Winget** (Windows)

## Design Principles

1. **Trait-Based**: Package managers implement core traits for operations
2. **Plugin System**: New package managers are plugins, not core modifications
3. **Type-Safe**: Leverage Rust's type system for correctness
4. **Zero-Copy Where Possible**: Minimize allocations and copies
5. **Error Transparency**: Clear error messages that help users understand what went wrong

## Quick Start

### Installation

```bash
cargo install --path .
```

### Usage

```bash
# Install packages
pacmagic install nginx

# Search for packages
pacmagic search python

# Update package database
pacmagic update

# Upgrade all packages
pacmagic upgrade

# Remove packages
pacmagic remove nginx

# List installed packages
pacmagic list

# Show package info
pacmagic info nginx

# Only official repositories, or only user repositories (e.g. the AUR)
pacmagic install --user paru-bin
pacmagic search --official python
pacmagic list --user
```

`--official`/`--user` apply to `install`, `upgrade`, `search`, `list` and `info`, and are ignored by other commands. Support depends on the package manager: paru supports both everywhere; pacman supports `--official` everywhere and `--user` only for `list` (foreign packages). apt, brew and winget don't support them yet.

### Configuration

Initialize a configuration file:

```bash
pacmagic init-config
```

Edit `~/.config/pacmagic/config.toml` (Linux) or `~/Library/Application Support/pacmagic/config.toml` (macOS) to customize behavior.

### Shell Completions

PacMagic supports **dynamic shell completions** for bash, fish, and zsh. The completions are context-aware and include:

- **Commands and aliases**: All commands plus any custom aliases from your config
- **Package manager names**: Dynamic completion for `-p/--package-manager` flag
- **Package names**: Dynamic completion for install/remove/upgrade/info commands (with 500ms timeout)
- **Flags and options**: Planned (not yet implemented)

The completions automatically detect your configuration and available package managers, making them fast and accurate.

#### Bash

```bash
# Generate and save completions
pacmagic completions bash > ~/.local/share/bash-completion/completions/pacmagic

# Or for system-wide installation (requires sudo)
sudo pacmagic completions bash > /usr/share/bash-completion/completions/pacmagic
```

If your system uses a different location for bash completions, you may need to adjust the path. Common locations include:
- `~/.bash_completion.d/pacmagic`
- `/etc/bash_completion.d/pacmagic`

After installation, reload your shell or source the completion file:
```bash
source ~/.local/share/bash-completion/completions/pacmagic
```

#### Fish

```fish
# Generate and save completions
pacmagic completions fish > ~/.config/fish/completions/pacmagic.fish
```

After installation, restart your fish shell or run:
```fish
source ~/.config/fish/completions/pacmagic.fish
```

#### Zsh

```zsh
# Generate and save completions
pacmagic completions zsh > ~/.zsh/completions/_pacmagic

# Ensure the completions directory is in your fpath
# Add this to your ~/.zshrc if not already present:
# fpath=(~/.zsh/completions $fpath)

# Then reload completions
autoload -U compinit && compinit
```

Alternatively, if you use Oh My Zsh:
```zsh
pacmagic completions zsh > ~/.oh-my-zsh/completions/_pacmagic
```

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for the development setup, testing, and PR process, and [ARCHITECTURE.md](ARCHITECTURE.md) for how the codebase is put together.

## Comparison with Existing Tools

| Feature | PacMagic | pacaptr | upt | upm_lib |
|---------|----------|---------|-----|---------|
| Extensible Plugin System | ✅ | ⚠️ | ⚠️ | ⚠️ |
| Custom Aliases | ✅ | ❌ | ❌ | ❌ |
| Library + CLI | ✅ | ❌ | ⚠️ | ✅ |
| AUR/PPA Support | ✅ (planned) | ⚠️ | ❌ | ❌ |
| Clean API | ✅ | ❌ | ⚠️ | ✅ |

## License

MIT
