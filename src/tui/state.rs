use crate::models::{CleanupCategory, CleanupItem};
use crate::tui::action::SafetyLevel;

#[derive(Debug, Clone)]
pub struct State {
    pub current_tab: usize,
    pub selected_index: usize,
    pub items: Vec<CleanupItem>,
    pub total_size: u64,
    pub selected_size: u64,
    pub safety_level: SafetyLevel,
    pub cleanup_in_progress: bool,
    pub cleanup_progress: f64,
    pub should_exit: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            current_tab: 0,
            selected_index: 0,
            items: Vec::new(),
            total_size: 0,
            selected_size: 0,
            safety_level: SafetyLevel::Safe,
            cleanup_in_progress: false,
            cleanup_progress: 0.0,
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
            .filter(|item| item.category == category)
            .count()
    }

    pub fn visible_items(&self) -> Vec<&CleanupItem> {
        let category = self.current_category();
        self.items
            .iter()
            .filter(|item| item.category == category)
            .collect()
    }

    pub fn visible_item_indices(&self) -> Vec<usize> {
        let category = self.current_category();
        self.items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.category == category)
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
}
