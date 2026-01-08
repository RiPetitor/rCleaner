use crate::tui::action::SafetyLevel;
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
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Min(0)])
        .split(chunks[1]);

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
    frame.render_stateful_widget(list, content_chunks[0], &mut list_state);

    let hint = Paragraph::new(format!(
        "Profile: {}\nAuto confirm: {}\nConfig: {}\nEdit whitelist/blacklist in config.",
        match state.safety_level {
            SafetyLevel::Safe => "safe",
            SafetyLevel::Aggressive => "aggressive",
        },
        if auto_confirm { "on" } else { "off" },
        config_path,
    ))
    .block(Block::default().borders(Borders::ALL).title("Info"));
    frame.render_widget(hint, content_chunks[1]);

    let keys = vec![
        "[Left/Right] Change".to_string(),
        "[Enter] Back".to_string(),
        "[Esc] Back".to_string(),
    ];
    render_status_bar(frame, chunks[2], &keys);
}
