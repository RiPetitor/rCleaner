//! Модуль безопасности для проверки элементов перед очисткой.
//!
//! Проверяет:
//! - Права доступа к файлам
//! - Правила whitelist/blacklist
//! - Зависимости пакетов
//! - Защищённые системные пути

mod dependency_check;
mod permissions;
mod rules;

use self::dependency_check::check_dependencies_for_manager;
use self::permissions::{can_clean_path, is_root};
use self::rules::SafetyRules;
use crate::config::Config;
use crate::error::Result;
use crate::models::{CleanupItem, CleanupSource};

/// Проверяет безопасность очистки элементов.
pub struct SafetyChecker {
    config: Config,
    rules: SafetyRules,
}

impl SafetyChecker {
    /// Создаёт новый экземпляр с указанной конфигурацией.
    pub fn new(config: Config) -> Self {
        let rules = SafetyRules::from_config(&config);
        Self { config, rules }
    }

    /// Проверяет, безопасно ли очистить элемент.
    ///
    /// Возвращает `true`, если элемент можно безопасно удалить.
    pub fn is_safe_to_clean(&self, item: &CleanupItem) -> Result<bool> {
        if !self.config.safety.enabled && (!self.config.safety.only_root_can_disable || is_root()) {
            return Ok(true);
        }

        if let Some(ref path) = item.path
            && !can_clean_path(path)
        {
            return Ok(false);
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

    /// Применяет проверки безопасности к элементу.
    ///
    /// Устанавливает `can_clean = false` и `blocked_reason`, если элемент заблокирован.
    pub fn apply_to_item(&self, item: &mut CleanupItem) -> Result<()> {
        if !self.config.safety.enabled && (!self.config.safety.only_root_can_disable || is_root()) {
            return Ok(());
        }

        if let Some(ref path) = item.path
            && !can_clean_path(path)
        {
            mark_blocked(item, "Insufficient permissions to clean path");
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

/// Помечает элемент как заблокированный.
fn mark_blocked(item: &mut CleanupItem, reason: &str) {
    item.can_clean = false;
    if item.blocked_reason.is_none() {
        item.blocked_reason = Some(reason.to_string());
    }
}
