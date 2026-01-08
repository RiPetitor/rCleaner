use crate::backup::BackupManager;
use crate::cleaner::base::Cleaner;
use crate::error::Result;
use crate::models::{CleanupCategory, CleanupItem, CleanupResult, CleanupSource};
use std::path::Path;
use walkdir::WalkDir;

pub struct CacheCleaner;

impl CacheCleaner {
    pub fn new() -> Self {
        Self {}
    }

    fn calculate_directory_size(&self, path: &str) -> Result<u64> {
        self.calculate_directory_size_path(Path::new(path))
    }

    fn calculate_directory_size_path(&self, path: &Path) -> Result<u64> {
        if !path.exists() {
            return Ok(0);
        }

        let mut total_size = 0u64;

        for entry in WalkDir::new(path).into_iter().flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    total_size += metadata.len();
                }
            }
        }

        Ok(total_size)
    }
}

impl Cleaner for CacheCleaner {
    fn name(&self) -> &str {
        "Cache Cleaner"
    }

    fn category(&self) -> CleanupCategory {
        CleanupCategory::Cache
    }

    fn scan(&self) -> Result<Vec<CleanupItem>> {
        let mut items = Vec::new();

        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let cache_dirs = [
            ("User cache", format!("{}/.cache", home)),
            ("Thumbnails", format!("{}/.cache/thumbnails", home)),
            ("Firefox cache", format!("{}/.cache/mozilla/firefox", home)),
            ("Chrome cache", format!("{}/.cache/google-chrome", home)),
            ("Chromium cache", format!("{}/.cache/chromium", home)),
            ("Brave cache", format!("{}/.cache/BraveSoftware", home)),
            ("Shader cache", format!("{}/.cache/mesa_shader_cache", home)),
        ];

        for (label, cache_dir) in &cache_dirs {
            if let Ok(size) = self.calculate_directory_size(cache_dir) {
                if size > 0 {
                    items.push(CleanupItem {
                        id: cache_dir.clone(),
                        name: label.to_string(),
                        path: Some(cache_dir.clone()),
                        size,
                        description: format!("Cache directory: {}", cache_dir),
                        category: self.category(),
                        source: CleanupSource::FileSystem,
                        selected: false,
                        can_clean: true,
                        blocked_reason: None,
                        dependencies: Vec::new(),
                    });
                }
            }
        }

        let flatpak_root = format!("{}/.var/app", home);
        if let Ok(entries) = std::fs::read_dir(&flatpak_root) {
            for entry in entries.flatten() {
                let app_path = entry.path();
                let app_name = entry.file_name().to_string_lossy().trim().to_string();
                let cache_path = app_path.join("cache");
                if let Ok(size) = self.calculate_directory_size_path(&cache_path) {
                    if size > 0 {
                        items.push(CleanupItem {
                            id: cache_path.to_string_lossy().to_string(),
                            name: format!("Flatpak cache: {}", app_name),
                            path: Some(cache_path.to_string_lossy().to_string()),
                            size,
                            description: format!(
                                "Flatpak cache directory: {}",
                                cache_path.to_string_lossy()
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
            if let Some(ref path) = item.path {
                if dry_run {
                    log::info!("[DRY RUN] Would clean: {}", path);
                    result.cleaned_items += 1;
                    result.freed_bytes += item.size;
                } else {
                    match std::fs::remove_dir_all(path) {
                        Ok(()) => {
                            result.cleaned_items += 1;
                            result.freed_bytes += item.size;
                        }
                        Err(err) => {
                            result.errors.push(format!("{}: {}", path, err));
                        }
                    }
                }
            } else {
                result.skipped_items += 1;
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CleanupCategory, CleanupItem, CleanupSource};
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_clean_dry_run_keeps_files() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "rcleaner-cache-test-{nanos}-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();

        let file_path = dir.join("file.txt");
        let mut file = std::fs::File::create(&file_path).unwrap();
        writeln!(file, "test").unwrap();
        let size = std::fs::metadata(&file_path).unwrap().len();

        let item = CleanupItem {
            id: "test".to_string(),
            name: "Test cache".to_string(),
            path: Some(dir.to_string_lossy().to_string()),
            size,
            description: "test".to_string(),
            category: CleanupCategory::Cache,
            source: CleanupSource::FileSystem,
            selected: true,
            can_clean: true,
            blocked_reason: None,
            dependencies: Vec::new(),
        };

        let cleaner = CacheCleaner::new();
        let result = cleaner.clean(&[item], true).unwrap();

        assert!(file_path.exists());
        assert_eq!(result.cleaned_items, 1);
        assert_eq!(result.freed_bytes, size);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
