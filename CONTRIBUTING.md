# Contributing to PacMagic

Thank you for your interest in contributing to PacMagic! This document provides guidelines and information for contributors.

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow

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

### Adding Package Managers

We're always looking to expand package manager support! Here's how:

1. **Research** the package manager's CLI interface
2. **Create** a new adapter in `src/adapters/`
3. **Implement** the `PackageManager` trait
4. **Test** thoroughly with different operations
5. **Document** any quirks or special behavior
6. **Submit** a PR with tests

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

All new code should include tests:

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

**Test types:**
- Unit tests in same file as code
- Integration tests in `tests/` directory
- Doc tests in documentation

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

- Update README.md for user-facing changes
- Update ARCHITECTURE.md for design changes

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

docs: update GETTING_STARTED with new examples
```

### Pull Request Process

1. **Fork** the repository
2. **Create** a feature branch:
   ```bash
   git checkout -b feat/my-feature
   ```
3. **Make** your changes
4. **Test** thoroughly (or `mise install` then `mise run all` if you use mise — see `mise.toml`):
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

## Development Setup

### Prerequisites

Minimum Rust **1.82** (see `rust-version` in `Cargo.toml`).

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install development tools
rustup component add clippy rustfmt
```

Optional: [mise](https://mise.jdx.dev/) reads `mise.toml` so `mise install` uses the repo-pinned Rust version. `mise run all` runs the bundled tasks defined there; put `cargo-deny` and `cargo-machete` on your `PATH` before running it.

### Local Development

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/pacmagic.git
cd pacmagic

# Build
cargo build --locked

# Run tests
cargo test --all-targets --all-features --locked

# Run locally
cargo run -- --help
```

### Debugging

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
- Validate package names
- Use `--no-confirm` carefully
- Document security implications
- Consider sandboxing for future features

### Performance

- Minimize subprocess calls
- Cache detection results when possible
- Parallelize when safe
- Profile before optimizing

## Release Process

(For maintainers)

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create git tag: `git tag v0.x.0`
4. Push tag: `git push --tags`
5. GitHub Actions will build and publish

## Questions?

- Open a discussion on GitHub
- Ask in pull request comments
- Check existing issues and PRs

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to PacMagic! 🪄
