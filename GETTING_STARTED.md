# Getting Started with PacMagic

Welcome to PacMagic! This guide will help you understand the project structure and start developing.

## Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)
- A supported package manager (apt, pacman, brew, etc.) for testing

## Installation

### From Source

```bash
git clone <your-repo-url>
cd pacmagic
cargo build --release
```

The compiled binary will be in `target/release/pacmagic`.

### Install Locally

```bash
cargo install --path .
```

## Project Structure

```
pacmagic/
├── src/
│   ├── adapters/           # Package manager adapters
│   │   ├── mod.rs         # Adapter registry
│   │   ├── apt.rs         # Debian/Ubuntu APT
│   │   ├── pacman.rs      # Arch Linux pacman
│   │   └── brew.rs        # macOS Homebrew
│   ├── config/            # Configuration system
│   │   └── mod.rs
│   ├── core/              # Core abstractions
│   │   ├── error.rs       # Error types
│   │   ├── executor.rs    # Command execution
│   │   ├── operation.rs   # Operations enum
│   │   ├── package.rs     # Package types
│   │   └── traits.rs      # Core traits
│   ├── platform/          # Platform detection
│   │   └── mod.rs
│   ├── lib.rs             # Library interface
│   └── main.rs            # CLI application
├── Cargo.toml
├── ARCHITECTURE.md        # Detailed architecture docs
└── README.md
```

## Quick Test

```bash
# Check for syntax errors
cargo check

# Run tests
cargo test

# Build and run
cargo run -- detect

# Install a package (will use detected package manager)
cargo run -- install neovim

# Get help
cargo run -- --help
```

## Your First Contribution

### Adding a New Package Manager

Let's add support for DNF (Fedora's package manager):

1. **Create the adapter**: `src/adapters/dnf.rs`

```rust
use crate::core::{
    CommandExecutor, Operation, OperationOptions, Package, 
    PackageManager, Result,
};
use std::process::Command;

pub struct Dnf {
    executor: CommandExecutor,
}

impl Dnf {
    pub fn new() -> Self {
        Self {
            executor: CommandExecutor::default(),
        }
    }
}

impl PackageManager for Dnf {
    fn name(&self) -> &str {
        "dnf"
    }

    fn command_names(&self) -> &[&str] {
        &["dnf"]
    }

    fn is_available(&self) -> bool {
        which::which("dnf").is_ok()
    }

    fn priority(&self) -> u8 {
        #[cfg(target_os = "linux")]
        {
            if std::path::Path::new("/etc/fedora-release").exists() {
                return 90;
            }
        }
        40
    }

    fn supports_operation(&self, operation: Operation) -> bool {
        // Implement based on what DNF supports
        todo!()
    }

    // Implement all required methods...
    fn install(&self, packages: &[&str], options: &OperationOptions) -> Result<()> {
        todo!()
    }

    // ... more methods
}
```

2. **Register the adapter**: In `src/adapters/mod.rs`

```rust
pub mod dnf;

// In AdapterRegistry::new():
Arc::new(dnf::Dnf::new()),
```

3. **Test it**:

```bash
cargo test
cargo run -- detect
```

## Development Workflow

1. **Make changes** to source files
2. **Check compilation**: `cargo check`
3. **Run tests**: `cargo test`
4. **Run linter**: `cargo clippy`
5. **Format code**: `cargo fmt`
6. **Build release**: `cargo build --release`

## Testing

### Unit Tests

Each module has its own tests:

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test adapters::apt

# Run with output
cargo test -- --nocapture
```

### Manual Testing

Since PacMagic interacts with system package managers, manual testing is important:

```bash
# Dry run mode (doesn't actually execute)
cargo run -- install --dry-run neovim

# Verbose mode
cargo run -- search -v python

# Detect package manager
cargo run -- detect
```

## Configuration

Initialize a config file:

```bash
cargo run -- init-config
```

Then edit `~/.config/pacmagic/config.toml` (Linux) or `~/Library/Application Support/pacmagic/config.toml` (macOS).

## Next Steps

1. **Read** `ARCHITECTURE.md` for design principles
2. **Explore** the code in `src/core/` to understand abstractions
3. **Look at** existing adapters (apt, pacman, brew) as examples
4. **Pick an issue** or feature to work on
5. **Submit a PR** with your changes!

## Common Issues

### "Package manager not found"

Make sure you have a supported package manager installed. Run `pacmagic detect` to see what's available.

### Compilation errors

Make sure you have the latest Rust:

```bash
rustup update
```

### Permission errors

Some operations require sudo. Use `--yes` to skip confirmations or configure in `config.toml`.

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Clap Documentation](https://docs.rs/clap/) - CLI parsing
- [Tokio](https://tokio.rs/) - Async runtime (optional feature)
- Package Manager Docs:
  - [APT](https://manpages.ubuntu.com/manpages/jammy/man8/apt.8.html)
  - [Pacman](https://wiki.archlinux.org/title/Pacman)
  - [Homebrew](https://docs.brew.sh/)

## Questions?

Open an issue or discussion on GitHub!


