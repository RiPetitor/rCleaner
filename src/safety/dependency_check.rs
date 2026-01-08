use crate::error::Result;
use crate::system::package_manager::PackageManager;
use crate::system::{apt::AptManager, dnf::DnfManager, pacman::PacmanManager, rpm::RpmManager};
use std::env;
use std::path::Path;

pub fn check_dependencies(package: &str) -> Result<Vec<String>> {
    if command_exists("apt-cache") {
        return AptManager::new().check_dependencies(package);
    }
    if command_exists("dnf") {
        return DnfManager::new().check_dependencies(package);
    }
    if command_exists("pacman") {
        return PacmanManager::new().check_dependencies(package);
    }
    if command_exists("rpm") {
        return RpmManager::new().check_dependencies(package);
    }
    Ok(Vec::new())
}

pub fn check_dependencies_for_manager(manager: &str, package: &str) -> Result<Vec<String>> {
    match manager {
        "apt" => {
            if command_exists("apt-cache") {
                AptManager::new().check_dependencies(package)
            } else {
                Ok(Vec::new())
            }
        }
        "dnf" => {
            if command_exists("dnf") {
                DnfManager::new().check_dependencies(package)
            } else {
                Ok(Vec::new())
            }
        }
        "pacman" => {
            if command_exists("pacman") {
                PacmanManager::new().check_dependencies(package)
            } else {
                Ok(Vec::new())
            }
        }
        "rpm" => {
            if command_exists("rpm") {
                RpmManager::new().check_dependencies(package)
            } else {
                Ok(Vec::new())
            }
        }
        _ => Ok(Vec::new()),
    }
}

pub fn is_safe_to_remove(package: &str, installed_packages: &[String]) -> bool {
    if !installed_packages.iter().any(|p| p == package) {
        return false;
    }
    match check_dependencies(package) {
        Ok(deps) => deps.is_empty(),
        Err(_) => false,
    }
}

fn command_exists(command: &str) -> bool {
    if command.contains('/') {
        return Path::new(command).exists();
    }
    if let Some(paths) = env::var_os("PATH") {
        for path in env::split_paths(&paths) {
            let candidate = path.join(command);
            if candidate.exists() {
                return true;
            }
        }
    }
    false
}
