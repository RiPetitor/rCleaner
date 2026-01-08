use crate::cleaner::base::Cleaner;
use crate::error::Result;
use crate::models::{CleanupCategory, CleanupItem, CleanupSource};
use walkdir::WalkDir;

pub struct CacheCleaner;

impl CacheCleaner {
    pub fn new() -> Self {
        Self {}
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
            format!("{}/.cache/thumbnails", home),
            format!("{}/.cache/mozilla/firefox", home),
            format!("{}/.cache/google-chrome", home),
        ];

        for cache_dir in &cache_dirs {
            if let Ok(size) = self.calculate_directory_size(cache_dir) {
                if size > 0 {
                    items.push(CleanupItem {
                        id: cache_dir.clone(),
                        name: cache_dir.split('/').last().unwrap_or("unknown").to_string(),
                        path: Some(cache_dir.clone()),
                        size,
                        description: format!("Cache directory: {}", cache_dir),
                        category: self.category(),
                        source: CleanupSource::FileSystem,
                        selected: false,
                        can_clean: true,
                    });
                }
            }
        }

        Ok(items)
    }

    fn clean(&self, items: &[CleanupItem], dry_run: bool) -> Result<()> {
        for item in items {
            if let Some(ref path) = item.path {
                if dry_run {
                    log::info!("[DRY RUN] Would clean: {}", path);
                } else {
                    log::info!("Cleaning: {}", path);
                    std::fs::remove_dir_all(path)?;
                }
            }
        }

        Ok(())
    }

    fn calculate_directory_size(&self, path: &str) -> Result<u64> {
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
