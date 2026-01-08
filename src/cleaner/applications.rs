use crate::error::Result;
use crate::cleaner::base::Cleaner;
use crate::models::{CleanupCategory, CleanupItem, CleanupSource};

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
        Ok(Vec::new())
    }

    fn clean(&self, _items: &[CleanupItem], _dry_run: bool) -> Result<()> {
        Ok(())
    }
}
