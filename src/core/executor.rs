use crate::core::error::{PacMagicError, Result};
use log::{debug, info};
use std::process::{Command, Output, Stdio};

/// Handles command execution with consistent error handling
pub struct CommandExecutor {
    dry_run: bool,
    verbose: bool,
    show_commands: bool,
}

impl CommandExecutor {
    pub fn new(dry_run: bool, verbose: bool) -> Self {
        Self {
            dry_run,
            verbose,
            show_commands: false,
        }
    }

    pub fn with_show_commands(mut self, show: bool) -> Self {
        self.show_commands = show;
        self
    }

    fn format_command(&self, command: &Command) -> String {
        let debug_str = format!("{:?}", command);
        let mut parts = Vec::new();
        let mut chars = debug_str.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '"' {
                let mut part = String::new();
                for next_ch in chars.by_ref() {
                    if next_ch == '"' {
                        break;
                    }
                    part.push(next_ch);
                }
                if !part.is_empty() {
                    parts.push(part);
                }
            }
        }

        if parts.is_empty() {
            debug_str
        } else {
            parts.join(" ")
        }
    }

    fn print_command(&self, command: &Command, prefix: &str) {
        if self.show_commands {
            let cmd_str = self.format_command(command);
            if self.dry_run {
                eprintln!("🔍 {}: {}", prefix, cmd_str);
            } else {
                eprintln!("⚡ {}: {}", prefix, cmd_str);
            }
        }
    }

    /// Execute a command and return the output
    pub fn execute(&self, command: Command) -> Result<Output> {
        self.execute_accepting(command, &[])
    }

    /// Execute a command and return the output, treating exit status 0 and
    /// any of `extra_ok_codes` as success
    fn execute_accepting(&self, mut command: Command, extra_ok_codes: &[i32]) -> Result<Output> {
        if self.verbose {
            info!("Executing: {:?}", command);
        }

        if self.dry_run {
            self.print_command(&command, "Would execute");
            return Ok(Output {
                status: std::process::ExitStatus::default(),
                stdout: b"[dry run - no output]".to_vec(),
                stderr: Vec::new(),
            });
        }

        self.print_command(&command, "Executing");
        debug!("Running command: {:?}", command);

        let output = command.stdin(Stdio::null()).output().map_err(|e| {
            PacMagicError::ExecutionError(format!("Failed to execute command: {}", e))
        })?;

        let accepted = output
            .status
            .code()
            .is_some_and(|code| extra_ok_codes.contains(&code));
        if !output.status.success() && !accepted {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);

            if !stdout.is_empty() {
                print!("{}", stdout);
            }
            if !stderr.is_empty() {
                eprint!("{}", stderr);
            }

            return Err(PacMagicError::ExecutionError(format!(
                "Command failed with exit code {:?}",
                output.status.code()
            )));
        }

        if self.verbose {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.is_empty() {
                info!("Output: {}", stdout);
            }
        }

        Ok(output)
    }

    /// Execute a command and return success/failure
    pub fn execute_checked(&self, command: Command) -> Result<()> {
        self.execute(command)?;
        Ok(())
    }

    /// Execute a command and return stdout as a string
    pub fn execute_capture(&self, command: Command) -> Result<String> {
        self.execute_capture_accepting(command, &[])
    }

    /// Execute a command and return stdout as a string, treating exit status 0
    /// and any of `extra_ok_codes` as success (e.g. for query commands that
    /// exit non-zero when there are no results)
    pub fn execute_capture_accepting(
        &self,
        command: Command,
        extra_ok_codes: &[i32],
    ) -> Result<String> {
        let output = self.execute_accepting(command, extra_ok_codes)?;
        let stdout = String::from_utf8(output.stdout).map_err(|e| {
            PacMagicError::ExecutionError(format!("Invalid UTF-8 in output: {}", e))
        })?;
        Ok(stdout)
    }

    /// Execute a command interactively (inheriting stdio)
    pub fn execute_interactive(&self, mut command: Command) -> Result<()> {
        if self.verbose {
            info!("Executing interactively: {:?}", command);
        }

        if self.dry_run {
            self.print_command(&command, "Would execute interactively");
            return Ok(());
        }

        self.print_command(&command, "Executing interactively");
        debug!("Running interactive command: {:?}", command);

        let status = command
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| {
                PacMagicError::ExecutionError(format!("Failed to execute command: {}", e))
            })?;

        if !status.success() {
            return Err(PacMagicError::ExecutionError(format!(
                "Command failed with exit code {:?}",
                status.code()
            )));
        }

        Ok(())
    }
}

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new(false, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_executor_new() {
        let executor = CommandExecutor::new(false, false);
        assert!(!executor.dry_run);
        assert!(!executor.verbose);
        assert!(!executor.show_commands);
    }

    #[test]
    fn test_command_executor_with_dry_run() {
        let executor = CommandExecutor::new(true, false);
        assert!(executor.dry_run);
    }

    #[test]
    fn test_command_executor_with_verbose() {
        let executor = CommandExecutor::new(false, true);
        assert!(executor.verbose);
    }

    #[test]
    fn test_command_executor_with_show_commands() {
        let executor = CommandExecutor::new(false, false).with_show_commands(true);
        assert!(executor.show_commands);
    }

    #[test]
    fn test_command_executor_default() {
        let executor = CommandExecutor::default();
        assert!(!executor.dry_run);
        assert!(!executor.verbose);
    }

    #[test]
    fn test_command_executor_dry_run() {
        let executor = CommandExecutor::new(true, false);
        let mut cmd = Command::new("echo");
        cmd.arg("test");
        let output = executor.execute(cmd).unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("dry run"));
    }

    #[test]
    fn test_command_executor_execute_success() {
        let executor = CommandExecutor::new(false, false);
        let mut cmd = Command::new("echo");
        cmd.arg("test");
        let output = executor.execute(cmd).unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.trim() == "test");
    }

    #[test]
    fn test_command_executor_execute_checked() {
        let executor = CommandExecutor::new(false, false);
        let cmd = Command::new("true");
        assert!(executor.execute_checked(cmd).is_ok());
    }

    #[test]
    fn test_command_executor_execute_checked_failure() {
        let executor = CommandExecutor::new(false, false);
        let cmd = Command::new("false");
        assert!(executor.execute_checked(cmd).is_err());
    }

    #[test]
    fn test_command_executor_execute_capture() {
        let executor = CommandExecutor::new(false, false);
        let mut cmd = Command::new("echo");
        cmd.arg("hello world");
        let output = executor.execute_capture(cmd).unwrap();
        assert_eq!(output.trim(), "hello world");
    }

    #[test]
    fn test_command_executor_execute_capture_accepting_listed_code() {
        let executor = CommandExecutor::new(false, false);
        let mut cmd = Command::new("sh");
        cmd.args(["-c", "exit 1"]);
        let output = executor.execute_capture_accepting(cmd, &[1]).unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn test_command_executor_execute_capture_accepting_unlisted_code() {
        let executor = CommandExecutor::new(false, false);
        let mut cmd = Command::new("sh");
        cmd.args(["-c", "exit 2"]);
        assert!(executor.execute_capture_accepting(cmd, &[1]).is_err());
    }

    #[test]
    fn test_command_executor_execute_interactive_dry_run() {
        let executor = CommandExecutor::new(true, false);
        let mut cmd = Command::new("echo");
        cmd.arg("test");
        assert!(executor.execute_interactive(cmd).is_ok());
    }

    #[test]
    fn test_command_executor_format_command() {
        let executor = CommandExecutor::new(false, false);
        let mut cmd = Command::new("echo");
        cmd.arg("test");
        cmd.arg("hello");
        let formatted = executor.format_command(&cmd);
        assert!(formatted.contains("echo"));
        assert!(formatted.contains("test"));
        assert!(formatted.contains("hello"));
    }

    #[test]
    fn test_command_executor_execute_nonexistent_command() {
        let executor = CommandExecutor::new(false, false);
        let cmd = Command::new("nonexistent_command_xyz123");
        assert!(executor.execute(cmd).is_err());
    }

    #[test]
    fn test_command_executor_execute_capture_empty_output() {
        let executor = CommandExecutor::new(false, false);
        let cmd = Command::new("true");
        let output = executor.execute_capture(cmd).unwrap();
        assert!(output.trim().is_empty());
    }

    #[test]
    fn test_command_executor_builder_chaining() {
        let executor = CommandExecutor::new(true, true).with_show_commands(true);
        assert!(executor.dry_run);
        assert!(executor.verbose);
        assert!(executor.show_commands);
    }
}
