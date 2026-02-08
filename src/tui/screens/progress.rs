// Copyright (C) 2026 RiPetitor
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::i18n;
use crate::tui::screens::common::{Theme, render_header, styled_block};
use crate::tui::state::State;
use crate::tui::widgets::progress_bar::render_progress_bar;
use crate::tui::widgets::status_bar::render_status_bar;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

pub fn render_progress_screen(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    state: &State,
    system_label: &str,
) {
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::ACCENT));
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

    let body = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(chunks[1]);

    render_progress_bar(
        frame,
        body[1],
        state.cleanup_progress,
        i18n::progress_label(),
    );

    let step_text = state.cleanup_step.as_deref().unwrap_or(i18n::working());
    let step = Paragraph::new(Line::from(vec![
        Span::raw("  "),
        Span::styled("▸ ", Style::default().fg(Theme::ACCENT)),
        Span::styled(step_text, Style::default().fg(Theme::TEXT)),
    ]))
    .block(styled_block(i18n::current_step()));
    frame.render_widget(step, body[2]);

    let keys = vec![i18n::key_cancel_esc().to_string()];
    render_status_bar(frame, chunks[2], &keys);
}
