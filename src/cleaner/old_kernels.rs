use crate::backup::BackupManager;
use crate::cleaner::base::Cleaner;
use crate::error::Result;
use crate::models::{CleanupCategory, CleanupItem, CleanupResult, CleanupSource};
use crate::system::{apt, rpm};
use std::collections::HashSet;

pub struct OldKernelsCleaner;

impl OldKernelsCleaner {
    pub fn new() -> Self {
        Self {}
    }
}

impl Cleaner for OldKernelsCleaner {
    fn name(&self) -> &str {
        "Old Kernels Cleaner"
    }

    fn category(&self) -> CleanupCategory {
        CleanupCategory::OldKernels
    }

    fn scan(&self) -> Result<Vec<CleanupItem>> {
        let mut items = Vec::new();

        let current_kernel = current_kernel_version();
        let mut seen = HashSet::new();

        items.extend(scan_rpm_kernels(&current_kernel, &mut seen)?);
        items.extend(scan_apt_kernels(&current_kernel, &mut seen)?);

        Ok(items)
    }

    fn clean(&self, items: &[CleanupItem], dry_run: bool) -> Result<CleanupResult> {
        let mut result = CleanupResult::default();
        let mut rpm_packages = Vec::new();
        let mut apt_packages = Vec::new();

        if !dry_run {
            let manager = BackupManager::from_config()?;
            let _backup = manager.create_backup(items)?;
        }

        for item in items {
            if !self.can_clean(item) {
                result.skipped_items += 1;
                continue;
            }

            match &item.source {
                CleanupSource::PackageManager(manager) => match manager.as_str() {
                    "rpm" => rpm_packages.push(item.name.clone()),
                    "apt" => apt_packages.push(item.name.clone()),
                    _ => result.skipped_items += 1,
                },
                _ => result.skipped_items += 1,
            }
        }

        if !rpm_packages.is_empty() {
            rpm::remove_packages(&rpm_packages, dry_run)?;
            result.cleaned_items += rpm_packages.len();
        }

        if !apt_packages.is_empty() {
            apt::remove_packages(&apt_packages, dry_run)?;
            result.cleaned_items += apt_packages.len();
        }

        Ok(result)
    }
}

fn current_kernel_version() -> String {
    let output = std::process::Command::new("uname").arg("-r").output();
    if let Ok(output) = output {
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
    }
    String::new()
}

fn scan_rpm_kernels(current: &str, seen: &mut HashSet<String>) -> Result<Vec<CleanupItem>> {
    let output = std::process::Command::new("rpm")
        .args(["-q", "kernel", "kernel-core", "kernel-modules"])
        .output();

    let Ok(output) = output else {
        return Ok(Vec::new());
    };

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut items = Vec::new();
    for line in stdout.lines() {
        let pkg = line.trim();
        if pkg.is_empty() || (current.len() > 0 && pkg.contains(current)) {
            continue;
        }
        if !seen.insert(pkg.to_string()) {
            continue;
        }
        items.push(CleanupItem {
            id: format!("rpm:{pkg}"),
            name: pkg.to_string(),
            path: None,
            size: 0,
            description: "Old kernel package (RPM)".to_string(),
            category: CleanupCategory::OldKernels,
            source: CleanupSource::PackageManager("rpm".to_string()),
            selected: false,
            can_clean: true,
            blocked_reason: None,
            dependencies: Vec::new(),
        });
    }

    Ok(items)
}

fn scan_apt_kernels(current: &str, seen: &mut HashSet<String>) -> Result<Vec<CleanupItem>> {
    let output = std::process::Command::new("dpkg")
        .args(["-l", "linux-image-*"])
        .output();

    let Ok(output) = output else {
        return Ok(Vec::new());
    };

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut items = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        if !line.starts_with("ii") {
            continue;
        }
        let mut parts = line.split_whitespace();
        let _status = parts.next();
        let pkg = match parts.next() {
            Some(pkg) => pkg,
            None => continue,
        };
        if pkg.contains("linux-image") {
            if !current.is_empty() && pkg.contains(current) {
                continue;
            }
            if !seen.insert(pkg.to_string()) {
                continue;
            }
            items.push(CleanupItem {
                id: format!("apt:{pkg}"),
                name: pkg.to_string(),
                path: None,
                size: 0,
                description: "Old kernel package (APT)".to_string(),
                category: CleanupCategory::OldKernels,
                source: CleanupSource::PackageManager("apt".to_string()),
                selected: false,
                can_clean: true,
                blocked_reason: None,
                dependencies: Vec::new(),
            });
        }
    }

    Ok(items)
}
