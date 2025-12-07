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
├──────────┬──────────┬──────────┬────────────┤
│   apt    │  pacman  │   brew   │   winget   │
│ adapter  │ adapter  │  adapter │  adapter   │
└──────────┴──────────┴──────────┴────────────┘
```

## Status

🚧 **Early Development** - This project is in the design and proof-of-concept phase.

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
```

### Configuration

Initialize a configuration file:

```bash
pacmagic init-config
```

Edit `~/.config/pacmagic/config.toml` (Linux) or `~/Library/Application Support/pacmagic/config.toml` (macOS) to customize behavior.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run clippy
cargo clippy --all-targets --all-features

# Format code
cargo fmt
```

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


