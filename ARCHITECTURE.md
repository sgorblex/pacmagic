# PacMagic Architecture

## Design Philosophy

PacMagic is built around three core principles:

1. **Separation of Concerns**: Clear boundaries between package manager implementations, core logic, and user interfaces
2. **Extensibility Through Traits**: New package managers are added by implementing well-defined traits
3. **Configuration Over Convention**: Users can customize behavior without modifying code

## Core Components

### 1. Package Manager Trait (`PackageManager`)

The foundation of the system. Every package manager adapter implements this trait:

```rust
trait PackageManager {
    fn name(&self) -> &str;
    fn is_available(&self) -> bool;
    fn install(&self, packages: &[&str], options: &Options) -> Result<()>;
    fn remove(&self, packages: &[&str], options: &Options) -> Result<()>;
    fn update(&self, packages: &[&str], options: &Options) -> Result<()>;
    fn search(&self, query: &str, options: &Options) -> Result<Vec<Package>>;
    fn list_installed(&self, options: &Options) -> Result<Vec<Package>>;
    // ... more operations
}
```

### 2. Operation System

Operations are first-class concepts:

```rust
enum Operation {
    Install,
    Remove,
    Update,
    Upgrade,    // Update all packages
    Search,
    List,
    Info,
    Clean,
}
```

Each operation can have modifiers (options) that affect behavior.

### 3. Adapter Pattern

Each package manager has an adapter:

```
src/adapters/
├── mod.rs          # Registry and common utilities
├── apt.rs          # Debian/Ubuntu apt
├── pacman.rs       # Arch Linux pacman
├── paru.rs         # Arch Linux AUR helper (paru)
├── brew.rs         # macOS Homebrew
├── dnf.rs          # Fedora DNF
├── winget.rs       # Windows Package Manager
└── ...
```

### 4. Configuration Layer

User preferences are stored in TOML:

```toml
[general]
default_package_manager = "auto"  # or specific: "apt", "pacman", etc.
confirm_before_install = true
parallel_downloads = true

[aliases]
# Map user commands to operations
i = "install"
rm = "remove"
up = "update"
upg = "upgrade"
s = "search"

[package_managers.apt]
enabled = true
use_sudo = true
extra_flags = ["--yes"]

[package_managers.pacman]
enabled = true
use_sudo = true
extra_flags = []

[package_managers.paru]
enabled = true
use_sudo = true
extra_flags = []
```

### 5. Command Router

The router maps user input to operations:

1. Parse CLI arguments (or API call)
2. Load configuration
3. Resolve aliases
4. Detect or select package manager
5. Execute operation through adapter
6. Return result or error

### 6. Platform Detection

Automatic detection of available package managers:

```rust
pub struct PlatformDetector {
    // Detects OS, available package managers, etc.
}
```

Priority order when multiple package managers are available:
1. User-specified in config
2. User-specified via CLI flag
3. Auto-detect based on platform conventions

### 7. Error Handling

Clear error hierarchy:

```rust
enum PacMagicError {
    PackageManagerNotFound,
    PackageManagerNotAvailable(String),
    OperationNotSupported { pm: String, op: Operation },
    ExecutionError { pm: String, details: String },
    ConfigError(String),
    // ...
}
```

## Data Flow

### Example: Installing a Package

```
User: `pacmagic install neovim`
  ↓
CLI Parser: Parse command + flags
  ↓
Config Loader: Load user preferences & aliases
  ↓
Command Router: Resolve "install" → Operation::Install
  ↓
PM Detector: Detect available package manager (e.g., apt)
  ↓
Adapter (apt): Translate to "sudo apt install neovim -y"
  ↓
Executor: Run command, capture output
  ↓
Result Handler: Format output, handle errors
  ↓
User: See results
```

## Extension Points

### Adding a New Package Manager

1. Create new file in `src/adapters/`
2. Implement `PackageManager` trait
3. Register in `src/adapters/mod.rs`
4. Add tests
5. Update documentation

Example:

```rust
// src/adapters/zypper.rs
pub struct Zypper;

impl PackageManager for Zypper {
    fn name(&self) -> &str {
        "zypper"
    }
    
    fn is_available(&self) -> bool {
        which::which("zypper").is_ok()
    }
    
    fn install(&self, packages: &[&str], options: &Options) -> Result<()> {
        let mut cmd = Command::new("sudo");
        cmd.arg("zypper").arg("install");
        
        if options.no_confirm {
            cmd.arg("-y");
        }
        
        cmd.args(packages);
        
        execute_command(cmd)
    }
    
    // ... implement other methods
}
```

### Adding a New Operation

1. Add variant to `Operation` enum
2. Add method to `PackageManager` trait
3. Implement in all adapters (return `Err(OperationNotSupported)` if not available)
4. Update command parser
5. Add tests

## Special Cases

### AUR (Arch User Repository)

AUR helpers are implemented as separate package manager adapters:
- Each helper (paru, yay, etc.) has its own adapter implementing `PackageManager`
- Paru adapter has higher priority (100) than pacman (95) on Arch Linux
- This allows users to explicitly choose: `pacmagic -p paru install package` vs `pacmagic -p pacman install package`
- Auto-detection will prefer paru when available, as it can handle both official repos and AUR

### PPAs (Personal Package Archives)

APT adapter has special handling:
- `add-ppa` operation → `add-apt-repository`
- `remove-ppa` operation → remove from sources
- Automatic `apt update` after PPA changes

### Flatpak/Snap/AppImage

These are additional package managers that can be enabled:
- Separate adapters
- Can coexist with system package manager
- Priority controlled via config

## Future Considerations

1. **Parallel Operations**: Install multiple packages concurrently
2. **Transaction System**: Rollback on failure
3. **Dependency Resolution**: Show dependency tree before install
4. **Package Source Priority**: Prefer certain sources (e.g., native > flatpak)
5. **Plugin API**: External plugins via dynamic loading
6. **GUI/TUI**: Build alternative interfaces using the library
7. **Remote Management**: Manage packages on remote systems
8. **Package Sets**: Define and install groups of packages

## Testing Strategy

1. **Unit Tests**: Test each adapter independently with mocked commands
2. **Integration Tests**: Test command routing and config loading
3. **Platform Tests**: CI tests on different OSes (Linux, macOS, Windows)
4. **Mock Package Managers**: Create fake package managers for testing


