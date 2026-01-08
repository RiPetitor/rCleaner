use crate::models::CleanupItem;
use crate::utils::size_format::format_size;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

pub fn render_selectable_list(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    title: &str,
    items: &[&CleanupItem],
    selected_index: usize,
) {
    let content_width = area.width.saturating_sub(4) as usize;
    let size_width = items
        .iter()
        .map(|item| format_size(item.size).len())
        .max()
        .unwrap_or(4);
    let size_width = size_width.max(4);
    let marker_width = 4usize;
    let name_width = content_width
        .saturating_sub(marker_width + 1 + size_width + 1)
        .max(10);

    let list_items: Vec<ListItem> = items
        .iter()
        .map(|item| {
            let marker = if !item.can_clean {
                "[!]"
            } else if item.selected {
                "[x]"
            } else {
                "[ ]"
            };

            let display_name = item.path.as_deref().unwrap_or(&item.name);
            let name = truncate_ascii(display_name, name_width);
            let size = format_size(item.size);
            let line = format!("{marker} {name:<name_width$} {size:>size_width$}");

            let style = if item.can_clean {
                Style::default()
            } else {
                Style::default().fg(Color::DarkGray)
            };

            ListItem::new(Line::from(vec![Span::styled(line, style)]))
        })
        .collect();

    let mut list_state = ListState::default();
    if !list_items.is_empty() {
        let selected = selected_index.min(list_items.len() - 1);
        list_state.select(Some(selected));
    }

    let list = List::new(list_items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut list_state);
}

fn truncate_ascii(value: &str, max_len: usize) -> String {
    if value.len() <= max_len {
        return value.to_string();
    }
    if max_len <= 3 {
        return value.chars().take(max_len).collect();
    }
    let mut truncated = value.chars().take(max_len - 3).collect::<String>();
    truncated.push_str("...");
    truncated
}
