use serde::{Deserialize, Serialize};

/// Represents a package in any package manager
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Package {
    /// Package name
    pub name: String,

    /// Package version
    pub version: Option<String>,

    /// Package description
    pub description: Option<String>,

    /// Is the package installed?
    pub installed: bool,

    /// Repository/source of the package
    pub repository: Option<String>,

    /// Package manager that manages this package
    pub manager: String,

    /// Additional metadata
    pub metadata: PackageMetadata,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageMetadata {
    /// Package size (in bytes)
    pub size: Option<u64>,

    /// Package architecture
    pub architecture: Option<String>,

    /// Package maintainer
    pub maintainer: Option<String>,

    /// Package homepage
    pub homepage: Option<String>,

    /// Package license
    pub license: Option<String>,

    /// Dependencies
    pub dependencies: Vec<String>,
}

impl Package {
    pub fn new(name: String, manager: String) -> Self {
        Self {
            name,
            version: None,
            description: None,
            installed: false,
            repository: None,
            manager,
            metadata: PackageMetadata::default(),
        }
    }

    pub fn with_version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_installed(mut self, installed: bool) -> Self {
        self.installed = installed;
        self
    }

    pub fn with_repository(mut self, repository: String) -> Self {
        self.repository = Some(repository);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_new() {
        let package = Package::new("nginx".to_string(), "apt".to_string());
        assert_eq!(package.name, "nginx");
        assert_eq!(package.manager, "apt");
        assert_eq!(package.version, None);
        assert_eq!(package.description, None);
        assert!(!package.installed);
        assert_eq!(package.repository, None);
    }

    #[test]
    fn test_package_with_version() {
        let package =
            Package::new("nginx".to_string(), "apt".to_string()).with_version("1.18.0".to_string());
        assert_eq!(package.version, Some("1.18.0".to_string()));
    }

    #[test]
    fn test_package_with_description() {
        let package = Package::new("nginx".to_string(), "apt".to_string())
            .with_description("Web server".to_string());
        assert_eq!(package.description, Some("Web server".to_string()));
    }

    #[test]
    fn test_package_with_installed() {
        let package = Package::new("nginx".to_string(), "apt".to_string()).with_installed(true);
        assert!(package.installed);
        let package = Package::new("nginx".to_string(), "apt".to_string()).with_installed(false);
        assert!(!package.installed);
    }

    #[test]
    fn test_package_with_repository() {
        let package = Package::new("nginx".to_string(), "apt".to_string())
            .with_repository("main".to_string());
        assert_eq!(package.repository, Some("main".to_string()));
    }

    #[test]
    fn test_package_builder_chaining() {
        let package = Package::new("nginx".to_string(), "apt".to_string())
            .with_version("1.18.0".to_string())
            .with_description("Web server".to_string())
            .with_installed(true)
            .with_repository("main".to_string());

        assert_eq!(package.name, "nginx");
        assert_eq!(package.manager, "apt");
        assert_eq!(package.version, Some("1.18.0".to_string()));
        assert_eq!(package.description, Some("Web server".to_string()));
        assert!(package.installed);
        assert_eq!(package.repository, Some("main".to_string()));
    }

    #[test]
    fn test_package_equality() {
        let p1 =
            Package::new("nginx".to_string(), "apt".to_string()).with_version("1.18.0".to_string());
        let p2 =
            Package::new("nginx".to_string(), "apt".to_string()).with_version("1.18.0".to_string());
        assert_eq!(p1, p2);
    }

    #[test]
    fn test_package_inequality() {
        let p1 = Package::new("nginx".to_string(), "apt".to_string());
        let p2 = Package::new("apache".to_string(), "apt".to_string());
        assert_ne!(p1, p2);
    }

    #[test]
    fn test_package_clone() {
        let p1 = Package::new("nginx".to_string(), "apt".to_string())
            .with_version("1.18.0".to_string())
            .with_installed(true);
        let p2 = p1.clone();
        assert_eq!(p1, p2);
    }

    #[test]
    fn test_package_metadata_default() {
        let metadata = PackageMetadata::default();
        assert_eq!(metadata.size, None);
        assert_eq!(metadata.architecture, None);
        assert_eq!(metadata.maintainer, None);
        assert_eq!(metadata.homepage, None);
        assert_eq!(metadata.license, None);
        assert!(metadata.dependencies.is_empty());
    }

    #[test]
    fn test_package_metadata_equality() {
        let m1 = PackageMetadata::default();
        let m2 = PackageMetadata::default();
        assert_eq!(m1, m2);
    }

    #[test]
    fn test_package_serde() {
        let package = Package::new("nginx".to_string(), "apt".to_string())
            .with_version("1.18.0".to_string())
            .with_description("Web server".to_string())
            .with_installed(true);

        let json = serde_json::to_string(&package).unwrap();
        let parsed: Package = serde_json::from_str(&json).unwrap();
        assert_eq!(package, parsed);
    }

    #[test]
    fn test_package_metadata_serde() {
        let mut metadata = PackageMetadata::default();
        metadata.size = Some(1024);
        metadata.architecture = Some("amd64".to_string());
        metadata.dependencies = vec!["libc6".to_string()];

        let json = serde_json::to_string(&metadata).unwrap();
        let parsed: PackageMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(metadata, parsed);
    }
}
