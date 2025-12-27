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

    /// Returns true if this is a command failure (output already printed)
    pub fn is_command_failure(&self) -> bool {
        matches!(self, PacMagicError::ExecutionError(msg) if msg.contains("Command failed with exit code"))
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
            PacMagicError::PackageManagerNotAvailable(_) => {
                Some("The specified package manager is not installed or not in PATH")
            }
            PacMagicError::InvalidOperation(_) => {
                Some("Check your input for invalid characters or format")
            }
            PacMagicError::ConfigError(_) => {
                Some("Check your configuration file syntax at ~/.config/pacmagic/config.toml")
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_is_recoverable() {
        assert!(PacMagicError::PackageNotFound("test".to_string()).is_recoverable());
        assert!(PacMagicError::OperationNotSupported {
            pm: "apt".to_string(),
            op: "test".to_string(),
        }
        .is_recoverable());
        assert!(!PacMagicError::NoPackageManagerFound.is_recoverable());
        assert!(!PacMagicError::PermissionDenied("test".to_string()).is_recoverable());
        assert!(!PacMagicError::ExecutionError("test".to_string()).is_recoverable());
    }

    #[test]
    fn test_error_is_command_failure() {
        assert!(
            PacMagicError::ExecutionError("Command failed with exit code 1".to_string())
                .is_command_failure()
        );
        assert!(
            PacMagicError::ExecutionError("Command failed with exit code 127".to_string())
                .is_command_failure()
        );
        assert!(
            !PacMagicError::ExecutionError("Some other error".to_string()).is_command_failure()
        );
        assert!(!PacMagicError::NoPackageManagerFound.is_command_failure());
    }

    #[test]
    fn test_error_help_text() {
        assert!(PacMagicError::NoPackageManagerFound.help_text().is_some());
        assert!(PacMagicError::PermissionDenied("test".to_string())
            .help_text()
            .is_some());
        assert!(PacMagicError::PackageManagerNotAvailable("apt".to_string())
            .help_text()
            .is_some());
        assert!(PacMagicError::InvalidOperation("test".to_string())
            .help_text()
            .is_some());
        assert!(PacMagicError::ConfigError("test".to_string())
            .help_text()
            .is_some());
        assert!(PacMagicError::PackageNotFound("test".to_string())
            .help_text()
            .is_none());
    }

    #[test]
    fn test_error_display() {
        let err = PacMagicError::NoPackageManagerFound;
        assert!(err.to_string().contains("No package manager found"));

        let err = PacMagicError::PackageManagerNotAvailable("apt".to_string());
        assert!(err.to_string().contains("apt"));

        let err = PacMagicError::OperationNotSupported {
            pm: "apt".to_string(),
            op: "test".to_string(),
        };
        assert!(err.to_string().contains("apt"));
        assert!(err.to_string().contains("test"));
    }

    #[test]
    fn test_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: PacMagicError = io_err.into();
        match err {
            PacMagicError::IoError(_) => {}
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_error_from_serde_error() {
        let json = "invalid json";
        let result: std::result::Result<serde_json::Value, _> = serde_json::from_str(json);
        if let Err(serde_err) = result {
            let err: PacMagicError = serde_err.into();
            match err {
                PacMagicError::SerializationError(_) => {}
                _ => panic!("Expected SerializationError"),
            }
        }
    }

    #[test]
    fn test_error_from_anyhow() {
        let anyhow_err = anyhow::anyhow!("test error");
        let err: PacMagicError = anyhow_err.into();
        match err {
            PacMagicError::Other(_) => {}
            _ => panic!("Expected Other error"),
        }
    }

    #[test]
    fn test_error_result_type() {
        fn returns_result() -> Result<()> {
            Ok(())
        }
        assert!(returns_result().is_ok());

        fn returns_error() -> Result<()> {
            Err(PacMagicError::NoPackageManagerFound)
        }
        assert!(returns_error().is_err());
    }
}
