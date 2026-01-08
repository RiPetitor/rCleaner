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
        let safety_disabled =
            !self.config.safety.enabled && (!self.config.safety.only_root_can_disable || is_root());

        if item.id == "systemd-journal" && !is_root() {
            return Ok(false);
        }

        if let CleanupSource::PackageManager(manager) = &item.source
            && requires_root(manager)
            && !is_root()
        {
            return Ok(false);
        }

        if let Some(ref path) = item.path
            && !can_clean_path(path)
        {
            return Ok(false);
        }

        if !safety_disabled {
            if !self.rules.check_item(item) {
                return Ok(false);
            }

            if let CleanupSource::PackageManager(manager) = &item.source {
                let deps = check_dependencies_for_manager(manager, &item.name)?;
                if !deps.is_empty() {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// Применяет проверки безопасности к элементу.
    ///
    /// Устанавливает `can_clean = false` и `blocked_reason`, если элемент заблокирован.
    pub fn apply_to_item(&self, item: &mut CleanupItem) -> Result<()> {
        let safety_disabled =
            !self.config.safety.enabled && (!self.config.safety.only_root_can_disable || is_root());

        if item.id == "systemd-journal" && !is_root() {
            mark_blocked(item, "Root required to manage systemd journal");
            return Ok(());
        }

        if let CleanupSource::PackageManager(manager) = &item.source
            && requires_root(manager)
            && !is_root()
        {
            mark_blocked(item, "Root required to manage packages");
            return Ok(());
        }

        if let Some(ref path) = item.path
            && !can_clean_path(path)
        {
            mark_blocked(item, "Insufficient permissions to clean path");
            if safety_disabled {
                return Ok(());
            }
        }

        if !safety_disabled {
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

fn requires_root(manager: &str) -> bool {
    matches!(
        manager,
        "apt" | "dnf" | "rpm" | "pacman" | "snap" | "rpm-ostree"
    )
}
