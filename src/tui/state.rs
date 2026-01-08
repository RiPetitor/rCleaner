use crate::models::{CleanupCategory, CleanupItem, CleanupResult};
use crate::tui::action::{SafetyLevel, Screen, SettingsEdit};

#[derive(Debug, Clone)]
pub struct State {
    pub active_screen: Screen,
    pub current_tab: usize,
    pub selected_index: usize,
    pub items: Vec<CleanupItem>,
    pub total_size: u64,
    pub selected_size: u64,
    pub safety_level: SafetyLevel,
    pub cleanup_in_progress: bool,
    pub cleanup_progress: f64,
    pub cleanup_step: Option<String>,
    pub last_result: Option<CleanupResult>,
    pub status_message: Option<String>,
    pub search_query: String,
    pub search_active: bool,
    pub settings_edit: Option<SettingsEdit>,
    pub settings_input: String,
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
    pub fn new() -> Self {
        Self::default()
    }

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

    pub fn visible_items_len(&self) -> usize {
        let category = self.current_category();
        self.items
            .iter()
            .filter(|item| item.category == category && self.matches_search(item))
            .count()
    }

    pub fn visible_items(&self) -> Vec<&CleanupItem> {
        let category = self.current_category();
        self.items
            .iter()
            .filter(|item| item.category == category && self.matches_search(item))
            .collect()
    }

    pub fn visible_item_indices(&self) -> Vec<usize> {
        let category = self.current_category();
        self.items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.category == category && self.matches_search(item))
            .map(|(index, _)| index)
            .collect()
    }

    pub fn selected_item_index(&self) -> Option<usize> {
        self.visible_item_indices()
            .get(self.selected_index)
            .copied()
    }

    pub fn selected_item(&self) -> Option<&CleanupItem> {
        self.selected_item_index()
            .and_then(|index| self.items.get(index))
    }

    pub fn update_total_size(&mut self) {
        self.total_size = self.items.iter().map(|item| item.size).sum();
    }

    pub fn update_selected_size(&mut self) {
        self.selected_size = self
            .items
            .iter()
            .filter(|item| item.selected)
            .map(|item| item.size)
            .sum();
    }

    pub fn selected_items(&self) -> Vec<CleanupItem> {
        self.items
            .iter()
            .filter(|item| item.selected)
            .cloned()
            .collect()
    }

    pub fn selected_count(&self) -> usize {
        self.items.iter().filter(|item| item.selected).count()
    }

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
        if let Some(path) = &item.path {
            if path.to_lowercase().contains(&query) {
                return true;
            }
        }
        false
    }
}
