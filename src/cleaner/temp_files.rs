use crate::error::Result;
use crate::cleaner::base::Cleaner;
use crate::models::{CleanupCategory, CleanupItem, CleanupSource};

pub struct TempFilesCleaner;

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
        Ok(Vec::new())
    }

    fn clean(&self, _items: &[CleanupItem], _dry_run: bool) -> Result<()> {
        Ok(())
    }
}
