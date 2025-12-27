use crate::core::{
    CommandExecutor, InstallBehavior, Operation, OperationOptions, Package, PackageManager, Result,
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
                | Operation::Autoremove
                | Operation::Owns
        )
    }

    fn install(&self, packages: &[&str], options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);
        let mut cmd = self.build_command(options);

        cmd.arg("-S");

        match options.install_behavior {
            InstallBehavior::Reinstall => {
                cmd.arg("--overwrite");
            }
            InstallBehavior::Skip => {
                cmd.arg("--needed");
            }
            InstallBehavior::Upgrade | InstallBehavior::Default => {}
        }

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
        // Pacman doesn't have selective upgrade, so we install specific versions
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
        let mut cmd = Command::new("pacman");

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

                    let mut pkg = Package::new(name, "pacman".to_string())
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
        let mut cmd = Command::new("pacman");

        cmd.arg("-Q");

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
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);
        let mut cmd = Command::new("pacman");

        cmd.arg("-Si");
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
        let mut cmd = Command::new("pacman");

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
        let mut cmd = Command::new("pacman");

        cmd.arg("-Dk");
        cmd.args(&options.extra_flags);

        executor.execute_checked(cmd)
    }

    fn autoremove(&self, options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);

        let mut list_cmd = Command::new("pacman");
        list_cmd.arg("-Qtdq");

        let output = executor.execute_capture(list_cmd)?;
        let orphans: Vec<&str> = output.lines().filter(|line| !line.is_empty()).collect();

        if orphans.is_empty() {
            return Ok(());
        }

        let mut remove_cmd = self.build_command(options);
        remove_cmd.arg("-Rns");
        remove_cmd.args(orphans);
        remove_cmd.args(&options.extra_flags);

        executor.execute_interactive(remove_cmd)
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

        let mut cmd = Command::new("pacman");
        cmd.arg("-Qo");
        cmd.arg(&path);

        let output = executor.execute_capture(cmd)?;

        let package = output
            .lines()
            .next()
            .and_then(|line| {
                if let Some(pos) = line.find("owned by") {
                    line[pos + 8..]
                        .split_whitespace()
                        .next()
                        .map(|s| s.to_string())
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                crate::core::error::PacMagicError::ExecutionError(
                    "No package found for the given file or command".to_string(),
                )
            })?;

        Ok(package)
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
        assert!(pacman.supports_operation(Operation::Autoremove));
    }
}
