//! Состояние приложения.

use crate::models::{CleanupCategory, CleanupItem, CleanupResult};
use crate::tui::action::{SafetyLevel, Screen, SettingsEdit};

/// Состояние TUI приложения.
#[derive(Debug, Clone)]
pub struct State {
    /// Текущий экран.
    pub active_screen: Screen,
    /// Индекс текущей вкладки (0-5).
    pub current_tab: usize,
    /// Индекс выбранного элемента в текущей вкладке.
    pub selected_index: usize,
    /// Все элементы для очистки.
    pub items: Vec<CleanupItem>,
    /// Общий размер всех элементов.
    pub total_size: u64,
    /// Размер выбранных элементов.
    pub selected_size: u64,
    /// Уровень безопасности.
    pub safety_level: SafetyLevel,
    /// Идёт ли очистка.
    pub cleanup_in_progress: bool,
    /// Прогресс очистки (0.0..1.0).
    pub cleanup_progress: f64,
    /// Текущий шаг очистки.
    pub cleanup_step: Option<String>,
    /// Результат последней очистки.
    pub last_result: Option<CleanupResult>,
    /// Статусное сообщение.
    pub status_message: Option<String>,
    /// Поисковый запрос.
    pub search_query: String,
    /// Активен ли режим поиска.
    pub search_active: bool,
    /// Текущее редактирование настроек.
    pub settings_edit: Option<SettingsEdit>,
    /// Ввод в настройках.
    pub settings_input: String,
    /// Флаг выхода из приложения.
    pub should_exit: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            active_screen: Screen::Main,
            current_tab: 0,
            selected_index: 0,
            items: Vec::new(),
            total_size: 0,
            selected_size: 0,
            safety_level: SafetyLevel::Safe,
            cleanup_in_progress: false,
            cleanup_progress: 0.0,
            cleanup_step: None,
            last_result: None,
            status_message: None,
            search_query: String::new(),
            search_active: false,
            settings_edit: None,
            settings_input: String::new(),
            should_exit: false,
        }
    }
}

impl State {
    /// Создаёт новое состояние по умолчанию.
    pub fn new() -> Self {
        Self::default()
    }

    /// Возвращает категорию текущей вкладки.
    pub fn current_category(&self) -> CleanupCategory {
        match self.current_tab {
            0 => CleanupCategory::Cache,
            1 => CleanupCategory::Applications,
            2 => CleanupCategory::TempFiles,
            3 => CleanupCategory::Logs,
            4 => CleanupCategory::OldPackages,
            _ => CleanupCategory::OldKernels,
        }
    }

    /// Возвращает количество видимых элементов в текущей вкладке.
    pub fn visible_items_len(&self) -> usize {
        let category = self.current_category();
        self.items
            .iter()
            .filter(|item| item.category == category && self.matches_search(item))
            .count()
    }

    /// Возвращает видимые элементы в текущей вкладке.
    pub fn visible_items(&self) -> Vec<&CleanupItem> {
        let category = self.current_category();
        self.items
            .iter()
            .filter(|item| item.category == category && self.matches_search(item))
            .collect()
    }

    /// Возвращает индексы видимых элементов.
    pub fn visible_item_indices(&self) -> Vec<usize> {
        let category = self.current_category();
        self.items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.category == category && self.matches_search(item))
            .map(|(index, _)| index)
            .collect()
    }

    /// Возвращает глобальный индекс выбранного элемента.
    pub fn selected_item_index(&self) -> Option<usize> {
        self.visible_item_indices()
            .get(self.selected_index)
            .copied()
    }

    /// Возвращает выбранный элемент.
    pub fn selected_item(&self) -> Option<&CleanupItem> {
        self.selected_item_index()
            .and_then(|index| self.items.get(index))
    }

    /// Обновляет общий размер всех элементов.
    pub fn update_total_size(&mut self) {
        self.total_size = self.items.iter().map(|item| item.size).sum();
    }

    /// Обновляет размер выбранных элементов.
    pub fn update_selected_size(&mut self) {
        self.selected_size = self
            .items
            .iter()
            .filter(|item| item.selected)
            .map(|item| item.size)
            .sum();
    }

    /// Возвращает список выбранных элементов.
    pub fn selected_items(&self) -> Vec<CleanupItem> {
        self.items
            .iter()
            .filter(|item| item.selected)
            .cloned()
            .collect()
    }

    /// Возвращает количество выбранных элементов.
    pub fn selected_count(&self) -> usize {
        self.items.iter().filter(|item| item.selected).count()
    }

    /// Проверяет, соответствует ли элемент поисковому запросу.
    fn matches_search(&self, item: &CleanupItem) -> bool {
        let query = self.search_query.trim();
        if query.is_empty() {
            return true;
        }
        let query = query.to_lowercase();
        if item.name.to_lowercase().contains(&query) {
            return true;
        }
        if item.description.to_lowercase().contains(&query) {
            return true;
        }
        if let Some(path) = &item.path
            && path.to_lowercase().contains(&query)
        {
            return true;
        }
        false
    }
}
