use crate::cleaner;
use crate::config::Config;
use crate::error::{RcleanerError, Result};
use crate::models::CleanupItem;
use crate::system::detection::{SystemInfo, SystemType, detect_system};
use crate::tui::action::{Action, SafetyLevel, Screen};
use crate::tui::dispatcher::Dispatcher;
use crate::tui::screens::{confirm, main, progress, results, settings};
use crate::tui::state::State;
use crate::utils::cache;
use ratatui::crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::{DefaultTerminal, Frame};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

type ScanResult = std::result::Result<Vec<CleanupItem>, String>;

pub struct App {
    dispatcher: Dispatcher,
    system_label: String,
    config: Config,
    config_path: PathBuf,
    scan_tx: mpsc::Sender<ScanResult>,
    scan_rx: mpsc::Receiver<ScanResult>,
    scan_in_progress: bool,
}

impl App {
    pub fn new() -> Self {
        let mut dispatcher = Dispatcher::new();
        dispatcher.dispatch(Action::Init);

        let (scan_tx, scan_rx) = mpsc::channel();
        let config_path = Config::default_path();
        let (config, status_message) = load_config(&config_path);

        let mut app = Self {
            dispatcher,
            system_label: build_system_label(),
            config,
            config_path,
            scan_tx,
            scan_rx,
            scan_in_progress: false,
        };

        app.apply_config_to_state();
        if let Some(message) = status_message {
            app.dispatcher.dispatch(Action::SetStatus(Some(message)));
        }
        app.load_cached_items();
        app.request_scan("Startup scan");

        app
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            self.poll_scan_results();
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
            Screen::Settings => settings::render_settings_screen(
                frame,
                area,
                state,
                &self.system_label,
                self.config.current_profile().auto_confirm,
                &self.config_path.to_string_lossy(),
            ),
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
            Screen::Main => self.handle_main_keys(key, terminal)?,
            Screen::Confirm => self.handle_confirm_keys(key, terminal)?,
            Screen::Settings => self.handle_settings_keys(key),
            Screen::Results => self.handle_results_keys(key),
            Screen::Progress => {}
        }

        Ok(())
    }

    fn handle_main_keys(
        &mut self,
        key: event::KeyEvent,
        terminal: &mut DefaultTerminal,
    ) -> Result<()> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.dispatcher.dispatch(Action::Exit);
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.request_scan("Manual refresh");
            }
            KeyCode::Enter => {
                if self.dispatcher.store().state().selected_count() > 0 {
                    if self.config.current_profile().auto_confirm {
                        self.perform_cleanup(terminal)?;
                    } else {
                        self.dispatcher.dispatch(Action::OpenConfirm);
                    }
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
        Ok(())
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
                self.apply_safety_level(next);
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
                self.request_scan("Post-cleanup refresh");
            }
            _ => {}
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

    fn load_cached_items(&mut self) {
        match cache::load_cached_items() {
            Ok(Some(items)) => {
                self.dispatcher.dispatch(Action::SetItems(items));
                self.dispatcher.dispatch(Action::SetStatus(Some(
                    "Loaded cached results.".to_string(),
                )));
            }
            Ok(None) => {}
            Err(err) => {
                log::warn!("Failed to load cached results: {}", err);
            }
        }
    }

    fn request_scan(&mut self, reason: &str) {
        if self.scan_in_progress {
            self.dispatcher.dispatch(Action::SetStatus(Some(
                "Scan already in progress.".to_string(),
            )));
            return;
        }

        self.reload_config();
        self.scan_in_progress = true;
        self.dispatcher.dispatch(Action::Refresh);
        self.dispatcher
            .dispatch(Action::SetStatus(Some(format!("{reason}..."))));

        let tx = self.scan_tx.clone();
        thread::spawn(move || {
            let result = cleaner::scan_all().map_err(|err| err.to_string());
            let _ = tx.send(result);
        });
    }

    fn poll_scan_results(&mut self) {
        while let Ok(result) = self.scan_rx.try_recv() {
            self.scan_in_progress = false;
            match result {
                Ok(items) => {
                    if let Err(err) = cache::save_cached_items(&items) {
                        log::warn!("Failed to save cache: {}", err);
                    }
                    self.dispatcher.dispatch(Action::SetItems(items));
                    self.dispatcher
                        .dispatch(Action::SetStatus(Some("Scan complete.".to_string())));
                }
                Err(err) => {
                    log::error!("Failed to scan items: {}", err);
                    self.dispatcher.dispatch(Action::SetStatus(Some(
                        "Scan failed. See logs.".to_string(),
                    )));
                }
            }
        }
    }

    fn draw_current(&self, terminal: &mut DefaultTerminal) {
        let state = self.dispatcher.store().state().clone();
        if let Err(err) = terminal.draw(|frame| self.draw(frame, &state)) {
            log::warn!("Failed to render: {}", err);
        }
    }

    fn apply_config_to_state(&mut self) {
        let level = safety_level_from_config(&self.config);
        self.dispatcher.dispatch(Action::ChangeSafetyLevel(level));
    }

    fn apply_safety_level(&mut self, level: SafetyLevel) {
        self.dispatcher.dispatch(Action::ChangeSafetyLevel(level));
        self.config.safety.level = match level {
            SafetyLevel::Safe => "safe".to_string(),
            SafetyLevel::Aggressive => "aggressive".to_string(),
        };

        if let Err(err) = self.config.save(&self.config_path) {
            log::warn!("Failed to save config: {}", err);
            self.dispatcher.dispatch(Action::SetStatus(Some(
                "Failed to save config.".to_string(),
            )));
        } else {
            self.dispatcher
                .dispatch(Action::SetStatus(Some("Config saved.".to_string())));
        }
    }

    fn reload_config(&mut self) {
        match Config::load(&self.config_path) {
            Ok(config) => {
                self.config = config;
                self.apply_config_to_state();
            }
            Err(err) => {
                log::warn!("Failed to reload config: {}", err);
            }
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

fn load_config(path: &PathBuf) -> (Config, Option<String>) {
    match Config::load(path) {
        Ok(config) => (config, None),
        Err(RcleanerError::Io(err)) if err.kind() == std::io::ErrorKind::NotFound => {
            let config = Config::default();
            if let Err(save_err) = config.save(path) {
                log::warn!("Failed to write default config: {}", save_err);
                return (
                    config,
                    Some("Using defaults (config not saved).".to_string()),
                );
            }
            (
                config,
                Some(format!(
                    "Created default config at {}.",
                    path.to_string_lossy()
                )),
            )
        }
        Err(err) => {
            log::warn!("Failed to load config: {}", err);
            (Config::default(), Some("Using default config.".to_string()))
        }
    }
}

fn safety_level_from_config(config: &Config) -> SafetyLevel {
    if config.safety.level.to_lowercase() == "aggressive" {
        SafetyLevel::Aggressive
    } else {
        SafetyLevel::Safe
    }
}
