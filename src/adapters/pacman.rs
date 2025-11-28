use crate::core::{
    CommandExecutor, Operation, OperationOptions, PacMagicError, Package, PackageManager, Result,
};
use std::process::Command;

pub struct Pacman;

impl Pacman {
    pub fn new() -> Self {
        Self
    }

    fn build_command(&self, options: &OperationOptions) -> Command {
        let mut cmd = if options.use_sudo {
            let mut c = Command::new("sudo");
            c.arg("pacman");
            c
        } else {
            Command::new("pacman")
        };

        if options.no_confirm {
            cmd.arg("--noconfirm");
        }

        if options.verbose {
            cmd.arg("-v");
        }

        cmd
    }
}

impl Default for Pacman {
    fn default() -> Self {
        Self::new()
    }
}

impl PackageManager for Pacman {
    fn name(&self) -> &str {
        "pacman"
    }

    fn command_names(&self) -> &[&str] {
        &["pacman"]
    }

    fn is_available(&self) -> bool {
        which::which("pacman").is_ok()
    }

    fn priority(&self) -> u8 {
        // High priority on Arch Linux
        #[cfg(target_os = "linux")]
        {
            if std::path::Path::new("/etc/arch-release").exists() {
                return 95;
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
        )
    }

    fn install(&self, packages: &[&str], options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose);
        let mut cmd = self.build_command(options);

        cmd.arg("-S"); // Sync (install)
        cmd.args(packages);
        cmd.args(&options.extra_flags);

        executor.execute_interactive(cmd)
    }

    fn remove(&self, packages: &[&str], options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose);
        let mut cmd = self.build_command(options);

        cmd.arg("-R"); // Remove
        cmd.args(packages);
        cmd.args(&options.extra_flags);

        executor.execute_interactive(cmd)
    }

    fn update(&self, options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose);
        let mut cmd = self.build_command(options);

        cmd.arg("-Sy"); // Sync database
        cmd.args(&options.extra_flags);

        executor.execute_interactive(cmd)
    }

    fn upgrade(&self, packages: &[&str], options: &OperationOptions) -> Result<()> {
        // Pacman doesn't have selective upgrade, so we install specific versions
        self.install(packages, options)
    }

    fn upgrade_all(&self, options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose);
        let mut cmd = self.build_command(options);

        cmd.arg("-Syu"); // Sync + refresh + upgrade
        cmd.args(&options.extra_flags);

        executor.execute_interactive(cmd)
    }

    fn search(&self, query: &str, options: &OperationOptions) -> Result<Vec<Package>> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose);
        let mut cmd = Command::new("pacman");

        cmd.arg("-Ss"); // Search sync database
        cmd.arg(query);

        let output = executor.execute_capture(cmd)?;

        // Parse pacman search output
        let mut packages = Vec::new();
        let lines: Vec<&str> = output.lines().collect();

        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];
            if line.is_empty() {
                i += 1;
                continue;
            }

            // First line: repo/name version [installed]
            if let Some((repo_name, rest)) = line.split_once(' ') {
                let parts: Vec<&str> = repo_name.split('/').collect();
                if parts.len() == 2 {
                    let name = parts[1].to_string();
                    let version = rest.split_whitespace().next().map(|s| s.to_string());
                    let installed = line.contains("[installed]");

                    let mut pkg = Package::new(name, "pacman".to_string())
                        .with_installed(installed)
                        .with_repository(parts[0].to_string());

                    if let Some(v) = version {
                        pkg = pkg.with_version(v);
                    }

                    // Second line: description
                    if i + 1 < lines.len() {
                        let desc_line = lines[i + 1].trim();
                        if !desc_line.is_empty() && !desc_line.starts_with("    ") {
                            pkg.description = Some(desc_line.to_string());
                        }
                    }

                    packages.push(pkg);
                }
            }

            i += 1;
        }

        Ok(packages)
    }

    fn list_installed(&self, options: &OperationOptions) -> Result<Vec<Package>> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose);
        let mut cmd = Command::new("pacman");

        cmd.arg("-Q"); // Query local database

        let output = executor.execute_capture(cmd)?;

        let packages: Vec<Package> = output
            .lines()
            .filter(|line| !line.is_empty())
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    Some(
                        Package::new(parts[0].to_string(), "pacman".to_string())
                            .with_version(parts[1].to_string())
                            .with_installed(true),
                    )
                } else {
                    None
                }
            })
            .collect();

        Ok(packages)
    }

    fn info(&self, package: &str, options: &OperationOptions) -> Result<Package> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose);
        let mut cmd = Command::new("pacman");

        cmd.arg("-Si"); // Show info from sync database
        cmd.arg(package);

        let output = executor.execute_capture(cmd)?;

        let mut pkg = Package::new(package.to_string(), "pacman".to_string());

        for line in output.lines() {
            if let Some(value) = line.strip_prefix("Version         : ") {
                pkg.version = Some(value.trim().to_string());
            } else if let Some(value) = line.strip_prefix("Description     : ") {
                pkg.description = Some(value.trim().to_string());
            } else if let Some(value) = line.strip_prefix("Repository      : ") {
                pkg.repository = Some(value.trim().to_string());
            }
        }

        Ok(pkg)
    }

    fn clean(&self, options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose);
        let mut cmd = self.build_command(options);

        cmd.arg("-Sc"); // Clean cache
        cmd.args(&options.extra_flags);

        executor.execute_interactive(cmd)
    }

    fn files(&self, package: &str, options: &OperationOptions) -> Result<Vec<String>> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose);
        let mut cmd = Command::new("pacman");

        cmd.arg("-Ql"); // Query files
        cmd.arg(package);

        let output = executor.execute_capture(cmd)?;

        Ok(output
            .lines()
            .filter_map(|line| {
                // Format: "package /path/to/file"
                line.split_once(' ').map(|(_, path)| path.to_string())
            })
            .collect())
    }

    fn check(&self, options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose);
        let mut cmd = Command::new("pacman");

        cmd.arg("-Dk"); // Check database
        cmd.args(&options.extra_flags);

        executor.execute_checked(cmd)
    }

    fn autoremove(&self, _options: &OperationOptions) -> Result<()> {
        // Pacman doesn't have autoremove, but orphans can be removed with -Qtdq | pacman -Rns -
        Err(PacMagicError::OperationNotSupported {
            pm: "pacman".to_string(),
            op: "autoremove".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pacman_creation() {
        let pacman = Pacman::new();
        assert_eq!(pacman.name(), "pacman");
    }

    #[test]
    fn test_operation_support() {
        let pacman = Pacman::new();
        assert!(pacman.supports_operation(Operation::Install));
        assert!(pacman.supports_operation(Operation::Files));
        assert!(!pacman.supports_operation(Operation::Autoremove));
    }
}
