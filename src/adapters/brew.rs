use crate::core::{
    CommandExecutor, Operation, OperationOptions, Package, PackageManager,
    PacMagicError, Result,
};
use std::process::Command;

pub struct Brew {
    executor: CommandExecutor,
}

impl Brew {
    pub fn new() -> Self {
        Self {
            executor: CommandExecutor::default(),
        }
    }

    fn build_command(&self, options: &OperationOptions) -> Command {
        let mut cmd = Command::new("brew");

        if options.verbose {
            cmd.arg("--verbose");
        }

        cmd
    }
}

impl Default for Brew {
    fn default() -> Self {
        Self::new()
    }
}

impl PackageManager for Brew {
    fn name(&self) -> &str {
        "brew"
    }

    fn command_names(&self) -> &[&str] {
        &["brew"]
    }

    fn is_available(&self) -> bool {
        which::which("brew").is_ok()
    }

    fn priority(&self) -> u8 {
        // High priority on macOS
        #[cfg(target_os = "macos")]
        {
            return 90;
        }
        #[cfg(target_os = "linux")]
        {
            return 30; // Lower priority on Linux (Linuxbrew)
        }
        #[allow(unreachable_code)]
        50
    }

    fn supports_operation(&self, operation: Operation) -> bool {
        matches!(
            operation,
            Operation::Install
                | Operation::Remove
                | Operation::Update
                | Operation::Upgrade
                | Operation::UpgradeAll
                | Operation::Search
                | Operation::List
                | Operation::Info
                | Operation::Clean
                | Operation::Check
        )
    }

    fn install(&self, packages: &[&str], options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose);
        let mut cmd = self.build_command(options);
        
        cmd.arg("install");
        cmd.args(packages);
        cmd.args(&options.extra_flags);

        executor.execute_interactive(cmd)
    }

    fn remove(&self, packages: &[&str], options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose);
        let mut cmd = self.build_command(options);
        
        cmd.arg("uninstall");
        cmd.args(packages);
        cmd.args(&options.extra_flags);

        executor.execute_interactive(cmd)
    }

    fn update(&self, options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose);
        let mut cmd = self.build_command(options);
        
        cmd.arg("update");
        cmd.args(&options.extra_flags);

        executor.execute_interactive(cmd)
    }

    fn upgrade(&self, packages: &[&str], options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose);
        let mut cmd = self.build_command(options);
        
        cmd.arg("upgrade");
        cmd.args(packages);
        cmd.args(&options.extra_flags);

        executor.execute_interactive(cmd)
    }

    fn upgrade_all(&self, options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose);
        let mut cmd = self.build_command(options);
        
        cmd.arg("upgrade");
        cmd.args(&options.extra_flags);

        executor.execute_interactive(cmd)
    }

    fn search(&self, query: &str, options: &OperationOptions) -> Result<Vec<Package>> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose);
        let mut cmd = Command::new("brew");
        
        cmd.arg("search");
        cmd.arg(query);

        let output = executor.execute_capture(cmd)?;
        
        // Parse brew search output
        let packages: Vec<Package> = output
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with("==>"))
            .map(|line| {
                let name = line.trim().to_string();
                Package::new(name, "brew".to_string())
            })
            .collect();

        Ok(packages)
    }

    fn list_installed(&self, options: &OperationOptions) -> Result<Vec<Package>> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose);
        let mut cmd = Command::new("brew");
        
        cmd.arg("list");
        cmd.arg("--versions");

        let output = executor.execute_capture(cmd)?;
        
        let packages: Vec<Package> = output
            .lines()
            .filter(|line| !line.is_empty())
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.is_empty() {
                    return None;
                }
                
                let name = parts[0].to_string();
                let version = parts.get(1).map(|s| s.to_string());
                
                let mut pkg = Package::new(name, "brew".to_string())
                    .with_installed(true);
                
                if let Some(v) = version {
                    pkg = pkg.with_version(v);
                }
                
                Some(pkg)
            })
            .collect();

        Ok(packages)
    }

    fn info(&self, package: &str, options: &OperationOptions) -> Result<Package> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose);
        let mut cmd = Command::new("brew");
        
        cmd.arg("info");
        cmd.arg("--json=v2");
        cmd.arg(package);

        let output = executor.execute_capture(cmd)?;
        
        // Parse JSON output
        let json: serde_json::Value = serde_json::from_str(&output)
            .map_err(|e| PacMagicError::ExecutionError(format!("Failed to parse JSON: {}", e)))?;
        
        let mut pkg = Package::new(package.to_string(), "brew".to_string());
        
        if let Some(formulae) = json["formulae"].as_array() {
            if let Some(formula) = formulae.first() {
                if let Some(version) = formula["versions"]["stable"].as_str() {
                    pkg.version = Some(version.to_string());
                }
                if let Some(desc) = formula["desc"].as_str() {
                    pkg.description = Some(desc.to_string());
                }
                if let Some(homepage) = formula["homepage"].as_str() {
                    pkg.metadata.homepage = Some(homepage.to_string());
                }
            }
        }

        Ok(pkg)
    }

    fn clean(&self, options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose);
        let mut cmd = self.build_command(options);
        
        cmd.arg("cleanup");
        cmd.args(&options.extra_flags);

        executor.execute_checked(cmd)
    }

    fn files(&self, package: &str, options: &OperationOptions) -> Result<Vec<String>> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose);
        let mut cmd = Command::new("brew");
        
        cmd.arg("list");
        cmd.arg(package);

        let output = executor.execute_capture(cmd)?;
        
        Ok(output.lines().map(|s| s.to_string()).collect())
    }

    fn check(&self, options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose);
        let mut cmd = Command::new("brew");
        
        cmd.arg("doctor");
        cmd.args(&options.extra_flags);

        executor.execute_checked(cmd)
    }

    fn autoremove(&self, _options: &OperationOptions) -> Result<()> {
        // Brew doesn't have a direct autoremove command
        Err(PacMagicError::OperationNotSupported {
            pm: "brew".to_string(),
            op: "autoremove".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brew_creation() {
        let brew = Brew::new();
        assert_eq!(brew.name(), "brew");
    }

    #[test]
    fn test_operation_support() {
        let brew = Brew::new();
        assert!(brew.supports_operation(Operation::Install));
        assert!(brew.supports_operation(Operation::Search));
        assert!(!brew.supports_operation(Operation::Autoremove));
    }
}


