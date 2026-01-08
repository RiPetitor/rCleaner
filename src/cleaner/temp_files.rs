use crate::backup::BackupManager;
use crate::cleaner::base::Cleaner;
use crate::error::Result;
use crate::models::{CleanupCategory, CleanupItem, CleanupResult, CleanupSource};
use std::path::{Path, PathBuf};
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

        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let temp_paths = [
            ("Temporary files (/tmp)", PathBuf::from("/tmp")),
            ("Temporary files (/var/tmp)", PathBuf::from("/var/tmp")),
            (
                "Trash",
                PathBuf::from(format!("{}/.local/share/Trash", home)),
            ),
        ];

        for (label, path) in temp_paths {
            if let Ok(size) = self.calculate_directory_size(&path)
                && size > 0 {
                    items.push(CleanupItem {
                        id: path.to_string_lossy().to_string(),
                        name: label.to_string(),
                        path: Some(path.to_string_lossy().to_string()),
                        size,
                        description: format!("Temporary directory: {}", path.to_string_lossy()),
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

        if !dry_run {
            let manager = BackupManager::from_config()?;
            let _backup = manager.create_backup(items)?;
        }

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

            match remove_directory_contents(path) {
                Ok(()) => {
                    result.cleaned_items += 1;
                    result.freed_bytes += item.size;
                }
                Err(err) => {
                    result.errors.push(format!("{}: {}", path.display(), err));
                }
            }
        }

        Ok(result)
    }
}

fn remove_directory_contents(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        if entry_path.is_dir() {
            std::fs::remove_dir_all(&entry_path)?;
        } else {
            std::fs::remove_file(&entry_path)?;
        }
    }
    Ok(())
}

fn calculate_directory_size(path: &Path) -> Result<u64> {
    if !path.exists() {
        return Ok(0);
    }

    let mut total_size = 0u64;
    for entry in WalkDir::new(path).into_iter().flatten() {
        if let Ok(metadata) = entry.metadata()
            && metadata.is_file() {
                total_size += metadata.len();
            }
    }
    Ok(total_size)
}

impl TempFilesCleaner {
    fn calculate_directory_size(&self, path: &Path) -> Result<u64> {
        calculate_directory_size(path)
    }
}
