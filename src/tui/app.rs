use crate::cleaner;
use crate::config::Config;
use crate::error::{RcleanerError, Result};
use crate::i18n;
use crate::models::CleanupItem;
use crate::system::detection::{SystemInfo, SystemType, detect_system};
use crate::tui::action::{Action, SafetyLevel, Screen, SettingsEdit};
use crate::tui::dispatcher::Dispatcher;
use crate::tui::screens::{confirm, main, progress, results, settings};
use crate::utils::cache;
use crate::utils::command;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEventKind, MouseButton,
    MouseEventKind,
};
use ratatui::widgets::Clear;
use ratatui::{DefaultTerminal, Frame};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

type ScanResult = std::result::Result<Vec<CleanupItem>, String>;
type CleanResult = std::result::Result<crate::models::CleanupResult, String>;

pub struct App {
    dispatcher: Dispatcher,
    system_label: String,
    config: Config,
    config_path: PathBuf,
    scan_tx: mpsc::Sender<ScanResult>,
    scan_rx: mpsc::Receiver<ScanResult>,
    scan_in_progress: bool,
    clean_tx: mpsc::Sender<CleanResult>,
    clean_rx: mpsc::Receiver<CleanResult>,
    cli_dry_run: bool,
    terminal_height: u16,
}

impl App {
    pub fn new(config_path: PathBuf, cli_dry_run: bool) -> Self {
        let mut dispatcher = Dispatcher::new();
        dispatcher.dispatch(Action::Init);

        let (scan_tx, scan_rx) = mpsc::channel();
        let (clean_tx, clean_rx) = mpsc::channel();
        let (config, status_message) = load_config(&config_path);

        let mut app = Self {
            dispatcher,
            system_label: build_system_label(),
            config,
            config_path,
            scan_tx,
            scan_rx,
            scan_in_progress: false,
            clean_tx,
            clean_rx,
            cli_dry_run,
            terminal_height: 24,
        };

        app.apply_config_to_state();
        app.apply_language_from_config();
        if let Some(message) = status_message {
            app.dispatcher.dispatch(Action::SetStatus(Some(message)));
        }
        app.load_cached_items();
        app.request_scan("Startup scan");

        app
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        // Enable mouse capture
        crossterm::execute!(std::io::stdout(), EnableMouseCapture)?;

        let result = self.event_loop(terminal);

        // Disable mouse capture on exit
        let _ = crossterm::execute!(std::io::stdout(), DisableMouseCapture);

        result
    }

    fn event_loop(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            self.poll_scan_results();
            self.poll_clean_results();

            let state = self.dispatcher.store().state();
            if state.should_exit {
                break;
            }

            terminal.draw(|frame| {
                self.terminal_height = frame.area().height;
                self.draw(frame);
            })?;

            if event::poll(Duration::from_millis(100))? {
                match event::read()? {
                    event::Event::Key(key) if key.kind == KeyEventKind::Press => {
                        self.handle_key_event(key)?;
                    }
                    event::Event::Mouse(mouse) => {
                        self.handle_mouse_event(mouse);
                    }
                    event::Event::Resize(_, h) => {
                        self.terminal_height = h;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(Clear, area);
        let state = self.dispatcher.store().state();

        match state.active_screen {
            Screen::Main => main::render_main_screen(frame, area, state, &self.system_label),
            Screen::Confirm => confirm::render_confirm_screen(
                frame,
                area,
                state,
                &self.system_label,
                self.effective_dry_run(),
            ),
            Screen::Settings => settings::render_settings_screen(
                frame,
                area,
                state,
                &self.system_label,
                self.config.current_profile().auto_confirm,
                self.effective_dry_run(),
                self.config.current_profile().temp_max_age_days,
                &self.config_path.to_string_lossy(),
                self.config.safety.enabled,
                self.config.safety.only_root_can_disable,
                &self.config.rules.whitelist.paths,
                &self.config.rules.blacklist.patterns,
            ),
            Screen::Progress => {
                progress::render_progress_screen(frame, area, state, &self.system_label)
            }
            Screen::Results => {
                results::render_results_screen(frame, area, state, &self.system_label)
            }
        }
    }

    fn effective_dry_run(&self) -> bool {
        self.cli_dry_run || self.config.current_profile().dry_run
    }

    fn handle_key_event(&mut self, key: event::KeyEvent) -> Result<()> {
        let screen = self.dispatcher.store().state().active_screen;

        match screen {
            Screen::Main => self.handle_main_keys(key)?,
            Screen::Confirm => self.handle_confirm_keys(key)?,
            Screen::Settings => self.handle_settings_keys(key),
            Screen::Results => self.handle_results_keys(key),
            Screen::Progress => self.handle_progress_keys(key),
        }

        Ok(())
    }

    fn handle_mouse_event(&mut self, mouse: event::MouseEvent) {
        let screen = self.dispatcher.store().state().active_screen;
        if screen != Screen::Main {
            return;
        }

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Header is ~2 lines, tabs ~3 lines, border ~1 line = offset ~7
                // Status bar at bottom ~3 lines + border ~1
                let list_top = 7u16;
                let list_bottom = self.terminal_height.saturating_sub(4);

                if mouse.row >= list_top && mouse.row < list_bottom {
                    let clicked_index = (mouse.row - list_top) as usize;
                    let scroll_offset = self.dispatcher.store().state().scroll_offset;
                    let target = scroll_offset + clicked_index;
                    let visible_count = self.dispatcher.store().state().visible_items_len();
                    if target < visible_count {
                        self.dispatcher.dispatch(Action::SelectItem(target));
                    }
                }
            }
            MouseEventKind::ScrollDown => {
                self.dispatcher.dispatch(Action::SelectNext);
            }
            MouseEventKind::ScrollUp => {
                self.dispatcher.dispatch(Action::SelectPrev);
            }
            _ => {}
        }
    }

    fn handle_main_keys(&mut self, key: event::KeyEvent) -> Result<()> {
        if self.dispatcher.store().state().search_active {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => {
                    self.dispatcher.dispatch(Action::EndSearch);
                }
                KeyCode::Backspace => {
                    self.dispatcher.dispatch(Action::BackspaceSearch);
                }
                KeyCode::Char(ch) => {
                    if !ch.is_control() {
                        self.dispatcher.dispatch(Action::AppendSearch(ch));
                    }
                }
                _ => {}
            }
            return Ok(());
        }

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
                        self.start_cleanup();
                    } else {
                        self.dispatcher.dispatch(Action::OpenConfirm);
                    }
                } else {
                    self.dispatcher
                        .dispatch(Action::SetStatus(Some(i18n::no_selected().to_string())));
                }
            }
            KeyCode::Char('/') => {
                self.dispatcher.dispatch(Action::StartSearch);
            }
            KeyCode::Esc => {
                if !self.dispatcher.store().state().search_query.is_empty() {
                    self.dispatcher.dispatch(Action::ClearSearch);
                }
            }
            KeyCode::Tab => {
                self.dispatcher.dispatch(Action::NextTab);
            }
            KeyCode::BackTab => {
                self.dispatcher.dispatch(Action::PrevTab);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.dispatcher.dispatch(Action::SelectNext);
            }
            KeyCode::Up | KeyCode::Char('k') => {
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
            KeyCode::Char('1') => self.dispatcher.dispatch(Action::ChangeTab(0)),
            KeyCode::Char('2') => self.dispatcher.dispatch(Action::ChangeTab(1)),
            KeyCode::Char('3') => self.dispatcher.dispatch(Action::ChangeTab(2)),
            KeyCode::Char('4') => self.dispatcher.dispatch(Action::ChangeTab(3)),
            KeyCode::Char('5') => self.dispatcher.dispatch(Action::ChangeTab(4)),
            KeyCode::Char('6') => self.dispatcher.dispatch(Action::ChangeTab(5)),
            KeyCode::PageDown => {
                let page = self.page_size();
                self.dispatcher.dispatch(Action::SelectPageDown(page));
            }
            KeyCode::PageUp => {
                let page = self.page_size();
                self.dispatcher.dispatch(Action::SelectPageUp(page));
            }
            KeyCode::Home => self.dispatcher.dispatch(Action::SelectFirst),
            KeyCode::End => self.dispatcher.dispatch(Action::SelectLast),
            _ => {}
        }
        Ok(())
    }

    fn handle_confirm_keys(&mut self, key: event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.start_cleanup();
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.dispatcher.dispatch(Action::BackToMain);
            }
            _ => {}
        }
        Ok(())
    }

    /// Row counts per settings block.
    const SETTINGS_BLOCK_ROWS: [usize; 3] = [4, 2, 2];

    fn handle_settings_keys(&mut self, key: event::KeyEvent) {
        let state = self.dispatcher.store().state().clone();

        // Text editing mode (whitelist/blacklist input)
        if let Some(edit_target) = state.settings_edit {
            match key.code {
                KeyCode::Enter => {
                    self.apply_settings_edit(edit_target, &state.settings_input);
                }
                KeyCode::Esc => {
                    self.dispatcher.dispatch(Action::EndSettingsEdit);
                }
                KeyCode::Backspace => {
                    self.dispatcher.dispatch(Action::BackspaceSettingsInput);
                }
                KeyCode::Char(ch) => {
                    if !ch.is_control() {
                        self.dispatcher.dispatch(Action::AppendSettingsInput(ch));
                    }
                }
                _ => {}
            }
            return;
        }

        let block = state.settings_block;
        let max_rows = Self::SETTINGS_BLOCK_ROWS[block];

        match key.code {
            KeyCode::Tab => {
                self.dispatcher.dispatch(Action::NextSettingsBlock);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.dispatcher.dispatch(Action::SettingsRowNext(max_rows));
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.dispatcher.dispatch(Action::SettingsRowPrev);
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.activate_settings_row(block, state.settings_row);
            }
            KeyCode::Left | KeyCode::Right => {
                // Left/Right also toggle in Safety Level block
                if block == 1 {
                    self.activate_settings_row(block, state.settings_row);
                }
            }
            KeyCode::Esc => {
                self.dispatcher.dispatch(Action::BackToMain);
            }
            _ => {}
        }
    }

    fn activate_settings_row(&mut self, block: usize, row: usize) {
        match block {
            0 => match row {
                0 => self.toggle_safety_enabled(),
                1 => self.toggle_root_only_disable(),
                2 => self.toggle_dry_run(),
                3 => {
                    let lang = i18n::toggle_lang();
                    self.config.language = match lang {
                        i18n::Lang::En => "en".to_string(),
                        i18n::Lang::Ru => "ru".to_string(),
                    };
                    self.save_config(i18n::config_saved());
                }
                _ => {}
            },
            1 => {
                let level = if row == 0 {
                    SafetyLevel::Safe
                } else {
                    SafetyLevel::Aggressive
                };
                self.apply_safety_level(level);
            }
            2 => match row {
                0 => self.begin_settings_edit(SettingsEdit::Whitelist),
                1 => self.begin_settings_edit(SettingsEdit::Blacklist),
                _ => {}
            },
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

    fn handle_progress_keys(&mut self, key: event::KeyEvent) {
        if key.code == KeyCode::Esc {
            self.dispatcher.dispatch(Action::CancelCleanup);
        }
    }

    fn page_size(&self) -> usize {
        // Header(2) + Tabs(3) + SearchBox(3) + StatusBar(3) + borders(2) = ~13
        self.terminal_height.saturating_sub(13).max(5) as usize
    }

    fn start_cleanup(&mut self) {
        let selected_items = self.dispatcher.store().state().selected_items();
        if selected_items.is_empty() {
            self.dispatcher
                .dispatch(Action::SetStatus(Some(i18n::no_selected().to_string())));
            self.dispatcher.dispatch(Action::BackToMain);
            return;
        }

        self.dispatcher.dispatch(Action::StartCleanup);

        let dry_run = self.effective_dry_run();
        let tx = self.clean_tx.clone();
        thread::spawn(move || {
            let result =
                cleaner::clean_selected(&selected_items, dry_run).map_err(|e| e.to_string());
            let _ = tx.send(result);
        });
    }

    fn poll_clean_results(&mut self) {
        while let Ok(result) = self.clean_rx.try_recv() {
            match result {
                Ok(cleanup_result) => {
                    self.dispatcher
                        .dispatch(Action::FinishCleanup(cleanup_result));
                }
                Err(err) => {
                    let mut failed = crate::models::CleanupResult::default();
                    failed.errors.push(err);
                    self.dispatcher.dispatch(Action::FinishCleanup(failed));
                }
            }
        }
    }

    fn load_cached_items(&mut self) {
        match cache::load_cached_items() {
            Ok(Some(items)) => {
                self.dispatcher.dispatch(Action::SetItems(items));
                self.dispatcher
                    .dispatch(Action::SetStatus(Some(i18n::loaded_cache().to_string())));
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
                i18n::scan_in_progress().to_string(),
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
                        .dispatch(Action::SetStatus(Some(i18n::scan_complete().to_string())));
                }
                Err(err) => {
                    log::error!("Failed to scan items: {}", err);
                    self.dispatcher
                        .dispatch(Action::SetStatus(Some(i18n::scan_failed().to_string())));
                }
            }
        }
    }

    fn apply_config_to_state(&mut self) {
        let level = safety_level_from_config(&self.config);
        self.dispatcher.dispatch(Action::ChangeSafetyLevel(level));
    }

    fn apply_language_from_config(&self) {
        let lang = match self.config.language.as_str() {
            "ru" => i18n::Lang::Ru,
            _ => i18n::Lang::En,
        };
        i18n::set_lang(lang);
    }

    fn apply_safety_level(&mut self, level: SafetyLevel) {
        self.dispatcher.dispatch(Action::ChangeSafetyLevel(level));
        self.config.safety.level = match level {
            SafetyLevel::Safe => "safe".to_string(),
            SafetyLevel::Aggressive => "aggressive".to_string(),
        };
        self.save_config(i18n::config_saved());
    }

    fn begin_settings_edit(&mut self, target: SettingsEdit) {
        let input = match target {
            SettingsEdit::Whitelist => self.config.rules.whitelist.paths.join(", "),
            SettingsEdit::Blacklist => self.config.rules.blacklist.patterns.join(", "),
        };
        self.dispatcher
            .dispatch(Action::BeginSettingsEdit(target, input));
    }

    fn apply_settings_edit(&mut self, target: SettingsEdit, input: &str) {
        let values = parse_rules_input(input);
        if let Some(err) = validate_rules_input(&values) {
            self.dispatcher.dispatch(Action::SetStatus(Some(format!(
                "{}: {err}",
                i18n::invalid_input()
            ))));
            return;
        }
        match target {
            SettingsEdit::Whitelist => self.config.rules.whitelist.paths = values,
            SettingsEdit::Blacklist => self.config.rules.blacklist.patterns = values,
        }

        if self.save_config(i18n::rules_updated()) {
            self.request_scan("Rules updated");
        }
        self.dispatcher.dispatch(Action::EndSettingsEdit);
    }

    fn toggle_safety_enabled(&mut self) {
        if self.config.safety.enabled
            && self.config.safety.only_root_can_disable
            && !command::is_root()
        {
            self.dispatcher
                .dispatch(Action::SetStatus(Some(i18n::root_required().to_string())));
            return;
        }

        self.config.safety.enabled = !self.config.safety.enabled;
        if self.save_config(i18n::safety_updated()) {
            self.request_scan("Safety updated");
        }
    }

    fn toggle_root_only_disable(&mut self) {
        if !command::is_root() {
            self.dispatcher
                .dispatch(Action::SetStatus(Some(i18n::root_change().to_string())));
            return;
        }

        self.config.safety.only_root_can_disable = !self.config.safety.only_root_can_disable;
        self.save_config(i18n::policy_updated());
    }

    fn toggle_dry_run(&mut self) {
        let profile = if self.config.safety.level.to_lowercase() == "aggressive" {
            &mut self.config.profiles.aggressive
        } else {
            &mut self.config.profiles.safe
        };
        profile.dry_run = !profile.dry_run;
        self.save_config(i18n::dryrun_updated());
    }

    fn save_config(&mut self, message: &str) -> bool {
        if let Err(err) = self.config.save(&self.config_path) {
            log::warn!("Failed to save config: {}", err);
            self.dispatcher
                .dispatch(Action::SetStatus(Some(i18n::config_fail().to_string())));
            false
        } else {
            self.dispatcher
                .dispatch(Action::SetStatus(Some(message.to_string())));
            true
        }
    }

    fn reload_config(&mut self) {
        match Config::load(&self.config_path) {
            Ok(config) => {
                self.config = config;
                self.apply_config_to_state();
                self.apply_language_from_config();
            }
            Err(err) => {
                log::warn!("Failed to reload config: {}", err);
            }
        }
    }
}

fn parse_rules_input(input: &str) -> Vec<String> {
    input
        .split([',', '\n'])
        .map(|entry| entry.trim())
        .filter(|entry| !entry.is_empty())
        .map(String::from)
        .collect()
}

fn validate_rules_input(values: &[String]) -> Option<String> {
    for value in values {
        if value.contains("..") {
            return Some(format!("path traversal not allowed: {value}"));
        }
        if value.starts_with('/')
            && !value.starts_with("/tmp")
            && !value.starts_with("/home")
            && [
                "/boot", "/usr", "/bin", "/sbin", "/lib", "/etc", "/root", "/dev", "/proc", "/sys",
            ]
            .iter()
            .any(|p| value.starts_with(p))
        {
            return Some(format!("system path not allowed: {value}"));
        }
    }
    None
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
    if let Some(desktop) = info.desktop_environment.as_deref()
        && !desktop.is_empty()
    {
        label.push_str(" | ");
        label.push_str(desktop);
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
                return (config, Some(i18n::config_not_saved().to_string()));
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
            (Config::default(), Some(i18n::default_config().to_string()))
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
