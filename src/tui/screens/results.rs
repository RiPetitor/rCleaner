// Copyright (C) 2026 RiPetitor
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::i18n;
use crate::tui::screens::common::{Theme, render_header, styled_block};
use crate::tui::state::State;
use crate::tui::widgets::status_bar::render_status_bar;
use crate::utils::size_format::format_size;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph};

pub fn render_results_screen(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    state: &State,
    system_label: &str,
) {
    let border_color = match &state.last_result {
        Some(r) if r.errors.is_empty() => Theme::SUCCESS,
        _ => Theme::WARNING,
    };

    let outer = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));
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
        .constraints([Constraint::Length(7), Constraint::Min(0)])
        .split(chunks[1]);

    let summary_lines = match &state.last_result {
        Some(result) => {
            vec![
                Line::from(""),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        format!("{:10}", i18n::cleaned()),
                        Style::default().fg(Theme::TEXT_DIM),
                    ),
                    Span::styled(
                        format!("{}", result.cleaned_items),
                        Style::default()
                            .fg(Theme::SUCCESS)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        format!("{:10}", i18n::freed()),
                        Style::default().fg(Theme::TEXT_DIM),
                    ),
                    Span::styled(
                        format_size(result.freed_bytes),
                        Style::default()
                            .fg(Theme::ACCENT)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        format!("{:10}", i18n::skipped()),
                        Style::default().fg(Theme::TEXT_DIM),
                    ),
                    Span::styled(
                        format!("{}", result.skipped_items),
                        Style::default().fg(Theme::TEXT),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        format!("{:10}", i18n::errors_label()),
                        Style::default().fg(Theme::TEXT_DIM),
                    ),
                    Span::styled(
                        format!("{}", result.errors.len()),
                        if result.errors.is_empty() {
                            Style::default().fg(Theme::TEXT)
                        } else {
                            Style::default()
                                .fg(Theme::DANGER)
                                .add_modifier(Modifier::BOLD)
                        },
                    ),
                ]),
            ]
        }
        None => vec![Line::from(Span::styled(
            format!("  {}", i18n::no_results()),
            Style::default().fg(Theme::TEXT_MUTED),
        ))],
    };

    let summary = Paragraph::new(summary_lines).block(styled_block(i18n::results_title()));
    frame.render_widget(summary, body[0]);

    let error_items: Vec<ListItem> = match &state.last_result {
        Some(result) if !result.errors.is_empty() => result
            .errors
            .iter()
            .take(body[1].height.saturating_sub(2) as usize)
            .map(|err| {
                ListItem::new(Line::from(vec![
                    Span::styled("  ✗ ", Style::default().fg(Theme::DANGER)),
                    Span::styled(err.clone(), Style::default().fg(Theme::TEXT_DIM)),
                ]))
            })
            .collect(),
        _ => vec![ListItem::new(Line::from(Span::styled(
            format!("  {}", i18n::no_errors()),
            Style::default().fg(Theme::TEXT_MUTED),
        )))],
    };

    let errors = List::new(error_items).block(styled_block(i18n::errors_title()));
    frame.render_widget(errors, body[1]);

    let keys = vec![
        i18n::key_enter_back().to_string(),
        i18n::key_back().to_string(),
    ];
    render_status_bar(frame, chunks[2], &keys);
}
