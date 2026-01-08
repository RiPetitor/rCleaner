use crate::error::Result;
use crate::system::package_manager::{PackageManager, command_failed, run_command};
use crate::utils::size_format::parse_size_string;
use std::path::Path;

pub struct FlatpakManager;

impl Default for FlatpakManager {
    fn default() -> Self {
        Self::new()
    }
}

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

/// Возвращает список установленных Flatpak приложений с размерами.
pub fn list_installed_with_sizes() -> Result<Vec<(String, u64)>> {
    // Колонка "size" — правильное имя для размера
    if let Ok(items) = list_with_columns("application,size")
        && !items.is_empty()
    {
        return Ok(items);
    }

    // Fallback без размеров
    Ok(list_installed()?.into_iter().map(|app| (app, 0)).collect())
}

fn list_with_columns(columns: &str) -> Result<Vec<(String, u64)>> {
    let args = [
        "list".to_string(),
        "--app".to_string(),
        format!("--columns={columns}"),
    ];
    let args_ref = args.iter().map(String::as_str).collect::<Vec<_>>();
    let output = run_command("flatpak", &args_ref)?;
    if !output.status.success() {
        return Err(command_failed("flatpak", &output));
    }

    let mut items = Vec::new();
    for line in output.stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let app = match parts.next() {
            Some(app) => app.to_string(),
            None => continue,
        };
        let size_text = parts.collect::<Vec<_>>().join(" ");
        let size = parse_size_string(&size_text).unwrap_or(0);
        items.push((app, size));
    }

    Ok(items)
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
