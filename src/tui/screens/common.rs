// Copyright (C) 2026 RiPetitor
// SPDX-License-Identifier: GPL-3.0-or-later

//! Общие компоненты экранов.

use crate::tui::action::SafetyLevel;
use crate::{NAME, VERSION};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

/// Цветовая палитра приложения.
pub struct Theme;

impl Theme {
    pub const ACCENT: Color = Color::Rgb(100, 180, 255);
    pub const ACCENT_DIM: Color = Color::Rgb(60, 120, 200);
    pub const SUCCESS: Color = Color::Rgb(80, 220, 120);
    pub const WARNING: Color = Color::Rgb(255, 200, 60);
    pub const DANGER: Color = Color::Rgb(255, 90, 90);
    pub const TEXT: Color = Color::Rgb(220, 220, 230);
    pub const TEXT_DIM: Color = Color::Rgb(120, 125, 140);
    pub const TEXT_MUTED: Color = Color::Rgb(80, 85, 95);
    pub const BG_HIGHLIGHT: Color = Color::Rgb(40, 44, 52);
    pub const BORDER: Color = Color::Rgb(60, 65, 75);
    pub const BORDER_ACTIVE: Color = Color::Rgb(100, 180, 255);
    pub const SAFE: Color = Color::Rgb(80, 220, 120);
    pub const AGGRESSIVE: Color = Color::Rgb(255, 160, 50);
    pub const SELECTED: Color = Color::Rgb(80, 220, 120);
    pub const BLOCKED: Color = Color::Rgb(255, 90, 90);
}

pub fn styled_block(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(
            format!(" {title} "),
            Style::default()
                .fg(Theme::TEXT)
                .add_modifier(Modifier::BOLD),
        ))
}

pub fn active_block(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::BORDER_ACTIVE))
        .title(Span::styled(
            format!(" {title} "),
            Style::default()
                .fg(Theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ))
}

pub fn render_header(
    frame: &mut ratatui::Frame,
    area: Rect,
    system_label: &str,
    safety_level: SafetyLevel,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(area);

    let (safety_label, safety_color) = match safety_level {
        SafetyLevel::Safe => (" SAFE ", Theme::SAFE),
        SafetyLevel::Aggressive => (" AGGRESSIVE ", Theme::AGGRESSIVE),
    };

    let left = Line::from(vec![
        Span::raw("  "),
        Span::styled("Safety", Style::default().fg(Theme::TEXT_DIM)),
        Span::raw(" "),
        Span::styled(
            safety_label,
            Style::default()
                .fg(Color::Black)
                .bg(safety_color)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let center = Line::from(vec![
        Span::styled(
            format!(" {NAME} "),
            Style::default()
                .fg(Theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("v{VERSION}"), Style::default().fg(Theme::TEXT_DIM)),
    ]);

    let right = Line::from(vec![
        Span::styled(system_label, Style::default().fg(Theme::TEXT_DIM)),
        Span::raw("  "),
    ]);

    frame.render_widget(Paragraph::new(left).alignment(Alignment::Left), chunks[0]);
    frame.render_widget(
        Paragraph::new(center).alignment(Alignment::Center),
        chunks[1],
    );
    frame.render_widget(Paragraph::new(right).alignment(Alignment::Right), chunks[2]);
}
