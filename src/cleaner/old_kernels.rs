use crate::error::Result;
use crate::cleaner::base::Cleaner;
use crate::models::{CleanupCategory, CleanupItem, CleanupSource};

pub struct OldKernelsCleaner;

impl OldKernelsCleaner {
    pub fn new() -> Self {
        Self {}
    }
}

impl Cleaner for OldKernelsCleaner {
    fn name(&self) -> &str {
        "Old Kernels Cleaner"
    }

    fn category(&self) -> CleanupCategory {
        CleanupCategory::OldKernels
    }

    fn scan(&self) -> Result<Vec<CleanupItem>> {
        Ok(Vec::new())
    }

    fn clean(&self, _items: &[CleanupItem], _dry_run: bool) -> Result<()> {
        Ok(())
    }
}
