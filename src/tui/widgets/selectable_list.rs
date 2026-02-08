// Copyright (C) 2026 RiPetitor
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::models::CleanupItem;
use crate::tui::screens::common::Theme;
use crate::utils::size_format::format_size;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState};

pub fn render_selectable_list(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    title: &str,
    items: &[&CleanupItem],
    selected_index: usize,
) {
    let content_width = area.width.saturating_sub(6) as usize;
    let size_width = items
        .iter()
        .map(|item| format_size(item.size).len())
        .max()
        .unwrap_or(4)
        .max(4);
    let marker_width = 5usize; // "[x] " or "[ ] " or "[!] "
    let name_width = content_width
        .saturating_sub(marker_width + size_width + 1)
        .max(10);

    let list_items: Vec<ListItem> = items
        .iter()
        .map(|item| {
            let (marker, marker_style) = if !item.can_clean {
                (
                    " [!] ",
                    Style::default()
                        .fg(Theme::BLOCKED)
                        .add_modifier(Modifier::DIM),
                )
            } else if item.selected {
                (
                    " [x] ",
                    Style::default()
                        .fg(Theme::SELECTED)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                (" [ ] ", Style::default().fg(Theme::TEXT_MUTED))
            };

            let display_name = item.path.as_deref().unwrap_or(&item.name);
            let name = truncate_str(display_name, name_width);
            let size = format_size(item.size);

            let text_style = if !item.can_clean {
                Style::default().fg(Theme::TEXT_MUTED)
            } else if item.selected {
                Style::default().fg(Theme::TEXT)
            } else {
                Style::default().fg(Theme::TEXT_DIM)
            };

            let size_style = if item.can_clean {
                Style::default().fg(Theme::ACCENT_DIM)
            } else {
                Style::default().fg(Theme::TEXT_MUTED)
            };

            ListItem::new(Line::from(vec![
                Span::styled(marker, marker_style),
                Span::styled(format!("{name:<name_width$}"), text_style),
                Span::raw(" "),
                Span::styled(format!("{size:>size_width$}"), size_style),
            ]))
        })
        .collect();

    let mut list_state = ListState::default();
    if !list_items.is_empty() {
        let selected = selected_index.min(list_items.len() - 1);
        list_state.select(Some(selected));
    }

    let list = List::new(list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Theme::BORDER))
                .title(Span::styled(
                    format!(" {title} "),
                    Style::default()
                        .fg(Theme::TEXT)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .highlight_style(
            Style::default()
                .bg(Theme::BG_HIGHLIGHT)
                .fg(Theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");

    frame.render_stateful_widget(list, area, &mut list_state);
}

fn truncate_str(value: &str, max_len: usize) -> String {
    if value.len() <= max_len {
        return value.to_string();
    }
    if max_len <= 3 {
        return value.chars().take(max_len).collect();
    }
    let mut truncated: String = value.chars().take(max_len - 1).collect();
    truncated.push('…');
    truncated
}
