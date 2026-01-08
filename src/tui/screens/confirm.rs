use crate::tui::screens::common::render_header;
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
        .title("Confirm Cleanup");
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

    let selected_items: Vec<_> = state.items.iter().filter(|item| item.selected).collect();
    let selected_count = selected_items.len();
    let selected_size = format_size(state.selected_size);

    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(0)])
        .split(chunks[1]);

    let mode_label = if dry_run { "Dry run" } else { "Execute" };
    let summary = Paragraph::new(Line::from(vec![
        Span::styled(
            "Selected items: ",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("{selected_count}")),
        Span::raw(" | "),
        Span::styled(
            "Estimated size: ",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(selected_size),
        Span::raw(" | "),
        Span::styled("Mode: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(mode_label),
    ]))
    .block(Block::default().borders(Borders::ALL).title("Summary"));
    frame.render_widget(summary, content_chunks[0]);

    let max_items = content_chunks[1].height.saturating_sub(2) as usize;
    let mut list_items = Vec::new();
    for item in selected_items.iter().take(max_items) {
        let line = format!(
            "{} ({})",
            item.path.as_deref().unwrap_or(&item.name),
            format_size(item.size)
        );
        list_items.push(ListItem::new(line));
    }
    if selected_count > max_items && max_items > 0 {
        list_items.push(ListItem::new(format!(
            "... and {} more",
            selected_count - max_items
        )));
    }

    let list = List::new(list_items)
        .block(Block::default().borders(Borders::ALL).title("Items"))
        .style(Style::default().fg(Color::White));
    frame.render_widget(list, content_chunks[1]);

    let keys = vec![
        "[Y] Confirm".to_string(),
        "[N] Cancel".to_string(),
        "[Esc] Back".to_string(),
    ];
    render_status_bar(frame, chunks[2], &keys);
}
