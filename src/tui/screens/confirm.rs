// Copyright (C) 2026 RiPetitor
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::i18n;
use crate::tui::screens::common::{Theme, render_header, styled_block};
use crate::tui::state::State;
use crate::tui::widgets::status_bar::render_status_bar;
use crate::utils::size_format::format_size;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph};

pub fn render_confirm_screen(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    state: &State,
    system_label: &str,
    dry_run: bool,
) {
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::WARNING));
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

    let selected_items: Vec<_> = state.items.iter().filter(|item| item.selected).collect();
    let selected_count = selected_items.len();
    let selected_size = format_size(state.selected_size);

    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(0)])
        .split(chunks[1]);

    let (mode_label, mode_color) = if dry_run {
        (i18n::dry_run_label(), Theme::ACCENT)
    } else {
        (i18n::execute_label(), Theme::DANGER)
    };

    let summary = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("{}: ", i18n::items_count()),
                Style::default().fg(Theme::TEXT_DIM),
            ),
            Span::styled(
                format!("{selected_count}"),
                Style::default()
                    .fg(Theme::TEXT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("    "),
            Span::styled(
                format!("{}: ", i18n::size_label()),
                Style::default().fg(Theme::TEXT_DIM),
            ),
            Span::styled(
                selected_size,
                Style::default()
                    .fg(Theme::TEXT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("    "),
            Span::styled(
                format!("{}: ", i18n::mode_label()),
                Style::default().fg(Theme::TEXT_DIM),
            ),
            Span::styled(
                mode_label,
                Style::default()
                    .fg(Color::Black)
                    .bg(mode_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ])
    .block(styled_block(i18n::confirm_title()));
    frame.render_widget(summary, content_chunks[0]);

    let max_items = content_chunks[1].height.saturating_sub(2) as usize;
    let mut list_items = Vec::new();
    for item in selected_items.iter().take(max_items) {
        let line = Line::from(vec![
            Span::styled("  ● ", Style::default().fg(Theme::WARNING)),
            Span::styled(
                item.path.as_deref().unwrap_or(&item.name),
                Style::default().fg(Theme::TEXT),
            ),
            Span::raw(" "),
            Span::styled(
                format!("({})", format_size(item.size)),
                Style::default().fg(Theme::TEXT_DIM),
            ),
        ]);
        list_items.push(ListItem::new(line));
    }
    if selected_count > max_items && max_items > 0 {
        list_items.push(ListItem::new(Line::from(vec![Span::styled(
            format!("  … {} {} ", i18n::and_more(), selected_count - max_items),
            Style::default().fg(Theme::TEXT_MUTED),
        )])));
    }

    let list = List::new(list_items).block(styled_block(i18n::items_to_clean()));
    frame.render_widget(list, content_chunks[1]);

    let keys = vec![
        i18n::key_confirm().to_string(),
        i18n::key_cancel().to_string(),
        i18n::key_back().to_string(),
    ];
    render_status_bar(frame, chunks[2], &keys);
}
