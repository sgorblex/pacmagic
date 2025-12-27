use super::error::{PacMagicError, Result};

/// Validates a package name to prevent command injection using a whitelist approach
pub fn validate_package_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(PacMagicError::InvalidOperation(
            "Package name cannot be empty".to_string(),
        ));
    }

    if name.starts_with('-') {
        return Err(PacMagicError::InvalidOperation(
            "Package name cannot start with '-'".to_string(),
        ));
    }

    for ch in name.chars() {
        if !matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '+' | '~' | ':' | '@' | '/' | '*')
        {
            return Err(PacMagicError::InvalidOperation(format!(
                "Invalid character '{}' in package name '{}'",
                ch, name
            )));
        }
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

/// Validates a search query (more lenient than package names)
pub fn validate_search_query(query: &str) -> Result<()> {
    if query.is_empty() {
        return Err(PacMagicError::InvalidOperation(
            "Search query cannot be empty".to_string(),
        ));
    }

    for ch in query.chars() {
        if matches!(ch, ';' | '|' | '&' | '`' | '$' | '\n' | '\r') {
            return Err(PacMagicError::InvalidOperation(format!(
                "Invalid character '{}' in search query",
                ch
            )));
        }
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
        assert!(validate_package_name("core/gcc").is_ok());
        assert!(validate_package_name("package:arch").is_ok());
        assert!(validate_package_name("@scope/package").is_ok());
        assert!(validate_package_name("python*").is_ok());
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
        assert!(validate_package_name("-option").is_err());
    }

    #[test]
    fn test_validate_multiple() {
        assert!(validate_package_names(&["nginx", "python3"]).is_ok());
        assert!(validate_package_names(&["nginx", "bad;name"]).is_err());
    }

    #[test]
    fn test_search_query_validation() {
        assert!(validate_search_query("python").is_ok());
        assert!(validate_search_query("python3 lib").is_ok());
        assert!(validate_search_query("lib-ssl").is_ok());
        assert!(validate_search_query("").is_err());
        assert!(validate_search_query("bad;query").is_err());
        assert!(validate_search_query("bad|query").is_err());
        assert!(validate_search_query("bad`query").is_err());
    }
}
