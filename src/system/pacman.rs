use crate::error::{RcleanerError, Result};
use crate::system::package_manager::{PackageManager, command_failed, run_command};

pub struct PacmanManager;

impl PacmanManager {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManager for PacmanManager {
    fn name(&self) -> &str {
        "pacman"
    }

    fn version(&self) -> Result<String> {
        let output = run_command("pacman", &["-V"])?;
        if !output.status.success() {
            return Err(command_failed("pacman", &output));
        }
        Ok(first_line(&output.stdout))
    }

    fn list_installed(&self) -> Result<Vec<String>> {
        let output = run_command("pacman", &["-Qq"])?;
        if !output.status.success() {
            return Err(command_failed("pacman", &output));
        }
        Ok(split_lines(&output.stdout))
    }

    fn check_dependencies(&self, package: &str) -> Result<Vec<String>> {
        let output = run_command("pacman", &["-Qi", package])?;
        if !output.status.success() {
            return Err(command_failed("pacman", &output));
        }
        Ok(parse_pacman_required_by(&output.stdout))
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

        if dry_run {
            log::info!("[DRY RUN] pacman -R {:?}", packages);
            return Ok(());
        }

        let args = vec!["-R", "--noconfirm"];
        let package_args = packages.iter().map(String::as_str).collect::<Vec<_>>();
        let mut combined = args;
        combined.extend(package_args);

        let output = run_command("pacman", &combined)?;
        if !output.status.success() {
            return Err(command_failed("pacman", &output));
        }

        Ok(())
    }
}

pub fn list_installed() -> Result<Vec<String>> {
    PacmanManager::new().list_installed()
}

pub fn remove_packages(packages: &[String], dry_run: bool) -> Result<()> {
    PacmanManager::new().remove_packages(packages, dry_run)
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

fn parse_pacman_required_by(output: &str) -> Vec<String> {
    for line in output.lines() {
        let line = line.trim();
        if line.starts_with("Required By") {
            if let Some((_, value)) = line.split_once(':') {
                let value = value.trim();
                if value == "None" || value.is_empty() {
                    return Vec::new();
                }
                return value.split_whitespace().map(String::from).collect();
            }
        }
    }
    Vec::new()
}
