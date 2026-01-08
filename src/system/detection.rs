use crate::error::Result;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

pub fn detect_system() -> Result<SystemInfo> {
    let os_release = read_os_release();
    let os_name = os_release_value(&os_release, "PRETTY_NAME")
        .or_else(|| os_release_value(&os_release, "NAME"))
        .unwrap_or_else(|| "Unknown".to_string());
    let os_version = os_release_value(&os_release, "VERSION_ID")
        .or_else(|| os_release_value(&os_release, "VERSION"))
        .unwrap_or_else(|| "Unknown".to_string());

    let available_managers = detect_available_managers();
    let containers = detect_containers();
    let desktop_environment = detect_desktop_environment();

    let system_type = if is_atomic_rpm_ostree() {
        SystemType::AtomicRpmOstree
    } else {
        SystemType::Desktop(detect_desktop_type(&os_release, &available_managers))
    };

    Ok(SystemInfo {
        system_type,
        os_name,
        os_version,
        desktop_environment,
        available_managers,
        containers,
    })
}

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub system_type: SystemType,
    pub os_name: String,
    pub os_version: String,
    pub desktop_environment: Option<String>,
    pub available_managers: Vec<PackageManagerType>,
    pub containers: Vec<ContainerType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemType {
    AtomicRpmOstree,
    Desktop(DesktopType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopType {
    RpmFedora,
    RpmRHEL,
    AptDebian,
    AptUbuntu,
    PacmanArch,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManagerType {
    RpmOstree,
    Rpm,
    Dnf,
    Apt,
    Pacman,
    Flatpak,
    Snap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerType {
    Docker,
    Podman,
}

fn read_os_release() -> HashMap<String, String> {
    let mut values = HashMap::new();
    let paths = [
        Path::new("/etc/os-release"),
        Path::new("/usr/lib/os-release"),
    ];

    for path in paths {
        if let Ok(content) = fs::read_to_string(path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = line.split_once('=') {
                    let value = value
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string();
                    values.insert(key.to_string(), value);
                }
            }
            break;
        }
    }

    values
}

fn os_release_value(values: &HashMap<String, String>, key: &str) -> Option<String> {
    values.get(key).cloned()
}

fn is_atomic_rpm_ostree() -> bool {
    Path::new("/run/ostree-booted").exists() || command_exists("rpm-ostree")
}

fn detect_available_managers() -> Vec<PackageManagerType> {
    let mut managers = Vec::new();

    if command_exists("rpm-ostree") {
        managers.push(PackageManagerType::RpmOstree);
    }
    if command_exists("rpm") {
        managers.push(PackageManagerType::Rpm);
    }
    if command_exists("dnf") {
        managers.push(PackageManagerType::Dnf);
    }
    if command_exists("apt") || command_exists("apt-get") {
        managers.push(PackageManagerType::Apt);
    }
    if command_exists("pacman") {
        managers.push(PackageManagerType::Pacman);
    }
    if command_exists("flatpak") {
        managers.push(PackageManagerType::Flatpak);
    }
    if command_exists("snap") {
        managers.push(PackageManagerType::Snap);
    }

    managers
}

fn detect_containers() -> Vec<ContainerType> {
    let mut containers = Vec::new();
    if command_exists("podman") {
        containers.push(ContainerType::Podman);
    }
    if command_exists("docker") {
        containers.push(ContainerType::Docker);
    }
    containers
}

fn detect_desktop_environment() -> Option<String> {
    for key in ["XDG_CURRENT_DESKTOP", "DESKTOP_SESSION", "GDMSESSION"] {
        if let Ok(value) = env::var(key) {
            let normalized = normalize_desktop_environment(&value);
            if !normalized.is_empty() {
                return Some(normalized);
            }
        }
    }
    None
}

fn normalize_desktop_environment(value: &str) -> String {
    let lower = value.to_lowercase();
    if lower.contains("gnome") {
        "GNOME".to_string()
    } else if lower.contains("kde") || lower.contains("plasma") {
        "KDE".to_string()
    } else if lower.contains("xfce") {
        "XFCE".to_string()
    } else {
        value.trim().to_string()
    }
}

fn detect_desktop_type(
    os_release: &HashMap<String, String>,
    managers: &[PackageManagerType],
) -> DesktopType {
    let id = os_release
        .get("ID")
        .map(|value| value.to_lowercase())
        .unwrap_or_default();
    let id_like = os_release
        .get("ID_LIKE")
        .map(|value| value.to_lowercase())
        .unwrap_or_default();
    let id_like_values: Vec<&str> = id_like.split_whitespace().collect();

    if id == "fedora" {
        return DesktopType::RpmFedora;
    }
    if matches!(id.as_str(), "rhel" | "centos" | "rocky" | "almalinux") {
        return DesktopType::RpmRHEL;
    }
    if id == "ubuntu" || id == "linuxmint" || id == "pop" {
        return DesktopType::AptUbuntu;
    }
    if id == "debian" {
        return DesktopType::AptDebian;
    }
    if id == "arch" || id == "manjaro" || id == "endeavouros" {
        return DesktopType::PacmanArch;
    }

    if id_like_values.contains(&"fedora") {
        return DesktopType::RpmFedora;
    }
    if id_like_values.contains(&"rhel") {
        return DesktopType::RpmRHEL;
    }
    if id_like_values.contains(&"ubuntu") {
        return DesktopType::AptUbuntu;
    }
    if id_like_values.contains(&"debian") {
        return DesktopType::AptDebian;
    }
    if id_like_values.contains(&"arch") {
        return DesktopType::PacmanArch;
    }

    if managers.contains(&PackageManagerType::Pacman) {
        return DesktopType::PacmanArch;
    }
    if managers.contains(&PackageManagerType::Apt) {
        return DesktopType::AptDebian;
    }
    if managers.contains(&PackageManagerType::Dnf) || managers.contains(&PackageManagerType::Rpm) {
        return DesktopType::RpmFedora;
    }

    DesktopType::Unknown
}

fn command_exists(command: &str) -> bool {
    if command.contains('/') {
        return is_executable(Path::new(command));
    }

    if let Some(paths) = env::var_os("PATH") {
        for path in env::split_paths(&paths) {
            let candidate = path.join(command);
            if is_executable(&candidate) {
                return true;
            }
        }
    }

    false
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    if let Ok(metadata) = fs::metadata(path) {
        metadata.is_file() && (metadata.permissions().mode() & 0o111 != 0)
    } else {
        false
    }
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}
