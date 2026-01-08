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
            if is_no_requires_message(&output.stderr, package)
                || is_no_requires_message(&output.stdout, package)
            {
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

pub(crate) fn is_no_requires_message(stderr: &str, package: &str) -> bool {
    let lower = stderr.to_lowercase();
    let markers = [
        "no package requires",
        "no packages require",
        "ни один из пакетов не требует",
        "не требует",
        "не требуется",
    ];

    if !markers.iter().any(|marker| lower.contains(marker)) {
        return false;
    }

    let package_lower = package.to_lowercase();
    if package_lower.is_empty() {
        return true;
    }

    if lower.contains(&package_lower) {
        return true;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::is_no_requires_message;

    #[test]
    fn test_is_no_requires_message_english() {
        let msg = "no package requires libnsl";
        assert!(is_no_requires_message(msg, "libnsl"));
    }

    #[test]
    fn test_is_no_requires_message_russian() {
        let msg = "ни один из пакетов не требует libnsl";
        assert!(is_no_requires_message(msg, "libnsl"));
    }

    #[test]
    fn test_is_no_requires_message_without_package() {
        let msg = "no package requires";
        assert!(is_no_requires_message(msg, "libnsl"));
    }

    #[test]
    fn test_is_no_requires_message_other_error() {
        let msg = "error: package libnsl is not installed";
        assert!(!is_no_requires_message(msg, "libnsl"));
    }
}
