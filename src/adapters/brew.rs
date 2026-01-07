use crate::core::{
    CommandExecutor, InstallBehavior, Operation, OperationOptions, PacMagicError, Package,
    PackageManager, Result,
};
use std::process::Command;

pub struct Brew;

impl Brew {
    pub fn new() -> Self {
        Self
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
        #[cfg(target_os = "macos")]
        {
            return 90;
        }
        #[cfg(target_os = "linux")]
        {
            return 30;
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

        match options.install_behavior {
            InstallBehavior::Reinstall => {
                cmd.arg("reinstall");
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
                cmd.arg("install");
                cmd.args(&to_install);
            }
            InstallBehavior::Upgrade | InstallBehavior::Default | InstallBehavior::Abort => {
                cmd.arg("install");
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

        cmd.arg("uninstall");
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

        cmd.arg("upgrade");
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
        let mut cmd = Command::new("brew");

        cmd.arg("search");
        cmd.arg(query);

        let output = executor.execute_capture(cmd)?;

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
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);
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

                let mut pkg = Package::new(name, "brew".to_string()).with_installed(true);

                if let Some(v) = version {
                    pkg = pkg.with_version(v);
                }

                Some(pkg)
            })
            .collect();

        Ok(packages)
    }

    fn info(&self, package: &str, options: &OperationOptions) -> Result<Package> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);
        let mut cmd = Command::new("brew");

        cmd.arg("info");
        cmd.arg("--json=v2");
        cmd.arg(package);

        let output = executor.execute_capture(cmd)?;

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
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);
        let mut cmd = self.build_command(options);

        cmd.arg("cleanup");
        cmd.args(&options.extra_flags);

        executor.execute_checked(cmd)
    }

    fn files(&self, package: &str, options: &OperationOptions) -> Result<Vec<String>> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);
        let mut cmd = Command::new("brew");

        cmd.arg("list");
        cmd.arg(package);

        let output = executor.execute_capture(cmd)?;

        Ok(output.lines().map(|s| s.to_string()).collect())
    }

    fn check(&self, options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);
        let mut cmd = Command::new("brew");

        cmd.arg("doctor");
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
                        return Err(PacMagicError::ExecutionError(format!(
                            "Command '{}' not found in PATH",
                            file_or_command
                        )));
                    }
                    resolved
                }
                Err(_) => {
                    return Err(PacMagicError::ExecutionError(format!(
                        "File or command '{}' not found",
                        file_or_command
                    )));
                }
            }
        };

        let installed = self.list_installed(options)?;
        let path_buf = std::path::PathBuf::from(&path);

        for pkg in installed {
            let files = self.files(&pkg.name, options)?;
            for file in files {
                if std::path::Path::new(&file) == path_buf.as_path() {
                    return Ok(pkg.name);
                }
            }
        }

        Err(PacMagicError::ExecutionError(
            "No package found for the given file or command".to_string(),
        ))
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
        assert!(brew.supports_operation(Operation::Autoremove));
    }
}
