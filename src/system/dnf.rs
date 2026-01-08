use crate::error::{RcleanerError, Result};
use crate::system::package_manager::{PackageManager, command_failed, run_command};

pub struct DnfManager;

impl DnfManager {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManager for DnfManager {
    fn name(&self) -> &str {
        "dnf"
    }

    fn version(&self) -> Result<String> {
        let output = run_command("dnf", &["--version"])?;
        if !output.status.success() {
            return Err(command_failed("dnf", &output));
        }
        Ok(first_line(&output.stdout))
    }

    fn list_installed(&self) -> Result<Vec<String>> {
        let output = run_command("dnf", &["list", "installed"])?;
        if !output.status.success() {
            return Err(command_failed("dnf", &output));
        }
        Ok(parse_dnf_list(&output.stdout))
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

        let mut args = vec!["remove", "-y"];
        if dry_run {
            args.push("--assumeno");
        }
        let package_args = packages.iter().map(String::as_str).collect::<Vec<_>>();
        let mut combined = args;
        combined.extend(package_args);

        let output = run_command("dnf", &combined)?;
        if !output.status.success() {
            return Err(command_failed("dnf", &output));
        }

        Ok(())
    }
}

pub fn list_installed() -> Result<Vec<String>> {
    DnfManager::new().list_installed()
}

pub fn remove_packages(packages: &[String], dry_run: bool) -> Result<()> {
    DnfManager::new().remove_packages(packages, dry_run)
}

fn first_line(output: &str) -> String {
    output.lines().next().unwrap_or_default().trim().to_string()
}

fn parse_dnf_list(output: &str) -> Vec<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| {
            !line.is_empty()
                && !line.starts_with("Installed")
                && !line.starts_with("Available")
                && !line.starts_with("Last metadata")
        })
        .filter_map(|line| line.split_whitespace().next())
        .map(String::from)
        .collect()
}

fn split_lines(output: &str) -> Vec<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect()
}
