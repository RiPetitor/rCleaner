pub mod applications;
pub mod base;
pub mod cache;
pub mod logs;
pub mod old_kernels;
pub mod old_packages;
pub mod temp_files;

use crate::cleaner::base::Cleaner;
use crate::config::Config;
use crate::error::Result;
use crate::models::{CleanupCategory, CleanupItem, CleanupResult};
use crate::safety::SafetyChecker;

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

    let config = match Config::load(&Config::default_path()) {
        Ok(config) => config,
        Err(err) => {
            log::warn!("Failed to load config for safety: {}", err);
            Config::default()
        }
    };
    let checker = SafetyChecker::new(config);
    for item in items.iter_mut() {
        if let Err(err) = checker.apply_to_item(item) {
            log::warn!("Safety check failed for {}: {}", item.name, err);
            item.can_clean = false;
        }
    }

    Ok(items)
}

pub fn clean_selected(items: &[CleanupItem], dry_run: bool) -> Result<CleanupResult> {
    clean_selected_with_progress(items, dry_run, |_progress, _label| {})
}

pub fn clean_selected_with_progress<F>(
    items: &[CleanupItem],
    dry_run: bool,
    mut on_progress: F,
) -> Result<CleanupResult>
where
    F: FnMut(f64, &str),
{
    let mut cleaners: Vec<(CleanupCategory, Box<dyn Cleaner>)> = vec![
        (CleanupCategory::Cache, Box::new(cache::CacheCleaner::new())),
        (
            CleanupCategory::Applications,
            Box::new(applications::ApplicationsCleaner::new()),
        ),
        (
            CleanupCategory::TempFiles,
            Box::new(temp_files::TempFilesCleaner::new()),
        ),
        (CleanupCategory::Logs, Box::new(logs::LogsCleaner::new())),
        (
            CleanupCategory::OldPackages,
            Box::new(old_packages::OldPackagesCleaner::new()),
        ),
        (
            CleanupCategory::OldKernels,
            Box::new(old_kernels::OldKernelsCleaner::new()),
        ),
    ];

    let mut total = CleanupResult::default();

    let mut steps = 0usize;
    for (category, _) in cleaners.iter() {
        if items
            .iter()
            .any(|item| item.selected && item.category == *category)
        {
            steps += 1;
        }
    }
    let steps = steps.max(1);

    let mut completed = 0usize;
    for (category, cleaner) in cleaners.drain(..) {
        let selected: Vec<CleanupItem> = items
            .iter()
            .filter(|item| item.selected && item.category == category)
            .cloned()
            .collect();
        if selected.is_empty() {
            continue;
        }

        let progress = completed as f64 / steps as f64;
        on_progress(progress, cleaner.name());

        match cleaner.clean(&selected, dry_run) {
            Ok(result) => {
                total.cleaned_items += result.cleaned_items;
                total.freed_bytes += result.freed_bytes;
                total.skipped_items += result.skipped_items;
                total.errors.extend(result.errors);
            }
            Err(err) => {
                total.errors.push(format!("{}: {}", cleaner.name(), err));
            }
        }

        completed += 1;
    }

    on_progress(1.0, "Done");
    Ok(total)
}
