// Copyright (C) 2026 RiPetitor
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::tui::screens::common::{Theme, styled_block};
use ratatui::style::Style;
use ratatui::widgets::Paragraph;

pub fn render_info_panel(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    title: &str,
    info: &str,
) {
    let paragraph = Paragraph::new(info)
        .block(styled_block(title))
        .style(Style::default().fg(Theme::TEXT_DIM))
        .wrap(ratatui::widgets::Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
