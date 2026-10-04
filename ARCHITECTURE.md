# PacMagic Architecture

## Design Philosophy

PacMagic is built around three core principles:

1. **Separation of Concerns**: Clear boundaries between package manager implementations, core logic, and user interfaces
2. **Extensibility Through Traits**: New package managers are added by implementing a well-defined trait
3. **Configuration Over Convention**: Users can customize behavior without modifying code

### Why these choices

- **Rust**: memory safety without a garbage collector, a strong type system that catches integration errors (e.g. a missing adapter method) at compile time, and first-class cross-platform support (Linux/macOS/Windows) via `std::process::Command`.
- **Trait-based adapters over shell scripts**: a shell wrapper around each package manager would be faster to write but gives up compile-time completeness checks, structured error handling, and a reusable library API — PacMagic is both a CLI and a `pacmagic` crate.
- **TOML config**: human-editable, the standard choice in the Rust ecosystem, and maps cleanly onto `serde`.

## Core Components

### 1. Package Manager Trait (`PackageManager`)

The foundation of the system, defined in `src/core/traits.rs`. Every package manager adapter implements this trait:

```rust
pub trait PackageManager: Send + Sync {
    fn name(&self) -> &str;
    fn command_names(&self) -> &[&str];
    fn is_available(&self) -> bool;
    fn priority(&self) -> u8 { 50 } // higher = preferred when multiple are available
    fn supports_operation(&self, operation: Operation) -> bool;
    fn supports_source(&self, operation: Operation, source: PackageSource) -> bool { source == PackageSource::Any }

    fn install(&self, packages: &[&str], options: &OperationOptions) -> Result<()>;
    fn remove(&self, packages: &[&str], options: &OperationOptions) -> Result<()>;
    fn update(&self, options: &OperationOptions) -> Result<()>;
    fn upgrade(&self, packages: &[&str], options: &OperationOptions) -> Result<()>;
    fn upgrade_all(&self, options: &OperationOptions) -> Result<()>;
    fn search(&self, query: &str, options: &OperationOptions) -> Result<Vec<Package>>;
    fn list_installed(&self, options: &OperationOptions) -> Result<Vec<Package>>;
    fn info(&self, packages: &[&str], options: &OperationOptions) -> Result<()>;
    fn clean(&self, options: &OperationOptions) -> Result<()>;
    fn files(&self, package: &str, options: &OperationOptions) -> Result<Vec<String>>;
    fn check(&self, options: &OperationOptions) -> Result<()>;
    fn autoremove(&self, options: &OperationOptions) -> Result<()>;
    fn owns(&self, file_or_command: &str, options: &OperationOptions) -> Result<String>;
}
```

An extension trait, `PackageManagerExt`, adds optional repository operations (`add_repository`/`remove_repository`/`list_repositories`) with a blanket default implementation that returns `OperationNotSupported` — adapters only override what they actually need.

### 2. Operation System

Operations are first-class values, defined in `src/core/operation.rs`:

```rust
pub enum Operation {
    Install, Remove, Update, Upgrade, UpgradeAll,
    Search, List, Info, Clean, Files, Check, Autoremove, Owns,
}
```

Each `Operation` carries its own canonical CLI name (e.g. `install`) via `default_aliases()`, which is also how `Config`'s user-defined aliases are validated and resolved. `default_aliases()` returns only that single canonical name — shorthand aliases (`i`, `rm`, ...) aren't built in; users add their own under `[aliases]`.

`OperationOptions` carries the modifiers that affect behavior across all adapters: `no_confirm`, `use_sudo`, `verbose`, `dry_run`, `show_commands`, `install_behavior`, `extra_flags`, `force`, `source`.

`source` is a `PackageSource` (`Any`/`Official`/`User`, set by `--official`/`--user`) restricting which repositories an operation draws from (e.g. paru's `--repo`/`--aur`, or `-Qn`/`-Qm` for listing). It only applies to operations where `Operation::is_source_filterable()` is true (install, upgrade, upgrade-all, search, list, info); others ignore it. `PacMagic::execute` rejects a source the adapter's `supports_source()` doesn't accept with `SourceNotSupported`, and the pre-install update/upgrade always runs with `Any`, so restricting an install never causes a partial upgrade.

Two enums tune install semantics further:
- `InstallBehavior` — what to do if a package is already installed (`Upgrade`/`Reinstall`/`Skip`/`Abort`/`Default`)
- `InstallPreUpdateBehavior` — whether to update/upgrade before an install (`UpdateAndUpgrade`/`UpdateOnly`/`UpdateAndAsk`/`None`/`Default`, where `Default` means "update+upgrade only on rolling releases", decided via `PlatformDetector::is_rolling_release()`)

### 3. Adapter Pattern

Each package manager has an adapter, one file per manager:

```
src/adapters/
├── mod.rs          # AdapterRegistry: built-in adapter list, lookup, auto-detection
├── apt.rs          # Debian/Ubuntu apt
├── pacman.rs       # Arch Linux pacman
├── paru.rs         # Arch Linux AUR helper (paru)
├── brew.rs         # macOS Homebrew
└── winget.rs       # Windows Package Manager
```

Adapters are unit structs (`pub struct Apt;`) — no per-instance state. Each method builds a `std::process::Command` and runs it through a `CommandExecutor` (see below), never `Command::output()`/`status()` directly.

### 4. Configuration Layer

User preferences live in TOML, loaded by `src/config/mod.rs`:

```toml
[general]
default_package_manager = "auto"  # or specific: "apt", "pacman", etc.
confirm_before_install = true
install_behavior = "default"
install_pre_update_behavior = "default"

[aliases]
i = "install"
rm = "remove"
sync = ["update", "upgrade-all"]  # multi-command aliases run sequentially

[package_managers.apt]
enabled = true
use_sudo = true
extra_flags = ["--yes"]
```

Default aliases (from `Operation::default_aliases()`) are always merged in on load, so user config only needs to add or override entries. See `config.example.toml` for the full schema.

### 5. Command Router

`PacMagic::execute()` in `src/lib.rs` is the single entry point used by both the CLI and library consumers:

1. Validate package names / search query (`core::validation`) — this is the injection-prevention boundary, since adapters shell out to real package managers, and it runs before any adapter sees the input.
2. Resolve the active `PackageManager` via `AdapterRegistry` (explicit `-p/--package-manager` flag or config, else highest-`priority()` available adapter).
3. Confirm the adapter `supports_operation()` the requested `Operation`.
4. For `Install`, first run the pre-update/upgrade policy (`handle_install_pre_update`), then dispatch to the adapter method matching the `Operation` variant.

`main.rs` additionally handles: CLI alias resolution (including multi-command aliases run sequentially), a default-command-when-no-args flow, and a fallback re-parse path that lets unrecognized flags pass through to the underlying package manager via `--pm-args`/trailing `--`.

### 6. Platform Detection

`src/platform/mod.rs`'s `PlatformDetector` detects OS and (on Linux) distro from `/etc/*-release` files, maps distro → recommended package manager, and exposes `is_rolling_release()` (Arch/Gentoo/openSUSE Tumbleweed/Debian unstable/Fedora Rawhide).

Priority order when multiple package managers are available:
1. User-specified in config
2. User-specified via CLI flag (`-p`)
3. Auto-detect based on each adapter's `priority()`

### 7. Error Handling

Defined in `src/core/error.rs` (via `thiserror`):

```rust
pub enum PacMagicError {
    NoPackageManagerFound,
    PackageManagerNotAvailable(String),
    OperationNotSupported { pm: String, op: String },
    ExecutionError(String),
    ConfigError(String),
    InvalidOperation(String),
    PackageNotFound(String),
    PermissionDenied(String),
    IoError(#[from] std::io::Error),
    SerializationError(#[from] serde_json::Error),
    Other(#[from] anyhow::Error),
}
```

`is_recoverable()` flags errors like `PackageNotFound`/`OperationNotSupported` as non-fatal; `is_command_failure()` lets `main.rs` avoid re-printing an error when the underlying command already wrote to stderr; `help_text()` attaches a user-facing suggestion for the common cases.

## Data Flow

### Example: Installing a Package

```
User: `pacmagic install neovim`
  ↓
CLI Parser (main.rs): parse command + flags, resolve aliases via Config
  ↓
core::validation: reject invalid/shell-metacharacter package names
  ↓
AdapterRegistry: resolve the active PackageManager (config/flag/auto-detect by priority)
  ↓
PacMagic::execute: pre-update/upgrade policy, then supports_operation() check
  ↓
Adapter (apt): build a std::process::Command, e.g. "sudo apt install neovim -y"
  ↓
CommandExecutor: run it (or print it, in --dry-run), capture output
  ↓
Result Handler: format output, map failures to PacMagicError
  ↓
User: sees results
```

## Extension Points

### Adding a New Package Manager

See `CONTRIBUTING.md`'s "Adding a Package Manager" section for the concrete steps and a code skeleton.

### Adding a New Operation

1. Add a variant to the `Operation` enum in `src/core/operation.rs`, with default aliases
2. Add the corresponding method to the `PackageManager` trait in `src/core/traits.rs`
3. Implement it in every adapter (or return `Err(OperationNotSupported)` where not applicable)
4. Wire it into the CLI in `src/main.rs`
5. Add tests

## Special Cases

### AUR (Arch User Repository)

AUR helpers are implemented as separate package manager adapters:
- Each helper (currently paru) has its own adapter implementing `PackageManager`
- Paru's adapter has higher priority than pacman's, so auto-detection prefers it when available (it can handle both official repos and AUR)
- Users can still choose explicitly: `pacmagic -p paru install package` vs `pacmagic -p pacman install package`

### PPAs (Personal Package Archives)

Not yet implemented; the extension point is `PackageManagerExt::add_repository`/`remove_repository`/`list_repositories`, which the APT adapter is the natural candidate to override.

### Flatpak/Snap/AppImage

Not yet implemented; these would be additional adapters that can coexist with the system package manager, with priority controlled via config, same as any other adapter.

## Current Characteristics

An honest snapshot of the architecture as built today, kept here as the baseline for the planned design/performance refactor (see `ROADMAP.md`):

- **Synchronous, one subprocess per call**: every adapter method blocks on a single `std::process::Command` via `CommandExecutor`; there's no async runtime and no batching of multiple operations into one underlying invocation.
- **No caching**: `is_available()`/`priority()`/detection results are recomputed on every run; nothing is memoized across a single `pacmagic` invocation or between invocations.
- **Adapters are hardcoded**: `AdapterRegistry::new()` in `src/adapters/mod.rs` lists all built-in adapters directly; there is no dynamic plugin loading (`AdapterRegistry::register()` exists for library consumers to add their own at runtime, but nothing in the CLI uses it).
- **Sequential multi-command aliases**: alias sequences like `sync = ["update", "upgrade-all"]` run one operation after another in `main.rs`, not as a single combined package manager invocation — each step is its own separate subprocess call, even for pacman/paru where `update`'s `-Sy` and `upgrade-all`'s `-Su` could in principle be combined into one `-Syu`.

## Testing Strategy

1. **Unit Tests**: Each module's `#[cfg(test)] mod tests` block tests its logic in isolation, including adapters (command construction, output parsing) and validation (injection-prevention edge cases).
2. **Mock Package Managers**: `mockall` is available as a dev-dependency, and `AdapterRegistry::register()`/the `PackageManager` trait make it straightforward to substitute a fake adapter in tests (see `test_registry_register` in `src/adapters/mod.rs` for the pattern).
3. **Platform Tests**: CI (`.github/workflows/ci.yml`) runs the test suite on Linux, macOS, and Windows.
