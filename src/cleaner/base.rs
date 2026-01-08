//! Базовый trait для модулей очистки.

use crate::error::Result;
use crate::models::{CleanupCategory, CleanupItem, CleanupResult};

/// Trait для модулей очистки.
///
/// Каждый модуль очистки (кэш, логи, пакеты и т.д.) реализует этот trait.
pub trait Cleaner {
    /// Возвращает имя модуля очистки.
    fn name(&self) -> &str;

    /// Возвращает категорию очистки.
    fn category(&self) -> CleanupCategory;

    /// Сканирует систему и возвращает список элементов для очистки.
    fn scan(&self) -> Result<Vec<CleanupItem>>;

    /// Выполняет очистку выбранных элементов.
    ///
    /// # Arguments
    ///
    /// * `items` - элементы для очистки
    /// * `dry_run` - если `true`, только симуляция без реального удаления
    fn clean(&self, items: &[CleanupItem], dry_run: bool) -> Result<CleanupResult>;

    /// Проверяет, можно ли очистить элемент.
    fn can_clean(&self, item: &CleanupItem) -> bool {
        item.can_clean
    }
}
