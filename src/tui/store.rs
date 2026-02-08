// Copyright (C) 2026 RiPetitor
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::tui::action::Action;
use crate::tui::state::State;

pub struct Store {
    state: State,
}

const TAB_COUNT: usize = 7;

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

impl Store {
    pub fn new() -> Self {
        Self {
            state: State::new(),
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn update(&mut self, action: Action) {
        match action {
            Action::Init => {
                self.state = State::new();
            }

            Action::Exit => {
                self.state.should_exit = true;
            }

            Action::Refresh => {
                self.state.status_message = Some("Scanning...".to_string());
            }

            Action::SetItems(items) => {
                self.state.items = items;
                self.state.selected_index = 0;
                self.state.scroll_offset = 0;
                self.state.update_total_size();
                self.state.update_selected_size();
                self.state.status_message = None;
            }

            Action::SetStatus(message) => {
                self.state.status_message = message;
            }

            Action::ChangeTab(index) => {
                if index < TAB_COUNT {
                    self.state.current_tab = index;
                    self.state.selected_index = 0;
                    self.state.scroll_offset = 0;
                }
            }

            Action::NextTab => {
                self.state.current_tab = (self.state.current_tab + 1) % TAB_COUNT;
                self.state.selected_index = 0;
                self.state.scroll_offset = 0;
            }

            Action::PrevTab => {
                self.state.current_tab = if self.state.current_tab == 0 {
                    TAB_COUNT - 1
                } else {
                    self.state.current_tab - 1
                };
                self.state.selected_index = 0;
                self.state.scroll_offset = 0;
            }

            Action::SelectItem(index) => {
                let visible_count = self.state.visible_items_len();
                if visible_count == 0 {
                    self.state.selected_index = 0;
                } else {
                    self.state.selected_index = index.min(visible_count - 1);
                }
            }

            Action::SelectNext => {
                let visible_count = self.state.visible_items_len();
                if visible_count == 0 {
                    self.state.selected_index = 0;
                } else {
                    self.state.selected_index = (self.state.selected_index + 1) % visible_count;
                }
            }

            Action::SelectPrev => {
                let visible_count = self.state.visible_items_len();
                if visible_count == 0 {
                    self.state.selected_index = 0;
                } else if self.state.selected_index == 0 {
                    self.state.selected_index = visible_count - 1;
                } else {
                    self.state.selected_index -= 1;
                }
            }

            Action::SelectPageDown(page_size) => {
                let visible_count = self.state.visible_items_len();
                if visible_count == 0 {
                    self.state.selected_index = 0;
                } else {
                    self.state.selected_index =
                        (self.state.selected_index + page_size).min(visible_count - 1);
                }
            }

            Action::SelectPageUp(page_size) => {
                self.state.selected_index = self.state.selected_index.saturating_sub(page_size);
            }

            Action::SelectFirst => {
                self.state.selected_index = 0;
            }

            Action::SelectLast => {
                let visible_count = self.state.visible_items_len();
                if visible_count == 0 {
                    self.state.selected_index = 0;
                } else {
                    self.state.selected_index = visible_count - 1;
                }
            }

            Action::ToggleSelection => {
                if let Some(item_index) = self.state.selected_item_index()
                    && let Some(item) = self.state.items.get_mut(item_index)
                    && item.can_clean
                {
                    item.selected = !item.selected;
                    self.state.update_selected_size();
                }
            }

            Action::ToggleAllVisible => {
                let visible_indices = self.state.visible_item_indices();
                let mut selectable_indices = Vec::new();
                for index in visible_indices {
                    if let Some(item) = self.state.items.get(index)
                        && item.can_clean
                    {
                        selectable_indices.push(index);
                    }
                }

                let should_select = selectable_indices
                    .iter()
                    .any(|index| !self.state.items[*index].selected);

                for index in selectable_indices {
                    if let Some(item) = self.state.items.get_mut(index) {
                        item.selected = should_select;
                    }
                }

                self.state.update_selected_size();
            }

            Action::StartSearch => {
                self.state.search_active = true;
                self.state.status_message = Some("Search mode".to_string());
            }

            Action::EndSearch => {
                self.state.search_active = false;
                self.state.status_message = None;
            }

            Action::ClearSearch => {
                self.state.search_query.clear();
                self.state.selected_index = 0;
                self.state.scroll_offset = 0;
            }

            Action::AppendSearch(ch) => {
                self.state.search_query.push(ch);
                self.state.selected_index = 0;
                self.state.scroll_offset = 0;
            }

            Action::BackspaceSearch => {
                self.state.search_query.pop();
                self.state.selected_index = 0;
                self.state.scroll_offset = 0;
            }

            Action::BeginSettingsEdit(target, input) => {
                self.state.settings_edit = Some(target);
                self.state.settings_input = input;
            }

            Action::EndSettingsEdit => {
                self.state.settings_edit = None;
                self.state.settings_input.clear();
            }

            Action::AppendSettingsInput(ch) => {
                self.state.settings_input.push(ch);
            }

            Action::BackspaceSettingsInput => {
                self.state.settings_input.pop();
            }

            Action::OpenConfirm => {
                self.state.active_screen = crate::tui::action::Screen::Confirm;
                self.state.search_active = false;
                self.state.settings_edit = None;
            }

            Action::OpenSettings => {
                self.state.active_screen = crate::tui::action::Screen::Settings;
                self.state.search_active = false;
                self.state.settings_block = 0;
                self.state.settings_row = 0;
            }

            Action::NextSettingsBlock => {
                self.state.settings_block = (self.state.settings_block + 1) % 3;
                self.state.settings_row = 0;
            }

            Action::SettingsRowNext(max_rows) => {
                if max_rows > 0 {
                    self.state.settings_row = (self.state.settings_row + 1).min(max_rows - 1);
                }
            }

            Action::SettingsRowPrev => {
                self.state.settings_row = self.state.settings_row.saturating_sub(1);
            }

            Action::BackToMain => {
                self.state.active_screen = crate::tui::action::Screen::Main;
                self.state.cleanup_in_progress = false;
                self.state.cleanup_progress = 0.0;
                self.state.cleanup_step = None;
                self.state.search_active = false;
                self.state.settings_edit = None;
            }

            Action::StartCleanup => {
                self.state.active_screen = crate::tui::action::Screen::Progress;
                self.state.cleanup_in_progress = true;
                self.state.cleanup_progress = 0.0;
                self.state.cleanup_step = Some("Preparing cleanup...".to_string());
                self.state.search_active = false;
                self.state.settings_edit = None;
            }

            Action::CancelCleanup => {
                self.state.cleanup_in_progress = false;
                self.state.cleanup_progress = 0.0;
                self.state.cleanup_step = None;
                self.state.active_screen = crate::tui::action::Screen::Main;
            }

            Action::CleanupProgress { progress, step } => {
                self.state.cleanup_progress = progress;
                self.state.cleanup_step = step;
            }

            Action::ChangeSafetyLevel(level) => {
                self.state.safety_level = level;
            }

            Action::FinishCleanup(result) => {
                self.state.cleanup_in_progress = false;
                self.state.cleanup_progress = 1.0;
                self.state.cleanup_step = None;
                self.state.last_result = Some(result);
                self.state.active_screen = crate::tui::action::Screen::Results;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CleanupCategory, CleanupItem, CleanupResult, CleanupSource};
    use crate::tui::action::{SafetyLevel, Screen, SettingsEdit};

    fn make_item(id: &str, category: CleanupCategory, size: u64) -> CleanupItem {
        CleanupItem {
            id: id.to_string(),
            name: id.to_string(),
            path: Some(format!("/tmp/{id}")),
            size,
            description: format!("Test item {id}"),
            category,
            source: CleanupSource::FileSystem,
            selected: false,
            can_clean: true,
            blocked_reason: None,
            dependencies: Vec::new(),
        }
    }

    fn store_with_items() -> Store {
        let mut store = Store::new();
        store.update(Action::Init);
        store.update(Action::SetItems(vec![
            make_item("cache1", CleanupCategory::Cache, 1024),
            make_item("cache2", CleanupCategory::Cache, 2048),
            make_item("log1", CleanupCategory::Logs, 4096),
            make_item("temp1", CleanupCategory::TempFiles, 512),
        ]));
        store
    }

    #[test]
    fn test_init() {
        let mut store = Store::new();
        store.update(Action::Init);
        assert_eq!(store.state().active_screen, Screen::Main);
        assert_eq!(store.state().current_tab, 0);
        assert!(!store.state().should_exit);
    }

    #[test]
    fn test_exit() {
        let mut store = Store::new();
        store.update(Action::Exit);
        assert!(store.state().should_exit);
    }

    #[test]
    fn test_set_items_updates_sizes() {
        let store = store_with_items();
        assert_eq!(store.state().items.len(), 4);
        assert_eq!(store.state().total_size, 1024 + 2048 + 4096 + 512);
        assert_eq!(store.state().selected_size, 0);
    }

    #[test]
    fn test_tab_navigation() {
        let mut store = Store::new();
        store.update(Action::Init);

        store.update(Action::NextTab);
        assert_eq!(store.state().current_tab, 1);

        store.update(Action::PrevTab);
        assert_eq!(store.state().current_tab, 0);

        store.update(Action::PrevTab);
        assert_eq!(store.state().current_tab, 6);

        store.update(Action::NextTab);
        assert_eq!(store.state().current_tab, 0);

        store.update(Action::ChangeTab(3));
        assert_eq!(store.state().current_tab, 3);

        // Invalid tab ignored
        store.update(Action::ChangeTab(99));
        assert_eq!(store.state().current_tab, 3);
    }

    #[test]
    fn test_item_selection_navigation() {
        let mut store = store_with_items();
        // Tab 0 = Cache, has 2 items
        assert_eq!(store.state().visible_items_len(), 2);

        store.update(Action::SelectNext);
        assert_eq!(store.state().selected_index, 1);

        store.update(Action::SelectNext);
        assert_eq!(store.state().selected_index, 0); // wraps

        store.update(Action::SelectPrev);
        assert_eq!(store.state().selected_index, 1); // wraps back

        store.update(Action::SelectFirst);
        assert_eq!(store.state().selected_index, 0);

        store.update(Action::SelectLast);
        assert_eq!(store.state().selected_index, 1);
    }

    #[test]
    fn test_page_navigation() {
        let mut store = store_with_items();
        store.update(Action::SelectPageDown(10));
        assert_eq!(store.state().selected_index, 1); // clamped to max

        store.update(Action::SelectPageUp(10));
        assert_eq!(store.state().selected_index, 0); // clamped to 0
    }

    #[test]
    fn test_toggle_selection() {
        let mut store = store_with_items();
        assert_eq!(store.state().selected_count(), 0);

        store.update(Action::ToggleSelection);
        assert_eq!(store.state().selected_count(), 1);
        assert_eq!(store.state().selected_size, 1024);

        store.update(Action::ToggleSelection);
        assert_eq!(store.state().selected_count(), 0);
        assert_eq!(store.state().selected_size, 0);
    }

    #[test]
    fn test_toggle_all_visible() {
        let mut store = store_with_items();
        store.update(Action::ToggleAllVisible);
        assert_eq!(store.state().selected_count(), 2); // only cache tab
        assert_eq!(store.state().selected_size, 1024 + 2048);

        store.update(Action::ToggleAllVisible);
        assert_eq!(store.state().selected_count(), 0);
    }

    #[test]
    fn test_blocked_item_not_selectable() {
        let mut store = Store::new();
        store.update(Action::Init);
        let mut item = make_item("blocked", CleanupCategory::Cache, 100);
        item.can_clean = false;
        item.blocked_reason = Some("Protected".to_string());
        store.update(Action::SetItems(vec![item]));

        store.update(Action::ToggleSelection);
        assert_eq!(store.state().selected_count(), 0);

        store.update(Action::ToggleAllVisible);
        assert_eq!(store.state().selected_count(), 0);
    }

    #[test]
    fn test_search() {
        let mut store = store_with_items();

        store.update(Action::StartSearch);
        assert!(store.state().search_active);

        store.update(Action::AppendSearch('c'));
        store.update(Action::AppendSearch('a'));
        assert_eq!(store.state().search_query, "ca");
        assert_eq!(store.state().visible_items_len(), 2); // cache1, cache2

        store.update(Action::AppendSearch('c'));
        store.update(Action::AppendSearch('h'));
        store.update(Action::AppendSearch('e'));
        store.update(Action::AppendSearch('1'));
        assert_eq!(store.state().search_query, "cache1");
        assert_eq!(store.state().visible_items_len(), 1); // cache1 only

        store.update(Action::BackspaceSearch);
        assert_eq!(store.state().search_query, "cache");

        store.update(Action::EndSearch);
        assert!(!store.state().search_active);

        store.update(Action::ClearSearch);
        assert!(store.state().search_query.is_empty());
    }

    #[test]
    fn test_screen_navigation() {
        let mut store = store_with_items();

        store.update(Action::OpenSettings);
        assert_eq!(store.state().active_screen, Screen::Settings);

        store.update(Action::BackToMain);
        assert_eq!(store.state().active_screen, Screen::Main);

        // Select an item first
        store.update(Action::ToggleSelection);
        store.update(Action::OpenConfirm);
        assert_eq!(store.state().active_screen, Screen::Confirm);

        store.update(Action::BackToMain);
        assert_eq!(store.state().active_screen, Screen::Main);
    }

    #[test]
    fn test_cleanup_lifecycle() {
        let mut store = store_with_items();

        store.update(Action::StartCleanup);
        assert_eq!(store.state().active_screen, Screen::Progress);
        assert!(store.state().cleanup_in_progress);
        assert_eq!(store.state().cleanup_progress, 0.0);

        store.update(Action::CleanupProgress {
            progress: 0.5,
            step: Some("Cleaning cache...".to_string()),
        });
        assert_eq!(store.state().cleanup_progress, 0.5);

        let result = CleanupResult {
            cleaned_items: 2,
            freed_bytes: 3072,
            skipped_items: 0,
            errors: vec![],
        };
        store.update(Action::FinishCleanup(result));
        assert_eq!(store.state().active_screen, Screen::Results);
        assert!(!store.state().cleanup_in_progress);
        assert!(store.state().last_result.is_some());
        assert_eq!(store.state().last_result.as_ref().unwrap().cleaned_items, 2);
    }

    #[test]
    fn test_cancel_cleanup() {
        let mut store = store_with_items();
        store.update(Action::StartCleanup);
        store.update(Action::CancelCleanup);
        assert_eq!(store.state().active_screen, Screen::Main);
        assert!(!store.state().cleanup_in_progress);
    }

    #[test]
    fn test_safety_level() {
        let mut store = Store::new();
        store.update(Action::Init);
        assert_eq!(store.state().safety_level, SafetyLevel::Safe);

        store.update(Action::ChangeSafetyLevel(SafetyLevel::Aggressive));
        assert_eq!(store.state().safety_level, SafetyLevel::Aggressive);
    }

    #[test]
    fn test_settings_edit() {
        let mut store = Store::new();
        store.update(Action::Init);

        store.update(Action::BeginSettingsEdit(
            SettingsEdit::Whitelist,
            "~/Documents".to_string(),
        ));
        assert_eq!(store.state().settings_edit, Some(SettingsEdit::Whitelist));
        assert_eq!(store.state().settings_input, "~/Documents");

        store.update(Action::AppendSettingsInput(','));
        store.update(Action::AppendSettingsInput(' '));
        assert_eq!(store.state().settings_input, "~/Documents, ");

        store.update(Action::BackspaceSettingsInput);
        assert_eq!(store.state().settings_input, "~/Documents,");

        store.update(Action::EndSettingsEdit);
        assert_eq!(store.state().settings_edit, None);
        assert!(store.state().settings_input.is_empty());
    }

    #[test]
    fn test_status_message() {
        let mut store = Store::new();
        store.update(Action::Init);

        store.update(Action::SetStatus(Some("Hello".to_string())));
        assert_eq!(store.state().status_message.as_deref(), Some("Hello"));

        store.update(Action::SetStatus(None));
        assert!(store.state().status_message.is_none());
    }

    #[test]
    fn test_refresh() {
        let mut store = Store::new();
        store.update(Action::Init);
        store.update(Action::Refresh);
        assert_eq!(store.state().status_message.as_deref(), Some("Scanning..."));
    }

    #[test]
    fn test_tab_item_count() {
        let store = store_with_items();
        assert_eq!(store.state().tab_item_count(0), 2); // Cache
        assert_eq!(store.state().tab_item_count(3), 1); // TempFiles
        assert_eq!(store.state().tab_item_count(4), 1); // Logs
        assert_eq!(store.state().tab_item_count(1), 0); // Apps
    }
}
