use crate::error::Result;
use crate::cleaner::base::Cleaner;
use crate::models::{CleanupCategory, CleanupItem, CleanupSource};

pub struct LogsCleaner;

impl LogsCleaner {
    pub fn new() -> Self {
        Self {}
    }
}

impl Cleaner for LogsCleaner {
    fn name(&self) -> &str {
        "Logs Cleaner"
    }

    fn category(&self) -> CleanupCategory {
        CleanupCategory::Logs
    }

    fn scan(&self) -> Result<Vec<CleanupItem>> {
        Ok(Vec::new())
    }

    fn clean(&self, _items: &[CleanupItem], _dry_run: bool) -> Result<()> {
        Ok(())
    }
}
