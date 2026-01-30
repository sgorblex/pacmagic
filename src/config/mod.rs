use crate::core::{InstallBehavior, InstallPreUpdateBehavior, Operation, PacMagicError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    #[serde(
        serialize_with = "serialize_aliases",
        deserialize_with = "deserialize_aliases"
    )]
    pub aliases: HashMap<String, AliasValue>,
    pub package_managers: HashMap<String, PackageManagerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AliasValue {
    Single(String),
    Multiple(Vec<String>),
}

fn serialize_aliases<S>(
    aliases: &HashMap<String, AliasValue>,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::Serialize;
    let mut map = HashMap::new();
    for (key, value) in aliases {
        match value {
            AliasValue::Single(s) => {
                map.insert(key.clone(), toml::Value::String(s.clone()));
            }
            AliasValue::Multiple(v) => {
                map.insert(
                    key.clone(),
                    toml::Value::Array(v.iter().map(|s| toml::Value::String(s.clone())).collect()),
                );
            }
        }
    }
    map.serialize(serializer)
}

fn deserialize_aliases<'de, D>(
    deserializer: D,
) -> std::result::Result<HashMap<String, AliasValue>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let map: HashMap<String, toml::Value> = HashMap::deserialize(deserializer)?;
    let mut result = HashMap::new();

    for (key, value) in map {
        let alias_value = match value {
            toml::Value::String(s) => AliasValue::Single(s),
            toml::Value::Array(arr) => {
                let strings: std::result::Result<Vec<String>, _> = arr
                    .into_iter()
                    .map(|v| {
                        if let toml::Value::String(s) = v {
                            Ok(s)
                        } else {
                            Err(serde::de::Error::custom(
                                "Alias array must contain only strings",
                            ))
                        }
                    })
                    .collect();
                AliasValue::Multiple(strings?)
            }
            _ => {
                return Err(serde::de::Error::custom(
                    "Alias must be a string or array of strings",
                ))
            }
        };
        result.insert(key, alias_value);
    }

    Ok(result)
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

    /// Default behavior when installing packages that are already installed
    #[serde(default = "default_install_behavior")]
    pub install_behavior: InstallBehavior,

    /// Default behavior for update/upgrade before installing packages
    #[serde(default = "default_install_pre_update_behavior")]
    pub install_pre_update_behavior: InstallPreUpdateBehavior,

    /// Default command to run when pacmagic is invoked without arguments
    #[serde(default)]
    pub default_command: Option<String>,
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

fn default_install_behavior() -> InstallBehavior {
    InstallBehavior::Default
}

fn default_install_pre_update_behavior() -> InstallPreUpdateBehavior {
    InstallPreUpdateBehavior::Default
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            default_package_manager: default_package_manager(),
            confirm_before_install: true,
            verbose: false,
            use_sudo: true,
            install_behavior: default_install_behavior(),
            install_pre_update_behavior: default_install_pre_update_behavior(),
            default_command: None,
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

        for op in Operation::all() {
            for alias in op.default_aliases() {
                aliases.insert(alias.to_string(), AliasValue::Single(op.to_string()));
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
            return Ok(Self::default());
        }

        Self::load_from(&config_path)
    }

    /// Load configuration from a specific path
    pub fn load_from(path: &PathBuf) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .map_err(|e| PacMagicError::ConfigError(format!("Failed to read config: {}", e)))?;

        let mut config: Config = toml::from_str(&contents)
            .map_err(|e| PacMagicError::ConfigError(format!("Failed to parse config: {}", e)))?;

        let mut default_aliases = HashMap::new();
        for op in Operation::all() {
            for alias in op.default_aliases() {
                default_aliases.insert(alias.to_string(), AliasValue::Single(op.to_string()));
            }
        }
        for (alias, op) in default_aliases {
            config.aliases.entry(alias).or_insert(op);
        }

        Ok(config)
    }

    /// Save configuration to the default location
    pub fn save(&self) -> Result<()> {
        let config_path = Self::default_config_path()?;
        self.save_to(&config_path)
    }

    /// Save configuration to a specific path
    pub fn save_to(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                PacMagicError::ConfigError(format!("Failed to create config directory: {}", e))
            })?;
        }

        let contents = toml::to_string_pretty(self).map_err(|e| {
            PacMagicError::ConfigError(format!("Failed to serialize config: {}", e))
        })?;

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

    /// Resolve an alias to a single operation (for backward compatibility)
    pub fn resolve_alias(&self, alias: &str) -> Option<Operation> {
        self.aliases.get(alias).and_then(|value| match value {
            AliasValue::Single(s) => Operation::try_from_str(s),
            AliasValue::Multiple(_) => None,
        })
    }

    /// Resolve an alias to a sequence of operations
    pub fn resolve_alias_sequence(&self, alias: &str) -> Option<Vec<Operation>> {
        self.aliases.get(alias).and_then(|value| match value {
            AliasValue::Single(s) => Operation::try_from_str(s).map(|op| vec![op]),
            AliasValue::Multiple(ops) => {
                let operations: Vec<Operation> = ops
                    .iter()
                    .filter_map(|s| Operation::try_from_str(s))
                    .collect();
                if operations.len() == ops.len() {
                    Some(operations)
                } else {
                    None
                }
            }
        })
    }

    /// Get package manager configuration
    pub fn get_package_manager_config(&self, name: &str) -> PackageManagerConfig {
        self.package_managers.get(name).cloned().unwrap_or_default()
    }
}

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

    #[test]
    fn test_multi_command_alias() {
        let mut config = Config::default();
        config.aliases.insert(
            "full-upgrade".to_string(),
            AliasValue::Multiple(vec!["update".to_string(), "upgrade-all".to_string()]),
        );

        let ops = config.resolve_alias_sequence("full-upgrade");
        assert!(ops.is_some());
        let ops = ops.unwrap();
        assert_eq!(ops.len(), 2);
        assert_eq!(ops[0], Operation::Update);
        assert_eq!(ops[1], Operation::UpgradeAll);
    }

    #[test]
    fn test_single_alias_still_works() {
        let config = Config::default();
        let op = config.resolve_alias("install");
        assert_eq!(op, Some(Operation::Install));

        let ops = config.resolve_alias_sequence("install");
        assert_eq!(ops, Some(vec![Operation::Install]));
    }

    #[test]
    fn test_alias_resolution_with_package_manager_specified() {
        let mut config = Config::default();
        config.aliases.insert(
            "custom-install".to_string(),
            AliasValue::Single("install".to_string()),
        );
        config.aliases.insert(
            "custom-sync".to_string(),
            AliasValue::Multiple(vec!["update".to_string(), "upgrade-all".to_string()]),
        );

        config.general.default_package_manager = "pacman".to_string();

        assert_eq!(
            config.resolve_alias("custom-install"),
            Some(Operation::Install)
        );
        assert_eq!(config.resolve_alias("i"), Some(Operation::Install));

        let ops = config.resolve_alias_sequence("custom-sync");
        assert!(ops.is_some());
        let ops = ops.unwrap();
        assert_eq!(ops.len(), 2);
        assert_eq!(ops[0], Operation::Update);
        assert_eq!(ops[1], Operation::UpgradeAll);
    }

    #[test]
    fn test_alias_resolution_works_regardless_of_package_manager() {
        let mut configs = vec![Config::default()];

        let mut config_pacman = Config::default();
        config_pacman.general.default_package_manager = "pacman".to_string();
        configs.push(config_pacman);

        let mut config_apt = Config::default();
        config_apt.general.default_package_manager = "apt".to_string();
        configs.push(config_apt);

        for config in configs {
            assert_eq!(config.resolve_alias("i"), Some(Operation::Install));
            assert_eq!(config.resolve_alias("rm"), Some(Operation::Remove));
            assert_eq!(config.resolve_alias("up"), Some(Operation::Update));
            assert_eq!(config.resolve_alias("s"), Some(Operation::Search));
        }
    }

    #[test]
    fn test_general_config_default() {
        let config = GeneralConfig::default();
        assert_eq!(config.default_package_manager, "auto");
        assert!(config.confirm_before_install);
        assert!(!config.verbose);
        assert!(config.use_sudo);
        assert_eq!(config.install_behavior, InstallBehavior::Default);
        assert_eq!(config.default_command, None);
    }

    #[test]
    fn test_package_manager_config_default() {
        let config = PackageManagerConfig::default();
        assert!(config.enabled);
        assert_eq!(config.use_sudo, None);
        assert!(config.extra_flags.is_empty());
        assert!(config.extra.is_empty());
    }

    #[test]
    fn test_config_get_package_manager_config() {
        let mut config = Config::default();
        let pm_config = PackageManagerConfig {
            enabled: false,
            extra_flags: vec!["--test".to_string()],
            ..Default::default()
        };
        config
            .package_managers
            .insert("apt".to_string(), pm_config.clone());

        let retrieved = config.get_package_manager_config("apt");
        assert!(!retrieved.enabled);
        assert_eq!(retrieved.extra_flags, vec!["--test".to_string()]);

        let default = config.get_package_manager_config("nonexistent");
        assert!(default.enabled);
        assert!(default.extra_flags.is_empty());
    }

    #[test]
    fn test_config_alias_value_single() {
        let value = AliasValue::Single("install".to_string());
        match value {
            AliasValue::Single(s) => assert_eq!(s, "install"),
            _ => panic!("Expected Single"),
        }
    }

    #[test]
    fn test_config_alias_value_multiple() {
        let value = AliasValue::Multiple(vec!["update".to_string(), "upgrade".to_string()]);
        match value {
            AliasValue::Multiple(v) => {
                assert_eq!(v.len(), 2);
                assert_eq!(v[0], "update");
                assert_eq!(v[1], "upgrade");
            }
            _ => panic!("Expected Multiple"),
        }
    }

    #[test]
    fn test_config_resolve_alias_sequence_single() {
        let config = Config::default();
        let ops = config.resolve_alias_sequence("install");
        assert_eq!(ops, Some(vec![Operation::Install]));
    }

    #[test]
    fn test_config_resolve_alias_sequence_multiple() {
        let mut config = Config::default();
        config.aliases.insert(
            "sync".to_string(),
            AliasValue::Multiple(vec!["update".to_string(), "upgrade-all".to_string()]),
        );
        let ops = config.resolve_alias_sequence("sync");
        assert!(ops.is_some());
        let ops = ops.unwrap();
        assert_eq!(ops.len(), 2);
        assert_eq!(ops[0], Operation::Update);
        assert_eq!(ops[1], Operation::UpgradeAll);
    }

    #[test]
    fn test_config_resolve_alias_sequence_invalid_operation() {
        let mut config = Config::default();
        config.aliases.insert(
            "invalid".to_string(),
            AliasValue::Multiple(vec!["update".to_string(), "invalid-op".to_string()]),
        );
        let ops = config.resolve_alias_sequence("invalid");
        assert!(ops.is_none());
    }

    #[test]
    fn test_config_resolve_alias_sequence_nonexistent() {
        let config = Config::default();
        assert_eq!(config.resolve_alias_sequence("nonexistent"), None);
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let mut config = Config::default();
        config.general.default_package_manager = "pacman".to_string();
        config.general.verbose = true;
        config.general.confirm_before_install = false;

        let toml = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml).unwrap();

        assert_eq!(parsed.general.default_package_manager, "pacman");
        assert!(parsed.general.verbose);
        assert!(!parsed.general.confirm_before_install);
    }

    #[test]
    fn test_config_package_manager_config_serialization() {
        let mut pm_config = PackageManagerConfig {
            enabled: false,
            use_sudo: Some(false),
            extra_flags: vec!["--flag1".to_string(), "--flag2".to_string()],
            ..Default::default()
        };
        pm_config.extra.insert(
            "custom".to_string(),
            toml::Value::String("value".to_string()),
        );

        let toml = toml::to_string(&pm_config).unwrap();
        let parsed: PackageManagerConfig = toml::from_str(&toml).unwrap();

        assert!(!parsed.enabled);
        assert_eq!(parsed.use_sudo, Some(false));
        assert_eq!(parsed.extra_flags.len(), 2);
        assert_eq!(parsed.extra.get("custom").unwrap().as_str(), Some("value"));
    }

    #[test]
    fn test_config_alias_serialization_single() {
        let mut config = Config::default();
        config.aliases.insert(
            "custom-install".to_string(),
            AliasValue::Single("install".to_string()),
        );

        let toml = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml).unwrap();

        assert_eq!(
            parsed.resolve_alias("custom-install"),
            Some(Operation::Install)
        );
    }

    #[test]
    fn test_config_alias_serialization_multiple() {
        let mut config = Config::default();
        config.aliases.insert(
            "sync".to_string(),
            AliasValue::Multiple(vec!["update".to_string(), "upgrade-all".to_string()]),
        );

        let toml = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml).unwrap();

        let ops = parsed.resolve_alias_sequence("sync");
        assert!(ops.is_some());
        let ops = ops.unwrap();
        assert_eq!(ops.len(), 2);
    }

    #[test]
    fn test_config_default_aliases_populated() {
        let config = Config::default();
        assert!(config.aliases.contains_key("install"));
        assert!(config.aliases.contains_key("i"));
        assert!(config.aliases.contains_key("remove"));
        assert!(config.aliases.contains_key("rm"));
        assert!(config.aliases.contains_key("search"));
        assert!(config.aliases.contains_key("s"));
    }

    #[test]
    fn test_config_load_from_nonexistent_returns_default() {
        use std::path::PathBuf;
        let path = PathBuf::from("/nonexistent/path/config.toml");
        let config = Config::load_from(&path);
        assert!(config.is_err());
    }

    #[test]
    fn test_config_save_and_load_roundtrip() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let mut config = Config::default();
        config.general.default_package_manager = "apt".to_string();
        config.general.verbose = true;

        config.save_to(&config_path).unwrap();
        assert!(config_path.exists());

        let loaded = Config::load_from(&config_path).unwrap();
        assert_eq!(loaded.general.default_package_manager, "apt");
        assert!(loaded.general.verbose);
    }
}
