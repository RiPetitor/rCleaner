use crate::cleaner::base::Cleaner;
use crate::cleaner::old_packages::{query_dpkg_sizes, query_pacman_sizes};
use crate::error::Result;
use crate::models::{CleanupCategory, CleanupItem, CleanupResult, CleanupSource};
use crate::system::{apt, dnf};

pub struct SystemPackagesCleaner;

impl Default for SystemPackagesCleaner {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemPackagesCleaner {
    pub fn new() -> Self {
        Self {}
    }
}

impl Cleaner for SystemPackagesCleaner {
    fn name(&self) -> &str {
        "System Packages Cleaner"
    }

    fn category(&self) -> CleanupCategory {
        CleanupCategory::SystemPackages
    }

    fn scan(&self) -> Result<Vec<CleanupItem>> {
        let mut items = Vec::new();

        items.extend(scan_apt_manual()?);
        items.extend(scan_dnf_user()?);
        items.extend(scan_pacman_explicit()?);

        Ok(items)
    }

    fn clean(&self, items: &[CleanupItem], dry_run: bool) -> Result<CleanupResult> {
        let mut result = CleanupResult::default();
        let mut apt_packages = Vec::new();
        let mut dnf_packages = Vec::new();
        let mut pacman_packages = Vec::new();

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
            // Autoremove orphaned dependencies
            run_autoremove("apt-get", &["autoremove", "-y"], dry_run);
        }

        if !dnf_packages.is_empty() {
            dnf::remove_packages(&dnf_packages, dry_run)?;
            result.cleaned_items += dnf_packages.len();
            result.freed_bytes += dnf_packages
                .iter()
                .filter_map(|n| item_sizes.get(n.as_str()))
                .sum::<u64>();
            run_autoremove("dnf", &["autoremove", "-y"], dry_run);
        }

        if !pacman_packages.is_empty() {
            // Use -Rns to remove package + unneeded deps
            if dry_run {
                log::info!("[DRY RUN] pacman -Rns {:?}", pacman_packages);
            } else {
                let mut args = vec!["-Rns", "--noconfirm"];
                let pkg_args: Vec<&str> = pacman_packages.iter().map(String::as_str).collect();
                args.extend(pkg_args);
                let output = std::process::Command::new("pacman").args(&args).output();
                if let Ok(output) = output {
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        result.errors.push(format!("pacman: {}", stderr.trim()));
                    }
                }
            }
            result.cleaned_items += pacman_packages.len();
            result.freed_bytes += pacman_packages
                .iter()
                .filter_map(|n| item_sizes.get(n.as_str()))
                .sum::<u64>();
        }

        Ok(result)
    }
}

fn run_autoremove(cmd: &str, args: &[&str], dry_run: bool) {
    if dry_run {
        log::info!("[DRY RUN] {} {:?}", cmd, args);
        return;
    }
    let _ = std::process::Command::new(cmd).args(args).output();
}

fn scan_apt_manual() -> Result<Vec<CleanupItem>> {
    let output = std::process::Command::new("apt-mark")
        .arg("showmanual")
        .output();

    let Ok(output) = output else {
        return Ok(Vec::new());
    };
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

    let sizes = query_dpkg_sizes(&pkg_names);

    let items = pkg_names
        .into_iter()
        .map(|pkg| {
            let size = sizes.get(pkg.as_str()).copied().unwrap_or(0);
            CleanupItem {
                id: format!("apt:{pkg}"),
                name: pkg,
                path: None,
                size,
                description: "APT package".to_string(),
                category: CleanupCategory::SystemPackages,
                source: CleanupSource::PackageManager("apt".to_string()),
                selected: false,
                can_clean: true,
                blocked_reason: None,
                dependencies: Vec::new(),
            }
        })
        .collect();

    Ok(items)
}

fn scan_dnf_user() -> Result<Vec<CleanupItem>> {
    let output = std::process::Command::new("dnf")
        .args([
            "repoquery",
            "--userinstalled",
            "--qf",
            "%{name} %{installsize}",
        ])
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
        items.push(CleanupItem {
            id: format!("dnf:{pkg}"),
            name: pkg.to_string(),
            path: None,
            size,
            description: "DNF package".to_string(),
            category: CleanupCategory::SystemPackages,
            source: CleanupSource::PackageManager("dnf".to_string()),
            selected: false,
            can_clean: true,
            blocked_reason: None,
            dependencies: Vec::new(),
        });
    }

    Ok(items)
}

fn scan_pacman_explicit() -> Result<Vec<CleanupItem>> {
    let output = std::process::Command::new("pacman").args(["-Qeq"]).output();

    let Ok(output) = output else {
        return Ok(Vec::new());
    };
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

    let sizes = query_pacman_sizes(&pkg_names);

    let items = pkg_names
        .into_iter()
        .map(|pkg| {
            let size = sizes.get(pkg.as_str()).copied().unwrap_or(0);
            CleanupItem {
                id: format!("pacman:{pkg}"),
                name: pkg,
                path: None,
                size,
                description: "Pacman package".to_string(),
                category: CleanupCategory::SystemPackages,
                source: CleanupSource::PackageManager("pacman".to_string()),
                selected: false,
                can_clean: true,
                blocked_reason: None,
                dependencies: Vec::new(),
            }
        })
        .collect();

    Ok(items)
}
