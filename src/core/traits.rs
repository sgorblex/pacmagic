use super::{Operation, OperationOptions, Package, PackageSource, Result};

/// Core trait that all package manager adapters must implement
pub trait PackageManager: Send + Sync {
    /// Returns the name of this package manager (e.g., "apt", "pacman")
    fn name(&self) -> &str;

    /// Returns a list of common command names for this package manager
    fn command_names(&self) -> &[&str];

    /// Check if this package manager is available on the current system
    fn is_available(&self) -> bool;

    /// Returns the priority of this package manager (higher = preferred)
    /// Used when multiple package managers are available
    fn priority(&self) -> u8 {
        50 // Default priority
    }

    /// Check if this package manager supports a given operation
    fn supports_operation(&self, operation: Operation) -> bool;

    /// Check if this package manager can restrict a given operation to a
    /// package source. Only consulted for operations where
    /// [`Operation::is_source_filterable`] is true
    fn supports_source(&self, _operation: Operation, source: PackageSource) -> bool {
        source == PackageSource::Any
    }

    /// Install one or more packages
    fn install(&self, packages: &[&str], options: &OperationOptions) -> Result<()>;

    /// Remove one or more packages
    fn remove(&self, packages: &[&str], options: &OperationOptions) -> Result<()>;

    /// Update package database/repositories
    fn update(&self, options: &OperationOptions) -> Result<()>;

    /// Upgrade specific packages
    fn upgrade(&self, packages: &[&str], options: &OperationOptions) -> Result<()>;

    /// Upgrade all packages
    fn upgrade_all(&self, options: &OperationOptions) -> Result<()>;

    /// Search for packages
    fn search(&self, query: &str, options: &OperationOptions) -> Result<Vec<Package>>;

    /// List installed packages
    fn list_installed(&self, options: &OperationOptions) -> Result<Vec<Package>>;

    /// Show information about packages, forwarding the package manager's
    /// native output as-is
    fn info(&self, packages: &[&str], options: &OperationOptions) -> Result<()>;

    /// Clean package cache
    fn clean(&self, options: &OperationOptions) -> Result<()>;

    /// List files installed by a package
    fn files(&self, package: &str, options: &OperationOptions) -> Result<Vec<String>>;

    /// Check for broken dependencies or issues
    fn check(&self, options: &OperationOptions) -> Result<()>;

    /// Remove unused dependencies
    fn autoremove(&self, options: &OperationOptions) -> Result<()>;

    /// Find which package owns a file or command
    fn owns(&self, file_or_command: &str, options: &OperationOptions) -> Result<String>;
}

/// Extension trait for package managers with additional features
pub trait PackageManagerExt: PackageManager {
    /// Add a repository/PPA/etc
    fn add_repository(&self, repo: &str, options: &OperationOptions) -> Result<()> {
        let _ = (repo, options);
        Err(crate::core::error::PacMagicError::OperationNotSupported {
            pm: self.name().to_string(),
            op: "add_repository".to_string(),
        })
    }

    /// Remove a repository
    fn remove_repository(&self, repo: &str, options: &OperationOptions) -> Result<()> {
        let _ = (repo, options);
        Err(crate::core::error::PacMagicError::OperationNotSupported {
            pm: self.name().to_string(),
            op: "remove_repository".to_string(),
        })
    }

    /// List enabled repositories
    fn list_repositories(&self, options: &OperationOptions) -> Result<Vec<String>> {
        let _ = options;
        Err(crate::core::error::PacMagicError::OperationNotSupported {
            pm: self.name().to_string(),
            op: "list_repositories".to_string(),
        })
    }
}

// Blanket implementation
impl<T: PackageManager> PackageManagerExt for T {}
