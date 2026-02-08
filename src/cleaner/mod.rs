// Copyright (C) 2026 RiPetitor
// SPDX-License-Identifier: GPL-3.0-or-later

//! Модули очистки системы.
//!
//! Каждый модуль отвечает за свою категорию:
//! - [`cache`] - кэш браузеров и приложений
//! - [`applications`] - Flatpak и Snap приложения
//! - [`temp_files`] - временные файлы
//! - [`logs`] - журналы и логи
//! - [`old_packages`] - старые пакеты
//! - [`old_kernels`] - старые ядра

pub mod applications;
pub mod base;
pub mod cache;
pub mod logs;
pub mod old_kernels;
pub mod old_packages;
pub mod system_packages;
pub mod temp_files;

use crate::cleaner::base::Cleaner;
use crate::config::Config;
use crate::error::Result;
use crate::models::{CleanupCategory, CleanupItem, CleanupResult};
use crate::safety::SafetyChecker;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

fn all_cleaners() -> Vec<Box<dyn Cleaner + Send>> {
    vec![
        Box::new(cache::CacheCleaner::new()),
        Box::new(applications::ApplicationsCleaner::new()),
        Box::new(temp_files::TempFilesCleaner::new()),
        Box::new(logs::LogsCleaner::new()),
        Box::new(old_packages::OldPackagesCleaner::new()),
        Box::new(old_kernels::OldKernelsCleaner::new()),
        Box::new(system_packages::SystemPackagesCleaner::new()),
    ]
}

/// Сканирует все категории параллельно и возвращает список элементов для очистки.
pub fn scan_all() -> Result<Vec<CleanupItem>> {
    scan_all_with_progress(|_, _| {})
}

/// Сканирует все категории параллельно с отслеживанием прогресса.
pub fn scan_all_with_progress<F>(on_progress: F) -> Result<Vec<CleanupItem>>
where
    F: Fn(f64, &str) + Send + Sync,
{
    let cleaners = all_cleaners();
    let total = cleaners.len();
    let completed = Arc::new(Mutex::new(0usize));
    let on_progress = Arc::new(on_progress);

    let results: Vec<(String, std::result::Result<Vec<CleanupItem>, String>)> = cleaners
        .into_par_iter()
        .map(|cleaner| {
            let name = cleaner.name().to_string();
            let result = cleaner.scan().map_err(|e| e.to_string());

            let mut count = completed.lock().unwrap();
            *count += 1;
            let progress = *count as f64 / total as f64;
            on_progress(progress, &name);

            (name, result)
        })
        .collect();

    let mut items = Vec::new();
    for (name, result) in results {
        match result {
            Ok(mut scanned) => items.append(&mut scanned),
            Err(err) => log::warn!("{} scan failed: {}", name, err),
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
            if item.blocked_reason.is_none() {
                item.blocked_reason = Some(format!("Safety check failed: {}", err));
            }
        }
    }

    Ok(items)
}

/// Очищает выбранные элементы.
pub fn clean_selected(items: &[CleanupItem], dry_run: bool) -> Result<CleanupResult> {
    clean_selected_with_progress(items, dry_run, |_progress, _label| {})
}

/// Очищает выбранные элементы с отслеживанием прогресса.
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
        (
            CleanupCategory::SystemPackages,
            Box::new(system_packages::SystemPackagesCleaner::new()),
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
