use crate::error::Result;
use crate::system::package_manager::{PackageManager, command_failed, run_command};
use std::path::Path;

pub struct SnapManager;

impl SnapManager {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManager for SnapManager {
    fn name(&self) -> &str {
        "snap"
    }

    fn version(&self) -> Result<String> {
        let output = run_command("snap", &["version"])?;
        if !output.status.success() {
            return Err(command_failed("snap", &output));
        }
        Ok(first_line(&output.stdout))
    }

    fn list_installed(&self) -> Result<Vec<String>> {
        let output = run_command("snap", &["list"])?;
        if !output.status.success() {
            return Err(command_failed("snap", &output));
        }
        Ok(parse_snap_list(&output.stdout))
    }

    fn check_dependencies(&self, _package: &str) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    fn remove_packages(&self, packages: &[String], dry_run: bool) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        if dry_run {
            log::info!("[DRY RUN] snap remove {:?}", packages);
            return Ok(());
        }

        let args = vec!["remove"];
        let package_args = packages.iter().map(String::as_str).collect::<Vec<_>>();
        let mut combined = args;
        combined.extend(package_args);

        let output = run_command("snap", &combined)?;
        if !output.status.success() {
            return Err(command_failed("snap", &output));
        }

        Ok(())
    }
}

pub fn is_snap_available() -> bool {
    Path::new("/usr/bin/snap").exists() || Path::new("/usr/local/bin/snap").exists()
}

pub fn list_installed() -> Result<Vec<String>> {
    SnapManager::new().list_installed()
}

pub fn remove_packages(packages: &[String], dry_run: bool) -> Result<()> {
    SnapManager::new().remove_packages(packages, dry_run)
}

fn first_line(output: &str) -> String {
    output.lines().next().unwrap_or_default().trim().to_string()
}

fn parse_snap_list(output: &str) -> Vec<String> {
    output
        .lines()
        .skip(1)
        .filter_map(|line| line.split_whitespace().next())
        .map(String::from)
        .collect()
}
