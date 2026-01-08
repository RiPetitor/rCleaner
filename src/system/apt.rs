use crate::error::{RcleanerError, Result};
use crate::system::package_manager::{PackageManager, command_failed, run_command};

pub struct AptManager;

impl Default for AptManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AptManager {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManager for AptManager {
    fn name(&self) -> &str {
        "apt"
    }

    fn version(&self) -> Result<String> {
        let output = run_command("apt", &["--version"])?;
        if !output.status.success() {
            return Err(command_failed("apt", &output));
        }
        Ok(first_line(&output.stdout))
    }

    fn list_installed(&self) -> Result<Vec<String>> {
        let output = run_command("dpkg-query", &["-W", "-f=${binary:Package}\n"])?;
        if !output.status.success() {
            return Err(command_failed("dpkg-query", &output));
        }
        Ok(split_lines(&output.stdout))
    }

    fn check_dependencies(&self, package: &str) -> Result<Vec<String>> {
        let output = run_command("apt-cache", &["rdepends", "--installed", package])?;
        if !output.status.success() {
            return Err(command_failed("apt-cache", &output));
        }
        let mut deps = parse_apt_rdepends(&output.stdout);
        deps.retain(|dep| dep != package);
        Ok(deps)
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

        let mut args = vec!["remove"];
        if dry_run {
            args.push("-s");
        } else {
            args.push("-y");
        }
        let package_args = packages.iter().map(String::as_str).collect::<Vec<_>>();
        let mut combined = args;
        combined.extend(package_args);

        let output = run_command("apt-get", &combined)?;
        if !output.status.success() {
            return Err(command_failed("apt-get", &output));
        }

        Ok(())
    }
}

pub fn list_installed() -> Result<Vec<String>> {
    AptManager::new().list_installed()
}

pub fn remove_packages(packages: &[String], dry_run: bool) -> Result<()> {
    AptManager::new().remove_packages(packages, dry_run)
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

fn parse_apt_rdepends(output: &str) -> Vec<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| {
            !line.is_empty()
                && !line.starts_with("Reverse Depends")
                && !line.starts_with("Reverse Depends:")
        })
        .filter_map(|line| {
            let trimmed = line.trim_start_matches('|').trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .collect()
}
