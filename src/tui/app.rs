use crate::cleaner;
use crate::error::Result;
use crate::tui::action::Action;
use crate::tui::dispatcher::Dispatcher;
use crate::tui::state::State;
use crate::utils::size_format::format_size;
use crate::{NAME, VERSION};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, KeyCode, KeyEventKind},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
};

const TAB_TITLES: [&str; 6] = ["Cache", "Apps", "Temp", "Logs", "Packages", "Kernels"];

pub struct App {
    dispatcher: Dispatcher,
}

impl App {
    pub fn new() -> Self {
        let mut dispatcher = Dispatcher::new();
        dispatcher.dispatch(Action::Init);

        Self { dispatcher }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.refresh_items();

        loop {
            let state = self.dispatcher.store().state().clone();

            if state.should_exit {
                break;
            }

            terminal.draw(|frame| self.draw(frame, &state))?;

            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    self.handle_key_event(key);
                }
            }
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame, state: &State) {
        let size = frame.area();

        let chunks = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(size);

        let header = Paragraph::new(format!(
            "{} v{} | Safety: {:?}",
            NAME, VERSION, state.safety_level
        ))
        .block(Block::default().borders(Borders::ALL));

        frame.render_widget(header, chunks[0]);

        let content_chunks = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(chunks[1]);

        let titles = TAB_TITLES
            .iter()
            .map(|title| Line::from(format!(" {title} ")));
        let tabs = Tabs::new(titles)
            .select(state.current_tab)
            .block(Block::default().borders(Borders::ALL).title("Categories"))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().fg(Color::White))
            .divider("|");

        frame.render_widget(tabs, content_chunks[0]);

        let visible_items = state.visible_items();
        if visible_items.is_empty() {
            let tab_name = TAB_TITLES.get(state.current_tab).unwrap_or(&"Unknown");
            let empty = Paragraph::new(format!("No items in {tab_name}. Press [R] to refresh."))
                .block(Block::default().borders(Borders::ALL).title("Items"));
            frame.render_widget(empty, content_chunks[1]);
        } else {
            let items: Vec<ListItem> = visible_items
                .iter()
                .map(|item| {
                    let marker = if item.selected { "[x]" } else { "[ ]" };
                    let size = format_size(item.size);
                    ListItem::new(format!("{marker} {} ({size})", item.name))
                })
                .collect();

            let mut list_state = ListState::default();
            let selected = state.selected_index.min(items.len().saturating_sub(1));
            list_state.select(Some(selected));

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Items"))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                .highlight_symbol("> ");

            frame.render_stateful_widget(list, content_chunks[1], &mut list_state);
        }

        let info = match state.selected_item() {
            Some(item) => format!(
                "{} | {} | {}",
                item.name,
                format_size(item.size),
                item.description
            ),
            None => "No item selected.".to_string(),
        };

        let info_panel =
            Paragraph::new(info).block(Block::default().borders(Borders::ALL).title("Info"));
        frame.render_widget(info_panel, content_chunks[2]);

        let footer = Paragraph::new(format!(
            "[Q] Quit  [R] Refresh  [Enter] Clean  [Space] Toggle  Selected: {} / {}",
            format_size(state.selected_size),
            format_size(state.total_size)
        ))
        .block(Block::default().borders(Borders::ALL));

        frame.render_widget(footer, chunks[2]);
    }

    fn handle_key_event(&mut self, key: event::KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.dispatcher.dispatch(Action::Exit);
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.dispatcher.dispatch(Action::UpdateCache);
                self.refresh_items();
            }
            KeyCode::Enter => {
                self.dispatcher.dispatch(Action::StartCleanup);
            }
            KeyCode::Tab => {
                let current = self.dispatcher.store().state().current_tab;
                let max_tabs = TAB_TITLES.len();
                let next = (current + 1) % max_tabs;
                self.dispatcher.dispatch(Action::ChangeTab(next));
            }
            KeyCode::BackTab => {
                let current = self.dispatcher.store().state().current_tab;
                let max_tabs = TAB_TITLES.len();
                let next = if current == 0 {
                    max_tabs - 1
                } else {
                    current - 1
                };
                self.dispatcher.dispatch(Action::ChangeTab(next));
            }
            KeyCode::Down => {
                let current = self.dispatcher.store().state().selected_index;
                let items_count = self.dispatcher.store().state().visible_items_len();
                let next = if items_count == 0 {
                    0
                } else {
                    (current + 1) % items_count
                };
                self.dispatcher.dispatch(Action::SelectItem(next));
            }
            KeyCode::Up => {
                let current = self.dispatcher.store().state().selected_index;
                let items_count = self.dispatcher.store().state().visible_items_len();
                let next = if items_count == 0 {
                    0
                } else if current == 0 {
                    items_count - 1
                } else {
                    current - 1
                };
                self.dispatcher.dispatch(Action::SelectItem(next));
            }
            KeyCode::Char(' ') => {
                self.dispatcher.dispatch(Action::ToggleSelection);
            }
            _ => {}
        }
    }

    fn refresh_items(&mut self) {
        match cleaner::scan_all() {
            Ok(items) => self.dispatcher.store_mut().set_items(items),
            Err(err) => log::error!("Failed to scan items: {}", err),
        }
    }
}
