use crate::error::Result;
use crate::system::package_manager::{PackageManager, command_failed, run_command};
use std::path::Path;

pub struct FlatpakManager;

impl FlatpakManager {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManager for FlatpakManager {
    fn name(&self) -> &str {
        "flatpak"
    }

    fn version(&self) -> Result<String> {
        let output = run_command("flatpak", &["--version"])?;
        if !output.status.success() {
            return Err(command_failed("flatpak", &output));
        }
        Ok(first_line(&output.stdout))
    }

    fn list_installed(&self) -> Result<Vec<String>> {
        let output = run_command("flatpak", &["list", "--app", "--columns=application"])?;
        if !output.status.success() {
            return Err(command_failed("flatpak", &output));
        }
        Ok(split_lines(&output.stdout))
    }

    fn check_dependencies(&self, _package: &str) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    fn remove_packages(&self, packages: &[String], dry_run: bool) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        let mut args = vec!["uninstall", "-y"];
        if dry_run {
            args.push("--dry-run");
        }
        let package_args = packages.iter().map(String::as_str).collect::<Vec<_>>();
        let mut combined = args;
        combined.extend(package_args);

        let output = run_command("flatpak", &combined)?;
        if !output.status.success() {
            return Err(command_failed("flatpak", &output));
        }

        Ok(())
    }
}

pub fn is_flatpak_available() -> bool {
    Path::new("/usr/bin/flatpak").exists() || Path::new("/usr/local/bin/flatpak").exists()
}

pub fn list_installed() -> Result<Vec<String>> {
    FlatpakManager::new().list_installed()
}

pub fn remove_packages(packages: &[String], dry_run: bool) -> Result<()> {
    FlatpakManager::new().remove_packages(packages, dry_run)
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
