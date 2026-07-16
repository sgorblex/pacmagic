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

    /// Skip installation if package is already installed (with info message)
    #[serde(alias = "skip")]
    Skip,

    /// Abort with error if package is already installed
    #[serde(alias = "abort")]
    Abort,
}

impl fmt::Display for InstallBehavior {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            InstallBehavior::Default => "default",
            InstallBehavior::Upgrade => "upgrade",
            InstallBehavior::Reinstall => "reinstall",
            InstallBehavior::Skip => "skip",
            InstallBehavior::Abort => "abort",
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
            "abort" => Ok(InstallBehavior::Abort),
            _ => Err(format!("Unknown install behavior: {}", s)),
        }
    }
}

/// Behavior for update/upgrade before installing packages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InstallPreUpdateBehavior {
    /// Update and upgrade everything first
    #[serde(alias = "update-and-upgrade")]
    UpdateAndUpgrade,

    /// Update first, but do not upgrade
    #[serde(alias = "update-only")]
    UpdateOnly,

    /// Update first, ask if upgrade (only if there are upgrades, and show how many)
    #[serde(alias = "update-and-ask")]
    UpdateAndAsk,

    /// Just install (the user manually decides to run update and/or upgrade first)
    #[serde(alias = "none")]
    None,

    /// Default: Update and upgrade everything first, but only on rolling releases such as Arch
    #[serde(alias = "default")]
    Default,
}

impl fmt::Display for InstallPreUpdateBehavior {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            InstallPreUpdateBehavior::UpdateAndUpgrade => "update-and-upgrade",
            InstallPreUpdateBehavior::UpdateOnly => "update-only",
            InstallPreUpdateBehavior::UpdateAndAsk => "update-and-ask",
            InstallPreUpdateBehavior::None => "none",
            InstallPreUpdateBehavior::Default => "default",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for InstallPreUpdateBehavior {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "update-and-upgrade" | "update_and_upgrade" => {
                Ok(InstallPreUpdateBehavior::UpdateAndUpgrade)
            }
            "update-only" | "update_only" => Ok(InstallPreUpdateBehavior::UpdateOnly),
            "update-and-ask" | "update_and_ask" => Ok(InstallPreUpdateBehavior::UpdateAndAsk),
            "none" => Ok(InstallPreUpdateBehavior::None),
            "default" => Ok(InstallPreUpdateBehavior::Default),
            _ => Err(format!("Unknown install pre-update behavior: {}", s)),
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
            Operation::Install => &["install"],
            Operation::Remove => &["remove"],
            Operation::Update => &["update"],
            Operation::Upgrade => &["upgrade"],
            Operation::UpgradeAll => &["upgrade-all"],
            Operation::Search => &["search"],
            Operation::List => &["list"],
            Operation::Info => &["info"],
            Operation::Clean => &["clean"],
            Operation::Files => &["files"],
            Operation::Check => &["check"],
            Operation::Autoremove => &["autoremove"],
            Operation::Owns => &["owns"],
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

    mod install_behavior_tests {
        use super::*;

        #[test]
        fn test_install_behavior_display() {
            assert_eq!(InstallBehavior::Default.to_string(), "default");
            assert_eq!(InstallBehavior::Upgrade.to_string(), "upgrade");
            assert_eq!(InstallBehavior::Reinstall.to_string(), "reinstall");
            assert_eq!(InstallBehavior::Skip.to_string(), "skip");
            assert_eq!(InstallBehavior::Abort.to_string(), "abort");
        }

        #[test]
        fn test_install_behavior_from_str() {
            assert_eq!(
                InstallBehavior::from_str("default").unwrap(),
                InstallBehavior::Default
            );
            assert_eq!(
                InstallBehavior::from_str("upgrade").unwrap(),
                InstallBehavior::Upgrade
            );
            assert_eq!(
                InstallBehavior::from_str("reinstall").unwrap(),
                InstallBehavior::Reinstall
            );
            assert_eq!(
                InstallBehavior::from_str("skip").unwrap(),
                InstallBehavior::Skip
            );
            assert_eq!(
                InstallBehavior::from_str("abort").unwrap(),
                InstallBehavior::Abort
            );
        }

        #[test]
        fn test_install_behavior_from_str_case_insensitive() {
            assert_eq!(
                InstallBehavior::from_str("DEFAULT").unwrap(),
                InstallBehavior::Default
            );
            assert_eq!(
                InstallBehavior::from_str("Upgrade").unwrap(),
                InstallBehavior::Upgrade
            );
            assert_eq!(
                InstallBehavior::from_str("REINSTALL").unwrap(),
                InstallBehavior::Reinstall
            );
            assert_eq!(
                InstallBehavior::from_str("Skip").unwrap(),
                InstallBehavior::Skip
            );
            assert_eq!(
                InstallBehavior::from_str("Abort").unwrap(),
                InstallBehavior::Abort
            );
        }

        #[test]
        fn test_install_behavior_from_str_invalid() {
            assert!(InstallBehavior::from_str("invalid").is_err());
            assert!(InstallBehavior::from_str("").is_err());
            assert!(InstallBehavior::from_str("upgrade-all").is_err());
        }

        #[test]
        fn test_install_behavior_serde() {
            let json = serde_json::to_string(&InstallBehavior::Default).unwrap();
            assert_eq!(json, "\"default\"");
            let parsed: InstallBehavior = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, InstallBehavior::Default);
        }

        #[test]
        fn test_install_behavior_serde_all_variants() {
            for behavior in [
                InstallBehavior::Default,
                InstallBehavior::Upgrade,
                InstallBehavior::Reinstall,
                InstallBehavior::Skip,
                InstallBehavior::Abort,
            ] {
                let json = serde_json::to_string(&behavior).unwrap();
                let parsed: InstallBehavior = serde_json::from_str(&json).unwrap();
                assert_eq!(parsed, behavior);
            }
        }

        #[test]
        fn test_install_behavior_serde_alias() {
            let json = r#""default""#;
            let parsed: InstallBehavior = serde_json::from_str(json).unwrap();
            assert_eq!(parsed, InstallBehavior::Default);
        }
    }

    mod operation_tests {
        use super::*;

        #[test]
        fn test_operation_all() {
            let all_ops = Operation::all();
            assert_eq!(all_ops.len(), 13);
            assert!(all_ops.contains(&Operation::Install));
            assert!(all_ops.contains(&Operation::Remove));
            assert!(all_ops.contains(&Operation::Update));
            assert!(all_ops.contains(&Operation::Upgrade));
            assert!(all_ops.contains(&Operation::UpgradeAll));
            assert!(all_ops.contains(&Operation::Search));
            assert!(all_ops.contains(&Operation::List));
            assert!(all_ops.contains(&Operation::Info));
            assert!(all_ops.contains(&Operation::Clean));
            assert!(all_ops.contains(&Operation::Files));
            assert!(all_ops.contains(&Operation::Check));
            assert!(all_ops.contains(&Operation::Autoremove));
            assert!(all_ops.contains(&Operation::Owns));
        }

        #[test]
        fn test_operation_default_aliases() {
            assert_eq!(Operation::Install.default_aliases(), &["install"]);
            assert_eq!(Operation::Remove.default_aliases(), &["remove"]);
            assert_eq!(Operation::Update.default_aliases(), &["update"]);
            assert_eq!(Operation::Upgrade.default_aliases(), &["upgrade"]);
            assert_eq!(Operation::UpgradeAll.default_aliases(), &["upgrade-all"]);
            assert_eq!(Operation::Search.default_aliases(), &["search"]);
            assert_eq!(Operation::List.default_aliases(), &["list"]);
            assert_eq!(Operation::Info.default_aliases(), &["info"]);
            assert_eq!(Operation::Clean.default_aliases(), &["clean"]);
            assert_eq!(Operation::Files.default_aliases(), &["files"]);
            assert_eq!(Operation::Check.default_aliases(), &["check"]);
            assert_eq!(Operation::Autoremove.default_aliases(), &["autoremove"]);
            assert_eq!(Operation::Owns.default_aliases(), &["owns"]);
        }

        #[test]
        fn test_operation_try_from_str() {
            assert_eq!(Operation::try_from_str("install"), Some(Operation::Install));
            assert_eq!(Operation::try_from_str("remove"), Some(Operation::Remove));
            assert_eq!(Operation::try_from_str("search"), Some(Operation::Search));
            assert_eq!(Operation::try_from_str("update"), Some(Operation::Update));
            assert_eq!(
                Operation::try_from_str("upgrade-all"),
                Some(Operation::UpgradeAll)
            );
        }

        #[test]
        fn test_operation_try_from_str_invalid() {
            assert_eq!(Operation::try_from_str("invalid"), None);
            assert_eq!(Operation::try_from_str(""), None);
            assert_eq!(Operation::try_from_str("instal"), None);
            // These used to be built-in shorthand aliases; they're no longer
            // recognized unless a user adds them to their own config.
            assert_eq!(Operation::try_from_str("i"), None);
            assert_eq!(Operation::try_from_str("add"), None);
            assert_eq!(Operation::try_from_str("rm"), None);
            assert_eq!(Operation::try_from_str("s"), None);
            assert_eq!(Operation::try_from_str("find"), None);
            assert_eq!(Operation::try_from_str("up"), None);
            assert_eq!(Operation::try_from_str("full-upgrade"), None);
        }

        #[test]
        fn test_operation_from_str() {
            assert_eq!(Operation::from_str("install").unwrap(), Operation::Install);
            assert_eq!(Operation::from_str("remove").unwrap(), Operation::Remove);
        }

        #[test]
        fn test_operation_from_str_error() {
            assert!(Operation::from_str("invalid").is_err());
            assert!(Operation::from_str("").is_err());
        }

        #[test]
        fn test_operation_display() {
            assert_eq!(Operation::Install.to_string(), "install");
            assert_eq!(Operation::Remove.to_string(), "remove");
            assert_eq!(Operation::Update.to_string(), "update");
            assert_eq!(Operation::Upgrade.to_string(), "upgrade");
            assert_eq!(Operation::UpgradeAll.to_string(), "upgrade-all");
            assert_eq!(Operation::Search.to_string(), "search");
            assert_eq!(Operation::List.to_string(), "list");
            assert_eq!(Operation::Info.to_string(), "info");
            assert_eq!(Operation::Clean.to_string(), "clean");
            assert_eq!(Operation::Files.to_string(), "files");
            assert_eq!(Operation::Check.to_string(), "check");
            assert_eq!(Operation::Autoremove.to_string(), "autoremove");
            assert_eq!(Operation::Owns.to_string(), "owns");
        }

        #[test]
        fn test_operation_serde() {
            let json = serde_json::to_string(&Operation::Install).unwrap();
            assert_eq!(json, "\"Install\"");
            let parsed: Operation = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, Operation::Install);
        }

        #[test]
        fn test_operation_serde_all_variants() {
            for op in Operation::all() {
                let json = serde_json::to_string(op).unwrap();
                let parsed: Operation = serde_json::from_str(&json).unwrap();
                assert_eq!(*op, parsed);
            }
        }

        #[test]
        fn test_operation_hash_eq() {
            use std::collections::HashSet;
            let mut set = HashSet::new();
            set.insert(Operation::Install);
            set.insert(Operation::Remove);
            assert_eq!(set.len(), 2);
            assert!(set.contains(&Operation::Install));
            assert!(!set.contains(&Operation::Update));
        }
    }

    mod operation_options_tests {
        use super::*;

        #[test]
        fn test_operation_options_default() {
            let options = OperationOptions::default();
            assert!(!options.no_confirm);
            assert!(options.use_sudo);
            assert!(!options.verbose);
            assert!(!options.dry_run);
            assert!(options.show_commands);
            assert_eq!(options.install_behavior, InstallBehavior::Default);
            assert!(options.extra_flags.is_empty());
            assert!(!options.force);
        }

        #[test]
        fn test_operation_options_new() {
            let options = OperationOptions::new();
            assert_eq!(options.no_confirm, OperationOptions::default().no_confirm);
        }

        #[test]
        fn test_operation_options_with_no_confirm() {
            let options = OperationOptions::new().with_no_confirm(true);
            assert!(options.no_confirm);
            let options = OperationOptions::new().with_no_confirm(false);
            assert!(!options.no_confirm);
        }

        #[test]
        fn test_operation_options_with_sudo() {
            let options = OperationOptions::new().with_sudo(false);
            assert!(!options.use_sudo);
            let options = OperationOptions::new().with_sudo(true);
            assert!(options.use_sudo);
        }

        #[test]
        fn test_operation_options_with_verbose() {
            let options = OperationOptions::new().with_verbose(true);
            assert!(options.verbose);
            let options = OperationOptions::new().with_verbose(false);
            assert!(!options.verbose);
        }

        #[test]
        fn test_operation_options_with_dry_run() {
            let options = OperationOptions::new().with_dry_run(true);
            assert!(options.dry_run);
            let options = OperationOptions::new().with_dry_run(false);
            assert!(!options.dry_run);
        }

        #[test]
        fn test_operation_options_with_force() {
            let options = OperationOptions::new().with_force(true);
            assert!(options.force);
            let options = OperationOptions::new().with_force(false);
            assert!(!options.force);
        }

        #[test]
        fn test_operation_options_with_show_commands() {
            let options = OperationOptions::new().with_show_commands(false);
            assert!(!options.show_commands);
            let options = OperationOptions::new().with_show_commands(true);
            assert!(options.show_commands);
        }

        #[test]
        fn test_operation_options_with_install_behavior() {
            let options = OperationOptions::new().with_install_behavior(InstallBehavior::Skip);
            assert_eq!(options.install_behavior, InstallBehavior::Skip);
            let options = OperationOptions::new().with_install_behavior(InstallBehavior::Reinstall);
            assert_eq!(options.install_behavior, InstallBehavior::Reinstall);
        }

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

        #[test]
        fn test_operation_options_append_empty_flags() {
            let options = OperationOptions::new().append_extra_flags(&[]);
            assert_eq!(options.extra_flags.len(), 0);
        }

        #[test]
        fn test_operation_options_builder_chaining() {
            let options = OperationOptions::new()
                .with_no_confirm(true)
                .with_sudo(false)
                .with_verbose(true)
                .with_dry_run(true)
                .with_force(true)
                .with_show_commands(false)
                .with_install_behavior(InstallBehavior::Skip)
                .with_extra_flags(vec!["--flag1".to_string()])
                .append_extra_flags(&["--flag2".to_string()]);

            assert!(options.no_confirm);
            assert!(!options.use_sudo);
            assert!(options.verbose);
            assert!(options.dry_run);
            assert!(options.force);
            assert!(!options.show_commands);
            assert_eq!(options.install_behavior, InstallBehavior::Skip);
            assert_eq!(options.extra_flags.len(), 2);
        }

        #[test]
        fn test_operation_options_clone() {
            let options1 = OperationOptions::new()
                .with_verbose(true)
                .with_extra_flags(vec!["--test".to_string()]);
            let options2 = options1.clone();
            assert_eq!(options1.verbose, options2.verbose);
            assert_eq!(options1.extra_flags, options2.extra_flags);
        }
    }
}
