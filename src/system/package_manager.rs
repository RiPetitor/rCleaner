use crate::error::{RcleanerError, Result};
use std::process::{Command, ExitStatus};

pub trait PackageManager {
    fn name(&self) -> &str;
    fn version(&self) -> Result<String>;
    fn list_installed(&self) -> Result<Vec<String>>;
    fn check_dependencies(&self, package: &str) -> Result<Vec<String>>;
    fn remove_packages(&self, packages: &[String], dry_run: bool) -> Result<()>;
}

pub(crate) struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub status: ExitStatus,
}

pub(crate) fn run_command(program: &str, args: &[&str]) -> Result<CommandOutput> {
    let output = Command::new(program).args(args).output()?;
    Ok(CommandOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        status: output.status,
    })
}

pub(crate) fn command_failed(program: &str, output: &CommandOutput) -> RcleanerError {
    let message = if output.stderr.trim().is_empty() {
        output.stdout.trim().to_string()
    } else {
        output.stderr.trim().to_string()
    };
    RcleanerError::Command(format!("{program} command failed: {message}"))
}
