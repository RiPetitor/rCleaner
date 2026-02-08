use crate::i18n;
use crate::tui::action::{SafetyLevel, SettingsEdit};
use crate::tui::screens::common::{Theme, active_block, render_header, styled_block};
use crate::tui::state::State;
use crate::tui::widgets::status_bar::render_status_bar;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

#[allow(clippy::too_many_arguments)]
pub fn render_settings_screen(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    state: &State,
    system_label: &str,
    auto_confirm: bool,
    dry_run: bool,
    temp_max_age_days: u64,
    config_path: &str,
    safety_enabled: bool,
    only_root_can_disable: bool,
    whitelist: &[String],
    blacklist: &[String],
) {
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::BORDER));
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(inner);

    render_header(frame, chunks[0], system_label, state.safety_level);

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(chunks[1]);

    // Left: Config (top, grows) + Safety Level (bottom, compact)
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(5)])
        .split(content_chunks[0]);

    let active_block_idx = state.settings_block;
    let active_row = state.settings_row;

    // ── Block 0: Configuration ──────────────────────────────

    let config_block = if active_block_idx == 0 {
        active_block(i18n::config_title())
    } else {
        styled_block(i18n::config_title())
    };

    let config_rows: Vec<(&str, String, bool)> = vec![
        (
            i18n::safety_label(),
            if safety_enabled {
                "ON".to_string()
            } else {
                "OFF".to_string()
            },
            true,
        ),
        (
            i18n::root_only(),
            if only_root_can_disable {
                "ON".to_string()
            } else {
                "OFF".to_string()
            },
            true,
        ),
        (
            i18n::dry_run_setting(),
            if dry_run {
                "ON".to_string()
            } else {
                "OFF".to_string()
            },
            true,
        ),
        (
            i18n::lang_setting(),
            i18n::current_lang().label().to_string(),
            true,
        ),
    ];

    let mut info_lines: Vec<Line> = vec![Line::from("")];

    for (i, (label, value, _interactive)) in config_rows.iter().enumerate() {
        let is_active = active_block_idx == 0 && active_row == i;
        let prefix = if is_active { "  ▸ " } else { "    " };

        let value_color = match value.as_str() {
            "ON" => Theme::SUCCESS,
            "OFF" => Theme::DANGER,
            _ => Theme::ACCENT,
        };

        let bg = if is_active {
            Theme::BG_HIGHLIGHT
        } else {
            ratatui::style::Color::Reset
        };

        info_lines.push(Line::from(vec![
            Span::styled(prefix, Style::default().fg(Theme::TEXT).bg(bg)),
            Span::styled(
                format!("{:14}", label),
                Style::default().fg(Theme::TEXT_DIM).bg(bg),
            ),
            Span::styled(
                value.clone(),
                Style::default()
                    .fg(value_color)
                    .bg(bg)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    // Display-only rows (no highlight)
    info_lines.push(Line::from(""));
    info_lines.push(Line::from(vec![
        Span::raw("    "),
        Span::styled(
            format!("{:14}", i18n::auto_confirm()),
            Style::default().fg(Theme::TEXT_MUTED),
        ),
        Span::styled(
            if auto_confirm { "on" } else { "off" },
            Style::default().fg(Theme::TEXT_MUTED),
        ),
    ]));
    info_lines.push(Line::from(vec![
        Span::raw("    "),
        Span::styled(
            format!("{:14}", i18n::temp_age()),
            Style::default().fg(Theme::TEXT_MUTED),
        ),
        Span::styled(
            format!("{temp_max_age_days}d"),
            Style::default().fg(Theme::TEXT_MUTED),
        ),
    ]));
    info_lines.push(Line::from(vec![
        Span::raw("    "),
        Span::styled(
            format!("{:14}", i18n::config_label()),
            Style::default().fg(Theme::TEXT_MUTED),
        ),
        Span::styled(config_path, Style::default().fg(Theme::TEXT_MUTED)),
    ]));

    let info = Paragraph::new(info_lines).block(config_block);
    frame.render_widget(info, left_chunks[0]);

    // ── Block 1: Safety Level ───────────────────────────────

    let level_block = if active_block_idx == 1 {
        active_block(i18n::safety_level())
    } else {
        styled_block(i18n::safety_level())
    };

    let level_options = [
        ("Safe", Theme::SAFE, i18n::safe_desc()),
        ("Aggressive", Theme::AGGRESSIVE, i18n::aggressive_desc()),
    ];

    let current_level_idx = match state.safety_level {
        SafetyLevel::Safe => 0,
        SafetyLevel::Aggressive => 1,
    };

    let mut level_lines: Vec<Line> = vec![Line::from("")];
    for (i, (name, color, desc)) in level_options.iter().enumerate() {
        let is_active = active_block_idx == 1 && active_row == i;
        let is_current = i == current_level_idx;
        let prefix = if is_active { "  ▸ " } else { "    " };

        let bg = if is_active {
            Theme::BG_HIGHLIGHT
        } else {
            ratatui::style::Color::Reset
        };

        let check = if is_current { " ●" } else { "  " };

        level_lines.push(Line::from(vec![
            Span::styled(prefix, Style::default().fg(Theme::TEXT).bg(bg)),
            Span::styled(
                name.to_string(),
                Style::default()
                    .fg(*color)
                    .bg(bg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" — {desc}"),
                Style::default().fg(Theme::TEXT_DIM).bg(bg),
            ),
            Span::styled(
                check,
                Style::default()
                    .fg(Theme::ACCENT)
                    .bg(bg)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    let level_widget = Paragraph::new(level_lines).block(level_block);
    frame.render_widget(level_widget, left_chunks[1]);

    // ── Right side: Rules + Edit ────────────────────────────

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(content_chunks[1]);

    // ── Block 2: Rules ──────────────────────────────────────

    let rules_block_widget = if active_block_idx == 2 {
        active_block(i18n::rules_title())
    } else {
        styled_block(i18n::rules_title())
    };

    let rules_entries = [
        (i18n::whitelist_label(), whitelist),
        (i18n::blacklist_label(), blacklist),
    ];

    let mut rules_lines: Vec<Line> = vec![Line::from("")];
    for (i, (label, entries)) in rules_entries.iter().enumerate() {
        let is_active = active_block_idx == 2 && active_row == i;
        let prefix = if is_active { "  ▸ " } else { "    " };

        let bg = if is_active {
            Theme::BG_HIGHLIGHT
        } else {
            ratatui::style::Color::Reset
        };

        rules_lines.push(Line::from(vec![
            Span::styled(prefix, Style::default().fg(Theme::TEXT).bg(bg)),
            Span::styled(
                format!("{label}:"),
                Style::default()
                    .fg(Theme::TEXT)
                    .bg(bg)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        if entries.is_empty() {
            rules_lines.push(Line::from(Span::styled(
                "      (empty)",
                Style::default().fg(Theme::TEXT_MUTED),
            )));
        } else {
            for entry in *entries {
                rules_lines.push(Line::from(vec![
                    Span::raw("      "),
                    Span::styled(format!("● {entry}"), Style::default().fg(Theme::TEXT_DIM)),
                ]));
            }
        }
        rules_lines.push(Line::from(""));
    }

    let rules = Paragraph::new(rules_lines).block(rules_block_widget);
    frame.render_widget(rules, right_chunks[0]);

    // ── Edit area ───────────────────────────────────────────

    let edit_block_widget = if state.settings_edit.is_some() {
        active_block(i18n::edit_title())
    } else {
        styled_block(i18n::edit_title())
    };
    let edit_inner = edit_block_widget.inner(right_chunks[1]);
    frame.render_widget(edit_block_widget, right_chunks[1]);

    let edit_text = if let Some(edit) = state.settings_edit {
        let label = match edit {
            SettingsEdit::Whitelist => i18n::whitelist_label(),
            SettingsEdit::Blacklist => i18n::blacklist_label(),
        };
        format!("{label}: {}", state.settings_input)
    } else {
        String::new()
    };

    let edit_style = if state.settings_edit.is_some() {
        Style::default().fg(Theme::TEXT)
    } else {
        Style::default().fg(Theme::TEXT_MUTED)
    };
    frame.render_widget(Paragraph::new(edit_text).style(edit_style), edit_inner);

    if state.settings_edit.is_some() {
        // Use chars().count() for correct cursor position with multi-byte UTF-8 (Cyrillic)
        let label_len = match state.settings_edit {
            Some(SettingsEdit::Whitelist) => i18n::whitelist_label().chars().count() + 2,
            Some(SettingsEdit::Blacklist) => i18n::blacklist_label().chars().count() + 2,
            None => 0,
        };
        let cursor_offset = label_len + state.settings_input.chars().count();
        let max_x = edit_inner.width.saturating_sub(1) as usize;
        let cursor_x = edit_inner.x + cursor_offset.min(max_x) as u16;
        frame.set_cursor_position(ratatui::layout::Position {
            x: cursor_x,
            y: edit_inner.y,
        });
    }

    // ── Status bar ──────────────────────────────────────────

    let mut keys = if state.settings_edit.is_some() {
        vec![
            i18n::key_save().to_string(),
            i18n::key_esc_cancel().to_string(),
        ]
    } else {
        vec![
            i18n::key_tab_block().to_string(),
            i18n::key_arrows().to_string(),
            i18n::key_enter_toggle().to_string(),
            i18n::key_esc_back().to_string(),
        ]
    };
    if let Some(message) = state.status_message.as_deref() {
        keys.push(message.to_string());
    }
    render_status_bar(frame, chunks[2], &keys);
}
