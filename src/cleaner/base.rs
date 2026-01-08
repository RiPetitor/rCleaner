use crate::error::Result;

pub trait Cleaner {
    fn name(&self) -> &str;
    fn category(&self) -> crate::models::CleanupCategory;
    fn scan(&self) -> Result<Vec<crate::models::CleanupItem>>;
    fn clean(&self, items: &[crate::models::CleanupItem], dry_run: bool) -> Result<()>;
}
