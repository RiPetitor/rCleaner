use crate::tui::screens::common::render_header;
use crate::tui::state::State;
use crate::tui::widgets::info_panel::render_info_panel;
use crate::tui::widgets::progress_bar::render_progress_bar;
use crate::tui::widgets::selectable_list::render_selectable_list;
use crate::tui::widgets::status_bar::render_status_bar;
use crate::tui::widgets::tabs::render_tabs;
use crate::utils::size_format::{format_percentage, format_size};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

pub fn render_main_screen(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    state: &State,
    system_label: &str,
) {
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(inner);

    render_header(frame, chunks[0], system_label, state.safety_level);
    render_tabs(frame, chunks[1], state.current_tab);

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(chunks[2]);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(body_chunks[0]);

    let visible_items = state.visible_items();
    if visible_items.is_empty() {
        let empty = Paragraph::new("No items found. Press [R] to rescan.")
            .block(Block::default().borders(Borders::ALL).title("Items"))
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, left_chunks[0]);
    } else {
        render_selectable_list(
            frame,
            left_chunks[0],
            "Items",
            &visible_items,
            state.selected_index,
        );
    }

    let ratio = if state.total_size == 0 {
        0.0
    } else {
        state.selected_size as f64 / state.total_size as f64
    };
    render_progress_bar(frame, left_chunks[1], ratio, "Selected");

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(0)])
        .split(body_chunks[1]);

    let info_text = match state.selected_item() {
        Some(item) => format!(
            "Name: {}\nSize: {}\nSource: {}\n{}\n",
            item.name,
            format_size(item.size),
            format_source(item),
            item.description
        ),
        None => "No item selected.".to_string(),
    };
    render_info_panel(frame, right_chunks[0], "Details", &info_text);

    let summary_text = format!(
        "Selected: {} items\nSelected size: {} ({})\nTotal: {} items\nTotal size: {}\n",
        state.selected_count(),
        format_size(state.selected_size),
        format_percentage(state.selected_size, state.total_size),
        state.items.len(),
        format_size(state.total_size)
    );
    render_info_panel(frame, right_chunks[1], "Summary", &summary_text);

    let mut keys = vec![
        "[Tab] Next",
        "[Shift+Tab] Prev",
        "[Up/Down] Move",
        "[Space] Select",
        "[A] All",
        "[Enter] Clean",
        "[S] Settings",
        "[R] Refresh",
        "[Q] Quit",
    ]
    .into_iter()
    .map(String::from)
    .collect::<Vec<String>>();

    if let Some(message) = state.status_message.as_deref() {
        keys.push(message.to_string());
    }

    render_status_bar(frame, chunks[3], &keys);
}

fn format_source(item: &crate::models::CleanupItem) -> String {
    match &item.source {
        crate::models::CleanupSource::FileSystem => "Files".to_string(),
        crate::models::CleanupSource::PackageManager(name) => format!("Package: {name}"),
        crate::models::CleanupSource::Container(name) => format!("Container: {name}"),
    }
}
