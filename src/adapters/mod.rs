pub mod apt;
pub mod brew;
pub mod pacman;

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
            Arc::new(brew::Brew::new()),
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

        // Sort by priority (highest first)
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
        assert!(registry.all().len() >= 3); // At least apt, pacman, brew
    }

    #[test]
    fn test_get_adapter_by_name() {
        let registry = AdapterRegistry::new();

        assert!(registry.get("apt").is_some());
        assert!(registry.get("pacman").is_some());
        assert!(registry.get("brew").is_some());
        assert!(registry.get("nonexistent").is_none());
    }
}
