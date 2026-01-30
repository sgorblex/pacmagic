use crate::core::{CommandExecutor, InstallBehavior, Operation, OperationOptions, Package};
use crate::core::{PacMagicError, PackageManager, Result};
use std::process::Command;

pub struct Winget;

impl Winget {
    pub fn new() -> Self {
        Self
    }

    fn push_mutating_confirm_flags(cmd: &mut Command, options: &OperationOptions) {
        if options.no_confirm {
            cmd.arg("--accept-package-agreements");
            cmd.arg("--accept-source-agreements");
            cmd.arg("--disable-interactivity");
        }
    }

    fn push_capture_mode_flags(cmd: &mut Command, options: &OperationOptions) {
        cmd.arg("--disable-interactivity");
        cmd.arg("--accept-source-agreements");
        if options.no_confirm {
            cmd.arg("--accept-package-agreements");
        }
    }
}

pub(crate) fn parse_table_rows(output: &str) -> Vec<std::collections::HashMap<String, String>> {
    let lines: Vec<&str> = output.lines().collect();

    let header_idx = lines.iter().position(|line| {
        let l = line.trim();
        l.contains("Name") && l.contains("Id") && l.contains("Version")
    });

    let Some(header_idx) = header_idx else {
        return vec![];
    };
    let header = lines[header_idx];

    let mut cols: Vec<(&'static str, usize)> = Vec::new();
    for col in ["Name", "Id", "Version", "Available", "Match", "Source"] {
        if let Some(byte_idx) = header.find(col) {
            let char_pos = header[..byte_idx].chars().count();
            cols.push((col, char_pos));
        }
    }
    cols.sort_by_key(|(_, p)| *p);

    let sep_idx = lines
        .iter()
        .skip(header_idx + 1)
        .position(|line| {
            let t = line.trim();
            !t.is_empty() && t.chars().all(|c| c == '-' || c == '─' || c == '—')
        })
        .map(|i| i + header_idx + 1)
        .unwrap_or(header_idx);

    let mut rows = Vec::new();
    for line in lines.iter().skip(sep_idx + 1) {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }

        let mut row = std::collections::HashMap::new();
        for (i, (col, start_char)) in cols.iter().enumerate() {
            let end_char = cols.get(i + 1).map(|(_, p)| *p).unwrap_or(usize::MAX);

            let start_byte = char_pos_to_byte_idx(line, *start_char);
            let end_byte = char_pos_to_byte_idx(line, end_char);

            let val = if start_byte >= line.len() {
                ""
            } else {
                &line[start_byte..std::cmp::min(end_byte, line.len())]
            };
            row.insert((*col).to_string(), val.trim().to_string());
        }

        if row.values().all(|v| v.is_empty()) {
            continue;
        }

        rows.push(row);
    }

    rows
}

fn char_pos_to_byte_idx(s: &str, char_pos: usize) -> usize {
    if char_pos == usize::MAX {
        return s.len();
    }
    if char_pos == 0 {
        return 0;
    }
    s.char_indices()
        .nth(char_pos)
        .map(|(i, _)| i)
        .unwrap_or_else(|| s.len())
}

fn unsupported<T>(pm: &str, op: &str) -> Result<T> {
    Err(PacMagicError::OperationNotSupported {
        pm: pm.to_string(),
        op: op.to_string(),
    })
}

impl Default for Winget {
    fn default() -> Self {
        Self::new()
    }
}

impl PackageManager for Winget {
    fn name(&self) -> &str {
        "winget"
    }

    fn command_names(&self) -> &[&str] {
        &["winget"]
    }

    fn is_available(&self) -> bool {
        which::which("winget").is_ok()
    }

    fn priority(&self) -> u8 {
        #[cfg(target_os = "windows")]
        {
            return 90;
        }
        #[allow(unreachable_code)]
        10
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
        )
    }

    fn install(&self, packages: &[&str], options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);

        let needs_installed_check = !packages.is_empty()
            && matches!(
                options.install_behavior,
                InstallBehavior::Abort
                    | InstallBehavior::Skip
                    | InstallBehavior::Default
                    | InstallBehavior::Upgrade
            );

        let installed_ids: Option<std::collections::HashSet<String>> = if needs_installed_check {
            Some(
                self.list_installed(options)?
                    .into_iter()
                    .map(|p| p.name)
                    .collect(),
            )
        } else {
            None
        };

        if options.install_behavior == InstallBehavior::Abort {
            let already_installed: Vec<&str> = installed_ids
                .as_ref()
                .map(|ids| {
                    packages
                        .iter()
                        .copied()
                        .filter(|p| ids.contains(*p))
                        .collect()
                })
                .unwrap_or_default();

            if !already_installed.is_empty() {
                return Err(PacMagicError::ExecutionError(format!(
                    "Package(s) already installed: {}",
                    already_installed.join(", ")
                )));
            }
        }

        for package in packages {
            let is_installed = installed_ids
                .as_ref()
                .map(|ids| ids.contains(*package))
                .unwrap_or(false);

            if is_installed && options.install_behavior == InstallBehavior::Skip {
                log::info!("Skipping already installed package: {}", package);
                continue;
            }

            if is_installed
                && matches!(
                    options.install_behavior,
                    InstallBehavior::Default | InstallBehavior::Upgrade
                )
            {
                self.upgrade(&[*package], options)?;
                continue;
            }

            let mut cmd = Command::new("winget");
            cmd.arg("install");
            Self::push_mutating_confirm_flags(&mut cmd, options);
            cmd.arg("--id");
            cmd.arg(package);
            cmd.arg("--exact");

            if options.force || options.install_behavior == InstallBehavior::Reinstall {
                cmd.arg("--force");
            }

            cmd.args(&options.extra_flags);
            executor.execute_interactive(cmd)?;
        }

        Ok(())
    }

    fn remove(&self, packages: &[&str], options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);

        for package in packages {
            let mut cmd = Command::new("winget");
            cmd.arg("uninstall");
            Self::push_mutating_confirm_flags(&mut cmd, options);
            cmd.arg("--id");
            cmd.arg(package);
            cmd.arg("--exact");
            cmd.args(&options.extra_flags);
            executor.execute_interactive(cmd)?;
        }

        Ok(())
    }

    fn update(&self, options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);

        let mut cmd = Command::new("winget");
        cmd.arg("source");
        cmd.arg("update");
        Self::push_mutating_confirm_flags(&mut cmd, options);
        cmd.args(&options.extra_flags);

        executor.execute_interactive(cmd)
    }

    fn upgrade(&self, packages: &[&str], options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);

        for package in packages {
            let mut cmd = Command::new("winget");
            cmd.arg("upgrade");
            Self::push_mutating_confirm_flags(&mut cmd, options);
            cmd.arg("--id");
            cmd.arg(package);
            cmd.arg("--exact");
            cmd.args(&options.extra_flags);
            executor.execute_interactive(cmd)?;
        }

        Ok(())
    }

    fn upgrade_all(&self, options: &OperationOptions) -> Result<()> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);

        let mut cmd = Command::new("winget");
        cmd.arg("upgrade");
        Self::push_mutating_confirm_flags(&mut cmd, options);
        cmd.arg("--all");
        cmd.args(&options.extra_flags);

        executor.execute_interactive(cmd)
    }

    fn search(&self, query: &str, options: &OperationOptions) -> Result<Vec<Package>> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);

        let mut cmd = Command::new("winget");
        cmd.arg("search");
        cmd.arg(query);
        Self::push_capture_mode_flags(&mut cmd, options);

        let output = executor.execute_capture(cmd)?;
        let rows = parse_table_rows(&output);

        let mut packages = Vec::new();
        for row in rows {
            let id = row.get("Id").cloned().unwrap_or_default();
            if id.is_empty() {
                continue;
            }

            let display_name = row.get("Name").cloned().unwrap_or_default();
            let version = row.get("Version").cloned().unwrap_or_default();
            let source = row.get("Source").cloned().unwrap_or_default();

            let mut pkg = Package::new(id, "winget".to_string());
            if !display_name.is_empty() {
                pkg = pkg.with_description(display_name);
            }
            if !version.is_empty() {
                pkg = pkg.with_version(version);
            }
            if !source.is_empty() {
                pkg = pkg.with_repository(source);
            }
            packages.push(pkg);
        }

        Ok(packages)
    }

    fn list_installed(&self, options: &OperationOptions) -> Result<Vec<Package>> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);

        let mut cmd = Command::new("winget");
        cmd.arg("list");
        Self::push_capture_mode_flags(&mut cmd, options);

        let output = executor.execute_capture(cmd)?;
        let rows = parse_table_rows(&output);

        let mut packages = Vec::new();
        for row in rows {
            let id = row.get("Id").cloned().unwrap_or_default();
            if id.is_empty() {
                continue;
            }

            let display_name = row.get("Name").cloned().unwrap_or_default();
            let version = row.get("Version").cloned().unwrap_or_default();
            let source = row.get("Source").cloned().unwrap_or_default();

            let mut pkg = Package::new(id, "winget".to_string()).with_installed(true);
            if !display_name.is_empty() {
                pkg = pkg.with_description(display_name);
            }
            if !version.is_empty() {
                pkg = pkg.with_version(version);
            }
            if !source.is_empty() {
                pkg = pkg.with_repository(source);
            }
            packages.push(pkg);
        }

        Ok(packages)
    }

    fn info(&self, package: &str, options: &OperationOptions) -> Result<Package> {
        let executor = CommandExecutor::new(options.dry_run, options.verbose)
            .with_show_commands(options.show_commands);

        let mut cmd = Command::new("winget");
        cmd.arg("show");
        cmd.arg("--id");
        cmd.arg(package);
        cmd.arg("--exact");
        Self::push_capture_mode_flags(&mut cmd, options);

        let output = executor.execute_capture(cmd)?;

        let mut pkg = Package::new(package.to_string(), "winget".to_string());
        for line in output.lines() {
            let Some((k, v)) = line.split_once(':') else {
                continue;
            };
            let key = k.trim().to_lowercase();
            let val = v.trim();
            if val.is_empty() {
                continue;
            }
            match key.as_str() {
                "version" => pkg.version = Some(val.to_string()),
                "description" => pkg.description = Some(val.to_string()),
                "homepage" => pkg.metadata.homepage = Some(val.to_string()),
                _ => {}
            }
        }

        Ok(pkg)
    }

    fn clean(&self, _options: &OperationOptions) -> Result<()> {
        unsupported("winget", "clean")
    }

    fn files(&self, _package: &str, _options: &OperationOptions) -> Result<Vec<String>> {
        unsupported("winget", "files")
    }

    fn check(&self, _options: &OperationOptions) -> Result<()> {
        unsupported("winget", "check")
    }

    fn autoremove(&self, _options: &OperationOptions) -> Result<()> {
        unsupported("winget", "autoremove")
    }

    fn owns(&self, _file_or_command: &str, _options: &OperationOptions) -> Result<String> {
        unsupported("winget", "owns")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_winget_creation() {
        let winget = Winget::new();
        assert_eq!(winget.name(), "winget");
    }

    #[test]
    fn test_operation_support() {
        let winget = Winget::new();
        assert!(winget.supports_operation(Operation::Install));
        assert!(winget.supports_operation(Operation::Search));
        assert!(!winget.supports_operation(Operation::Owns));
    }
}
