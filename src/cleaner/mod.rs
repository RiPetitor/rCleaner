pub mod applications;
pub mod base;
pub mod cache;
pub mod logs;
pub mod old_kernels;
pub mod old_packages;
pub mod temp_files;

use crate::cleaner::base::Cleaner;
use crate::error::Result;
use crate::models::CleanupItem;

pub fn scan_all() -> Result<Vec<CleanupItem>> {
    let cleaners: Vec<Box<dyn Cleaner>> = vec![
        Box::new(cache::CacheCleaner::new()),
        Box::new(applications::ApplicationsCleaner::new()),
        Box::new(temp_files::TempFilesCleaner::new()),
        Box::new(logs::LogsCleaner::new()),
        Box::new(old_packages::OldPackagesCleaner::new()),
        Box::new(old_kernels::OldKernelsCleaner::new()),
    ];

    let mut items = Vec::new();
    for cleaner in cleaners {
        match cleaner.scan() {
            Ok(mut cleaned) => items.append(&mut cleaned),
            Err(err) => {
                log::warn!("{} scan failed: {}", cleaner.name(), err);
            }
        }
    }

    Ok(items)
}
