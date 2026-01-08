use crate::error::Result;
use crate::system::package_manager::{PackageManager, command_failed, run_command};
use std::path::Path;

pub struct RpmOstreeManager;

impl RpmOstreeManager {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManager for RpmOstreeManager {
    fn name(&self) -> &str {
        "rpm-ostree"
    }

    fn version(&self) -> Result<String> {
        let output = run_command("rpm-ostree", &["--version"])?;
        if !output.status.success() {
            return Err(command_failed("rpm-ostree", &output));
        }
        Ok(first_line(&output.stdout))
    }

    fn list_installed(&self) -> Result<Vec<String>> {
        if let Ok(output) = run_command("rpm-ostree", &["status", "--json"]) {
            if output.status.success() {
                if let Ok(packages) = parse_rpm_ostree_json(&output.stdout) {
                    if !packages.is_empty() {
                        return Ok(packages);
                    }
                }
            }
        }

        let output = run_command("rpm-ostree", &["db", "list"])?;
        if !output.status.success() {
            return Err(command_failed("rpm-ostree", &output));
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
        if dry_run {
            log::info!("[DRY RUN] rpm-ostree uninstall {:?}", packages);
            return Ok(());
        }

        let args = packages.iter().map(String::as_str).collect::<Vec<_>>();
        let mut command_args = vec!["uninstall"];
        command_args.extend(args);

        let output = run_command("rpm-ostree", &command_args)?;
        if !output.status.success() {
            return Err(command_failed("rpm-ostree", &output));
        }

        Ok(())
    }
}

pub fn is_rpm_ostree_available() -> bool {
    Path::new("/usr/bin/rpm-ostree").exists() || Path::new("/usr/local/bin/rpm-ostree").exists()
}

pub fn list_installed() -> Result<Vec<String>> {
    RpmOstreeManager::new().list_installed()
}

pub fn remove_packages(packages: &[String], dry_run: bool) -> Result<()> {
    RpmOstreeManager::new().remove_packages(packages, dry_run)
}

fn parse_rpm_ostree_json(content: &str) -> Result<Vec<String>> {
    let value: serde_json::Value = serde_json::from_str(content)?;
    let mut packages = Vec::new();

    if let Some(list) = value
        .pointer("/deployments/0/requested-packages")
        .and_then(|v| v.as_array())
    {
        for entry in list {
            if let Some(pkg) = entry.as_str() {
                packages.push(pkg.to_string());
            }
        }
    }

    if packages.is_empty() {
        if let Some(list) = value
            .pointer("/deployments/0/packages")
            .and_then(|v| v.as_array())
        {
            for entry in list {
                if let Some(pkg) = entry.as_str() {
                    packages.push(pkg.to_string());
                }
            }
        }
    }

    Ok(packages)
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
