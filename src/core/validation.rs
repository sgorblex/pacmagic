use super::error::{PacMagicError, Result};

/// Validates a package name to prevent command injection
pub fn validate_package_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(PacMagicError::InvalidOperation(
            "Package name cannot be empty".to_string(),
        ));
    }

    // Check for dangerous characters that could be used for command injection
    // Allow alphanumeric, hyphen, underscore, period, plus, tilde, and colon (for arch packages)
    for ch in name.chars() {
        if !ch.is_alphanumeric()
            && ch != '-'
            && ch != '_'
            && ch != '.'
            && ch != '+'
            && ch != '~'
            && ch != ':'
            && ch != '@'
            && ch != '/'
        {
            return Err(PacMagicError::InvalidOperation(format!(
                "Invalid character '{}' in package name '{}'",
                ch, name
            )));
        }
    }

    // Check for shell metacharacters at the start
    if name.starts_with('-') {
        return Err(PacMagicError::InvalidOperation(
            "Package name cannot start with '-'".to_string(),
        ));
    }

    Ok(())
}

/// Validates multiple package names
pub fn validate_package_names(names: &[&str]) -> Result<()> {
    for name in names {
        validate_package_name(name)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_package_names() {
        assert!(validate_package_name("nginx").is_ok());
        assert!(validate_package_name("python3.11").is_ok());
        assert!(validate_package_name("lib-ssl1.0.0").is_ok());
        assert!(validate_package_name("package_name").is_ok());
        assert!(validate_package_name("my+package").is_ok());
        assert!(validate_package_name("package~test").is_ok());
        assert!(validate_package_name("core/gcc").is_ok()); // pacman style
        assert!(validate_package_name("package:arch").is_ok()); // debian style
        assert!(validate_package_name("@scope/package").is_ok()); // npm style
    }

    #[test]
    fn test_invalid_package_names() {
        assert!(validate_package_name("").is_err());
        assert!(validate_package_name("package;ls").is_err());
        assert!(validate_package_name("package|ls").is_err());
        assert!(validate_package_name("package&ls").is_err());
        assert!(validate_package_name("package$(ls)").is_err());
        assert!(validate_package_name("package`ls`").is_err());
        assert!(validate_package_name("package\nls").is_err());
        assert!(validate_package_name("-option").is_err()); // Looks like option
    }

    #[test]
    fn test_validate_multiple() {
        assert!(validate_package_names(&["nginx", "python3"]).is_ok());
        assert!(validate_package_names(&["nginx", "bad;name"]).is_err());
    }
}
