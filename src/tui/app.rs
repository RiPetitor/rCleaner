use crate::cleaner;
use crate::error::Result;
use crate::system::detection::{SystemInfo, SystemType, detect_system};
use crate::tui::action::{Action, SafetyLevel, Screen};
use crate::tui::dispatcher::Dispatcher;
use crate::tui::screens::{confirm, main, progress, results, settings};
use crate::tui::state::State;
use ratatui::crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::{DefaultTerminal, Frame};
use std::time::Duration;

pub struct App {
    dispatcher: Dispatcher,
    system_label: String,
}

impl App {
    pub fn new() -> Self {
        let mut dispatcher = Dispatcher::new();
        dispatcher.dispatch(Action::Init);

        Self {
            dispatcher,
            system_label: build_system_label(),
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.refresh_items();

        loop {
            let state = self.dispatcher.store().state().clone();
            if state.should_exit {
                break;
            }

            terminal.draw(|frame| self.draw(frame, &state))?;

            if event::poll(Duration::from_millis(150))? {
                if let event::Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key_event(key, terminal)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame, state: &State) {
        let area = frame.area();

        match state.active_screen {
            Screen::Main => main::render_main_screen(frame, area, state, &self.system_label),
            Screen::Confirm => {
                confirm::render_confirm_screen(frame, area, state, &self.system_label)
            }
            Screen::Settings => {
                settings::render_settings_screen(frame, area, state, &self.system_label)
            }
            Screen::Progress => {
                progress::render_progress_screen(frame, area, state, &self.system_label)
            }
            Screen::Results => {
                results::render_results_screen(frame, area, state, &self.system_label)
            }
        }
    }

    fn handle_key_event(
        &mut self,
        key: event::KeyEvent,
        terminal: &mut DefaultTerminal,
    ) -> Result<()> {
        let screen = self.dispatcher.store().state().active_screen;

        match screen {
            Screen::Main => self.handle_main_keys(key),
            Screen::Confirm => self.handle_confirm_keys(key, terminal)?,
            Screen::Settings => self.handle_settings_keys(key),
            Screen::Results => self.handle_results_keys(key),
            Screen::Progress => {}
        }

        Ok(())
    }

    fn handle_main_keys(&mut self, key: event::KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.dispatcher.dispatch(Action::Exit);
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.dispatcher.dispatch(Action::Refresh);
                self.refresh_items();
            }
            KeyCode::Enter => {
                if self.dispatcher.store().state().selected_count() > 0 {
                    self.dispatcher.dispatch(Action::OpenConfirm);
                } else {
                    self.dispatcher
                        .dispatch(Action::SetStatus(Some("No items selected.".to_string())));
                }
            }
            KeyCode::Tab => {
                self.dispatcher.dispatch(Action::NextTab);
            }
            KeyCode::BackTab => {
                self.dispatcher.dispatch(Action::PrevTab);
            }
            KeyCode::Down => {
                self.dispatcher.dispatch(Action::SelectNext);
            }
            KeyCode::Up => {
                self.dispatcher.dispatch(Action::SelectPrev);
            }
            KeyCode::Char(' ') => {
                self.dispatcher.dispatch(Action::ToggleSelection);
            }
            KeyCode::Char('a') | KeyCode::Char('A') => {
                self.dispatcher.dispatch(Action::ToggleAllVisible);
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.dispatcher.dispatch(Action::OpenSettings);
            }
            _ => {}
        }
    }

    fn handle_confirm_keys(
        &mut self,
        key: event::KeyEvent,
        terminal: &mut DefaultTerminal,
    ) -> Result<()> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.perform_cleanup(terminal)?;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.dispatcher.dispatch(Action::BackToMain);
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_settings_keys(&mut self, key: event::KeyEvent) {
        match key.code {
            KeyCode::Left | KeyCode::Right => {
                let next = match self.dispatcher.store().state().safety_level {
                    SafetyLevel::Safe => SafetyLevel::Aggressive,
                    SafetyLevel::Aggressive => SafetyLevel::Safe,
                };
                self.dispatcher.dispatch(Action::ChangeSafetyLevel(next));
            }
            KeyCode::Enter | KeyCode::Esc => {
                self.dispatcher.dispatch(Action::BackToMain);
            }
            _ => {}
        }
    }

    fn handle_results_keys(&mut self, key: event::KeyEvent) {
        match key.code {
            KeyCode::Enter | KeyCode::Esc => {
                self.dispatcher.dispatch(Action::BackToMain);
                self.dispatcher.dispatch(Action::Refresh);
                self.refresh_items();
            }
            _ => {}
        }
    }

    fn refresh_items(&mut self) {
        match cleaner::scan_all() {
            Ok(items) => self.dispatcher.dispatch(Action::SetItems(items)),
            Err(err) => {
                log::error!("Failed to scan items: {}", err);
                self.dispatcher.dispatch(Action::SetStatus(Some(
                    "Scan failed. See logs.".to_string(),
                )));
            }
        }
    }

    fn perform_cleanup(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let selected_items = self.dispatcher.store().state().selected_items();
        if selected_items.is_empty() {
            self.dispatcher
                .dispatch(Action::SetStatus(Some("No items selected.".to_string())));
            self.dispatcher.dispatch(Action::BackToMain);
            return Ok(());
        }

        self.dispatcher.dispatch(Action::StartCleanup);
        self.draw_current(terminal);

        let mut last_error = None;
        let result =
            cleaner::clean_selected_with_progress(&selected_items, false, |progress, step| {
                self.dispatcher.dispatch(Action::CleanupProgress {
                    progress,
                    step: Some(step.to_string()),
                });
                if let Err(err) = terminal.draw(|frame| {
                    let state = self.dispatcher.store().state().clone();
                    self.draw(frame, &state);
                }) {
                    last_error = Some(err.to_string());
                }
            });

        if let Some(message) = last_error {
            log::warn!("Failed to render progress: {}", message);
        }

        match result {
            Ok(result) => self.dispatcher.dispatch(Action::FinishCleanup(result)),
            Err(err) => {
                let mut failed = crate::models::CleanupResult::default();
                failed.errors.push(err.to_string());
                self.dispatcher.dispatch(Action::FinishCleanup(failed));
            }
        }

        Ok(())
    }

    fn draw_current(&self, terminal: &mut DefaultTerminal) {
        let state = self.dispatcher.store().state().clone();
        if let Err(err) = terminal.draw(|frame| self.draw(frame, &state)) {
            log::warn!("Failed to render: {}", err);
        }
    }
}

fn build_system_label() -> String {
    match detect_system() {
        Ok(info) => format_system_label(&info),
        Err(_) => "Unknown system".to_string(),
    }
}

fn format_system_label(info: &SystemInfo) -> String {
    let mut label = format!("{} {}", info.os_name, info.os_version);
    if matches!(info.system_type, SystemType::AtomicRpmOstree) {
        label.push_str(" Atomic");
    }
    if let Some(desktop) = info.desktop_environment.as_deref() {
        if !desktop.is_empty() {
            label.push_str(" | ");
            label.push_str(desktop);
        }
    }
    label
}
