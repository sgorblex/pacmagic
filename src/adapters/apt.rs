use crate::core::{
    CommandExecutor, Operation, OperationOptions, Package, PackageManager, 
    PacMagicError, Result,
};
use std::process::Command;

pub struct Apt {
    executor: CommandExecutor,
}

impl Apt {
    pub fn new() -> Self {
        Self {
            executor: CommandExecutor::default(),
        }
    }

    fn build_command(&self, options: &OperationOptions) -> Command {
        let mut cmd = if options.use_sudo {
            let mut c = Command::new("sudo");
            c.arg("apt");
            c
        } else {
            Command::new("apt")
        };

        if options.no_confirm {
            cmd.arg("-y");
        }

        if options.verbose {
            cmd.arg("-V");
        }

        cmd
    }
}

impl Default for Apt {
    fn default() -> Self {
        Self::new()
    }
}

impl PackageManager for Apt {
    fn name(&self) -> &str {
        "apt"
    }

    fn command_names(&self) -> &[&str] {
        &["apt", "apt-get"]
    }

    fn is_available(&self) -> bool {
        which::which("apt").is_ok()
    }

    fn priority(&self) -> u8 {
        // High priority on Debian/Ubuntu systems
        #[cfg(target_os = "linux")]
        {
            if std::path::Path::new("/etc/debian_version").exists() {
                return 90;
            }
        }
        40
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
                | Operation::Autoremove
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
        
        cmd.arg("remove");
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
        
        cmd.arg("install");
        cmd.arg("--only-upgrade");
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
        let mut cmd = Command::new("apt");
        
        cmd.arg("search");
        cmd.arg(query);

        let output = executor.execute_capture(cmd)?;
        
        // Parse apt search output
        let packages: Vec<Package> = output
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with("Sorting"))
            .filter_map(|line| {
                // apt search format: "package/suite version arch [installed]"
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.is_empty() {
                    return None;
                }
                
                let name = parts[0].split('/').next()?.to_string();
                let version = parts.get(1).map(|s| s.to_string());
                let installed = line.contains("[installed]");
                
                Some(
                    Package::new(name, "apt".to_string())
                        .with_version(version.unwrap_or_default())
                        .with_installed(installed)
                )
            })
            .collect();

        Ok(packages)
    }

    fn list_installed(&self, options: &OperationOptions) -> Result<Vec<Package>> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose);
        let mut cmd = Command::new("apt");
        
        cmd.arg("list");
        cmd.arg("--installed");

        let output = executor.execute_capture(cmd)?;
        
        let packages: Vec<Package> = output
            .lines()
            .skip(1) // Skip header line
            .filter(|line| !line.is_empty())
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.is_empty() {
                    return None;
                }
                
                let name = parts[0].split('/').next()?.to_string();
                let version = parts.get(1).map(|s| s.to_string());
                
                Some(
                    Package::new(name, "apt".to_string())
                        .with_version(version.unwrap_or_default())
                        .with_installed(true)
                )
            })
            .collect();

        Ok(packages)
    }

    fn info(&self, package: &str, options: &OperationOptions) -> Result<Package> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose);
        let mut cmd = Command::new("apt");
        
        cmd.arg("show");
        cmd.arg(package);

        let output = executor.execute_capture(cmd)?;
        
        // Parse apt show output
        let mut pkg = Package::new(package.to_string(), "apt".to_string());
        
        for line in output.lines() {
            if let Some(value) = line.strip_prefix("Version: ") {
                pkg.version = Some(value.to_string());
            } else if let Some(value) = line.strip_prefix("Description: ") {
                pkg.description = Some(value.to_string());
            }
        }

        Ok(pkg)
    }

    fn clean(&self, options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose);
        let mut cmd = self.build_command(options);
        
        cmd.arg("clean");
        cmd.args(&options.extra_flags);

        executor.execute_checked(cmd)
    }

    fn files(&self, package: &str, options: &OperationOptions) -> Result<Vec<String>> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose);
        let mut cmd = Command::new("dpkg");
        
        cmd.arg("-L");
        cmd.arg(package);

        let output = executor.execute_capture(cmd)?;
        
        Ok(output.lines().map(|s| s.to_string()).collect())
    }

    fn check(&self, options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose);
        let mut cmd = self.build_command(options);
        
        cmd.arg("check");
        cmd.args(&options.extra_flags);

        executor.execute_checked(cmd)
    }

    fn autoremove(&self, options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose);
        let mut cmd = self.build_command(options);
        
        cmd.arg("autoremove");
        cmd.args(&options.extra_flags);

        executor.execute_interactive(cmd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apt_creation() {
        let apt = Apt::new();
        assert_eq!(apt.name(), "apt");
    }

    #[test]
    fn test_operation_support() {
        let apt = Apt::new();
        assert!(apt.supports_operation(Operation::Install));
        assert!(apt.supports_operation(Operation::Search));
        assert!(!apt.supports_operation(Operation::Files)); // dpkg, but through files()
    }
}


