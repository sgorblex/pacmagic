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

    /// Format a command for display
    /// Note: Command's Debug format is: "program" "arg1" "arg2" ...
    /// We need to extract and unquote each part
    fn format_command(&self, command: &Command) -> String {
        let debug_str = format!("{:?}", command);

        // The Debug format is simply: "program" "arg1" "arg2" ...
        // We need to extract all quoted strings and join them with spaces
        let mut parts = Vec::new();
        let mut chars = debug_str.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '"' {
                // Found start of quoted string, collect until closing quote
                let mut part = String::new();
                while let Some(next_ch) = chars.next() {
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
            // Fallback to original if no quotes found
            debug_str
        } else {
            // Join all parts with spaces
            parts.join(" ")
        }
    }

    /// Print command if show_commands is enabled with pretty formatting
    fn print_command(&self, command: &Command, prefix: &str) {
        if self.show_commands {
            let cmd_str = self.format_command(command);
            // Use pretty formatting with emojis for better visibility
            if self.dry_run {
                eprintln!("🔍 {}: {}", prefix, cmd_str);
            } else {
                eprintln!("⚡ {}: {}", prefix, cmd_str);
            }
        }
    }

    /// Execute a command and return the output
    pub fn execute(&self, mut command: Command) -> Result<Output> {
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

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);

            return Err(PacMagicError::ExecutionError(format!(
                "Command failed with exit code {:?}\nStdout: {}\nStderr: {}",
                output.status.code(),
                stdout,
                stderr
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
        let output = self.execute(command)?;
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
