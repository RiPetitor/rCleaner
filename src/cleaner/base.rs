use crate::error::Result;
use crate::models::{CleanupCategory, CleanupItem, CleanupResult};

pub trait Cleaner {
    fn name(&self) -> &str;
    fn category(&self) -> CleanupCategory;
    fn scan(&self) -> Result<Vec<CleanupItem>>;
    fn clean(&self, items: &[CleanupItem], dry_run: bool) -> Result<CleanupResult>;
    fn can_clean(&self, item: &CleanupItem) -> bool {
        item.can_clean
    }
}
