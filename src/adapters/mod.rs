pub mod apt;
pub mod brew;
pub mod pacman;
pub mod paru;
pub mod winget;

use crate::core::{PacMagicError, PackageManager, Result};
use std::sync::Arc;

/// Registry of all available package manager adapters
pub struct AdapterRegistry {
    adapters: Vec<Arc<dyn PackageManager>>,
}

impl AdapterRegistry {
    /// Create a new registry with all built-in adapters
    pub fn new() -> Self {
        let adapters: Vec<Arc<dyn PackageManager>> = vec![
            Arc::new(apt::Apt::new()),
            Arc::new(pacman::Pacman::new()),
            Arc::new(paru::Paru::new()),
            Arc::new(brew::Brew::new()),
            Arc::new(winget::Winget::new()),
        ];

        Self { adapters }
    }

    /// Get all registered adapters
    pub fn all(&self) -> &[Arc<dyn PackageManager>] {
        &self.adapters
    }

    /// Get all available (installed) package managers
    pub fn available(&self) -> Vec<Arc<dyn PackageManager>> {
        self.adapters
            .iter()
            .filter(|pm| pm.is_available())
            .cloned()
            .collect()
    }

    /// Get a specific package manager by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn PackageManager>> {
        self.adapters.iter().find(|pm| pm.name() == name).cloned()
    }

    /// Auto-detect the best package manager for the current system
    pub fn auto_detect(&self) -> Result<Arc<dyn PackageManager>> {
        let mut available = self.available();

        if available.is_empty() {
            return Err(PacMagicError::NoPackageManagerFound);
        }

        available.sort_by_key(|pm| std::cmp::Reverse(pm.priority()));

        Ok(available[0].clone())
    }

    /// Register a custom adapter
    pub fn register(&mut self, adapter: Arc<dyn PackageManager>) {
        self.adapters.push(adapter);
    }
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = AdapterRegistry::new();
        assert!(registry.all().len() >= 4);
    }

    #[test]
    fn test_get_adapter_by_name() {
        let registry = AdapterRegistry::new();

        assert!(registry.get("apt").is_some());
        assert!(registry.get("pacman").is_some());
        assert!(registry.get("paru").is_some());
        assert!(registry.get("brew").is_some());
        assert!(registry.get("winget").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_registry_default() {
        let registry = AdapterRegistry::default();
        assert!(registry.all().len() >= 4);
    }

    #[test]
    fn test_registry_all() {
        let registry = AdapterRegistry::new();
        let all = registry.all();
        assert!(all.len() >= 4);

        let names: Vec<&str> = all.iter().map(|pm| pm.name()).collect();
        assert!(names.contains(&"apt"));
        assert!(names.contains(&"pacman"));
        assert!(names.contains(&"paru"));
        assert!(names.contains(&"brew"));
        assert!(names.contains(&"winget"));
    }

    #[test]
    fn test_registry_available() {
        let registry = AdapterRegistry::new();
        let available = registry.available();
        assert!(available.len() <= registry.all().len());
    }

    #[test]
    fn test_registry_get_names() {
        let registry = AdapterRegistry::new();
        let apt = registry
            .get("apt")
            .expect("apt adapter should be registered");
        assert_eq!(apt.name(), "apt");

        let pacman = registry
            .get("pacman")
            .expect("pacman adapter should be registered");
        assert_eq!(pacman.name(), "pacman");
    }

    #[test]
    fn test_registry_auto_detect() {
        let registry = AdapterRegistry::new();
        let result = registry.auto_detect();
        if let Ok(pm) = result {
            assert!(!pm.name().is_empty());
            assert!(pm.is_available());
        }
    }

    #[test]
    fn test_registry_register() {
        use crate::core::{Operation, OperationOptions, Package, PackageManager, Result};
        use std::sync::Arc;

        struct MockPackageManager {
            name: String,
        }

        impl PackageManager for MockPackageManager {
            fn name(&self) -> &str {
                &self.name
            }

            fn command_names(&self) -> &[&str] {
                &["mock"]
            }

            fn is_available(&self) -> bool {
                true
            }

            fn priority(&self) -> u8 {
                100
            }

            fn supports_operation(&self, _op: Operation) -> bool {
                true
            }

            fn install(&self, _packages: &[&str], _options: &OperationOptions) -> Result<()> {
                Ok(())
            }

            fn remove(&self, _packages: &[&str], _options: &OperationOptions) -> Result<()> {
                Ok(())
            }

            fn update(&self, _options: &OperationOptions) -> Result<()> {
                Ok(())
            }

            fn upgrade(&self, _packages: &[&str], _options: &OperationOptions) -> Result<()> {
                Ok(())
            }

            fn upgrade_all(&self, _options: &OperationOptions) -> Result<()> {
                Ok(())
            }

            fn search(&self, _query: &str, _options: &OperationOptions) -> Result<Vec<Package>> {
                Ok(vec![])
            }

            fn list_installed(&self, _options: &OperationOptions) -> Result<Vec<Package>> {
                Ok(vec![])
            }

            fn info(&self, _packages: &[&str], _options: &OperationOptions) -> Result<()> {
                Ok(())
            }

            fn clean(&self, _options: &OperationOptions) -> Result<()> {
                Ok(())
            }

            fn files(&self, _package: &str, _options: &OperationOptions) -> Result<Vec<String>> {
                Ok(vec![])
            }

            fn check(&self, _options: &OperationOptions) -> Result<()> {
                Ok(())
            }

            fn autoremove(&self, _options: &OperationOptions) -> Result<()> {
                Ok(())
            }

            fn owns(&self, _file_or_command: &str, _options: &OperationOptions) -> Result<String> {
                Ok("mock".to_string())
            }
        }

        let mut registry = AdapterRegistry::new();
        let initial_count = registry.all().len();

        let mock_pm = Arc::new(MockPackageManager {
            name: "mock".to_string(),
        });
        registry.register(mock_pm);

        assert_eq!(registry.all().len(), initial_count + 1);
        assert!(registry.get("mock").is_some());
        assert_eq!(registry.get("mock").unwrap().name(), "mock");
    }
}
