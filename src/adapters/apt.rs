use crate::core::{
    CommandExecutor, InstallBehavior, Operation, OperationOptions, Package, PackageManager, Result,
};
use std::process::Command;

pub struct Apt;

impl Apt {
    pub fn new() -> Self {
        Self
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
                | Operation::Files
                | Operation::Check
                | Operation::Autoremove
                | Operation::Owns
        )
    }

    fn install(&self, packages: &[&str], options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);

        if options.install_behavior == InstallBehavior::Abort {
            let installed = self.list_installed(options)?;
            let installed_names: std::collections::HashSet<String> =
                installed.iter().map(|p| p.name.clone()).collect();
            let mut already_installed = Vec::new();
            for package in packages {
                if installed_names.contains(*package) {
                    already_installed.push(*package);
                }
            }
            if !already_installed.is_empty() {
                return Err(crate::core::error::PacMagicError::ExecutionError(format!(
                    "Package(s) already installed: {}",
                    already_installed.join(", ")
                )));
            }
        }

        let mut cmd = self.build_command(options);
        cmd.arg("install");

        match options.install_behavior {
            InstallBehavior::Reinstall => {
                cmd.arg("--reinstall");
                cmd.args(packages);
            }
            InstallBehavior::Skip => {
                let installed = self.list_installed(options)?;
                let installed_names: std::collections::HashSet<String> =
                    installed.iter().map(|p| p.name.clone()).collect();
                let mut skipped = Vec::new();
                let mut to_install = Vec::new();
                for package in packages {
                    if installed_names.contains(*package) {
                        skipped.push(*package);
                    } else {
                        to_install.push(*package);
                    }
                }
                if !skipped.is_empty() {
                    log::info!(
                        "Skipping already installed package(s): {}",
                        skipped.join(", ")
                    );
                }
                if to_install.is_empty() {
                    return Ok(());
                }
                cmd.args(&to_install);
            }
            InstallBehavior::Upgrade | InstallBehavior::Default | InstallBehavior::Abort => {
                cmd.args(packages);
            }
        }

        cmd.args(&options.extra_flags);

        executor.execute_interactive(cmd)
    }

    fn remove(&self, packages: &[&str], options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);
        let mut cmd = self.build_command(options);

        cmd.arg("remove");
        cmd.args(packages);
        cmd.args(&options.extra_flags);

        executor.execute_interactive(cmd)
    }

    fn update(&self, options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);
        let mut cmd = self.build_command(options);

        cmd.arg("update");
        cmd.args(&options.extra_flags);

        executor.execute_interactive(cmd)
    }

    fn upgrade(&self, packages: &[&str], options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);
        let mut cmd = self.build_command(options);

        cmd.arg("install");
        cmd.arg("--only-upgrade");
        cmd.args(packages);
        cmd.args(&options.extra_flags);

        executor.execute_interactive(cmd)
    }

    fn upgrade_all(&self, options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);
        let mut cmd = self.build_command(options);

        cmd.arg("upgrade");
        cmd.args(&options.extra_flags);

        executor.execute_interactive(cmd)
    }

    fn search(&self, query: &str, options: &OperationOptions) -> Result<Vec<Package>> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);
        let mut cmd = Command::new("apt");

        cmd.arg("search");
        cmd.arg(query);

        let output = executor.execute_capture(cmd)?;

        let packages: Vec<Package> = output
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with("Sorting"))
            .filter_map(|line| {
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
                        .with_installed(installed),
                )
            })
            .collect();

        Ok(packages)
    }

    fn list_installed(&self, options: &OperationOptions) -> Result<Vec<Package>> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);
        let mut cmd = Command::new("apt");

        cmd.arg("list");
        cmd.arg("--installed");

        let output = executor.execute_capture(cmd)?;

        let packages: Vec<Package> = output
            .lines()
            .skip(1)
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
                        .with_installed(true),
                )
            })
            .collect();

        Ok(packages)
    }

    fn info(&self, packages: &[&str], options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);
        let mut cmd = Command::new("apt");

        cmd.arg("show");
        cmd.args(packages);
        cmd.args(&options.extra_flags);

        executor.execute_interactive(cmd)
    }

    fn clean(&self, options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);
        let mut cmd = self.build_command(options);

        cmd.arg("clean");
        cmd.args(&options.extra_flags);

        executor.execute_checked(cmd)
    }

    fn files(&self, package: &str, options: &OperationOptions) -> Result<Vec<String>> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);
        let mut cmd = Command::new("dpkg");

        cmd.arg("-L");
        cmd.arg(package);

        let output = executor.execute_capture(cmd)?;

        Ok(output.lines().map(|s| s.to_string()).collect())
    }

    fn check(&self, options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);
        let mut cmd = self.build_command(options);

        cmd.arg("check");
        cmd.args(&options.extra_flags);

        executor.execute_checked(cmd)
    }

    fn autoremove(&self, options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);
        let mut cmd = self.build_command(options);

        cmd.arg("autoremove");
        cmd.args(&options.extra_flags);

        executor.execute_interactive(cmd)
    }

    fn owns(&self, file_or_command: &str, options: &OperationOptions) -> Result<String> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);

        let path = if std::path::Path::new(file_or_command).exists() {
            file_or_command.to_string()
        } else {
            let mut which_cmd = Command::new("which");
            which_cmd.arg(file_or_command);
            match executor.execute_capture(which_cmd) {
                Ok(output) => {
                    let resolved = output.trim().to_string();
                    if resolved.is_empty() {
                        return Err(crate::core::error::PacMagicError::ExecutionError(format!(
                            "Command '{}' not found in PATH",
                            file_or_command
                        )));
                    }
                    resolved
                }
                Err(_) => {
                    return Err(crate::core::error::PacMagicError::ExecutionError(format!(
                        "File or command '{}' not found",
                        file_or_command
                    )));
                }
            }
        };

        let mut cmd = Command::new("dpkg");
        cmd.arg("-S");
        cmd.arg(&path);

        let output = executor.execute_capture(cmd)?;

        let package = output
            .lines()
            .next()
            .and_then(|line| line.split(':').next())
            .ok_or_else(|| {
                crate::core::error::PacMagicError::ExecutionError(
                    "No package found for the given file or command".to_string(),
                )
            })?;

        Ok(package.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::PackageSource;

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
        assert!(apt.supports_operation(Operation::Files)); // Supported via dpkg
    }

    #[test]
    fn test_source_support() {
        let apt = Apt::new();
        assert!(apt.supports_source(Operation::Install, PackageSource::Any));
        assert!(!apt.supports_source(Operation::Install, PackageSource::User));
        assert!(!apt.supports_source(Operation::Search, PackageSource::Official));
    }
}
