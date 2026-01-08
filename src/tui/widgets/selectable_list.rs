use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem};

pub struct SelectableList {
    items: Vec<(String, bool)>,
    selected: usize,
}

impl SelectableList {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
        }
    }

    pub fn set_items(&mut self, items: Vec<(String, bool)>) {
        self.items = items;
    }

    pub fn next(&mut self) {
        if !self.items.is_empty() {
            self.selected = (self.selected + 1) % self.items.len();
        }
    }

    pub fn previous(&mut self) {
        if !self.items.is_empty() {
            if self.selected == 0 {
                self.selected = self.items.len() - 1;
            } else {
                self.selected -= 1;
            }
        }
    }
}
