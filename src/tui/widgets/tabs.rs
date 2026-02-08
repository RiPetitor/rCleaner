// Copyright (C) 2026 RiPetitor
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::i18n;
use crate::tui::screens::common::Theme;
use crate::tui::state::State;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Tabs};

pub fn tab_titles() -> [&'static str; 7] {
    [
        i18n::tab_cache(),
        i18n::tab_apps(),
        i18n::tab_system(),
        i18n::tab_temp(),
        i18n::tab_logs(),
        i18n::tab_packages(),
        i18n::tab_kernels(),
    ]
}

pub fn render_tabs(frame: &mut ratatui::Frame, area: ratatui::layout::Rect, state: &State) {
    let titles: Vec<Line> = tab_titles()
        .iter()
        .enumerate()
        .map(|(i, title)| {
            let count = state.tab_item_count(i);
            let label = if count > 0 {
                format!(" {title} ({count}) ")
            } else {
                format!(" {title} ")
            };
            Line::from(label)
        })
        .collect();

    let tabs = Tabs::new(titles)
        .select(state.current_tab)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Theme::BORDER)),
        )
        .highlight_style(
            Style::default()
                .fg(Theme::ACCENT)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )
        .style(Style::default().fg(Theme::TEXT_DIM))
        .divider(Span::styled("│", Style::default().fg(Theme::TEXT_MUTED)));

    frame.render_widget(tabs, area);
}
