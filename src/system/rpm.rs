use crate::error::{RcleanerError, Result};
use crate::system::package_manager::{PackageManager, command_failed, run_command};

pub struct RpmManager;

impl RpmManager {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManager for RpmManager {
    fn name(&self) -> &str {
        "rpm"
    }

    fn version(&self) -> Result<String> {
        let output = run_command("rpm", &["--version"])?;
        if !output.status.success() {
            return Err(command_failed("rpm", &output));
        }
        Ok(first_line(&output.stdout))
    }

    fn list_installed(&self) -> Result<Vec<String>> {
        let output = run_command("rpm", &["-qa", "--qf", "%{NAME}\n"])?;
        if !output.status.success() {
            return Err(command_failed("rpm", &output));
        }
        Ok(split_lines(&output.stdout))
    }

    fn check_dependencies(&self, package: &str) -> Result<Vec<String>> {
        let output = run_command("rpm", &["-q", "--whatrequires", package])?;
        if !output.status.success() {
            let stderr = output.stderr.to_lowercase();
            if stderr.contains("no package requires") || output.stdout.trim().is_empty() {
                return Ok(Vec::new());
            }
            return Err(command_failed("rpm", &output));
        }
        Ok(split_lines(&output.stdout))
    }

    fn remove_packages(&self, packages: &[String], dry_run: bool) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        for package in packages {
            let deps = self.check_dependencies(package)?;
            if !deps.is_empty() {
                return Err(RcleanerError::Dependency(format!(
                    "Package {package} is required by: {}",
                    deps.join(", ")
                )));
            }
        }

        let mut args: Vec<String> = vec!["-e".to_string()];
        if dry_run {
            args.push("--test".to_string());
        }
        args.extend(packages.iter().cloned());
        let args_ref = args.iter().map(String::as_str).collect::<Vec<_>>();

        let output = run_command("rpm", &args_ref)?;
        if !output.status.success() {
            return Err(command_failed("rpm", &output));
        }

        Ok(())
    }
}

pub fn list_installed() -> Result<Vec<String>> {
    RpmManager::new().list_installed()
}

pub fn remove_packages(packages: &[String], dry_run: bool) -> Result<()> {
    RpmManager::new().remove_packages(packages, dry_run)
}

fn first_line(output: &str) -> String {
    output.lines().next().unwrap_or_default().trim().to_string()
}

fn split_lines(output: &str) -> Vec<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect()
}
