use crate::core::{
    CommandExecutor, Operation, OperationOptions, PacMagicError, Package, PackageManager, Result,
};
use std::process::Command;

pub struct Paru;

impl Paru {
    pub fn new() -> Self {
        Self
    }

    fn build_command(&self, options: &OperationOptions) -> Command {
        let mut cmd = Command::new("paru");

        if options.no_confirm {
            cmd.arg("--noconfirm");
        }

        if options.verbose {
            cmd.arg("-v");
        }

        cmd
    }
}

impl Default for Paru {
    fn default() -> Self {
        Self::new()
    }
}

impl PackageManager for Paru {
    fn name(&self) -> &str {
        "paru"
    }

    fn command_names(&self) -> &[&str] {
        &["paru"]
    }

    fn is_available(&self) -> bool {
        which::which("paru").is_ok()
    }

    fn priority(&self) -> u8 {
        #[cfg(target_os = "linux")]
        {
            if std::path::Path::new("/etc/arch-release").exists() {
                return 100;
            }
        }
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
                | Operation::Files
                | Operation::Check
        )
    }

    fn install(&self, packages: &[&str], options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);
        let mut cmd = self.build_command(options);

        cmd.arg("-S");
        cmd.args(packages);
        cmd.args(&options.extra_flags);

        executor.execute_interactive(cmd)
    }

    fn remove(&self, packages: &[&str], options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);
        let mut cmd = self.build_command(options);

        cmd.arg("-R");
        cmd.args(packages);
        cmd.args(&options.extra_flags);

        executor.execute_interactive(cmd)
    }

    fn update(&self, options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);
        let mut cmd = self.build_command(options);

        cmd.arg("-Sy");
        cmd.args(&options.extra_flags);

        executor.execute_interactive(cmd)
    }

    fn upgrade(&self, packages: &[&str], options: &OperationOptions) -> Result<()> {
        self.install(packages, options)
    }

    fn upgrade_all(&self, options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);
        let mut cmd = self.build_command(options);

        cmd.arg("-Syu");
        cmd.args(&options.extra_flags);

        executor.execute_interactive(cmd)
    }

    fn search(&self, query: &str, options: &OperationOptions) -> Result<Vec<Package>> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);
        let mut cmd = Command::new("paru");

        cmd.arg("-Ss");
        cmd.arg(query);

        let output = executor.execute_capture(cmd)?;

        let mut packages = Vec::new();
        let lines: Vec<&str> = output.lines().collect();

        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];
            if line.is_empty() {
                i += 1;
                continue;
            }

            if let Some((repo_name, rest)) = line.split_once(' ') {
                let parts: Vec<&str> = repo_name.split('/').collect();
                if parts.len() == 2 {
                    let name = parts[1].to_string();
                    let version = rest.split_whitespace().next().map(|s| s.to_string());
                    let installed = line.contains("[installed]");

                    let mut pkg = Package::new(name, "paru".to_string())
                        .with_installed(installed)
                        .with_repository(parts[0].to_string());

                    if let Some(v) = version {
                        pkg = pkg.with_version(v);
                    }

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
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);
        let mut cmd = Command::new("paru");

        cmd.arg("-Q");

        let output = executor.execute_capture(cmd)?;

        let packages: Vec<Package> = output
            .lines()
            .filter(|line| !line.is_empty())
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    Some(
                        Package::new(parts[0].to_string(), "paru".to_string())
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
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);
        let mut cmd = Command::new("paru");

        cmd.arg("-Si");
        cmd.arg(package);

        let output = executor.execute_capture(cmd)?;

        let mut pkg = Package::new(package.to_string(), "paru".to_string());

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
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);
        let mut cmd = self.build_command(options);

        cmd.arg("-Sc");
        cmd.args(&options.extra_flags);

        executor.execute_interactive(cmd)
    }

    fn files(&self, package: &str, options: &OperationOptions) -> Result<Vec<String>> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);
        let mut cmd = Command::new("paru");

        cmd.arg("-Ql");
        cmd.arg(package);

        let output = executor.execute_capture(cmd)?;

        Ok(output
            .lines()
            .filter_map(|line| line.split_once(' ').map(|(_, path)| path.to_string()))
            .collect())
    }

    fn check(&self, options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);
        let mut cmd = Command::new("paru");

        cmd.arg("-Dk");
        cmd.args(&options.extra_flags);

        executor.execute_checked(cmd)
    }

    fn autoremove(&self, _options: &OperationOptions) -> Result<()> {
        Err(PacMagicError::OperationNotSupported {
            pm: "paru".to_string(),
            op: "autoremove".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paru_creation() {
        let paru = Paru::new();
        assert_eq!(paru.name(), "paru");
    }

    #[test]
    fn test_operation_support() {
        let paru = Paru::new();
        assert!(paru.supports_operation(Operation::Install));
        assert!(paru.supports_operation(Operation::Search));
        assert!(!paru.supports_operation(Operation::Autoremove));
    }

    #[test]
    fn test_priority_higher_than_pacman() {
        let paru = Paru::new();
        let pacman = crate::adapters::pacman::Pacman::new();
        assert!(paru.priority() > pacman.priority());
    }
}
