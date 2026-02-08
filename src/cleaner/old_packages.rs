// Copyright (C) 2026 RiPetitor
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::backup::BackupManager;
use crate::cleaner::base::Cleaner;
use crate::error::Result;
use crate::models::{CleanupCategory, CleanupItem, CleanupResult, CleanupSource};
use crate::system::{apt, dnf, pacman, rpm};

pub struct OldPackagesCleaner;

impl Default for OldPackagesCleaner {
    fn default() -> Self {
        Self::new()
    }
}

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
                    "apt" => apt_packages.push(item.name.clone()),
                    "dnf" => dnf_packages.push(item.name.clone()),
                    "pacman" => pacman_packages.push(item.name.clone()),
                    "rpm" => rpm_packages.push(item.name.clone()),
                    _ => result.skipped_items += 1,
                },
                _ => result.skipped_items += 1,
            }
        }

        let item_sizes: std::collections::HashMap<&str, u64> = items
            .iter()
            .map(|item| (item.name.as_str(), item.size))
            .collect();

        if !apt_packages.is_empty() {
            apt::remove_packages(&apt_packages, dry_run)?;
            result.cleaned_items += apt_packages.len();
            result.freed_bytes += apt_packages
                .iter()
                .filter_map(|n| item_sizes.get(n.as_str()))
                .sum::<u64>();
        }
        if !dnf_packages.is_empty() {
            dnf::remove_packages(&dnf_packages, dry_run)?;
            result.cleaned_items += dnf_packages.len();
            result.freed_bytes += dnf_packages
                .iter()
                .filter_map(|n| item_sizes.get(n.as_str()))
                .sum::<u64>();
        }
        if !pacman_packages.is_empty() {
            pacman::remove_packages(&pacman_packages, dry_run)?;
            result.cleaned_items += pacman_packages.len();
            result.freed_bytes += pacman_packages
                .iter()
                .filter_map(|n| item_sizes.get(n.as_str()))
                .sum::<u64>();
        }
        if !rpm_packages.is_empty() {
            rpm::remove_packages(&rpm_packages, dry_run)?;
            result.cleaned_items += rpm_packages.len();
            result.freed_bytes += rpm_packages
                .iter()
                .filter_map(|n| item_sizes.get(n.as_str()))
                .sum::<u64>();
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
    let mut pkg_names: Vec<String> = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("Remv ")
            && let Some(pkg) = rest.split_whitespace().next()
        {
            pkg_names.push(pkg.to_string());
        }
    }

    // Try to get installed sizes via dpkg-query
    let sizes = query_dpkg_sizes(&pkg_names);

    let items = pkg_names
        .into_iter()
        .map(|pkg| {
            let size = sizes.get(pkg.as_str()).copied().unwrap_or(0);
            make_package_item_with_size(&pkg, "APT autoremove candidate", "apt", size)
        })
        .collect();

    Ok(items)
}

pub(crate) fn query_dpkg_sizes(packages: &[String]) -> std::collections::HashMap<String, u64> {
    let mut sizes = std::collections::HashMap::new();
    if packages.is_empty() {
        return sizes;
    }

    let mut args = vec![
        "-W".to_string(),
        "--showformat=${Package} ${Installed-Size}\n".to_string(),
    ];
    args.extend(packages.iter().cloned());

    let output = std::process::Command::new("dpkg-query")
        .args(&args)
        .output();
    let Ok(output) = output else {
        return sizes;
    };

    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 2
            && let Ok(kb) = parts[1].parse::<u64>()
        {
            // dpkg reports size in KB
            sizes.insert(parts[0].to_string(), kb * 1024);
        }
    }
    sizes
}

fn scan_dnf_unneeded() -> Result<Vec<CleanupItem>> {
    // Get unneeded packages with their sizes
    let output = std::process::Command::new("dnf")
        .args(["repoquery", "--unneeded", "--qf", "%{name} %{installsize}"])
        .output();

    let Ok(output) = output else {
        return Ok(Vec::new());
    };
    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut seen = std::collections::HashSet::new();
    let mut items = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.rsplitn(2, ' ').collect();
        let (pkg, size) = if parts.len() == 2 {
            let size = parts[0].parse::<u64>().unwrap_or(0);
            (parts[1], size)
        } else {
            (line, 0)
        };
        if pkg.is_empty() || !seen.insert(pkg.to_string()) {
            continue;
        }
        items.push(make_package_item_with_size(
            pkg,
            "DNF unneeded package",
            "dnf",
            size,
        ));
    }

    // Also check dnf autoremove candidates
    let autoremove = std::process::Command::new("dnf")
        .args(["autoremove", "--assumeno", "-q"])
        .output();

    if let Ok(output) = autoremove
        && output.status.success()
    {
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let pkg = line.split_whitespace().next().unwrap_or("");
            if !pkg.is_empty() && seen.insert(pkg.to_string()) {
                items.push(make_package_item(pkg, "DNF autoremove candidate", "dnf"));
            }
        }
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
    // pacman -Qtdq returns exit code 1 when no orphans found
    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let pkg_names: Vec<String> = stdout
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect();

    // Get installed sizes via pacman -Qi
    let sizes = query_pacman_sizes(&pkg_names);

    let mut items: Vec<CleanupItem> = pkg_names
        .into_iter()
        .map(|pkg| {
            let size = sizes.get(pkg.as_str()).copied().unwrap_or(0);
            make_package_item_with_size(&pkg, "Pacman orphaned package", "pacman", size)
        })
        .collect();

    // Also scan pacman package cache for old versions
    items.extend(scan_pacman_cache()?);

    Ok(items)
}

pub(crate) fn query_pacman_sizes(packages: &[String]) -> std::collections::HashMap<String, u64> {
    let mut sizes = std::collections::HashMap::new();
    for pkg in packages {
        let output = std::process::Command::new("pacman")
            .args(["-Qi", pkg])
            .env("LANG", "C")
            .output();
        let Ok(output) = output else { continue };
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("Installed Size")
                && let Some((_, value)) = rest.split_once(':')
                && let Some(bytes) = parse_human_size(value.trim())
            {
                sizes.insert(pkg.clone(), bytes);
            }
        }
    }
    sizes
}

pub(crate) fn parse_human_size(s: &str) -> Option<u64> {
    let s = s.trim();
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 2 {
        return None;
    }
    let num: f64 = parts[0].replace(',', ".").parse().ok()?;
    let multiplier = match parts[1].to_uppercase().as_str() {
        "B" => 1.0,
        "KIB" | "KB" => 1024.0,
        "MIB" | "MB" => 1024.0 * 1024.0,
        "GIB" | "GB" => 1024.0 * 1024.0 * 1024.0,
        _ => return None,
    };
    Some((num * multiplier) as u64)
}

fn scan_pacman_cache() -> Result<Vec<CleanupItem>> {
    // Scan /var/cache/pacman/pkg for old package versions
    let cache_dir = std::path::Path::new("/var/cache/pacman/pkg");
    if !cache_dir.exists() {
        return Ok(Vec::new());
    }

    let total_size: u64 = std::fs::read_dir(cache_dir)?
        .flatten()
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum();

    if total_size == 0 {
        return Ok(Vec::new());
    }

    Ok(vec![CleanupItem {
        id: "pacman:cache".to_string(),
        name: "Pacman package cache".to_string(),
        path: Some("/var/cache/pacman/pkg".to_string()),
        size: total_size,
        description: "Cached package archives".to_string(),
        category: CleanupCategory::OldPackages,
        source: CleanupSource::FileSystem,
        selected: false,
        can_clean: true,
        blocked_reason: None,
        dependencies: Vec::new(),
    }])
}

fn make_package_item(name: &str, description: &str, manager: &str) -> CleanupItem {
    make_package_item_with_size(name, description, manager, 0)
}

fn make_package_item_with_size(
    name: &str,
    description: &str,
    manager: &str,
    size: u64,
) -> CleanupItem {
    CleanupItem {
        id: format!("{manager}:{name}"),
        name: name.to_string(),
        path: None,
        size,
        description: description.to_string(),
        category: CleanupCategory::OldPackages,
        source: CleanupSource::PackageManager(manager.to_string()),
        selected: false,
        can_clean: true,
        blocked_reason: None,
        dependencies: Vec::new(),
    }
}
