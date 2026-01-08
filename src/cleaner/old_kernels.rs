use crate::cleaner::base::Cleaner;
use crate::config::Config;
use crate::error::Result;
use crate::models::{CleanupCategory, CleanupItem, CleanupResult, CleanupSource};
use crate::system::{apt, rpm};
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

        Ok(items)
    }

    fn clean(&self, items: &[CleanupItem], dry_run: bool) -> Result<CleanupResult> {
        let mut result = CleanupResult::default();
        let mut rpm_packages = Vec::new();
        let mut apt_packages = Vec::new();

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
        .args(["-q", "kernel", "kernel-core", "kernel-modules"])
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
        KernelPrefixes::Rpm => vec!["kernel-core-", "kernel-modules-", "kernel-"],
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
        if let Some(rest) = package.strip_prefix(prefix) {
            if is_version_like(rest) {
                return Some(rest.to_string());
            }
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
