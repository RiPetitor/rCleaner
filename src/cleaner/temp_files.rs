use crate::cleaner::base::Cleaner;
use crate::config::Config;
use crate::error::Result;
use crate::models::{CleanupCategory, CleanupItem, CleanupResult, CleanupSource};
use crate::utils::command;
use libc;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;

pub struct TempFilesCleaner;

impl Default for TempFilesCleaner {
    fn default() -> Self {
        Self::new()
    }
}

impl TempFilesCleaner {
    pub fn new() -> Self {
        Self {}
    }
}

impl Cleaner for TempFilesCleaner {
    fn name(&self) -> &str {
        "Temp Files Cleaner"
    }

    fn category(&self) -> CleanupCategory {
        CleanupCategory::TempFiles
    }

    fn scan(&self) -> Result<Vec<CleanupItem>> {
        let mut items = Vec::new();
        let options = TempCleanupOptions::from_config();

        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let temp_paths = [
            ("Temporary files (/tmp)", PathBuf::from("/tmp")),
            ("Temporary files (/var/tmp)", PathBuf::from("/var/tmp")),
            (
                "Trash (files)",
                PathBuf::from(format!("{}/.local/share/Trash/files", home)),
            ),
        ];

        for (label, path) in temp_paths {
            let entries = collect_eligible_entries(&path, &options)?;
            let size = calculate_entries_size(&entries);
            if size > 0 {
                items.push(CleanupItem {
                    id: path.to_string_lossy().to_string(),
                    name: label.to_string(),
                    path: Some(path.to_string_lossy().to_string()),
                    size,
                    description: format!(
                        "Temporary directory: {} (older than {} days)",
                        path.to_string_lossy(),
                        options.max_age_days
                    ),
                    category: self.category(),
                    source: CleanupSource::FileSystem,
                    selected: false,
                    can_clean: true,
                    blocked_reason: None,
                    dependencies: Vec::new(),
                });
            }
        }

        Ok(items)
    }

    fn clean(&self, items: &[CleanupItem], dry_run: bool) -> Result<CleanupResult> {
        let mut result = CleanupResult::default();
        let options = TempCleanupOptions::from_config();

        for item in items {
            if !self.can_clean(item) {
                result.skipped_items += 1;
                continue;
            }

            let Some(ref path) = item.path else {
                result.skipped_items += 1;
                continue;
            };
            let path = Path::new(path);

            if dry_run {
                log::info!("[DRY RUN] Would clean: {}", path.display());
                result.cleaned_items += 1;
                result.freed_bytes += item.size;
                continue;
            }

            let entries = collect_eligible_entries(path, &options)?;
            if entries.is_empty() {
                result.skipped_items += 1;
                continue;
            }

            let mut removed_bytes = 0u64;
            let mut removed_any = false;
            for entry_path in entries {
                let entry_size = entry_size(&entry_path).unwrap_or(0);
                if dry_run {
                    log::info!("[DRY RUN] Would remove: {}", entry_path.display());
                    removed_bytes = removed_bytes.saturating_add(entry_size);
                    removed_any = true;
                    continue;
                }

                match remove_entry(&entry_path) {
                    Ok(()) => {
                        removed_bytes = removed_bytes.saturating_add(entry_size);
                        removed_any = true;
                        if let Some(info_path) = trash_info_path(&entry_path) {
                            let _ = fs::remove_file(info_path);
                        }
                    }
                    Err(err) => {
                        result
                            .errors
                            .push(format!("{}: {}", entry_path.display(), err));
                    }
                }
            }

            if removed_any {
                result.cleaned_items += 1;
                result.freed_bytes += removed_bytes;
            } else if !dry_run {
                result.skipped_items += 1;
            }
        }

        Ok(result)
    }
}

struct TempCleanupOptions {
    cutoff: SystemTime,
    max_age_days: u64,
    only_owner: bool,
    owner_uid: u32,
}

impl TempCleanupOptions {
    fn from_config() -> Self {
        let config = Config::load(&Config::default_path()).unwrap_or_default();
        let max_age_days = config.current_profile().temp_max_age_days;
        let max_age = Duration::from_secs(max_age_days.saturating_mul(24 * 60 * 60));
        let cutoff = SystemTime::now().checked_sub(max_age).unwrap_or(UNIX_EPOCH);

        Self {
            cutoff,
            max_age_days,
            only_owner: !command::is_root(),
            owner_uid: unsafe { libc::geteuid() } as u32,
        }
    }
}

fn collect_eligible_entries(path: &Path, options: &TempCleanupOptions) -> Result<Vec<PathBuf>> {
    let mut entries = Vec::new();
    if !path.exists() {
        return Ok(entries);
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let metadata = match fs::symlink_metadata(&entry_path) {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };

        if options.only_owner && metadata.uid() != options.owner_uid {
            continue;
        }

        let modified = match metadata.modified() {
            Ok(time) => time,
            Err(_) => continue,
        };

        if modified > options.cutoff {
            continue;
        }

        entries.push(entry_path);
    }

    Ok(entries)
}

fn calculate_entries_size(entries: &[PathBuf]) -> u64 {
    entries
        .iter()
        .filter_map(|path| entry_size(path).ok())
        .sum()
}

fn entry_size(path: &Path) -> Result<u64> {
    let metadata = fs::symlink_metadata(path)?;
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        return Ok(0);
    }

    if file_type.is_file() {
        return Ok(metadata.len());
    }

    if !path.exists() {
        return Ok(0);
    }

    let mut total = 0u64;
    for entry in WalkDir::new(path).into_iter().flatten() {
        let entry_type = entry.file_type();
        if entry_type.is_symlink() {
            continue;
        }
        if entry_type.is_file() {
            if let Ok(metadata) = entry.metadata() {
                total += metadata.len();
            }
        }
    }
    Ok(total)
}

fn remove_entry(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn trash_info_path(entry_path: &Path) -> Option<PathBuf> {
    let file_name = entry_path.file_name()?.to_string_lossy();
    let files_dir = entry_path.parent()?;
    if files_dir.file_name()?.to_string_lossy() != "files" {
        return None;
    }
    let trash_dir = files_dir.parent()?;
    let info_dir = trash_dir.join("info");
    Some(info_dir.join(format!("{file_name}.trashinfo")))
}
