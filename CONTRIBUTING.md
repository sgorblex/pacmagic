# Contributing to PacMagic

Thank you for your interest in contributing to PacMagic! This document covers everything you need to set up, develop, test, and submit changes.

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow

## Development Setup

### Prerequisites

- Rust **1.85** or later (**MSRV**, see `rust-version` in `Cargo.toml`).
- A supported package manager (apt, pacman, paru, brew, winget) if you want to exercise real (non-dry-run) operations.

Optional but recommended: [mise](https://mise.jdx.dev/) reads `mise.toml` and installs the repo-pinned Rust version, plus `cargo-deny` and `cargo-machete` (needed for `mise run check-all`):

```bash
mise install
```

If you skip mise, use any rustup toolchain meeting the MSRV above, run `rustup component add rustfmt clippy` if `cargo fmt`/`cargo clippy` are missing, and install `cargo-deny`/`cargo-machete` yourself if you want to run `mise run check-all`'s full check bundle.

### Building

```bash
git clone https://github.com/YOUR_USERNAME/pacmagic.git
cd pacmagic
cargo build --locked          # or: mise run build (release profile)
cargo run --locked -- --help
```

### Project Structure

```
pacmagic/
├── src/
│   ├── adapters/           # One file per package manager
│   │   ├── mod.rs          # AdapterRegistry: built-in adapter list, auto-detection
│   │   ├── apt.rs          # Debian/Ubuntu APT
│   │   ├── pacman.rs       # Arch Linux pacman
│   │   ├── paru.rs         # Arch Linux AUR helper (paru)
│   │   ├── brew.rs         # macOS Homebrew
│   │   └── winget.rs       # Windows Package Manager
│   ├── config/mod.rs       # TOML config, command aliases, per-PM overrides
│   ├── core/
│   │   ├── traits.rs       # The PackageManager trait every adapter implements
│   │   ├── operation.rs    # Operation enum, OperationOptions
│   │   ├── package.rs      # Package type
│   │   ├── error.rs        # PacMagicError
│   │   ├── executor.rs     # CommandExecutor: dry-run/verbose-aware subprocess wrapper
│   │   └── validation.rs   # Package name / search query validation (injection prevention)
│   ├── platform/mod.rs     # OS/distro detection
│   ├── command_names.rs    # Known CLI command names, used when parsing pass-through args
│   ├── completion.rs       # Dynamic bash/fish/zsh shell completions
│   ├── lib.rs              # Library entry point (the `PacMagic` struct)
│   └── main.rs             # CLI entry point (clap)
├── Cargo.toml
├── ARCHITECTURE.md         # How the pieces fit together, and why
├── ROADMAP.md
└── CHANGELOG.md
```

### Quick Checks

```bash
cargo check --all-targets --all-features --locked   # fast syntax/type check
cargo test --all-targets --all-features --locked    # run tests
cargo run --locked -- detect                         # show detected package manager
cargo run --locked -- install --dry-run neovim        # dry run, no side effects
```

## How to Contribute

### Reporting Bugs

1. Check if the bug has already been reported
2. Use the bug report template
3. Include:
   - Your OS and version
   - Package manager(s) you're using
   - Steps to reproduce
   - Expected vs actual behavior
   - Relevant logs (use `--verbose`)

### Suggesting Features

1. Check if the feature has been suggested
2. Describe the use case
3. Explain how it fits PacMagic's goals
4. Consider backward compatibility

### Adding a Package Manager

1. **Research** the package manager's CLI interface.
2. **Create** `src/adapters/<name>.rs`, implementing the `PackageManager` trait (see `src/core/traits.rs` for the full method list; `src/adapters/apt.rs` is a good reference implementation).
3. **Register** it in `AdapterRegistry::new()` in `src/adapters/mod.rs`.
4. **Set `priority()`** so auto-detection prefers the right adapter when multiple are available (see `ARCHITECTURE.md` for how APT/pacman/paru resolve this today).
5. **Test** thoroughly with different operations, and add unit tests inline in the same file (see the `#[cfg(test)] mod tests` block at the bottom of any existing adapter for the pattern).
6. **Document** any quirks or special behavior, and update the supported-package-managers list in `README.md`.

Example skeleton, following the same shape as `src/adapters/apt.rs`:

```rust
// src/adapters/zypper.rs
use crate::core::{CommandExecutor, Operation, OperationOptions, Package, PackageManager, Result};
use std::process::Command;

pub struct Zypper;

impl Zypper {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Zypper {
    fn default() -> Self {
        Self::new()
    }
}

impl PackageManager for Zypper {
    fn name(&self) -> &str {
        "zypper"
    }

    fn command_names(&self) -> &[&str] {
        &["zypper"]
    }

    fn is_available(&self) -> bool {
        which::which("zypper").is_ok()
    }

    fn priority(&self) -> u8 {
        50
    }

    fn supports_operation(&self, operation: Operation) -> bool {
        matches!(
            operation,
            Operation::Install | Operation::Remove | Operation::Update | Operation::Search
        )
    }

    fn install(&self, packages: &[&str], options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);
        let mut cmd = Command::new("zypper");
        cmd.arg("install").args(packages);
        if options.no_confirm {
            cmd.arg("-y");
        }
        executor.execute_checked(cmd)
    }

    // ...implement the rest of PackageManager: remove, update, upgrade, upgrade_all,
    // search, list_installed, info, clean, files, check, autoremove, owns.
}
```

#### Package Manager Checklist

- [ ] Basic operations (install, remove, update)
- [ ] Search functionality
- [ ] List installed packages
- [ ] Package information
- [ ] Files listing (if supported)
- [ ] Clean cache
- [ ] Tests for parsing output
- [ ] Documentation in README
- [ ] Priority setting for auto-detection

### Code Style

We follow standard Rust conventions:

```bash
# Apply formatting (writes files)
cargo fmt --all

# Verify formatting without modifying files
cargo fmt --all -- --check

# Linter
cargo clippy --all-targets --all-features --locked -- -D warnings

# Fix common clippy issues
cargo clippy --all-targets --all-features --locked --fix
```

**Guidelines:**
- Use descriptive variable names
- Add comments for complex logic
- Keep functions focused and small
- Prefer immutability
- Use `?` for error propagation
- Add doc comments for public APIs

### Testing

All new code should include tests. Every module keeps its unit tests inline, in a `#[cfg(test)] mod tests` block at the bottom of the same file — there is no separate `tests/` directory.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Test code here
    }
}
```

```bash
cargo test --all-targets --all-features --locked                  # run everything
cargo test --all-targets --all-features --locked adapters::apt    # scope to a module path
cargo test --all-targets --all-features --locked -- --nocapture   # show test output
```

Since PacMagic shells out to real system package managers, manual testing matters too:

```bash
cargo run --locked -- install --dry-run neovim   # dry run, no side effects
cargo run --locked -- search -v python            # verbose
cargo run --locked -- detect                      # show detected package manager
```

### Documentation

- Add doc comments to public functions:
  ```rust
  /// Installs packages using the detected package manager.
  ///
  /// # Arguments
  ///
  /// * `packages` - Slice of package names to install
  /// * `options` - Configuration options for the operation
  ///
  /// # Errors
  ///
  /// Returns an error if package manager is not found or installation fails.
  pub fn install(&self, packages: &[&str], options: &Options) -> Result<()> {
      // ...
  }
  ```

- Update `README.md` for user-facing changes
- Update `ARCHITECTURE.md` for design changes
- Don't edit `CHANGELOG.md` — release-please generates it from commit messages (see Release Process)

### Commit Messages

Use conventional commits:

```
type(scope): description

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting
- `refactor`: Code restructuring
- `test`: Adding tests
- `chore`: Maintenance

**Examples:**
```
feat(adapters): add DNF package manager support

fix(apt): correct package version parsing
Fixes #123

docs: update CONTRIBUTING with new examples
```

These aren't just style — commit types on `main` directly drive [release-please](https://github.com/googleapis/release-please)'s version bumps and `CHANGELOG.md` entries (see Release Process below). `feat:` bumps the minor version and appears under "Features"; `fix:` bumps patch and appears under "Bug Fixes"; a `!` after the type or a `BREAKING CHANGE:` footer bumps the major version (or minor, while pre-1.0). Other types (`chore`, `docs`, `style`, `refactor`, `test`, `build`, `ci`) don't bump the version and are omitted from the changelog unless they contain a breaking-change marker.

### Pull Request Process

1. **Fork** the repository
2. **Create** a feature branch:
   ```bash
   git checkout -b feat/my_feature   # <type>/<short_name>, snake_case
   ```
3. **Make** your changes
4. **Test** thoroughly (or `mise install` then `mise run check-all` if you use mise — see `mise.toml`):
   ```bash
   cargo test --all-targets --all-features --locked
   cargo clippy --all-targets --all-features --locked -- -D warnings
   cargo fmt --all -- --check
   ```
5. **Commit** with clear messages
6. **Push** to your fork
7. **Open** a pull request

**PR Guidelines:**
- Link related issues
- Describe what and why
- Include test results
- Keep changes focused
- Update documentation
- Respond to feedback

### Review Process

1. Maintainers will review within a few days
2. Address feedback and comments
3. Make requested changes
4. Once approved, PR will be merged

## Debugging

```bash
# Enable debug logging
RUST_LOG=debug cargo run -- install neovim

# Use dry-run to test without side effects
cargo run -- install --dry-run some-package
```

## Architecture Guidelines

### Adding New Core Features

1. **Discuss first** - Open an issue to discuss the design
2. **Keep abstractions clean** - Core traits should be minimal
3. **Maintain backward compatibility** - Don't break existing adapters
4. **Document trade-offs** - Explain design decisions

### Extending the API

When adding to the public API:

1. Consider use cases beyond CLI
2. Make it generic and reusable
3. Add examples in doc comments
4. Update `lib.rs` exports if needed

### Platform-Specific Code

Use conditional compilation:

```rust
#[cfg(target_os = "linux")]
{
    // Linux-specific code
}

#[cfg(target_os = "macos")]
{
    // macOS-specific code
}
```

## Special Considerations

### Package Manager Paradigms

Different package managers have different paradigms:

- **AUR (Arch)**: Community packages separate from official repos
- **PPAs (Ubuntu)**: Third-party repositories
- **Flatpak/Snap**: Sandboxed applications
- **Chocolatey/Scoop (Windows)**: Community-driven repositories

When implementing support for these, consider:
1. How to detect them
2. How they interact with the main package manager
3. Configuration options needed
4. Priority/preference ordering

### Security

- Never execute arbitrary user input
- All package names and search queries must go through `core::validation` (see `ARCHITECTURE.md`) before reaching an adapter
- Use `--no-confirm` carefully
- Document security implications
- Consider sandboxing for future features

### Performance

- Minimize subprocess calls
- Cache detection results when possible
- Parallelize when safe
- Profile before optimizing

This area is a near-term focus — see `ROADMAP.md` for the planned design/performance refactor.

## Release Process

(For maintainers)

Releases are automated by [release-please](https://github.com/googleapis/release-please) (`.github/workflows/release-please.yml`), driven entirely by Conventional Commits merged to `main` — there's no manual version bumping or tagging:

1. Every push to `main` with releasable commits (`feat`, `fix`, or a breaking change) since the last release makes release-please open or update a "release PR" that bumps `Cargo.toml`'s `version`, updates `Cargo.lock`, and rewrites `CHANGELOG.md`.
2. Review and merge that PR when you're ready to cut a release.
3. On merge, release-please creates the git tag and GitHub Release (with the changelog as the release body); the same workflow then builds release binaries for Linux/macOS/Windows and attaches them to that release.
4. Config lives in `release-please-config.json` / `.release-please-manifest.json` at the repo root.

Not published to crates.io yet — see `ROADMAP.md`. To rebuild/re-upload binaries for an existing tag without cutting a new release (e.g. after a failed build), run the workflow manually via `workflow_dispatch` with the existing tag.

## Questions?

- Open a discussion on GitHub
- Ask in pull request comments
- Check existing issues and PRs

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to PacMagic! 🪄
