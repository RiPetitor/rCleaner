use crate::tui::action::Action;
use crate::tui::state::State;

pub struct Store {
    state: State,
}

const TAB_COUNT: usize = 6;

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
                self.state.active_screen = crate::tui::action::Screen::Main;
                self.state.current_tab = 0;
                self.state.selected_index = 0;
                self.state.cleanup_in_progress = false;
                self.state.cleanup_progress = 0.0;
                self.state.cleanup_step = None;
                self.state.last_result = None;
                self.state.status_message = None;
                self.state.search_query.clear();
                self.state.search_active = false;
            }

            Action::Exit => {
                self.state.should_exit = true;
            }

            Action::Refresh => {
                self.state.status_message = Some("Refreshing...".to_string());
            }

            Action::SetItems(items) => {
                self.state.items = items;
                self.state.selected_index = 0;
                self.state.update_total_size();
                self.state.update_selected_size();
                self.state.status_message = None;
            }

            Action::SetStatus(message) => {
                self.state.status_message = message;
            }

            Action::ChangeTab(index) => {
                self.state.current_tab = index;
                self.state.selected_index = 0;
            }

            Action::NextTab => {
                self.state.current_tab = (self.state.current_tab + 1) % TAB_COUNT;
                self.state.selected_index = 0;
            }

            Action::PrevTab => {
                self.state.current_tab = if self.state.current_tab == 0 {
                    TAB_COUNT - 1
                } else {
                    self.state.current_tab - 1
                };
                self.state.selected_index = 0;
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

            Action::ToggleSelection => {
                if let Some(item_index) = self.state.selected_item_index() {
                    if let Some(item) = self.state.items.get_mut(item_index) {
                        if item.can_clean {
                            item.selected = !item.selected;
                            self.state.update_selected_size();
                        }
                    }
                }
            }

            Action::ToggleAllVisible => {
                let visible_indices = self.state.visible_item_indices();
                let mut selectable_indices = Vec::new();
                for index in visible_indices {
                    if let Some(item) = self.state.items.get(index) {
                        if item.can_clean {
                            selectable_indices.push(index);
                        }
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
            }

            Action::AppendSearch(ch) => {
                self.state.search_query.push(ch);
                self.state.selected_index = 0;
            }

            Action::BackspaceSearch => {
                self.state.search_query.pop();
                self.state.selected_index = 0;
            }

            Action::OpenConfirm => {
                self.state.active_screen = crate::tui::action::Screen::Confirm;
                self.state.search_active = false;
            }

            Action::OpenSettings => {
                self.state.active_screen = crate::tui::action::Screen::Settings;
                self.state.search_active = false;
            }

            Action::BackToMain => {
                self.state.active_screen = crate::tui::action::Screen::Main;
                self.state.cleanup_in_progress = false;
                self.state.cleanup_progress = 0.0;
                self.state.cleanup_step = None;
                self.state.search_active = false;
            }

            Action::StartCleanup => {
                self.state.active_screen = crate::tui::action::Screen::Progress;
                self.state.cleanup_in_progress = true;
                self.state.cleanup_progress = 0.0;
                self.state.cleanup_step = Some("Preparing cleanup...".to_string());
                self.state.search_active = false;
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
