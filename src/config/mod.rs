use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use crate::core::{Operation, PacMagicError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub aliases: HashMap<String, String>,
    pub package_managers: HashMap<String, PackageManagerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Default package manager to use ("auto" for auto-detection)
    #[serde(default = "default_package_manager")]
    pub default_package_manager: String,
    
    /// Confirm before installing packages
    #[serde(default = "default_true")]
    pub confirm_before_install: bool,
    
    /// Enable verbose output by default
    #[serde(default)]
    pub verbose: bool,
    
    /// Use sudo by default
    #[serde(default = "default_true")]
    pub use_sudo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManagerConfig {
    /// Is this package manager enabled?
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Use sudo for this package manager?
    pub use_sudo: Option<bool>,
    
    /// Extra flags to always pass
    #[serde(default)]
    pub extra_flags: Vec<String>,
    
    /// Additional configuration (package manager specific)
    #[serde(flatten)]
    pub extra: HashMap<String, toml::Value>,
}

fn default_package_manager() -> String {
    "auto".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            default_package_manager: default_package_manager(),
            confirm_before_install: true,
            verbose: false,
            use_sudo: true,
        }
    }
}

impl Default for PackageManagerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            use_sudo: None,
            extra_flags: vec![],
            extra: HashMap::new(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut aliases = HashMap::new();
        
        // Add default aliases for all operations
        for op in Operation::all() {
            for alias in op.default_aliases() {
                aliases.insert(alias.to_string(), op.to_string());
            }
        }

        Self {
            general: GeneralConfig::default(),
            aliases,
            package_managers: HashMap::new(),
        }
    }
}

impl Config {
    /// Load configuration from the default location
    pub fn load() -> Result<Self> {
        let config_path = Self::default_config_path()?;
        
        if !config_path.exists() {
            // Return default config if file doesn't exist
            return Ok(Self::default());
        }

        Self::load_from(&config_path)
    }

    /// Load configuration from a specific path
    pub fn load_from(path: &PathBuf) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .map_err(|e| PacMagicError::ConfigError(format!("Failed to read config: {}", e)))?;

        let config: Config = toml::from_str(&contents)
            .map_err(|e| PacMagicError::ConfigError(format!("Failed to parse config: {}", e)))?;

        Ok(config)
    }

    /// Save configuration to the default location
    pub fn save(&self) -> Result<()> {
        let config_path = Self::default_config_path()?;
        self.save_to(&config_path)
    }

    /// Save configuration to a specific path
    pub fn save_to(&self, path: &PathBuf) -> Result<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| PacMagicError::ConfigError(format!("Failed to create config directory: {}", e)))?;
        }

        let contents = toml::to_string_pretty(self)
            .map_err(|e| PacMagicError::ConfigError(format!("Failed to serialize config: {}", e)))?;

        fs::write(path, contents)
            .map_err(|e| PacMagicError::ConfigError(format!("Failed to write config: {}", e)))?;

        Ok(())
    }

    /// Get the default configuration path
    pub fn default_config_path() -> Result<PathBuf> {
        let config_dir = if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(xdg_config)
        } else if let Some(home) = dirs::home_dir() {
            #[cfg(target_os = "macos")]
            {
                home.join("Library").join("Application Support")
            }
            #[cfg(not(target_os = "macos"))]
            {
                home.join(".config")
            }
        } else {
            return Err(PacMagicError::ConfigError(
                "Unable to determine config directory".to_string(),
            ));
        };

        Ok(config_dir.join("pacmagic").join("config.toml"))
    }

    /// Resolve an alias to an operation
    pub fn resolve_alias(&self, alias: &str) -> Option<Operation> {
        self.aliases
            .get(alias)
            .and_then(|op| Operation::from_str(op))
    }

    /// Get package manager configuration
    pub fn get_package_manager_config(&self, name: &str) -> PackageManagerConfig {
        self.package_managers
            .get(name)
            .cloned()
            .unwrap_or_default()
    }
}

// Note: Using `dirs` crate for cross-platform home directory detection
// We need to add it to Cargo.toml

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.general.default_package_manager, "auto");
        assert!(config.general.confirm_before_install);
        assert!(!config.aliases.is_empty());
    }

    #[test]
    fn test_resolve_alias() {
        let config = Config::default();
        assert_eq!(config.resolve_alias("install"), Some(Operation::Install));
        assert_eq!(config.resolve_alias("i"), Some(Operation::Install));
        assert_eq!(config.resolve_alias("search"), Some(Operation::Search));
        assert_eq!(config.resolve_alias("nonexistent"), None);
    }

    #[test]
    fn test_serialization() {
        let config = Config::default();
        let toml = toml::to_string(&config).unwrap();
        assert!(!toml.is_empty());
        
        let parsed: Config = toml::from_str(&toml).unwrap();
        assert_eq!(parsed.general.default_package_manager, "auto");
    }
}


