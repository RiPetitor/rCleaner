use crate::cleaner::base::Cleaner;
use crate::error::Result;
use crate::models::{CleanupCategory, CleanupItem, CleanupResult, CleanupSource};
use crate::system::{apt, dnf, pacman, rpm};

pub struct OldPackagesCleaner;

impl OldPackagesCleaner {
    pub fn new() -> Self {
        Self {}
    }
}

impl Cleaner for OldPackagesCleaner {
    fn name(&self) -> &str {
        "Old Packages Cleaner"
    }

    fn category(&self) -> CleanupCategory {
        CleanupCategory::OldPackages
    }

    fn scan(&self) -> Result<Vec<CleanupItem>> {
        let mut items = Vec::new();

        items.extend(scan_apt_autoremove()?);
        items.extend(scan_dnf_unneeded()?);
        items.extend(scan_pacman_orphans()?);

        Ok(items)
    }

    fn clean(&self, items: &[CleanupItem], dry_run: bool) -> Result<CleanupResult> {
        let mut result = CleanupResult::default();
        let mut apt_packages = Vec::new();
        let mut dnf_packages = Vec::new();
        let mut pacman_packages = Vec::new();
        let mut rpm_packages = Vec::new();

        for item in items {
            if !self.can_clean(item) {
                result.skipped_items += 1;
                continue;
            }

            match &item.source {
                CleanupSource::PackageManager(manager) => match manager.as_str() {
                    "apt" => apt_packages.push(item.name.clone()),
                    "dnf" => dnf_packages.push(item.name.clone()),
                    "pacman" => pacman_packages.push(item.name.clone()),
                    "rpm" => rpm_packages.push(item.name.clone()),
                    _ => result.skipped_items += 1,
                },
                _ => result.skipped_items += 1,
            }
        }

        if !apt_packages.is_empty() {
            apt::remove_packages(&apt_packages, dry_run)?;
            result.cleaned_items += apt_packages.len();
        }
        if !dnf_packages.is_empty() {
            dnf::remove_packages(&dnf_packages, dry_run)?;
            result.cleaned_items += dnf_packages.len();
        }
        if !pacman_packages.is_empty() {
            pacman::remove_packages(&pacman_packages, dry_run)?;
            result.cleaned_items += pacman_packages.len();
        }
        if !rpm_packages.is_empty() {
            rpm::remove_packages(&rpm_packages, dry_run)?;
            result.cleaned_items += rpm_packages.len();
        }

        Ok(result)
    }
}

fn scan_apt_autoremove() -> Result<Vec<CleanupItem>> {
    let output = std::process::Command::new("apt-get")
        .args(["-s", "autoremove"])
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
        if let Some(rest) = line.strip_prefix("Remv ") {
            if let Some(pkg) = rest.split_whitespace().next() {
                items.push(make_package_item(pkg, "APT autoremove candidate", "apt"));
            }
        }
    }

    Ok(items)
}

fn scan_dnf_unneeded() -> Result<Vec<CleanupItem>> {
    let output = std::process::Command::new("dnf")
        .args(["repoquery", "--unneeded", "--qf", "%{name}"])
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
        if pkg.is_empty() {
            continue;
        }
        items.push(make_package_item(pkg, "DNF unneeded package", "dnf"));
    }

    Ok(items)
}

fn scan_pacman_orphans() -> Result<Vec<CleanupItem>> {
    let output = std::process::Command::new("pacman")
        .args(["-Qtdq"])
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
        if pkg.is_empty() {
            continue;
        }
        items.push(make_package_item(pkg, "Pacman orphaned package", "pacman"));
    }

    Ok(items)
}

fn make_package_item(name: &str, description: &str, manager: &str) -> CleanupItem {
    CleanupItem {
        id: format!("{manager}:{name}"),
        name: name.to_string(),
        path: None,
        size: 0,
        description: description.to_string(),
        category: CleanupCategory::OldPackages,
        source: CleanupSource::PackageManager(manager.to_string()),
        selected: false,
        can_clean: true,
        dependencies: Vec::new(),
    }
}
