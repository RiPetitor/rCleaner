use crate::tui::action::{SafetyLevel, SettingsEdit};
use crate::tui::screens::common::render_header;
use crate::tui::state::State;
use crate::tui::widgets::status_bar::render_status_bar;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph};

pub fn render_settings_screen(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    state: &State,
    system_label: &str,
    auto_confirm: bool,
    config_path: &str,
    safety_enabled: bool,
    only_root_can_disable: bool,
    whitelist: &[String],
    blacklist: &[String],
) {
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("Settings");
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(inner);

    render_header(frame, chunks[0], system_label, state.safety_level);

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(chunks[1]);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(0)])
        .split(content_chunks[0]);

    let safety_status = if safety_enabled {
        "enabled"
    } else {
        "disabled"
    };
    let root_status = if only_root_can_disable { "on" } else { "off" };
    let level_label = match state.safety_level {
        SafetyLevel::Safe => "safe",
        SafetyLevel::Aggressive => "aggressive",
    };

    let info = Paragraph::new(format!(
        "Safety: {safety_status} (E)\nRoot-only disable: {root_status} (O)\nLevel: {level_label}\nAuto confirm: {}\nConfig: {}",
        if auto_confirm { "on" } else { "off" },
        config_path
    ))
    .block(Block::default().borders(Borders::ALL).title("Config"));
    frame.render_widget(info, left_chunks[0]);

    let safety_items = vec![
        ListItem::new(Line::from(vec![
            Span::styled("Safe", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" - recommended, more conservative"),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("Aggressive", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" - faster cleanup, fewer checks"),
        ])),
    ];

    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(match state.safety_level {
        SafetyLevel::Safe => 0,
        SafetyLevel::Aggressive => 1,
    }));

    let list = List::new(safety_items)
        .block(Block::default().borders(Borders::ALL).title("Safety Level"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");
    frame.render_stateful_widget(list, left_chunks[1], &mut list_state);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(content_chunks[1]);

    let whitelist_lines = format_rules_list(whitelist);
    let blacklist_lines = format_rules_list(blacklist);
    let rules_text = format!("Whitelist:\n{whitelist_lines}\n\nBlacklist:\n{blacklist_lines}");
    let rules =
        Paragraph::new(rules_text).block(Block::default().borders(Borders::ALL).title("Rules"));
    frame.render_widget(rules, right_chunks[0]);

    let edit_label = match state.settings_edit {
        Some(SettingsEdit::Whitelist) => "Edit whitelist",
        Some(SettingsEdit::Blacklist) => "Edit blacklist",
        None => "Edit rules",
    };

    let edit_text = if let Some(edit) = state.settings_edit {
        let input = state.settings_input.as_str();
        let hint = match edit {
            SettingsEdit::Whitelist => "Comma-separated paths",
            SettingsEdit::Blacklist => "Comma-separated patterns",
        };
        format!("{edit_label}: {input}\n{hint} (Enter: save, Esc: cancel)")
    } else {
        "W: edit whitelist  B: edit blacklist".to_string()
    };

    let edit_block = Block::default().borders(Borders::ALL).title("Edit");
    let edit_inner = edit_block.inner(right_chunks[1]);
    frame.render_widget(edit_block, right_chunks[1]);
    frame.render_widget(Paragraph::new(edit_text), edit_inner);

    if state.settings_edit.is_some() {
        let cursor_offset = edit_label.len() + 2 + state.settings_input.len();
        let max_x = edit_inner.width.saturating_sub(1) as usize;
        let cursor_x = edit_inner.x + cursor_offset.min(max_x) as u16;
        frame.set_cursor_position(ratatui::layout::Position {
            x: cursor_x,
            y: edit_inner.y,
        });
    }

    let keys = if state.settings_edit.is_some() {
        vec![
            "[Enter] Save".to_string(),
            "[Esc] Cancel".to_string(),
            "[Backspace] Delete".to_string(),
        ]
    } else {
        vec![
            "[Left/Right] Level".to_string(),
            "[E] Toggle safety".to_string(),
            "[O] Root-only".to_string(),
            "[W/B] Edit rules".to_string(),
            "[Enter] Back".to_string(),
            "[Esc] Back".to_string(),
        ]
    };
    render_status_bar(frame, chunks[2], &keys);
}

fn format_rules_list(rules: &[String]) -> String {
    if rules.is_empty() {
        return "(empty)".to_string();
    }
    rules
        .iter()
        .map(|rule| format!("- {rule}"))
        .collect::<Vec<_>>()
        .join("\n")
}
