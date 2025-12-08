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
