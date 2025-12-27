pub mod adapters;
pub mod config;
pub mod core;
pub mod platform;

pub use adapters::AdapterRegistry;
pub use config::Config;
pub use core::{
    InstallBehavior, InstallPreUpdateBehavior, Operation, OperationOptions, PacMagicError, Package,
    PackageManager, Result,
};
pub use platform::PlatformDetector;

/// Main context for PacMagic operations
pub struct PacMagic {
    config: Config,
    registry: AdapterRegistry,
}

impl PacMagic {
    /// Create a new PacMagic instance with default configuration
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        let registry = AdapterRegistry::new();
        Ok(Self { config, registry })
    }

    /// Create a new PacMagic instance with custom configuration
    pub fn with_config(config: Config) -> Self {
        let registry = AdapterRegistry::new();
        Self { config, registry }
    }

    /// Get the current configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get a mutable reference to the configuration
    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    /// Get the adapter registry
    pub fn registry(&self) -> &AdapterRegistry {
        &self.registry
    }

    /// Auto-detect the best package manager for the current system
    pub fn detect_package_manager(&self) -> Result<std::sync::Arc<dyn PackageManager>> {
        if self.config.general.default_package_manager != "auto" {
            if let Some(pm) = self
                .registry
                .get(&self.config.general.default_package_manager)
            {
                if pm.is_available() {
                    return Ok(pm);
                }
            }
        }

        self.registry.auto_detect()
    }

    /// Handle pre-update/upgrade behavior before installing packages
    fn handle_install_pre_update(
        &self,
        _packages: &[&str],
        options: &OperationOptions,
        pm: &dyn PackageManager,
    ) -> Result<()> {
        use InstallPreUpdateBehavior::*;
        let behavior = self.config.general.install_pre_update_behavior;

        let effective_behavior = match behavior {
            Default => {
                if PlatformDetector::is_rolling_release() {
                    UpdateAndUpgrade
                } else {
                    None
                }
            }
            other => other,
        };

        match effective_behavior {
            UpdateAndUpgrade => {
                pm.update(options)?;
                pm.upgrade_all(options)?;
            }
            UpdateOnly => {
                pm.update(options)?;
            }
            UpdateAndAsk => {
                pm.update(options)?;
                let upgrade_count = self.get_upgrade_count(pm, options).unwrap_or(0);
                if upgrade_count > 0 && self.prompt_upgrade(upgrade_count, options)? {
                    pm.upgrade_all(options)?;
                }
            }
            None => {}
            Default => {}
        }

        Ok(())
    }

    /// Get the number of packages that can be upgraded
    fn get_upgrade_count(
        &self,
        pm: &dyn PackageManager,
        options: &OperationOptions,
    ) -> Result<usize> {
        let pm_name = pm.name();
        let executor =
            core::CommandExecutor::new(options.dry_run, options.verbose).with_show_commands(false);

        let count = match pm_name {
            "pacman" => if options.use_sudo {
                let mut sudo_cmd = std::process::Command::new("sudo");
                sudo_cmd.arg("pacman");
                sudo_cmd.arg("-Qu");
                executor.execute_capture(sudo_cmd).ok()
            } else {
                let mut cmd = std::process::Command::new("pacman");
                cmd.arg("-Qu");
                executor.execute_capture(cmd).ok()
            }
            .map(|output| output.lines().count())
            .unwrap_or(0),
            "paru" => {
                let mut cmd = std::process::Command::new("paru");
                cmd.arg("-Qu");
                executor
                    .execute_capture(cmd)
                    .ok()
                    .map(|output| output.lines().count())
                    .unwrap_or(0)
            }
            "apt" => {
                let mut cmd = std::process::Command::new("apt");
                cmd.arg("list");
                cmd.arg("--upgradable");
                executor
                    .execute_capture(cmd)
                    .ok()
                    .map(|output| {
                        output
                            .lines()
                            .filter(|line| line.contains("/") && !line.starts_with("Listing"))
                            .count()
                            .saturating_sub(1)
                    })
                    .unwrap_or(0)
            }
            _ => {
                return Ok(0);
            }
        };

        Ok(count)
    }

    /// Prompt the user about upgrading packages
    fn prompt_upgrade(&self, upgrade_count: usize, options: &OperationOptions) -> Result<bool> {
        if options.no_confirm {
            return Ok(false);
        }

        if options.dry_run {
            println!(
                "[dry run] Would prompt: Upgrade {} package(s)?",
                upgrade_count
            );
            return Ok(false);
        }

        print!(
            "{} package(s) can be upgraded. Upgrade now? [y/N]: ",
            upgrade_count
        );
        use std::io::{self, Write};
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        Ok(input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes")
    }

    /// Execute an operation with the detected package manager
    pub fn execute(
        &self,
        operation: Operation,
        packages: &[&str],
        options: &OperationOptions,
    ) -> Result<()> {
        let options = options.clone();
        if !packages.is_empty() && operation != Operation::Search {
            core::validate_package_names(packages)?;
        }

        let pm = self.detect_package_manager()?;

        if !pm.supports_operation(operation) {
            return Err(PacMagicError::OperationNotSupported {
                pm: pm.name().to_string(),
                op: operation.to_string(),
            });
        }

        match operation {
            Operation::Install => {
                self.handle_install_pre_update(packages, &options, &*pm)?;
                pm.install(packages, &options)
            }
            Operation::Remove => pm.remove(packages, &options),
            Operation::Update => pm.update(&options),
            Operation::Upgrade => pm.upgrade(packages, &options),
            Operation::UpgradeAll => pm.upgrade_all(&options),
            Operation::Search => {
                let query = packages.first().unwrap_or(&"");
                core::validate_search_query(query)?;
                let results = pm.search(query, &options)?;
                for pkg in results {
                    println!("{} - {}", pkg.name, pkg.description.unwrap_or_default());
                }
                Ok(())
            }
            Operation::List => {
                let results = pm.list_installed(&options)?;
                for pkg in results {
                    println!("{} {}", pkg.name, pkg.version.as_deref().unwrap_or(""));
                }
                Ok(())
            }
            Operation::Info => {
                for package in packages {
                    let info = pm.info(package, &options)?;
                    println!("Package: {}", info.name);
                    if let Some(version) = &info.version {
                        println!("Version: {}", version);
                    }
                    if let Some(desc) = &info.description {
                        println!("Description: {}", desc);
                    }
                }
                Ok(())
            }
            Operation::Clean => pm.clean(&options),
            Operation::Files => {
                for package in packages {
                    let files = pm.files(package, &options)?;
                    for file in files {
                        println!("{}", file);
                    }
                }
                Ok(())
            }
            Operation::Check => pm.check(&options),
            Operation::Autoremove => pm.autoremove(&options),
            Operation::Owns => {
                let file_or_command = packages.first().ok_or_else(|| {
                    PacMagicError::InvalidOperation(
                        "File or command name required for owns operation".to_string(),
                    )
                })?;
                let package = pm.owns(file_or_command, &options)?;
                println!("{}", package);
                Ok(())
            }
        }
    }
}

impl Default for PacMagic {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self::with_config(Config::default()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pacmagic_creation() {
        let pm = PacMagic::new();
        assert!(pm.is_ok());
    }

    #[test]
    fn test_pacmagic_with_config() {
        let config = Config::default();
        let pm = PacMagic::with_config(config);
        assert_eq!(pm.config().general.default_package_manager, "auto");
    }

    #[test]
    fn test_validation_prevents_injection() {
        let config = Config::default();
        let pm = PacMagic::with_config(config);
        let options = OperationOptions::default();

        let result = pm.execute(Operation::Install, &["package;ls"], &options);
        assert!(result.is_err());

        let result = pm.execute(Operation::Install, &["package|cat"], &options);
        assert!(result.is_err());
    }

    #[test]
    fn test_search_query_validation() {
        let config = Config::default();
        let pm = PacMagic::with_config(config);
        let options = OperationOptions::default();

        let result = pm.execute(Operation::Search, &["python lib"], &options);
        if let Err(e) = result {
            if matches!(e, PacMagicError::InvalidOperation(_)) {
                let msg = format!("{}", e);
                assert!(
                    !msg.contains("Invalid character"),
                    "Search should allow spaces"
                );
            }
        }

        let result = pm.execute(Operation::Search, &["python;ls"], &options);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, PacMagicError::InvalidOperation(_)));
        }
    }
}
