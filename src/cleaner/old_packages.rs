use crate::error::Result;
use crate::cleaner::base::Cleaner;
use crate::models::{CleanupCategory, CleanupItem, CleanupSource};

pub struct OldPackagesCleaner;

impl OldPackagesCleaner {
    pub fn new() -> Self {
        Self {}
    }
}

impl Cleaner for OldPackagesCleaner {
    fn name(&self) -> &str {
        "Old Packages Cleaner"
    }

    fn category(&self) -> CleanupCategory {
        CleanupCategory::OldPackages
    }

    fn scan(&self) -> Result<Vec<CleanupItem>> {
        Ok(Vec::new())
    }

    fn clean(&self, _items: &[CleanupItem], _dry_run: bool) -> Result<()> {
        Ok(())
    }
}
