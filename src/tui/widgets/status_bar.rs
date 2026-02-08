// Copyright (C) 2026 RiPetitor
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::tui::screens::common::Theme;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub fn render_status_bar(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    segments: &[String],
) {
    let spans: Vec<Span> = segments
        .iter()
        .enumerate()
        .flat_map(|(i, seg)| {
            let mut result = Vec::new();
            if i > 0 {
                result.push(Span::styled(
                    "  │  ",
                    Style::default().fg(Theme::TEXT_MUTED),
                ));
            }
            // Highlight key shortcuts in brackets
            if let Some((key, rest)) = seg.strip_prefix('[').and_then(|s| s.split_once(']')) {
                result.push(Span::styled(
                    format!("[{key}]"),
                    Style::default()
                        .fg(Theme::ACCENT)
                        .add_modifier(Modifier::BOLD),
                ));
                result.push(Span::styled(
                    rest.to_string(),
                    Style::default().fg(Theme::TEXT_DIM),
                ));
            } else {
                result.push(Span::styled(
                    seg.clone(),
                    Style::default().fg(Theme::TEXT_DIM),
                ));
            }
            result
        })
        .collect();

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}
