pub mod adapters;
pub mod config;
pub mod core;
pub mod platform;

pub use adapters::AdapterRegistry;
pub use config::Config;
pub use core::{Operation, OperationOptions, PacMagicError, Package, PackageManager, Result};
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

    /// Get a mutable reference to the adapter registry
    pub fn registry_mut(&mut self) -> &mut AdapterRegistry {
        &mut self.registry
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

    /// Execute an operation with the detected package manager
    pub fn execute(
        &self,
        operation: Operation,
        packages: &[&str],
        options: &OperationOptions,
    ) -> Result<()> {
        // Validate package names to prevent command injection
        if !packages.is_empty() {
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
            Operation::Install => pm.install(packages, options),
            Operation::Remove => pm.remove(packages, options),
            Operation::Update => pm.update(options),
            Operation::Upgrade => pm.upgrade(packages, options),
            Operation::UpgradeAll => pm.upgrade_all(options),
            Operation::Search => {
                let query = packages.first().unwrap_or(&"");
                core::validate_search_query(query)?;
                let results = pm.search(query, options)?;
                for pkg in results {
                    println!("{} - {}", pkg.name, pkg.description.unwrap_or_default());
                }
                Ok(())
            }
            Operation::List => {
                let results = pm.list_installed(options)?;
                for pkg in results {
                    println!("{} {}", pkg.name, pkg.version.as_deref().unwrap_or(""));
                }
                Ok(())
            }
            Operation::Info => {
                for package in packages {
                    let info = pm.info(package, options)?;
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
            Operation::Clean => pm.clean(options),
            Operation::Files => {
                for package in packages {
                    let files = pm.files(package, options)?;
                    for file in files {
                        println!("{}", file);
                    }
                }
                Ok(())
            }
            Operation::Check => pm.check(options),
            Operation::Autoremove => pm.autoremove(options),
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
}
