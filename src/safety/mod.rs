mod blacklist;
mod dependency_check;
mod permissions;
mod rules;
mod whitelist;

use self::dependency_check::check_dependencies_for_manager;
use self::permissions::{can_clean_path, is_root};
use self::rules::SafetyRules;
use crate::config::Config;
use crate::error::Result;
use crate::models::{CleanupItem, CleanupSource};

pub struct SafetyChecker {
    config: Config,
    rules: SafetyRules,
}

impl SafetyChecker {
    pub fn new(config: Config) -> Self {
        let rules = SafetyRules::from_config(&config);
        Self { config, rules }
    }

    pub fn is_safe_to_clean(&self, item: &CleanupItem) -> Result<bool> {
        if !self.config.safety.enabled {
            if !self.config.safety.only_root_can_disable || is_root() {
                return Ok(true);
            }
        }

        if let Some(ref path) = item.path {
            if !can_clean_path(path) {
                return Ok(false);
            }
        }

        if !self.rules.check_item(item) {
            return Ok(false);
        }

        if let CleanupSource::PackageManager(manager) = &item.source {
            let deps = check_dependencies_for_manager(manager, &item.name)?;
            if !deps.is_empty() {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn apply_to_item(&self, item: &mut CleanupItem) -> Result<()> {
        if !self.config.safety.enabled {
            if !self.config.safety.only_root_can_disable || is_root() {
                return Ok(());
            }
        }

        if let Some(ref path) = item.path {
            if !can_clean_path(path) {
                mark_blocked(item, "Insufficient permissions to clean path");
            }
        }

        if let Some(reason) = self.rules.check_item_reason(item) {
            mark_blocked(item, &reason);
        }

        if let CleanupSource::PackageManager(manager) = &item.source {
            let deps = check_dependencies_for_manager(manager, &item.name)?;
            if !deps.is_empty() {
                item.dependencies = deps;
                mark_blocked(item, "Package has dependents");
            }
        }

        Ok(())
    }
}

fn mark_blocked(item: &mut CleanupItem, reason: &str) {
    item.can_clean = false;
    if item.blocked_reason.is_none() {
        item.blocked_reason = Some(reason.to_string());
    }
}
