use crate::tui::action::Action;
use crate::tui::state::State;

pub struct Store {
    state: State,
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
                self.state.current_tab = 0;
                self.state.selected_index = 0;
                self.state.cleanup_in_progress = false;
                self.state.cleanup_progress = 0.0;
            }

            Action::Exit => {
                self.state.should_exit = true;
            }

            Action::UpdateCache => {}

            Action::ChangeTab(index) => {
                self.state.current_tab = index;
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

            Action::StartCleanup => {
                self.state.cleanup_in_progress = true;
                self.state.cleanup_progress = 0.0;
            }

            Action::CancelCleanup => {
                self.state.cleanup_in_progress = false;
                self.state.cleanup_progress = 0.0;
            }

            Action::CleanupProgress(progress) => {
                self.state.cleanup_progress = progress;
            }

            Action::OpenSettings => {}

            Action::ChangeSafetyLevel(level) => {
                self.state.safety_level = level;
            }

            Action::ConfirmCleanup => {
                self.state.should_exit = true;
            }
        }
    }

    pub fn set_items(&mut self, items: Vec<crate::models::CleanupItem>) {
        self.state.items = items;
        self.state.selected_index = 0;
        self.state.update_total_size();
        self.state.update_selected_size();
    }
}
