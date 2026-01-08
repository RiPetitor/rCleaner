use crate::backup::BackupManager;
use crate::cleaner::base::Cleaner;
use crate::error::{RcleanerError, Result};
use crate::models::{CleanupCategory, CleanupItem, CleanupResult, CleanupSource};
use crate::system::{flatpak, snap};
use crate::utils::size_format::parse_size_string;
use std::env;
use std::path::Path;

pub struct ApplicationsCleaner;

impl ApplicationsCleaner {
    pub fn new() -> Self {
        Self {}
    }
}

impl Cleaner for ApplicationsCleaner {
    fn name(&self) -> &str {
        "Applications Cleaner"
    }

    fn category(&self) -> CleanupCategory {
        CleanupCategory::Applications
    }

    fn scan(&self) -> Result<Vec<CleanupItem>> {
        let mut items = Vec::new();

        if flatpak::is_flatpak_available() {
            if let Ok(apps) = flatpak::list_installed_with_sizes() {
                for (app, size) in apps {
                    if app.trim().is_empty() {
                        continue;
                    }
                    items.push(CleanupItem {
                        id: format!("flatpak:{}", app),
                        name: app.clone(),
                        path: None,
                        size,
                        description: "Flatpak application".to_string(),
                        category: self.category(),
                        source: CleanupSource::PackageManager("flatpak".to_string()),
                        selected: false,
                        can_clean: true,
                        dependencies: Vec::new(),
                    });
                }
            }
        }

        if snap::is_snap_available() {
            if let Ok(apps) = snap::list_installed_with_sizes() {
                for (app, size) in apps {
                    if app.trim().is_empty() || app == "Name" {
                        continue;
                    }
                    items.push(CleanupItem {
                        id: format!("snap:{}", app),
                        name: app.clone(),
                        path: None,
                        size,
                        description: "Snap application".to_string(),
                        category: self.category(),
                        source: CleanupSource::PackageManager("snap".to_string()),
                        selected: false,
                        can_clean: true,
                        dependencies: Vec::new(),
                    });
                }
            }
        }

        items.extend(list_container_images("docker")?);
        items.extend(list_container_images("podman")?);

        Ok(items)
    }

    fn clean(&self, items: &[CleanupItem], dry_run: bool) -> Result<CleanupResult> {
        let mut result = CleanupResult::default();
        let mut flatpak_apps = Vec::new();
        let mut snap_apps = Vec::new();
        let mut docker_images = Vec::new();
        let mut podman_images = Vec::new();

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
                    "flatpak" => flatpak_apps.push(item.name.clone()),
                    "snap" => snap_apps.push(item.name.clone()),
                    _ => result.skipped_items += 1,
                },
                CleanupSource::Container(runtime) => match runtime.as_str() {
                    "docker" => docker_images.push(item.name.clone()),
                    "podman" => podman_images.push(item.name.clone()),
                    _ => result.skipped_items += 1,
                },
                CleanupSource::FileSystem => result.skipped_items += 1,
            }
        }

        if !flatpak_apps.is_empty() {
            flatpak::remove_packages(&flatpak_apps, dry_run)?;
            result.cleaned_items += flatpak_apps.len();
        }

        if !snap_apps.is_empty() {
            snap::remove_packages(&snap_apps, dry_run)?;
            result.cleaned_items += snap_apps.len();
        }

        if !docker_images.is_empty() {
            remove_container_images("docker", &docker_images, dry_run)
                .map_err(|err| RcleanerError::Command(err))?;
            result.cleaned_items += docker_images.len();
        }

        if !podman_images.is_empty() {
            remove_container_images("podman", &podman_images, dry_run)
                .map_err(|err| RcleanerError::Command(err))?;
            result.cleaned_items += podman_images.len();
        }

        Ok(result)
    }
}

fn list_container_images(runtime: &str) -> Result<Vec<CleanupItem>> {
    if !command_exists(runtime) {
        return Ok(Vec::new());
    }

    let output = std::process::Command::new(runtime)
        .args(["images", "--format", "{{.Repository}}:{{.Tag}}\t{{.Size}}"])
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
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split('\t');
        let name = parts.next().unwrap_or("").trim();
        let size_text = parts.next().unwrap_or("").trim();
        if name.is_empty() || name == "<none>:<none>" {
            continue;
        }
        items.push(CleanupItem {
            id: format!("{runtime}:{name}"),
            name: name.to_string(),
            path: None,
            size: parse_size_string(size_text).unwrap_or(0),
            description: format!("Container image ({runtime})"),
            category: CleanupCategory::Applications,
            source: CleanupSource::Container(runtime.to_string()),
            selected: false,
            can_clean: true,
            dependencies: Vec::new(),
        });
    }

    Ok(items)
}

fn remove_container_images(
    runtime: &str,
    images: &[String],
    dry_run: bool,
) -> std::result::Result<(), String> {
    if images.is_empty() {
        return Ok(());
    }

    if dry_run {
        log::info!("[DRY RUN] {} rmi {:?}", runtime, images);
        return Ok(());
    }

    let mut args = vec!["rmi", "-f"];
    let image_args = images.iter().map(String::as_str);
    args.extend(image_args);

    let output = std::process::Command::new(runtime)
        .args(&args)
        .output()
        .map_err(|err| err.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.trim().to_string());
    }

    Ok(())
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
