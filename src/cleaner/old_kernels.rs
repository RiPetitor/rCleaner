use crate::cleaner::base::Cleaner;
use crate::config::Config;
use crate::error::Result;
use crate::models::{CleanupCategory, CleanupItem, CleanupResult, CleanupSource};
use crate::system::{apt, pacman, rpm};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::UNIX_EPOCH;

pub struct OldKernelsCleaner;

impl Default for OldKernelsCleaner {
    fn default() -> Self {
        Self::new()
    }
}

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

        let keep_recent = keep_recent_kernels();
        let current_kernel = current_kernel_version();

        items.extend(scan_rpm_kernels(&current_kernel, keep_recent)?);
        items.extend(scan_apt_kernels(&current_kernel, keep_recent)?);
        items.extend(scan_pacman_kernels(&current_kernel, keep_recent)?);
        items.extend(scan_modules_dir(&current_kernel, keep_recent)?);

        Ok(items)
    }

    fn clean(&self, items: &[CleanupItem], dry_run: bool) -> Result<CleanupResult> {
        let mut result = CleanupResult::default();
        let mut rpm_packages = Vec::new();
        let mut apt_packages = Vec::new();
        let mut pacman_packages = Vec::new();

        for item in items {
            if !self.can_clean(item) {
                result.skipped_items += 1;
                continue;
            }

            match &item.source {
                CleanupSource::PackageManager(manager) => match manager.as_str() {
                    "rpm" => rpm_packages.push(item.name.clone()),
                    "apt" => apt_packages.push(item.name.clone()),
                    "pacman" => pacman_packages.push(item.name.clone()),
                    _ => result.skipped_items += 1,
                },
                CleanupSource::FileSystem => {
                    // /lib/modules/ dirs
                    if let Some(ref path) = item.path {
                        if dry_run {
                            log::info!("[DRY RUN] Would remove: {path}");
                        } else if let Err(e) = std::fs::remove_dir_all(path) {
                            result.errors.push(format!("{path}: {e}"));
                            continue;
                        }
                        result.cleaned_items += 1;
                        result.freed_bytes += item.size;
                    }
                }
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

        if !pacman_packages.is_empty() {
            pacman::remove_packages(&pacman_packages, dry_run)?;
            result.cleaned_items += pacman_packages.len();
        }

        Ok(result)
    }
}

fn current_kernel_version() -> String {
    let output = std::process::Command::new("uname").arg("-r").output();
    if let Ok(output) = output
        && output.status.success()
    {
        return String::from_utf8_lossy(&output.stdout).trim().to_string();
    }
    String::new()
}

fn keep_recent_kernels() -> usize {
    Config::load(&Config::default_path())
        .map(|config| config.current_profile().keep_recent_kernels)
        .unwrap_or(2)
}

fn scan_rpm_kernels(current: &str, keep_recent: usize) -> Result<Vec<CleanupItem>> {
    let output = std::process::Command::new("rpm")
        .args([
            "-q",
            "kernel",
            "kernel-core",
            "kernel-modules",
            "kernel-modules-core",
            "kernel-modules-extra",
            "kernel-devel",
            "kernel-headers",
        ])
        .output();

    let Ok(output) = output else {
        return Ok(Vec::new());
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let packages = parse_rpm_kernel_packages(&stdout);
    Ok(build_kernel_items(
        packages,
        current,
        keep_recent,
        "rpm",
        KernelPrefixes::Rpm,
    ))
}

fn scan_apt_kernels(current: &str, keep_recent: usize) -> Result<Vec<CleanupItem>> {
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
    let packages = parse_apt_kernel_packages(&stdout);
    Ok(build_kernel_items(
        packages,
        current,
        keep_recent,
        "apt",
        KernelPrefixes::Apt,
    ))
}

fn scan_pacman_kernels(current: &str, keep_recent: usize) -> Result<Vec<CleanupItem>> {
    // Check if pacman is available
    let output = std::process::Command::new("pacman")
        .args(["-Qq", "linux", "linux-lts", "linux-zen", "linux-hardened"])
        .output();

    let Ok(output) = output else {
        return Ok(Vec::new());
    };

    // pacman -Qq returns 1 if none of the packages are installed
    let stdout = String::from_utf8_lossy(&output.stdout);
    let packages: Vec<String> = stdout
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect();

    if packages.is_empty() {
        return Ok(Vec::new());
    }

    // Also look for linux*-headers packages
    let headers_output = std::process::Command::new("pacman").args(["-Qq"]).output();

    let mut all_kernel_pkgs = packages;
    if let Ok(out) = headers_output
        && out.status.success()
    {
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            let pkg = line.trim();
            if (pkg.starts_with("linux") && pkg.ends_with("-headers"))
                || (pkg.starts_with("linux") && pkg.contains("-docs"))
            {
                all_kernel_pkgs.push(pkg.to_string());
            }
        }
    }

    // Get version info for each kernel package
    let mut version_pkgs: HashMap<String, Vec<String>> = HashMap::new();
    for pkg in &all_kernel_pkgs {
        let qi = std::process::Command::new("pacman")
            .args(["-Q", pkg])
            .output();
        if let Ok(out) = qi {
            let line = String::from_utf8_lossy(&out.stdout);
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 2 {
                version_pkgs
                    .entry(parts[1].to_string())
                    .or_default()
                    .push(parts[0].to_string());
            }
        }
    }

    if version_pkgs.len() <= 1 {
        // Only one kernel version installed, nothing to remove
        return Ok(Vec::new());
    }

    let mut version_times: Vec<(String, i64)> = version_pkgs
        .keys()
        .map(|v| (v.clone(), kernel_mtime_seconds(v)))
        .collect();
    version_times.sort_by(|(_, a), (_, b)| b.cmp(a));

    let keep = select_versions_to_keep(version_times, current, keep_recent);

    let mut items = Vec::new();
    for (version, pkgs) in &version_pkgs {
        if keep.contains(version) {
            continue;
        }
        for pkg in pkgs {
            items.push(CleanupItem {
                id: format!("pacman:{pkg}"),
                name: pkg.clone(),
                path: None,
                size: 0,
                description: format!("Old kernel package (pacman) {version}"),
                category: CleanupCategory::OldKernels,
                source: CleanupSource::PackageManager("pacman".to_string()),
                selected: false,
                can_clean: true,
                blocked_reason: None,
                dependencies: Vec::new(),
            });
        }
    }

    Ok(items)
}

fn scan_modules_dir(current: &str, keep_recent: usize) -> Result<Vec<CleanupItem>> {
    // Scan /lib/modules/ for old kernel module directories
    let modules_path = Path::new("/lib/modules");
    if !modules_path.exists() {
        return Ok(Vec::new());
    }

    let entries = match std::fs::read_dir(modules_path) {
        Ok(entries) => entries,
        Err(_) => return Ok(Vec::new()),
    };

    let mut version_dirs: Vec<(String, std::path::PathBuf, u64)> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let version = entry.file_name().to_string_lossy().to_string();
        if !is_version_like(&version) {
            continue;
        }
        // Calculate directory size
        let size: u64 = walkdir::WalkDir::new(&path)
            .into_iter()
            .flatten()
            .filter_map(|e| e.metadata().ok())
            .filter(|m| m.is_file())
            .map(|m| m.len())
            .sum();
        version_dirs.push((version, path, size));
    }

    if version_dirs.len() <= 1 {
        return Ok(Vec::new());
    }

    let version_times: Vec<(String, i64)> = version_dirs
        .iter()
        .map(|(v, _, _)| (v.clone(), kernel_mtime_seconds(v)))
        .collect();

    let keep = select_versions_to_keep(version_times, current, keep_recent);

    let mut items = Vec::new();
    for (version, path, size) in version_dirs {
        if keep.contains(&version) {
            continue;
        }
        items.push(CleanupItem {
            id: format!("modules:{version}"),
            name: format!("/lib/modules/{version}"),
            path: Some(path.to_string_lossy().to_string()),
            size,
            description: format!("Old kernel modules ({version})"),
            category: CleanupCategory::OldKernels,
            source: CleanupSource::FileSystem,
            selected: false,
            can_clean: true,
            blocked_reason: None,
            dependencies: Vec::new(),
        });
    }

    Ok(items)
}

enum KernelPrefixes {
    Rpm,
    Apt,
}

fn build_kernel_items(
    packages: Vec<String>,
    current: &str,
    keep_recent: usize,
    manager: &str,
    prefixes: KernelPrefixes,
) -> Vec<CleanupItem> {
    let prefix_list = match prefixes {
        KernelPrefixes::Rpm => vec![
            "kernel-modules-extra-",
            "kernel-modules-core-",
            "kernel-modules-",
            "kernel-headers-",
            "kernel-devel-",
            "kernel-core-",
            "kernel-",
        ],
        KernelPrefixes::Apt => vec!["linux-image-unsigned-", "linux-image-"],
    };

    let versions = group_by_version(&packages, &prefix_list);
    if versions.is_empty() {
        return Vec::new();
    }

    let mut version_times = Vec::new();
    for version in versions.keys() {
        version_times.push((version.clone(), kernel_mtime_seconds(version)));
    }

    let keep_versions = select_versions_to_keep(version_times, current, keep_recent);
    let mut to_remove = Vec::new();
    for (version, pkgs) in versions {
        if keep_versions.contains(&version) {
            continue;
        }
        for pkg in pkgs {
            to_remove.push(pkg);
        }
    }

    to_remove
        .into_iter()
        .map(|pkg| CleanupItem {
            id: format!("{manager}:{pkg}"),
            name: pkg.clone(),
            path: None,
            size: 0,
            description: format!("Old kernel package ({manager})"),
            category: CleanupCategory::OldKernels,
            source: CleanupSource::PackageManager(manager.to_string()),
            selected: false,
            can_clean: true,
            blocked_reason: None,
            dependencies: Vec::new(),
        })
        .collect()
}

fn parse_rpm_kernel_packages(output: &str) -> Vec<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| !line.contains("not installed"))
        .map(String::from)
        .collect()
}

fn parse_apt_kernel_packages(output: &str) -> Vec<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("ii"))
        .filter_map(|line| line.split_whitespace().nth(1))
        .map(String::from)
        .collect()
}

fn group_by_version(packages: &[String], prefixes: &[&str]) -> HashMap<String, Vec<String>> {
    let mut grouped: HashMap<String, Vec<String>> = HashMap::new();
    for pkg in packages {
        if let Some(version) = extract_kernel_version(pkg, prefixes) {
            grouped.entry(version).or_default().push(pkg.clone());
        }
    }
    grouped
}

fn extract_kernel_version(package: &str, prefixes: &[&str]) -> Option<String> {
    for prefix in prefixes {
        if let Some(rest) = package.strip_prefix(prefix)
            && is_version_like(rest)
        {
            return Some(rest.to_string());
        }
    }
    None
}

fn is_version_like(value: &str) -> bool {
    let has_digit = value.chars().any(|ch| ch.is_ascii_digit());
    let has_separator = value.contains('.') || value.contains('-');
    has_digit && has_separator
}

fn kernel_mtime_seconds(version: &str) -> i64 {
    let path = Path::new("/boot").join(format!("vmlinuz-{version}"));
    let metadata = match std::fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(_) => return 0,
    };
    match metadata.modified() {
        Ok(time) => time
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs() as i64)
            .unwrap_or(0),
        Err(_) => 0,
    }
}

fn select_versions_to_keep(
    mut versions: Vec<(String, i64)>,
    current: &str,
    keep_recent: usize,
) -> HashSet<String> {
    versions.sort_by(|(a_version, a_time), (b_version, b_time)| {
        b_time.cmp(a_time).then_with(|| b_version.cmp(a_version))
    });

    let mut keep = HashSet::new();
    if !current.is_empty() {
        keep.insert(current.to_string());
    }

    for (version, _) in versions.into_iter().take(keep_recent) {
        keep.insert(version);
    }

    keep
}

#[cfg(test)]
mod tests {
    use super::{extract_kernel_version, select_versions_to_keep};

    #[test]
    fn test_extract_kernel_version_skips_meta() {
        let prefixes = ["linux-image-unsigned-", "linux-image-"];
        assert!(extract_kernel_version("linux-image-generic", &prefixes).is_none());
        assert!(extract_kernel_version("linux-image-amd64", &prefixes).is_none());
        assert_eq!(
            extract_kernel_version("linux-image-6.1.0-13-amd64", &prefixes).as_deref(),
            Some("6.1.0-13-amd64")
        );
    }

    #[test]
    fn test_select_versions_to_keep_respects_current() {
        let versions = vec![
            ("6.1.0-12-amd64".to_string(), 10),
            ("6.1.0-13-amd64".to_string(), 20),
            ("6.1.0-14-amd64".to_string(), 30),
        ];
        let keep = select_versions_to_keep(versions, "6.1.0-12-amd64", 1);
        assert!(keep.contains("6.1.0-14-amd64"));
        assert!(keep.contains("6.1.0-12-amd64"));
        assert!(!keep.contains("6.1.0-13-amd64"));
    }
}
