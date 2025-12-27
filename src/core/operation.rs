use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Behavior when installing a package that is already installed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InstallBehavior {
    /// Default behavior for the package manager (usually upgrade)
    #[serde(alias = "default")]
    Default,

    /// Upgrade the package if a newer version is available
    #[serde(alias = "upgrade")]
    Upgrade,

    /// Reinstall the package even if already installed
    #[serde(alias = "reinstall")]
    Reinstall,

    /// Skip installation if package is already installed
    #[serde(alias = "skip")]
    Skip,
}

impl fmt::Display for InstallBehavior {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            InstallBehavior::Default => "default",
            InstallBehavior::Upgrade => "upgrade",
            InstallBehavior::Reinstall => "reinstall",
            InstallBehavior::Skip => "skip",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for InstallBehavior {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "default" => Ok(InstallBehavior::Default),
            "upgrade" => Ok(InstallBehavior::Upgrade),
            "reinstall" => Ok(InstallBehavior::Reinstall),
            "skip" => Ok(InstallBehavior::Skip),
            _ => Err(format!("Unknown install behavior: {}", s)),
        }
    }
}

/// Core package management operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Operation {
    /// Install one or more packages
    Install,

    /// Remove/uninstall one or more packages
    Remove,

    /// Update package database/repositories
    Update,

    /// Upgrade installed packages
    Upgrade,

    /// Upgrade all packages (system-wide)
    UpgradeAll,

    /// Search for packages
    Search,

    /// List installed packages
    List,

    /// Show information about a package
    Info,

    /// Clean package cache
    Clean,

    /// Show package files
    Files,

    /// Check for broken dependencies
    Check,

    /// Autoremove unused dependencies
    Autoremove,

    /// Find which package owns a file or command
    Owns,
}

impl Operation {
    /// Returns all available operations
    pub fn all() -> &'static [Operation] {
        &[
            Operation::Install,
            Operation::Remove,
            Operation::Update,
            Operation::Upgrade,
            Operation::UpgradeAll,
            Operation::Search,
            Operation::List,
            Operation::Info,
            Operation::Clean,
            Operation::Files,
            Operation::Check,
            Operation::Autoremove,
            Operation::Owns,
        ]
    }

    /// Returns common aliases for this operation
    pub fn default_aliases(&self) -> &[&str] {
        match self {
            Operation::Install => &["install", "i", "add"],
            Operation::Remove => &["remove", "rm", "uninstall"],
            Operation::Update => &["update", "up"],
            Operation::Upgrade => &["upgrade", "upg"],
            Operation::UpgradeAll => &["upgrade-all", "full-upgrade"],
            Operation::Search => &["search", "s", "find"],
            Operation::List => &["list", "ls"],
            Operation::Info => &["info", "show"],
            Operation::Clean => &["clean", "cleanup"],
            Operation::Files => &["files", "ls-files"],
            Operation::Check => &["check", "verify"],
            Operation::Autoremove => &["autoremove", "auto-remove"],
            Operation::Owns => &["owns", "owner", "which-package"],
        }
    }

    /// Parse an operation from a string
    pub fn try_from_str(s: &str) -> Option<Self> {
        for op in Self::all() {
            if op.default_aliases().contains(&s) {
                return Some(*op);
            }
        }
        None
    }
}

impl FromStr for Operation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from_str(s).ok_or_else(|| format!("Unknown operation: {}", s))
    }
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Operation::Install => "install",
            Operation::Remove => "remove",
            Operation::Update => "update",
            Operation::Upgrade => "upgrade",
            Operation::UpgradeAll => "upgrade-all",
            Operation::Search => "search",
            Operation::List => "list",
            Operation::Info => "info",
            Operation::Clean => "clean",
            Operation::Files => "files",
            Operation::Check => "check",
            Operation::Autoremove => "autoremove",
            Operation::Owns => "owns",
        };
        write!(f, "{}", s)
    }
}

/// Options that modify operation behavior
#[derive(Debug, Clone)]
pub struct OperationOptions {
    /// Skip confirmation prompts
    pub no_confirm: bool,

    /// Use sudo/elevated privileges
    pub use_sudo: bool,

    /// Verbose output
    pub verbose: bool,

    /// Dry run (don't actually execute)
    pub dry_run: bool,

    /// Print the command before executing (default: true)
    pub show_commands: bool,

    /// Behavior when installing packages that are already installed
    pub install_behavior: InstallBehavior,

    /// Additional flags to pass to the package manager
    pub extra_flags: Vec<String>,

    /// Force operation even if risky
    pub force: bool,
}

impl Default for OperationOptions {
    fn default() -> Self {
        Self {
            no_confirm: false,
            use_sudo: true,
            verbose: false,
            dry_run: false,
            show_commands: true,
            install_behavior: InstallBehavior::Default,
            extra_flags: Vec::new(),
            force: false,
        }
    }
}

impl OperationOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_no_confirm(mut self, value: bool) -> Self {
        self.no_confirm = value;
        self
    }

    pub fn with_sudo(mut self, value: bool) -> Self {
        self.use_sudo = value;
        self
    }

    pub fn with_verbose(mut self, value: bool) -> Self {
        self.verbose = value;
        self
    }

    pub fn with_dry_run(mut self, value: bool) -> Self {
        self.dry_run = value;
        self
    }

    pub fn with_force(mut self, value: bool) -> Self {
        self.force = value;
        self
    }

    pub fn with_show_commands(mut self, value: bool) -> Self {
        self.show_commands = value;
        self
    }

    pub fn with_install_behavior(mut self, behavior: InstallBehavior) -> Self {
        self.install_behavior = behavior;
        self
    }

    pub fn with_extra_flags(mut self, flags: Vec<String>) -> Self {
        self.extra_flags = flags;
        self
    }

    pub fn append_extra_flags(mut self, flags: &[String]) -> Self {
        self.extra_flags.extend_from_slice(flags);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_options_extra_flags() {
        let options = OperationOptions::new()
            .with_extra_flags(vec!["--test-flag".to_string(), "-v".to_string()]);

        assert_eq!(options.extra_flags.len(), 2);
        assert_eq!(options.extra_flags[0], "--test-flag");
        assert_eq!(options.extra_flags[1], "-v");
    }

    #[test]
    fn test_operation_options_append_extra_flags() {
        let options = OperationOptions::new()
            .append_extra_flags(&["--flag1".to_string()])
            .append_extra_flags(&["--flag2".to_string(), "--flag3".to_string()]);

        assert_eq!(options.extra_flags.len(), 3);
        assert_eq!(options.extra_flags[0], "--flag1");
        assert_eq!(options.extra_flags[1], "--flag2");
        assert_eq!(options.extra_flags[2], "--flag3");
    }
}
