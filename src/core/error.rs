use thiserror::Error;

pub type Result<T> = std::result::Result<T, PacMagicError>;

#[derive(Error, Debug)]
pub enum PacMagicError {
    #[error("No package manager found on this system")]
    NoPackageManagerFound,

    #[error("Package manager '{0}' is not available")]
    PackageManagerNotAvailable(String),

    #[error("Package manager '{pm}' does not support operation '{op}'")]
    OperationNotSupported { pm: String, op: String },

    #[error("Command execution failed: {0}")]
    ExecutionError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Package not found: {0}")]
    PackageNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl PacMagicError {
    /// Returns true if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            PacMagicError::PackageNotFound(_) | PacMagicError::OperationNotSupported { .. }
        )
    }

    /// Returns a user-friendly error message with suggestions
    pub fn help_text(&self) -> Option<&str> {
        match self {
            PacMagicError::NoPackageManagerFound => {
                Some("No supported package manager was detected. Please install one (apt, pacman, brew, etc.)")
            }
            PacMagicError::PermissionDenied(_) => {
                Some("Try running with elevated permissions (sudo) or check file permissions")
            }
            _ => None,
        }
    }
}
