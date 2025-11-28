use crate::core::error::{PacMagicError, Result};
use log::{debug, info};
use std::process::{Command, Output, Stdio};

/// Handles command execution with consistent error handling
pub struct CommandExecutor {
    dry_run: bool,
    verbose: bool,
}

impl CommandExecutor {
    pub fn new(dry_run: bool, verbose: bool) -> Self {
        Self { dry_run, verbose }
    }

    /// Execute a command and return the output
    pub fn execute(&self, mut command: Command) -> Result<Output> {
        if self.verbose {
            info!("Executing: {:?}", command);
        }

        if self.dry_run {
            info!("[DRY RUN] Would execute: {:?}", command);
            return Ok(Output {
                status: std::process::ExitStatus::default(),
                stdout: b"[dry run - no output]".to_vec(),
                stderr: Vec::new(),
            });
        }

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
            info!("[DRY RUN] Would execute interactively: {:?}", command);
            return Ok(());
        }

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
